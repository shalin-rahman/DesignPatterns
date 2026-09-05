"""Detector functions: LLMCallSite -> Finding | None, one per smell.

Each detector is independent and side-effect free, so new smells can be
added without touching existing ones.
"""
from __future__ import annotations

import re

from scent_llm.analysis.ast_analyzer import LLMCallSite
from scent_llm.smells.base import Finding, SmellKind

_STRUCTURED_OUTPUT_KEYS = ("response_format", "functions", "tools", "tool_choice", "format")
_MAX_TOKEN_KEYS = ("max_tokens", "num_predict", "max_output_tokens", "max_new_tokens")
_TEMPERATURE_KEYS = ("temperature",)

# A model string counts as "pinned" if it carries a concrete version marker:
# a dated snapshot (YYYY-MM-DD / YYYYMMDD), an explicit version tag after a
# colon (Ollama's "name:tag" convention), or a numeric build suffix.
_PINNED_MODEL_PATTERN = re.compile(
    r"(\d{4}-?\d{2}-?\d{2})"      # date snapshot, e.g. -20240620 or -2024-06-20
    r"|(:[\w.\-]+)$"              # ollama tag, e.g. llama3.1:8b
    r"|(-\d{3,})$"                # numeric build suffix, e.g. gpt-4-0613
)

_KNOWN_UNPINNED_ALIASES = {
    "gpt-4", "gpt-4-turbo", "gpt-4o", "gpt-3.5-turbo", "gpt-4o-mini",
    "o1", "o3", "o1-mini", "claude-3-opus", "claude-3-sonnet",
    "claude-3-haiku", "claude-3.5-sonnet", "llama3", "llama2", "mixtral",
    "gemini-pro", "gemini-1.5-pro", "latest",
}


def detect_no_structured_output(call: LLMCallSite) -> Finding | None:
    if call.has_any(*_STRUCTURED_OUTPUT_KEYS):
        return None
    return Finding(
        smell=SmellKind.NSO,
        file_path=call.file_path,
        line=call.line,
        col=call.col,
        snippet=call.source_snippet,
        detail=(
            "No response_format / tools / functions / format argument found on "
            f"`{call.call_expr}`; the model's output must be parsed as free text."
        ),
        severity="medium",
    )


def detect_unbounded_max_metrics(call: LLMCallSite) -> Finding | None:
    if call.has_any(*_MAX_TOKEN_KEYS):
        value = call.get_literal(*_MAX_TOKEN_KEYS)
        if value is None:
            return None  # bound is a non-literal expression; assume caller manages it
        if isinstance(value, (int, float)) and value <= 0:
            return Finding(
                smell=SmellKind.UMM,
                file_path=call.file_path,
                line=call.line,
                col=call.col,
                snippet=call.source_snippet,
                detail=f"`{call.call_expr}` sets a non-positive token bound ({value}), which most backends treat as unbounded.",
                severity="high",
            )
        return None
    return Finding(
        smell=SmellKind.UMM,
        file_path=call.file_path,
        line=call.line,
        col=call.col,
        snippet=call.source_snippet,
        detail=(
            f"No max_tokens / num_predict bound set on `{call.call_expr}`; "
            "generation length is limited only by the provider's own default."
        ),
        severity="medium",
    )


def detect_temperature_not_set(call: LLMCallSite) -> Finding | None:
    if call.has_any(*_TEMPERATURE_KEYS):
        return None
    return Finding(
        smell=SmellKind.TNES,
        file_path=call.file_path,
        line=call.line,
        col=call.col,
        snippet=call.source_snippet,
        detail=f"No explicit temperature passed to `{call.call_expr}`; behavior depends on the provider's current default.",
        severity="low",
    )


def detect_no_model_version_pinning(call: LLMCallSite) -> Finding | None:
    model_value = call.get_literal("model")
    if model_value is None:
        if not call.has_any("model"):
            return Finding(
                smell=SmellKind.NMVP,
                file_path=call.file_path,
                line=call.line,
                col=call.col,
                snippet=call.source_snippet,
                detail=f"`{call.call_expr}` does not pass a `model` argument at all.",
                severity="high",
            )
        return None  # model is a non-literal expression (e.g. from config) - can't judge statically
    if not isinstance(model_value, str):
        return None
    is_pinned = bool(_PINNED_MODEL_PATTERN.search(model_value))
    is_known_unpinned = model_value.strip().lower() in _KNOWN_UNPINNED_ALIASES
    if is_pinned and not is_known_unpinned:
        return None
    return Finding(
        smell=SmellKind.NMVP,
        file_path=call.file_path,
        line=call.line,
        col=call.col,
        snippet=call.source_snippet,
        detail=f"Model string '{model_value}' is a floating alias, not a pinned version/snapshot/tag.",
        severity="medium",
    )


def detect_no_system_message(call: LLMCallSite) -> Finding | None:
    if "system" in call.kwargs:
        return None  # Anthropic-style top-level system parameter
    messages = call.get_literal("messages")
    if messages is None:
        if not call.has_any("messages"):
            return None  # not a messages-based call (e.g. ollama.generate uses "prompt")
        return None  # messages is a non-literal expression - can't judge statically
    if not isinstance(messages, list):
        return None
    has_system = any(
        isinstance(m, dict) and m.get("role") == "system" for m in messages
    )
    if has_system:
        return None
    return Finding(
        smell=SmellKind.NSM,
        file_path=call.file_path,
        line=call.line,
        col=call.col,
        snippet=call.source_snippet,
        detail=f"`{call.call_expr}` sends messages with no system-role entry.",
        severity="medium",
    )


_ALL_DETECTORS = (
    detect_no_structured_output,
    detect_unbounded_max_metrics,
    detect_temperature_not_set,
    detect_no_model_version_pinning,
    detect_no_system_message,
)


def run_all_detectors(call: LLMCallSite) -> list[Finding]:
    findings = []
    for detector in _ALL_DETECTORS:
        finding = detector(call)
        if finding is not None:
            findings.append(finding)
    return findings
