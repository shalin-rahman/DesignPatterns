"""Minimal step-progress reporting for the CLI.

Every long-running command prints numbered "==> doing X..." / "done (Ns)"
lines to stderr, so stdout stays clean for piping generated code or JSON.
No extra dependency (no rich/tqdm) - this is deliberately just text.
"""
from __future__ import annotations

import sys
import time
from contextlib import contextmanager
from typing import IO


class StepReporter:
    """Tracks step count for one command invocation and prints progress."""

    def __init__(self, total: int, stream: IO[str] = sys.stderr, enabled: bool = True):
        self.total = total
        self.stream = stream
        self.enabled = enabled
        self._current = 0

    @contextmanager
    def step(self, description: str):
        self._current += 1
        if not self.enabled:
            yield
            return
        prefix = f"[{self._current}/{self.total}]"
        self.stream.write(f"{prefix} {description}...\n")
        self.stream.flush()
        start = time.monotonic()
        try:
            yield
        except Exception:
            self.stream.write(f"{prefix} failed\n")
            self.stream.flush()
            raise
        else:
            elapsed = time.monotonic() - start
            self.stream.write(f"{prefix} done ({elapsed:.1f}s)\n")
            self.stream.flush()
