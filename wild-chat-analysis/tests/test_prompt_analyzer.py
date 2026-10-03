from __future__ import annotations

import json

import pytest

from app.llm_client import InvalidOutputError, NonRetryableLLMError
from app.models import AnalysisResult, Smell, UserPrompt
from app.prompt_analyzer import (
    AnalysisFailedError,
    PromptAnalyzer,
    build_messages,
    build_output_records,
    normalize_smell_type,
    parse_analysis_response,
)

PROMPT = UserPrompt(
    prompt_id="p1",
    conversation_id="abc123",
    model="gpt-4",
    timestamp="2025-01-01T10:00:00",
    content="Help me fix this.",
    user_turn=0,
)


class FakeClient:
    def __init__(self, replies: list[object]) -> None:
        self.replies = list(replies)
        self.calls: list[list[dict[str, str]]] = []

    def complete(self, messages: list[dict[str, str]]) -> str:
        self.calls.append(messages)
        reply = self.replies.pop(0)
        if isinstance(reply, Exception):
            raise reply
        return str(reply)


def test_empty_smell_result() -> None:
    assert parse_analysis_response('{"smells": []}').smells == []


def test_multiple_smells_parsed_and_normalized() -> None:
    text = json.dumps(
        {
            "smells": [
                {"type": "vague/missing context", "reason": "No detail."},
                {"type": "Ambiguous References", "reason": "'this' is unclear."},
                {"type": "Prompt Bloat", "reason": "Padding."},
            ]
        }
    )
    types = [s.type for s in parse_analysis_response(text).smells]
    assert types == ["Vague / Missing Context", "Ambiguous References", "Prompt Bloat / Convoluted Prompt"]


def test_new_category_is_kept() -> None:
    result = parse_analysis_response('{"smells": [{"type": "Unsafe Request", "reason": "r"}]}')
    assert result.smells[0].type == "Unsafe Request"


def test_duplicate_types_are_merged() -> None:
    text = '{"smells": [{"type": "Overloaded Prompt", "reason": "a"}, {"type": "overloaded prompt", "reason": "b"}]}'
    assert len(parse_analysis_response(text).smells) == 1


def test_json_in_markdown_fence_and_extra_text() -> None:
    assert parse_analysis_response('```json\n{"smells": []}\n```').smells == []
    assert parse_analysis_response('Here you go: {"smells": []} done').smells == []


def test_backticks_inside_reason_do_not_break_parsing() -> None:
    text = json.dumps({"smells": [{"type": "Format Ambiguity", "reason": "Quotes ```code``` and ```more```"}]})
    assert parse_analysis_response(text).smells[0].reason.startswith("Quotes")


@pytest.mark.parametrize(
    "text",
    ["not json", "{}", '{"smells": "none"}', '{"smells": [{"type": ""}]}', '{"smells": [{"reason": "x"}]}'],
)
def test_invalid_response_raises(text: str) -> None:
    with pytest.raises(ValueError):
        parse_analysis_response(text)


def test_normalize_smell_type() -> None:
    assert normalize_smell_type("Format Ambiguity.") == "Format Ambiguity"
    assert normalize_smell_type("Bias") == "Bias / Loaded Framing"


def test_analyzer_retries_after_invalid_output() -> None:
    client = FakeClient(["garbage", InvalidOutputError("bad"), '{"smells": []}'])
    result = PromptAnalyzer(client, parse_retries=2).analyze(PROMPT)
    assert result.smells == []
    assert len(client.calls) == 3


def test_analyzer_gives_up_after_retries() -> None:
    client = FakeClient(["garbage", "still garbage"])
    with pytest.raises(AnalysisFailedError):
        PromptAnalyzer(client, parse_retries=1).analyze(PROMPT)


def test_api_errors_are_not_swallowed() -> None:
    client = FakeClient([NonRetryableLLMError("HTTP 401")])
    with pytest.raises(NonRetryableLLMError):
        PromptAnalyzer(client).analyze(PROMPT)


def test_messages_hold_only_the_prompt_and_mark_follow_ups() -> None:
    messages = build_messages("Hello", 1000, follow_up=True)
    assert [m["role"] for m in messages] == ["system", "user"]
    assert "Hello" in messages[1]["content"]
    assert "follow-up" in messages[1]["content"]


def test_long_prompt_is_truncated() -> None:
    messages = build_messages("x" * 500, 100)
    assert "x" * 101 not in messages[1]["content"]
    assert "truncated" in messages[1]["content"]


def test_output_records_one_per_smell() -> None:
    result = AnalysisResult(
        smells=[Smell(type="Vague / Missing Context", reason="a"), Smell(type="Ambiguous References", reason="b")]
    )
    records = build_output_records(PROMPT, result)
    assert [r.smell_type for r in records] == ["Vague / Missing Context", "Ambiguous References"]
    assert all(r.content == PROMPT.content and r.conversation_id == "abc123" for r in records)


def test_output_record_for_no_smell() -> None:
    records = build_output_records(PROMPT, AnalysisResult(smells=[]))
    assert len(records) == 1
    assert records[0].smell_type is None and records[0].smell_reason is None
