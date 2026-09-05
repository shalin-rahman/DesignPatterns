"""scent-llm command-line interface.

Subcommands:
  generate  - ask a configured LLM to write code for a described task
  analyze   - statically scan a file/directory for LLM code smells
  sandbox   - dry-run a code file in an isolated environment
  diagram   - render the project tree or the tool's own pipeline sequence
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from scent_llm.config import LLMConfig
from scent_llm.llm_client import LLMClient, LLMClientError
from scent_llm.progress import StepReporter
from scent_llm.runner import analyze_files, discover_python_files
from scent_llm.sandbox.dry_run import dry_run
from scent_llm.smells.refactor import render_finding_report
from scent_llm.visualization.architecture import render_pipeline_sequence, render_project_tree


def build_parser() -> argparse.ArgumentParser:
    quiet_parent = argparse.ArgumentParser(add_help=False)
    quiet_parent.add_argument(
        "-q", "--quiet", action="store_true",
        help="Suppress '[n/total] step...' progress lines (still prints results/errors)",
    )

    parser = argparse.ArgumentParser(prog="scent-llm", description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    gen = subparsers.add_parser("generate", help="Generate code with a configured LLM", parents=[quiet_parent])
    gen.add_argument("task", help="Natural-language description of the code to generate")
    gen.add_argument("--language", default="python")
    gen.add_argument("--provider", choices=["ollama", "groq"], default=None)
    gen.add_argument("--model", default=None)
    gen.add_argument("--temperature", type=float, default=None)
    gen.add_argument("--max-tokens", type=int, default=None)
    gen.add_argument("--out", type=Path, default=None, help="Write generated code to this file")
    gen.add_argument("--analyze", action="store_true", help="Run smell analysis on the generated code (Python only)")

    ana = subparsers.add_parser(
        "analyze", help="Detect LLM code smells in a file or directory", parents=[quiet_parent]
    )
    ana.add_argument("path", type=Path)
    ana.add_argument("--json", action="store_true", dest="as_json")

    box = subparsers.add_parser(
        "sandbox", help="Dry-run a source file in an isolated environment", parents=[quiet_parent]
    )
    box.add_argument("path", type=Path)
    box.add_argument("--language", required=True)
    box.add_argument("--timeout", type=int, default=10)
    box.add_argument("--no-docker", action="store_true", help="Skip Docker even if available")

    dia = subparsers.add_parser(
        "diagram", help="Render a textual architecture/sequence view", parents=[quiet_parent]
    )
    dia.add_argument("kind", choices=["tree", "sequence"])
    dia.add_argument("--path", type=Path, default=Path.cwd(), help="Root directory for 'tree'")

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        if args.command == "generate":
            return _run_generate(args)
        if args.command == "analyze":
            return _run_analyze(args)
        if args.command == "sandbox":
            return _run_sandbox(args)
        if args.command == "diagram":
            return _run_diagram(args)
    except LLMClientError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    parser.print_help()
    return 1


def _run_generate(args: argparse.Namespace) -> int:
    total_steps = 3 + (1 if args.analyze else 0)
    progress = StepReporter(total=total_steps, enabled=not args.quiet)

    with progress.step("Loading configuration"):
        overrides = {
            "provider": args.provider,
            "model": args.model,
            "temperature": args.temperature,
            "max_tokens": args.max_tokens,
        }
        config = LLMConfig.load(overrides=overrides)
        client = LLMClient(config)

    with progress.step(f"Requesting {args.language} code from {config.provider}:{config.model}"):
        result = client.generate_code(args.task, language=args.language)

    with progress.step("Writing output"):
        if args.out:
            args.out.write_text(result.code, encoding="utf-8")
        else:
            print(result.code)

    if args.out:
        print(f"Wrote generated code to {args.out}")

    if args.analyze:
        with progress.step("Analyzing generated code for LLM smells"):
            if args.language.lower() != "python":
                print("note: --analyze only supports Python source; skipping.", file=sys.stderr)
            else:
                _print_findings_for_source(result.code, source_label="<generated>")

    return 0


def _run_analyze(args: argparse.Namespace) -> int:
    if not args.path.exists():
        print(f"error: path not found: {args.path}", file=sys.stderr)
        return 1

    progress = StepReporter(total=2, enabled=not args.quiet)
    with progress.step(f"Discovering Python source under {args.path}"):
        files = discover_python_files(args.path)
    with progress.step(f"Parsing {len(files)} file(s) and running smell detectors"):
        report = analyze_files(files, root=args.path)

    if args.as_json:
        payload = {
            "root": report.root,
            "files_scanned": report.files_scanned,
            "call_sites_found": report.call_sites_found,
            "counts_by_smell": report.counts_by_smell,
            "findings": [
                {
                    "smell": f.smell.value,
                    "title": f.title,
                    "file": f.file_path,
                    "line": f.line,
                    "col": f.col,
                    "severity": f.severity,
                    "detail": f.detail,
                }
                for f in report.findings
            ],
        }
        print(json.dumps(payload, indent=2))
        return 0

    print(f"Scanned {report.files_scanned} file(s), {report.call_sites_found} LLM call site(s) found.")
    if not report.findings:
        print("No LLM code smells detected.")
        return 0

    for finding in report.findings:
        print()
        print(render_finding_report(finding))
    print()
    print(f"Total: {len(report.findings)} finding(s) - {report.counts_by_smell}")
    return 0


def _print_findings_for_source(source: str, source_label: str) -> None:
    from scent_llm.analysis.ast_analyzer import find_llm_call_sites
    from scent_llm.smells.detectors import run_all_detectors

    call_sites = find_llm_call_sites(source, source_label)
    findings = [f for call in call_sites for f in run_all_detectors(call)]
    if not findings:
        print("No LLM code smells detected in generated code.", file=sys.stderr)
        return
    for finding in findings:
        print(render_finding_report(finding), file=sys.stderr)


def _run_sandbox(args: argparse.Namespace) -> int:
    if not args.path.exists():
        print(f"error: path not found: {args.path}", file=sys.stderr)
        return 1

    progress = StepReporter(total=2, enabled=not args.quiet)
    with progress.step(f"Reading {args.path}"):
        code = args.path.read_text(encoding="utf-8")
    with progress.step(f"Running {args.language} in sandbox (timeout {args.timeout}s)"):
        result = dry_run(code, args.language, timeout_seconds=args.timeout, prefer_docker=not args.no_docker)

    if result.error:
        print(f"error: {result.error}", file=sys.stderr)
        return 1

    print(f"[{result.isolation}] exit={result.exit_code} timed_out={result.timed_out}")
    if result.stdout:
        print("--- stdout ---")
        print(result.stdout)
    if result.stderr:
        print("--- stderr ---", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
    return 0 if result.exit_code == 0 else 1


def _run_diagram(args: argparse.Namespace) -> int:
    if args.kind == "sequence":
        print(render_pipeline_sequence())
        return 0

    if not args.path.exists():
        print(f"error: path not found: {args.path}", file=sys.stderr)
        return 1
    print(render_project_tree(args.path))
    return 0


if __name__ == "__main__":
    sys.exit(main())
