"""Textual (no external graphics dependency) architecture rendering.

Two views:
  - `render_project_tree`: an ASCII directory tree of a scanned project,
    annotated with per-file LLM-call and finding counts.
  - `render_pipeline_sequence`: a fixed sequence diagram of scent-llm's own
    generate -> analyze -> detect -> refactor -> report pipeline, useful as
    living documentation of what a CLI run actually does.
"""
from __future__ import annotations

from pathlib import Path

_IGNORED_DIRS = {".git", "__pycache__", ".venv", "venv", "node_modules", ".mypy_cache", ".pytest_cache"}

_PIPELINE_STAGES = [
    ("User", "CLI", "invoke command (generate | analyze | sandbox | diagram)"),
    ("CLI", "Config", "load provider/model/temperature settings"),
    ("CLI", "LLMClient", "generate_code(task, language)  [generate only]"),
    ("LLMClient", "Provider", "HTTP POST /chat  [Ollama or Groq]"),
    ("Provider", "LLMClient", "raw completion JSON"),
    ("CLI", "ASTAnalyzer", "find_llm_call_sites(source)  [analyze only]"),
    ("ASTAnalyzer", "Detectors", "run_all_detectors(call_site) x5 smells"),
    ("Detectors", "Refactor", "suggest_refactor(finding)"),
    ("Refactor", "CLI", "Finding + suggestion"),
    ("CLI", "User", "report (text / JSON)"),
]


def render_project_tree(root: Path, annotate: dict[Path, str] | None = None) -> str:
    """ASCII tree of `root`. `annotate` maps a file path to a trailing label,
    e.g. {path: "2 findings"}; unlisted files render with no label."""
    annotate = annotate or {}
    lines = [f"{root.name}/"]
    _walk(root, prefix="", lines=lines, annotate=annotate)
    return "\n".join(lines)


def _walk(directory: Path, prefix: str, lines: list[str], annotate: dict[Path, str]) -> None:
    try:
        entries = sorted(
            (e for e in directory.iterdir() if e.name not in _IGNORED_DIRS),
            key=lambda e: (e.is_file(), e.name.lower()),
        )
    except OSError:
        return
    for index, entry in enumerate(entries):
        is_last = index == len(entries) - 1
        connector = "`-- " if is_last else "|-- "
        label = annotate.get(entry)
        suffix = f"  [{label}]" if label else ""
        lines.append(f"{prefix}{connector}{entry.name}{'/' if entry.is_dir() else ''}{suffix}")
        if entry.is_dir():
            extension = "    " if is_last else "|   "
            _walk(entry, prefix + extension, lines, annotate)


def render_pipeline_sequence() -> str:
    lines = ["scent-llm pipeline (sequence view)", "=" * 36]
    for source, target, message in _PIPELINE_STAGES:
        arrow_len = max(4, 20 - len(source) - len(target))
        lines.append(f"{source:>12} {'-' * arrow_len}> {target:<12} : {message}")
    return "\n".join(lines)
