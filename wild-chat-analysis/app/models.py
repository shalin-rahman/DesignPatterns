"""Data models shared across the pipeline."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator


class Smell(BaseModel):
    """One prompt smell reported by the LLM."""

    model_config = ConfigDict(extra="ignore")

    type: str = Field(min_length=1)
    reason: str = Field(min_length=1)

    @field_validator("type", "reason", mode="before")
    @classmethod
    def _strip(cls, value: Any) -> Any:
        return value.strip() if isinstance(value, str) else value


class AnalysisResult(BaseModel):
    """The full LLM answer for one prompt. `smells` is required; it may be empty."""

    model_config = ConfigDict(extra="ignore")

    smells: list[Smell]


@dataclass(frozen=True)
class UserPrompt:
    """One English user message taken from a conversation."""

    prompt_id: str
    conversation_id: str
    model: str | None
    timestamp: str | None
    content: str
    user_turn: int  # 0 for the first user message in the conversation


class OutputRecord(BaseModel):
    """One row of the output file. A prompt with N smells gives N rows."""

    conversation_id: str
    model: str | None
    timestamp: str | None
    content: str
    smell_type: str | None
    smell_reason: str | None
