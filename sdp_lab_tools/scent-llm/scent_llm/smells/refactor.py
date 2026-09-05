"""Actionable refactoring suggestions, one per smell kind.

Each suggestion is a short instruction plus a minimal before/after snippet.
Snippets are illustrative, not a literal patch of the caller's file — the
tool does not attempt to auto-rewrite arbitrary third-party SDK calls.
"""
from __future__ import annotations

from scent_llm.smells.base import Finding, SmellKind

_SUGGESTIONS: dict[SmellKind, str] = {
    SmellKind.NSO: (
        "Constrain the response to a schema instead of parsing free text:\n"
        "  # OpenAI / Groq (OpenAI-compatible):\n"
        "  response_format={\"type\": \"json_schema\", \"json_schema\": {...}}\n"
        "  # Ollama:\n"
        "  format=\"json\"  # or a JSON schema dict\n"
        "  # Anthropic: use tool-calling with a strict input schema instead of prose."
    ),
    SmellKind.UMM: (
        "Set an explicit upper bound sized to the task, e.g.:\n"
        "  max_tokens=512          # OpenAI / Groq / Anthropic\n"
        "  options={\"num_predict\": 512}  # Ollama\n"
        "Pick the smallest value that comfortably covers the expected output."
    ),
    SmellKind.TNES: (
        "Pass temperature explicitly rather than relying on the provider default:\n"
        "  temperature=0.0   # deterministic, best for extraction/classification\n"
        "  temperature=0.7   # more varied, better for creative generation\n"
        "Pick a value deliberately and comment why."
    ),
    SmellKind.NMVP: (
        "Pin to a concrete, dated snapshot or tagged build instead of a floating alias:\n"
        "  model=\"gpt-4o-2024-08-06\"        # not \"gpt-4o\"\n"
        "  model=\"claude-3-5-sonnet-20241022\"  # not \"claude-3.5-sonnet\"\n"
        "  model=\"llama3.1:8b\"              # Ollama tag, not bare \"llama3\"\n"
        "This keeps behavior stable when the provider rolls the default forward."
    ),
    SmellKind.NSM: (
        "Add a system-role message that states the model's role, constraints, "
        "and expected output format:\n"
        "  messages=[\n"
        "      {\"role\": \"system\", \"content\": \"You are ... Respond with ...\"},\n"
        "      {\"role\": \"user\", \"content\": user_input},\n"
        "  ]\n"
        "  # Anthropic: pass system=\"...\" as a top-level argument instead."
    ),
}


def suggest_refactor(finding: Finding) -> str:
    return _SUGGESTIONS[finding.smell]


def render_finding_report(finding: Finding) -> str:
    """Human-readable block combining what/why/where/fix for one finding."""
    location = f"{finding.file_path}:{finding.line}:{finding.col}"
    return (
        f"[{finding.smell.value}] {finding.title}  ({finding.severity})\n"
        f"  where: {location}\n"
        f"  code:  {finding.snippet.strip().splitlines()[0]}\n"
        f"  why:   {finding.description}\n"
        f"  what:  {finding.detail}\n"
        f"  fix:\n"
        + "\n".join(f"    {line}" for line in suggest_refactor(finding).splitlines())
    )
