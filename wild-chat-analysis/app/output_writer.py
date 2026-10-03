"""Checkpoint file (JSON Lines, one line per finished prompt) and the final JSON output."""

from __future__ import annotations

import json
import logging
import os
from collections.abc import Iterator
from pathlib import Path
from types import TracebackType
from typing import IO, Any

from app.models import OutputRecord

logger = logging.getLogger(__name__)


class CheckpointStore:
    """Each line is {"prompt_id": ..., "records": [...]}. A prompt is only written after
    it was analyzed successfully, so failed prompts are retried on the next run."""

    def __init__(self, path: Path) -> None:
        self.path = path
        self._handle: IO[str] | None = None

    def reset(self) -> None:
        self.path.unlink(missing_ok=True)

    def _iter_lines(self) -> Iterator[dict[str, Any]]:
        if not self.path.exists():
            return
        with open(self.path, encoding="utf-8") as handle:
            for line_number, line in enumerate(handle, start=1):
                line = line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    # A hard stop can leave a half-written last line.
                    logger.warning("Skipping broken checkpoint line %d", line_number)
                    continue
                if isinstance(entry, dict) and isinstance(entry.get("prompt_id"), str):
                    yield entry

    def load_processed_ids(self) -> set[str]:
        return {entry["prompt_id"] for entry in self._iter_lines()}

    def iter_records(self) -> Iterator[dict[str, Any]]:
        seen: set[str] = set()
        for entry in self._iter_lines():
            if entry["prompt_id"] in seen:
                continue
            seen.add(entry["prompt_id"])
            for record in entry.get("records") or []:
                if isinstance(record, dict):
                    yield record

    def open(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        needs_newline = False
        if self.path.exists() and self.path.stat().st_size > 0:
            with open(self.path, "rb") as handle:
                handle.seek(-1, os.SEEK_END)
                needs_newline = handle.read(1) != b"\n"
        self._handle = open(self.path, "a", encoding="utf-8")
        if needs_newline:
            self._handle.write("\n")

    def append(self, prompt_id: str, records: list[OutputRecord]) -> None:
        if self._handle is None:
            raise RuntimeError("CheckpointStore is not open")
        entry = {"prompt_id": prompt_id, "records": [r.model_dump() for r in records]}
        self._handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
        self._handle.flush()

    def close(self) -> None:
        if self._handle is not None:
            self._handle.close()
            self._handle = None

    def __enter__(self) -> CheckpointStore:
        self.open()
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> None:
        self.close()


class FailureLog:
    """JSON Lines list of prompts that failed in the current run. No prompt text is stored."""

    def __init__(self, path: Path) -> None:
        self.path = path
        self.count = 0

    def reset(self) -> None:
        self.path.unlink(missing_ok=True)
        self.count = 0

    def append(self, prompt_id: str, conversation_id: str, error: str) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        entry = {"prompt_id": prompt_id, "conversation_id": conversation_id, "error": error}
        with open(self.path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
        self.count += 1


def write_final_json(store: CheckpointStore, output_path: Path) -> int:
    """Stream every checkpoint record into a UTF-8 JSON array. Returns the record count."""
    output_path.parent.mkdir(parents=True, exist_ok=True)
    temp_path = output_path.with_name(output_path.name + ".tmp")
    count = 0
    with open(temp_path, "w", encoding="utf-8") as handle:
        handle.write("[")
        for record in store.iter_records():
            handle.write(",\n  " if count else "\n  ")
            handle.write(json.dumps(record, ensure_ascii=False))
            count += 1
        handle.write("\n]\n" if count else "]\n")
    os.replace(temp_path, output_path)
    return count
