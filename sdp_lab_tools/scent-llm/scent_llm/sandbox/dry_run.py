"""Best-effort sandboxed dry-run execution across languages.

Isolation strategy, strongest to weakest, chosen automatically per host:
  1. Docker, if installed: `--network none`, `--rm`, read-only bind mount,
     memory/pid caps. Strong isolation, works the same on every OS.
  2. Bare subprocess in a throwaway temp directory with a trimmed
     environment and a hard wall-clock timeout. No real syscall sandbox —
     documented as such so callers don't over-trust it.

This is a developer convenience for "does this generated snippet even run
and what does it print", not a security boundary for untrusted, adversarial
code. Treat Docker mode as the safe path for anything you don't trust.
"""
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

DEFAULT_TIMEOUT_SECONDS = 10
DEFAULT_MEMORY_LIMIT = "256m"


@dataclass
class LanguageRuntime:
    filename: str
    docker_image: str
    build_cmd: list[str] | None  # None if the run_cmd handles compile+run itself
    run_cmd: list[str]


def _cmd(*parts: str) -> list[str]:
    return list(parts)


# {file} is substituted with the sandbox-relative script filename at run time.
_RUNTIMES: dict[str, LanguageRuntime] = {
    "python": LanguageRuntime("main.py", "python:3.12-slim", None, _cmd("python", "{file}")),
    "javascript": LanguageRuntime("main.js", "node:20-slim", None, _cmd("node", "{file}")),
    "typescript": LanguageRuntime(
        "main.ts", "node:20-slim", None, _cmd("npx", "--yes", "tsx", "{file}")
    ),
    "bash": LanguageRuntime("main.sh", "bash:5", None, _cmd("bash", "{file}")),
    "ruby": LanguageRuntime("main.rb", "ruby:3-slim", None, _cmd("ruby", "{file}")),
    "go": LanguageRuntime("main.go", "golang:1.22-alpine", None, _cmd("go", "run", "{file}")),
    "java": LanguageRuntime(
        "Main.java", "eclipse-temurin:21-jdk",
        _cmd("javac", "{file}"), _cmd("java", "-cp", ".", "Main"),
    ),
    "csharp": LanguageRuntime("Program.cs", "mcr.microsoft.com/dotnet/sdk:8.0", None, _cmd("dotnet", "run", "--project", ".")),
    "c": LanguageRuntime("main.c", "gcc:13", _cmd("gcc", "-O0", "-o", "main", "{file}"), _cmd("./main")),
    "cpp": LanguageRuntime("main.cpp", "gcc:13", _cmd("g++", "-O0", "-o", "main", "{file}"), _cmd("./main")),
}

_NATIVE_TOOL_FOR_LANGUAGE = {
    "python": "python",
    "javascript": "node",
    "typescript": "npx",
    "bash": "bash",
    "ruby": "ruby",
    "go": "go",
    "java": "javac",
    "csharp": "dotnet",
    "c": "gcc",
    "cpp": "g++",
}


@dataclass
class DryRunResult:
    language: str
    isolation: str  # "docker" | "subprocess"
    exit_code: int | None
    stdout: str
    stderr: str
    timed_out: bool
    error: str | None = None


def dry_run(
    code: str,
    language: str,
    timeout_seconds: int = DEFAULT_TIMEOUT_SECONDS,
    prefer_docker: bool = True,
) -> DryRunResult:
    language = language.lower()
    runtime = _RUNTIMES.get(language)
    if runtime is None:
        return DryRunResult(
            language=language, isolation="none", exit_code=None,
            stdout="", stderr="", timed_out=False,
            error=f"Unsupported language '{language}'. Supported: {', '.join(sorted(_RUNTIMES))}",
        )

    use_docker = prefer_docker and shutil.which("docker") is not None
    if use_docker:
        return _run_in_docker(code, language, runtime, timeout_seconds)
    return _run_in_subprocess(code, language, runtime, timeout_seconds)


def _run_in_docker(code: str, language: str, runtime: LanguageRuntime, timeout_seconds: int) -> DryRunResult:
    with tempfile.TemporaryDirectory(prefix="scent_llm_sandbox_") as tmp:
        script_path = Path(tmp) / runtime.filename
        script_path.write_text(code, encoding="utf-8")

        inner_cmds = []
        if runtime.build_cmd:
            inner_cmds.append(" ".join(p.format(file=runtime.filename) for p in runtime.build_cmd))
        inner_cmds.append(" ".join(p.format(file=runtime.filename) for p in runtime.run_cmd))
        shell_command = " && ".join(inner_cmds)

        docker_cmd = [
            "docker", "run", "--rm",
            "--network", "none",
            "--memory", DEFAULT_MEMORY_LIMIT,
            "--pids-limit", "64",
            "-v", f"{tmp}:/sandbox:ro" if not runtime.build_cmd else f"{tmp}:/sandbox",
            "-w", "/sandbox",
            runtime.docker_image,
            "sh", "-c", shell_command,
        ]
        try:
            proc = subprocess.run(
                docker_cmd, capture_output=True, text=True, timeout=timeout_seconds,
            )
            return DryRunResult(
                language=language, isolation="docker", exit_code=proc.returncode,
                stdout=proc.stdout, stderr=proc.stderr, timed_out=False,
            )
        except subprocess.TimeoutExpired as exc:
            return DryRunResult(
                language=language, isolation="docker", exit_code=None,
                stdout=exc.stdout or "", stderr=exc.stderr or "", timed_out=True,
            )
        except OSError as exc:
            return DryRunResult(
                language=language, isolation="docker", exit_code=None,
                stdout="", stderr="", timed_out=False, error=str(exc),
            )


def _run_in_subprocess(code: str, language: str, runtime: LanguageRuntime, timeout_seconds: int) -> DryRunResult:
    native_tool = _NATIVE_TOOL_FOR_LANGUAGE[language]
    if shutil.which(native_tool) is None:
        return DryRunResult(
            language=language, isolation="subprocess", exit_code=None,
            stdout="", stderr="", timed_out=False,
            error=(
                f"Neither Docker nor a local '{native_tool}' toolchain is available. "
                f"Install Docker, or install {native_tool} to run {language} locally."
            ),
        )

    with tempfile.TemporaryDirectory(prefix="scent_llm_sandbox_") as tmp:
        script_path = Path(tmp) / runtime.filename
        script_path.write_text(code, encoding="utf-8")

        minimal_env = {
            "PATH": os.environ.get("PATH", ""),
            "SYSTEMROOT": os.environ.get("SYSTEMROOT", ""),  # required for Windows subprocess resolution
        }

        try:
            if runtime.build_cmd:
                build = [p.format(file=runtime.filename) for p in runtime.build_cmd]
                build_proc = subprocess.run(
                    build, cwd=tmp, capture_output=True, text=True,
                    timeout=timeout_seconds, env=minimal_env,
                )
                if build_proc.returncode != 0:
                    return DryRunResult(
                        language=language, isolation="subprocess", exit_code=build_proc.returncode,
                        stdout=build_proc.stdout, stderr=build_proc.stderr, timed_out=False,
                        error="Build step failed.",
                    )

            run = [p.format(file=runtime.filename) for p in runtime.run_cmd]
            proc = subprocess.run(
                run, cwd=tmp, capture_output=True, text=True,
                timeout=timeout_seconds, env=minimal_env,
            )
            return DryRunResult(
                language=language, isolation="subprocess", exit_code=proc.returncode,
                stdout=proc.stdout, stderr=proc.stderr, timed_out=False,
            )
        except subprocess.TimeoutExpired as exc:
            return DryRunResult(
                language=language, isolation="subprocess", exit_code=None,
                stdout=exc.stdout or "", stderr=exc.stderr or "", timed_out=True,
            )
        except OSError as exc:
            return DryRunResult(
                language=language, isolation="subprocess", exit_code=None,
                stdout="", stderr="", timed_out=False, error=str(exc),
            )
