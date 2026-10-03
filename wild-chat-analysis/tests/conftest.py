from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import pyarrow as pa
import pyarrow.parquet as pq
import pytest


def message(role: str, content: str, language: str = "English") -> dict[str, Any]:
    return {"content": content, "language": language, "redacted": False, "role": role, "toxic": False}


SAMPLE_ROWS: list[dict[str, Any]] = [
    {
        "conversation_id": "c1",
        "model": "gpt-4",
        "timestamp": datetime(2023, 5, 1, 10, 0, tzinfo=timezone.utc),
        "conversation": [
            message("user", "Help me fix this."),
            message("assistant", "Sure, what is broken?"),
            message("user", "The login page, café ünïcode"),
        ],
        "language": "English",
        "turn": 2,
    },
    {
        "conversation_id": "c2",
        "model": "gpt-3.5-turbo",
        "timestamp": datetime(2023, 5, 2, 11, 30, tzinfo=timezone.utc),
        "conversation": [message("user", "Bonjour, comment ça va ?", "French")],
        "language": "French",
        "turn": 1,
    },
    {
        "conversation_id": "c3",
        "model": "gpt-4",
        "timestamp": datetime(2023, 5, 3, 9, 15, tzinfo=timezone.utc),
        "conversation": [message("assistant", "Only an assistant message")],
        "language": "English",
        "turn": 1,
    },
]


@pytest.fixture
def parquet_file(tmp_path: Path) -> Path:
    path = tmp_path / "sample.parquet"
    pq.write_table(pa.Table.from_pylist(SAMPLE_ROWS), path)
    return path
