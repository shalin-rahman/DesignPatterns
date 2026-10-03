from __future__ import annotations

import json
from datetime import datetime, timezone

from app.conversation_parser import (
    extract_user_prompts,
    format_timestamp,
    in_sample,
    make_prompt_id,
    parse_conversation,
)
from tests.conftest import message


def record(conversation: object, **extra: object) -> dict[str, object]:
    return {
        "conversation_id": "abc",
        "model": "gpt-4",
        "timestamp": datetime(2023, 1, 1, 10, 0, tzinfo=timezone.utc),
        "conversation": conversation,
        **extra,
    }


def test_only_user_messages_are_kept() -> None:
    prompts = extract_user_prompts(
        record([message("system", "sys"), message("user", "Hi"), message("assistant", "Hello")])
    )
    assert [p.content for p in prompts] == ["Hi"]


def test_non_english_messages_are_dropped() -> None:
    prompts = extract_user_prompts(record([message("user", "Hola", "Spanish"), message("user", "Hello")]))
    assert [p.content for p in prompts] == ["Hello"]


def test_language_falls_back_to_conversation_level() -> None:
    conv = [{"role": "user", "content": "Hello"}]
    assert len(extract_user_prompts(record(conv, language="English"))) == 1
    assert extract_user_prompts(record(conv, language="German")) == []
    assert extract_user_prompts(record(conv)) == []


def test_multiple_user_messages_keep_metadata() -> None:
    prompts = extract_user_prompts(
        record([message("user", "First"), message("assistant", "ok"), message("user", "Second")])
    )
    assert [p.content for p in prompts] == ["First", "Second"]
    assert [p.user_turn for p in prompts] == [0, 1]
    assert all(p.conversation_id == "abc" and p.model == "gpt-4" for p in prompts)
    assert prompts[0].timestamp == "2023-01-01T10:00:00+00:00"


def test_duplicate_message_in_same_conversation_kept_once() -> None:
    prompts = extract_user_prompts(record([message("user", "Same"), message("user", "Same")]))
    assert len(prompts) == 1


def test_missing_conversation_gives_no_prompts() -> None:
    assert extract_user_prompts(record(None)) == []
    assert extract_user_prompts({"conversation_id": "x"}) == []


def test_missing_conversation_id_gives_no_prompts() -> None:
    assert extract_user_prompts(record([message("user", "Hi")], conversation_id=None)) == []


def test_malformed_conversation_is_handled() -> None:
    assert parse_conversation(42) == []
    assert parse_conversation("{not json") == []
    assert parse_conversation(["text", None, {"role": "user", "content": "ok"}]) == [
        {"role": "user", "content": "ok"}
    ]
    bad_messages = [
        {"role": "user", "content": None, "language": "English"},
        {"role": "user", "content": "   ", "language": "English"},
        {"role": "user", "content": 123, "language": "English"},
        {"content": "no role", "language": "English"},
    ]
    assert extract_user_prompts(record(bad_messages)) == []


def test_json_string_conversation_is_parsed() -> None:
    prompts = extract_user_prompts(record(json.dumps([message("user", "Hi")])))
    assert [p.content for p in prompts] == ["Hi"]


def test_role_and_language_matching_ignore_case() -> None:
    prompts = extract_user_prompts(record([{"role": "USER", "content": "Hi", "language": "english"}]))
    assert len(prompts) == 1


def test_unicode_content_is_preserved() -> None:
    text = "Explain naïve café — 日本語"
    assert extract_user_prompts(record([message("user", text)]))[0].content == text


def test_prompt_id_is_deterministic() -> None:
    assert make_prompt_id("a", "b") == make_prompt_id("a", "b")
    assert make_prompt_id("a", "b") != make_prompt_id("a", "c")


def test_sampling_is_deterministic() -> None:
    picked = [cid for cid in map(str, range(1000)) if in_sample(cid, 0.3, "seed")]
    assert picked == [cid for cid in map(str, range(1000)) if in_sample(cid, 0.3, "seed")]
    assert 200 < len(picked) < 400
    assert in_sample("anything", 1.0, "seed")


def test_format_timestamp() -> None:
    assert format_timestamp(None) is None
    assert format_timestamp("2023-01-01") == "2023-01-01"
