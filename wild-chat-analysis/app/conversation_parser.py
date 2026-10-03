"""Turn a raw dataset row into the English user prompts it contains."""

from __future__ import annotations

import hashlib
import json
import logging
from collections.abc import Mapping
from datetime import date, datetime
from typing import Any

from app.models import UserPrompt

logger = logging.getLogger(__name__)


def make_prompt_id(conversation_id: str, content: str) -> str:
    """Stable id for a prompt, used for resume."""
    return hashlib.sha256(f"{conversation_id}\x1f{content}".encode("utf-8")).hexdigest()


def in_sample(conversation_id: str, fraction: float, seed: str) -> bool:
    """Pick the same conversations on every run, so resume works with sampling."""
    if fraction >= 1.0:
        return True
    digest = hashlib.sha256(f"{seed}:{conversation_id}".encode("utf-8")).hexdigest()
    return int(digest[:15], 16) / float(16**15) < fraction


def format_timestamp(value: Any) -> str | None:
    """Turn a date or datetime into ISO text. Other values become plain strings; None stays None."""
    if value is None:
        return None
    if isinstance(value, (datetime, date)):
        return value.isoformat()
    return str(value)


def _clean_str(value: Any) -> str | None:
    """Strip a value to text. Returns None when it is None or blank."""
    if value is None:
        return None
    text = str(value).strip()
    return text or None


def parse_conversation(raw: Any) -> list[dict[str, Any]]:
    """Return the list of message dicts, or [] if the structure is not usable."""
    if raw is None:
        return []
    if isinstance(raw, (str, bytes)):
        try:
            raw = json.loads(raw)
        except (ValueError, TypeError):
            return []
    if hasattr(raw, "tolist") and not isinstance(raw, (list, tuple, dict)):
        raw = raw.tolist()  # numpy arrays from pandas
    if isinstance(raw, Mapping):
        raw = raw.get("messages", [raw])
    if not isinstance(raw, (list, tuple)):
        return []
    return [dict(message) for message in raw if isinstance(message, Mapping)]


def _language_matches(value: Any, language: str) -> bool:
    """True when the message language tag equals `language`, ignoring case and spaces."""
    return isinstance(value, str) and value.strip().lower() == language.strip().lower()


def extract_user_prompts(record: Mapping[str, Any], language: str = "English") -> list[UserPrompt]:
    """Return one UserPrompt per English user message. Repeated identical messages
    in the same conversation share an id, so only the first is kept."""
    conversation_id = _clean_str(record.get("conversation_id"))
    if conversation_id is None:
        return []
    model = _clean_str(record.get("model"))
    timestamp = format_timestamp(record.get("timestamp"))
    fallback_language = record.get("language")

    prompts: list[UserPrompt] = []
    seen: set[str] = set()
    user_turn = -1
    for message in parse_conversation(record.get("conversation")):
        if str(message.get("role", "")).strip().lower() != "user":
            continue
        user_turn += 1
        content = message.get("content")
        if not isinstance(content, str) or not content.strip():
            continue
        message_language = message.get("language") or fallback_language
        if not _language_matches(message_language, language):
            continue
        prompt_id = make_prompt_id(conversation_id, content)
        if prompt_id in seen:
            continue
        seen.add(prompt_id)
        prompts.append(
            UserPrompt(
                prompt_id=prompt_id,
                conversation_id=conversation_id,
                model=model,
                timestamp=timestamp,
                content=content,
                user_turn=user_turn,
            )
        )
    return prompts
