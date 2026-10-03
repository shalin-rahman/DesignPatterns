from __future__ import annotations

from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest
import requests

from app.dataset_loader import (
    DatasetError,
    count_rows,
    iter_records,
    normalize_dataset_url,
    resolve_dataset,
)


def test_blob_url_becomes_resolve_url() -> None:
    url = "https://huggingface.co/datasets/allenai/WildChat/blob/main/data/train-00003-of-00006.parquet"
    assert normalize_dataset_url(url) == url.replace("/blob/", "/resolve/")


def test_non_hf_url_is_unchanged() -> None:
    url = "https://example.com/blob/file.parquet"
    assert normalize_dataset_url(url) == url


def test_local_path_resolves(parquet_file: Path, tmp_path: Path) -> None:
    assert resolve_dataset(str(parquet_file), tmp_path / "cache") == parquet_file


def test_missing_local_file_raises(tmp_path: Path) -> None:
    with pytest.raises(DatasetError):
        resolve_dataset(str(tmp_path / "nope.parquet"), tmp_path)


def test_cached_remote_file_is_not_downloaded_again(tmp_path: Path) -> None:
    cached = tmp_path / "train-00003-of-00006.parquet"
    cached.write_bytes(b"x")
    url = "https://huggingface.co/datasets/allenai/WildChat/blob/main/data/train-00003-of-00006.parquet"
    assert resolve_dataset(url, tmp_path) == cached


def test_hf_token_only_sent_to_huggingface(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    sent: list[dict] = []

    def fake_get(url: str, headers: dict, **kwargs: object) -> None:
        sent.append(headers)
        raise requests.ConnectionError()

    monkeypatch.setattr(requests, "get", fake_get)
    for url in ("https://evil.example/huggingface.co/x.parquet", "https://huggingface.co/x/resolve/main/y.parquet"):
        with pytest.raises(Exception):
            resolve_dataset(url, tmp_path, hf_token="hf_secret")
    assert sent[0] == {}
    assert sent[1] == {"Authorization": "Bearer hf_secret"}


def test_reads_only_needed_columns(parquet_file: Path) -> None:
    rows = list(iter_records(parquet_file))
    assert len(rows) == 3
    assert set(rows[0]) == {"conversation_id", "model", "timestamp", "conversation", "language"}
    assert count_rows(parquet_file) == 3


def test_max_records_limits_rows(parquet_file: Path) -> None:
    assert len(list(iter_records(parquet_file, batch_size=1, max_records=2))) == 2


def test_missing_required_column_raises(tmp_path: Path) -> None:
    path = tmp_path / "bad.parquet"
    pq.write_table(pa.table({"conversation_id": ["x"]}), path)
    with pytest.raises(DatasetError, match="missing columns"):
        list(iter_records(path))


def test_not_a_parquet_file_raises(tmp_path: Path) -> None:
    path = tmp_path / "bad.parquet"
    path.write_text("not parquet")
    with pytest.raises(DatasetError):
        list(iter_records(path))
