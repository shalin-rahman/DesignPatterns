"""AST-based discovery of LLM API call sites in Python source.

This module answers exactly one question: "where in this file does code
call out to an LLM, and what arguments did it pass?" It knows nothing about
which of those arguments constitute a code smell — that judgment lives in
`scent_llm.smells.detectors`.
"""
from __future__ import annotations

import ast
import re
from dataclasses import dataclass, field
from pathlib import Path

# Dotted call suffixes that identify a chat/completion-style LLM call,
# regardless of which SDK or client-variable name the caller used.
_CALL_SUFFIX_PATTERNS = [
    re.compile(r"\.chat\.completions\.create$"),
    re.compile(r"\.completions\.create$"),
    re.compile(r"\.responses\.create$"),
    re.compile(r"\.messages\.create$"),
    re.compile(r"^ollama\.chat$"),
    re.compile(r"^ollama\.generate$"),
    re.compile(r"\.generate_content$"),
]


@dataclass
class LLMCallSite:
    file_path: str
    line: int
    col: int
    call_expr: str
    source_snippet: str
    kwargs: dict[str, ast.expr] = field(default_factory=dict)
    literal_kwargs: dict[str, object] = field(default_factory=dict)

    def get_literal(self, *names: str, default=None):
        """First literal keyword value found among `names`, else `default`."""
        for name in names:
            if name in self.literal_kwargs:
                return self.literal_kwargs[name]
        return default

    def has_any(self, *names: str) -> bool:
        return any(name in self.kwargs for name in names)


class _CallSiteVisitor(ast.NodeVisitor):
    def __init__(self, source: str, file_path: str):
        self._source = source
        self._file_path = file_path
        self.call_sites: list[LLMCallSite] = []

    def visit_Call(self, node: ast.Call) -> None:  # noqa: N802 (ast API name)
        call_expr = _unparse_safe(node.func)
        if _looks_like_llm_call(call_expr):
            self.call_sites.append(self._build_call_site(node, call_expr))
        self.generic_visit(node)

    def _build_call_site(self, node: ast.Call, call_expr: str) -> LLMCallSite:
        kwargs: dict[str, ast.expr] = {kw.arg: kw.value for kw in node.keywords if kw.arg}
        literal_kwargs: dict[str, object] = {}
        for name, value_node in kwargs.items():
            try:
                literal_kwargs[name] = ast.literal_eval(value_node)
            except (ValueError, TypeError, SyntaxError):
                pass  # non-literal argument (a variable, f-string, etc.) - skip silently
        snippet = ast.get_source_segment(self._source, node) or call_expr
        return LLMCallSite(
            file_path=self._file_path,
            line=node.lineno,
            col=node.col_offset,
            call_expr=call_expr,
            source_snippet=snippet,
            kwargs=kwargs,
            literal_kwargs=literal_kwargs,
        )


def _looks_like_llm_call(call_expr: str) -> bool:
    return any(pattern.search(call_expr) for pattern in _CALL_SUFFIX_PATTERNS)


def _unparse_safe(node: ast.AST) -> str:
    try:
        return ast.unparse(node)
    except Exception:  # pragma: no cover - defensive; ast.unparse is stable on 3.10+
        return ""


def find_llm_call_sites(source: str, file_path: str | Path) -> list[LLMCallSite]:
    """Parse `source` and return every detected LLM call site.

    Returns an empty list (rather than raising) on a syntax error, since a
    file that doesn't parse simply has nothing to report.
    """
    try:
        tree = ast.parse(source, filename=str(file_path))
    except SyntaxError:
        return []
    visitor = _CallSiteVisitor(source, str(file_path))
    visitor.visit(tree)
    return visitor.call_sites


def find_llm_call_sites_in_file(path: Path) -> list[LLMCallSite]:
    try:
        source = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return []
    return find_llm_call_sites(source, path)
