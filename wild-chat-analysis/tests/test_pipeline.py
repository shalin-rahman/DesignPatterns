"""End-to-end run with a fake analyzer: output format and resume behaviour."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from app.config import Settings, load_settings
from app.llm_client import FatalLLMError
from app.models import AnalysisResult, OutputRecord, Smell, UserPrompt
from app.output_writer import CheckpointStore, write_final_json
from app.pipeline import run_pipeline


class FakeAnalyzer:
    def __init__(self, fail_on: str | None = None) -> None:
        self.seen: list[str] = []
        self.fail_on = fail_on

    def analyze(self, prompt: UserPrompt) -> AnalysisResult:
        self.seen.append(prompt.content)
        if prompt.content == self.fail_on:
            raise RuntimeError("boom")
        if prompt.content == "Help me fix this.":
            return AnalysisResult(
                smells=[
                    Smell(type="Vague / Missing Context", reason="No detail."),
                    Smell(type="Ambiguous References", reason="'this' is unclear."),
                ]
            )
        return AnalysisResult(smells=[])


def make_settings(parquet_file: Path, tmp_path: Path, **extra: object) -> Settings:
    return Settings(
        dataset_url=str(parquet_file),
        output_file=tmp_path / "out" / "result.json",
        concurrency=2,
        **extra,
    )


def read_output(settings: Settings) -> list[dict]:
    return json.loads(settings.output_file.read_text(encoding="utf-8"))


def test_full_run_output(parquet_file: Path, tmp_path: Path) -> None:
    settings = make_settings(parquet_file, tmp_path)
    analyzer = FakeAnalyzer()
    stats = run_pipeline(settings, analyzer=analyzer)

    assert sorted(analyzer.seen) == ["Help me fix this.", "The login page, café ünïcode"]
    assert stats.prompts_extracted == 2 and stats.succeeded == 2
    records = read_output(settings)
    assert len(records) == 3
    assert set(records[0]) == {"conversation_id", "model", "timestamp", "content", "smell_type", "smell_reason"}
    clean = [r for r in records if r["smell_type"] is None]
    assert clean[0]["content"] == "The login page, café ünïcode"
    assert clean[0]["timestamp"] == "2023-05-01T10:00:00+00:00"
    assert "café" in settings.output_file.read_text(encoding="utf-8")  # ensure_ascii=False


def test_resume_skips_processed_prompts(parquet_file: Path, tmp_path: Path) -> None:
    settings = make_settings(parquet_file, tmp_path, max_prompts=1)
    first = FakeAnalyzer()
    run_pipeline(settings, analyzer=first)
    assert len(first.seen) == 1

    settings = make_settings(parquet_file, tmp_path)
    second = FakeAnalyzer()
    stats = run_pipeline(settings, analyzer=second)
    assert stats.already_processed == 1
    assert len(second.seen) == 1 and second.seen != first.seen
    assert len({r["content"] for r in read_output(settings)}) == 2


def test_no_resume_starts_over(parquet_file: Path, tmp_path: Path) -> None:
    run_pipeline(make_settings(parquet_file, tmp_path), analyzer=FakeAnalyzer())
    analyzer = FakeAnalyzer()
    run_pipeline(make_settings(parquet_file, tmp_path, resume=False), analyzer=analyzer)
    assert len(analyzer.seen) == 2


def test_failed_prompt_does_not_stop_run_and_is_retried(parquet_file: Path, tmp_path: Path) -> None:
    settings = make_settings(parquet_file, tmp_path)
    stats = run_pipeline(settings, analyzer=FakeAnalyzer(fail_on="Help me fix this."))
    assert stats.failed == 1 and stats.succeeded == 1
    failure = json.loads(settings.failures_file.read_text(encoding="utf-8"))
    assert "content" not in failure

    retry = FakeAnalyzer()
    run_pipeline(settings, analyzer=retry)
    assert retry.seen == ["Help me fix this."]
    assert len(read_output(settings)) == 3


def test_fatal_api_error_stops_run_but_keeps_output(parquet_file: Path, tmp_path: Path) -> None:
    class BadKeyAnalyzer(FakeAnalyzer):
        def analyze(self, prompt: UserPrompt) -> AnalysisResult:
            raise FatalLLMError("HTTP 401: invalid_api_key")

    settings = make_settings(parquet_file, tmp_path)
    with pytest.raises(FatalLLMError):
        run_pipeline(settings, analyzer=BadKeyAnalyzer())
    assert read_output(settings) == []


def test_dry_run_calls_nothing(parquet_file: Path, tmp_path: Path) -> None:
    settings = make_settings(parquet_file, tmp_path)
    stats = run_pipeline(settings, dry_run=True)
    assert stats.to_process == 2
    assert not settings.output_file.exists()


def test_checkpoint_survives_half_written_line(tmp_path: Path) -> None:
    store = CheckpointStore(tmp_path / "cp.jsonl")
    record = OutputRecord(conversation_id="c", model=None, timestamp=None, content="x", smell_type=None, smell_reason=None)
    with store:
        store.append("p1", [record])
    with open(store.path, "a", encoding="utf-8") as handle:
        handle.write('{"prompt_id": "p2", "rec')  # simulated crash
    with store:
        store.append("p3", [record])
    assert store.load_processed_ids() == {"p1", "p3"}
    assert write_final_json(store, tmp_path / "out.json") == 2


def test_empty_checkpoint_gives_empty_array(tmp_path: Path) -> None:
    out = tmp_path / "out.json"
    assert write_final_json(CheckpointStore(tmp_path / "none.jsonl"), out) == 0
    assert json.loads(out.read_text(encoding="utf-8")) == []


def test_settings_precedence(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    config = tmp_path / "config.yaml"
    config.write_text("concurrency: 3\nrequest_timeout: 30\nllm:\n  model: yaml-model\n", encoding="utf-8")
    monkeypatch.setenv("LLM_MODEL", "env-model")
    monkeypatch.setenv("LLM_API_KEY", "secret")
    settings = load_settings(config, {"concurrency": 7, "max_prompts": None}, env_file=None)
    assert settings.concurrency == 7
    assert settings.llm.model == "env-model"
    assert settings.llm.request_timeout == 30
    assert settings.llm.api_key is not None and settings.llm.api_key.get_secret_value() == "secret"
