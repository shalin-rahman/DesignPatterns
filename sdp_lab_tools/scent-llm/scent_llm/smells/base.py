"""Shared data model for LLM code smells.

Keeps the taxonomy (what a smell IS) separate from detection (how we FIND
it) and from refactoring (what to DO about it), so each concern can change
independently.
"""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class SmellKind(str, Enum):
    NSO = "NSO"    # No Structured Output
    UMM = "UMM"    # Unbounded Max Metrics
    TNES = "TNES"  # Temperature Not Explicitly Set
    NMVP = "NMVP"  # No Model Version Pinning
    NSM = "NSM"    # No System Message


SMELL_TITLES: dict[SmellKind, str] = {
    SmellKind.NSO: "No Structured Output",
    SmellKind.UMM: "Unbounded Max Metrics",
    SmellKind.TNES: "Temperature Not Explicitly Set",
    SmellKind.NMVP: "No Model Version Pinning",
    SmellKind.NSM: "No System Message",
}

SMELL_DESCRIPTIONS: dict[SmellKind, str] = {
    SmellKind.NSO: (
        "The LLM call has no schema, response_format, or tool/function-calling "
        "constraint, so the response is free-form text the caller must parse "
        "by hand."
    ),
    SmellKind.UMM: (
        "The call sets no upper bound on generated length (max_tokens / "
        "num_predict / equivalent), so a single request can run away in "
        "cost, latency, or memory."
    ),
    SmellKind.TNES: (
        "The call relies on the provider's default temperature instead of "
        "setting one explicitly, so output determinism can change silently "
        "when the provider changes its default."
    ),
    SmellKind.NMVP: (
        "The model identifier is a generic or floating alias (e.g. 'gpt-4', "
        "'latest') rather than a pinned, versioned snapshot, so behavior can "
        "shift under the caller without warning."
    ),
    SmellKind.NSM: (
        "The chat request has no system-role message, leaving the model's "
        "role, constraints, and output contract unspecified."
    ),
}


@dataclass(frozen=True)
class Finding:
    """One detected smell instance, tied to a concrete source location."""

    smell: SmellKind
    file_path: str
    line: int
    col: int
    snippet: str
    detail: str
    severity: str = "medium"  # "low" | "medium" | "high"

    @property
    def title(self) -> str:
        return SMELL_TITLES[self.smell]

    @property
    def description(self) -> str:
        return SMELL_DESCRIPTIONS[self.smell]
