"""Orchestrates a full analysis run over a file or directory tree.

Ties together the AST analyzer and the smell detectors; the CLI and any
future TUI/API layer both call through here rather than duplicating the
walk-and-detect logic.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from scent_llm.analysis.ast_analyzer import find_llm_call_sites_in_file
from scent_llm.smells.base import Finding
from scent_llm.smells.detectors import run_all_detectors

_IGNORED_DIRS = {".git", "__pycache__", ".venv", "venv", "node_modules"}


@dataclass
class AnalysisReport:
    root: str
    files_scanned: int = 0
    call_sites_found: int = 0
    findings: list[Finding] = field(default_factory=list)

    def findings_for(self, file_path: str) -> list[Finding]:
        return [f for f in self.findings if f.file_path == file_path]

    @property
    def counts_by_smell(self) -> dict[str, int]:
        counts: dict[str, int] = {}
        for finding in self.findings:
            counts[finding.smell.value] = counts.get(finding.smell.value, 0) + 1
        return counts


def discover_python_files(target: Path) -> list[Path]:
    """List of .py files under `target` (or just `target` itself if it's a file)."""
    if target.is_file():
        return [target]
    return [
        p for p in target.rglob("*.py")
        if not any(part in _IGNORED_DIRS for part in p.parts)
    ]


def analyze_files(files: list[Path], root: Path) -> AnalysisReport:
    report = AnalysisReport(root=str(root.resolve()))
    for path in files:
        report.files_scanned += 1
        call_sites = find_llm_call_sites_in_file(path)
        report.call_sites_found += len(call_sites)
        for call_site in call_sites:
            report.findings.extend(run_all_detectors(call_site))
    return report


def analyze_path(target: Path) -> AnalysisReport:
    target = target.resolve()
    return analyze_files(discover_python_files(target), root=target)
