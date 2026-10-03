"""Download the WildChat parquet file and read the needed columns in batches."""

from __future__ import annotations

import logging
import os
from collections.abc import Iterator
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

import pyarrow.parquet as pq
import requests
from tqdm import tqdm

logger = logging.getLogger(__name__)

REQUIRED_COLUMNS: tuple[str, ...] = ("conversation_id", "model", "timestamp", "conversation")
# Read only if present. Used as a fallback when a message has no language of its own.
OPTIONAL_COLUMNS: tuple[str, ...] = ("language",)

_CHUNK_SIZE = 1024 * 1024


class DatasetError(Exception):
    """Raised when the dataset cannot be found, downloaded, or read."""


def is_remote(source: str) -> bool:
    """True when the dataset source is an http(s) URL rather than a local path."""
    return source.startswith(("http://", "https://"))


def normalize_dataset_url(url: str) -> str:
    """Turn a Hugging Face web page link (/blob/) into a direct download link (/resolve/)."""
    if "huggingface.co" in urlparse(url).netloc:
        return url.replace("/blob/", "/resolve/", 1)
    return url


def resolve_dataset(
    source: str,
    cache_dir: Path,
    hf_token: str | None = None,
    timeout: float = 60.0,
) -> Path:
    """Return a local path to the parquet file, downloading it once if `source` is a URL."""
    if not is_remote(source):
        path = Path(source)
        if not path.exists():
            raise DatasetError(f"Dataset file not found: {path}")
        return path

    url = normalize_dataset_url(source)
    file_name = Path(urlparse(url).path).name or "dataset.parquet"
    target = cache_dir / file_name
    if target.exists() and target.stat().st_size > 0:
        logger.info("Using cached dataset: %s", target)
        return target

    cache_dir.mkdir(parents=True, exist_ok=True)
    partial = target.with_name(target.name + ".part")
    host = (urlparse(url).hostname or "").lower()
    send_token = bool(hf_token) and (host == "huggingface.co" or host.endswith(".huggingface.co"))
    headers = {"Authorization": f"Bearer {hf_token}"} if send_token else {}
    logger.info("Downloading dataset from %s", url)
    try:
        with requests.get(url, headers=headers, stream=True, timeout=timeout) as response:
            if response.status_code in (401, 403):
                raise DatasetError(
                    f"Access denied ({response.status_code}). Accept the dataset terms on "
                    "Hugging Face and set HF_TOKEN."
                )
            response.raise_for_status()
            total = int(response.headers.get("Content-Length", 0)) or None
            with open(partial, "wb") as handle, tqdm(
                total=total, unit="B", unit_scale=True, desc="download"
            ) as bar:
                for chunk in response.iter_content(chunk_size=_CHUNK_SIZE):
                    handle.write(chunk)
                    bar.update(len(chunk))
    except requests.RequestException as exc:
        partial.unlink(missing_ok=True)
        raise DatasetError(f"Dataset download failed: {exc}") from exc
    os.replace(partial, target)
    return target


def count_rows(path: Path) -> int:
    """Return the parquet row count from the file footer, without reading the data."""
    return pq.ParquetFile(path).metadata.num_rows


def iter_records(
    path: Path,
    batch_size: int = 1000,
    max_records: int | None = None,
) -> Iterator[dict[str, Any]]:
    """Yield rows as dicts holding only the needed columns."""
    try:
        parquet = pq.ParquetFile(path)
    except Exception as exc:  # pyarrow raises several error types for bad files
        raise DatasetError(f"Cannot read parquet file {path}: {exc}") from exc

    names = set(parquet.schema_arrow.names)
    missing = [column for column in REQUIRED_COLUMNS if column not in names]
    if missing:
        raise DatasetError(f"Dataset is missing columns: {', '.join(missing)}")
    columns = [*REQUIRED_COLUMNS, *(c for c in OPTIONAL_COLUMNS if c in names)]

    count = 0
    for batch in parquet.iter_batches(batch_size=batch_size, columns=columns):
        for row in batch.to_pylist():
            yield row
            count += 1
            if max_records is not None and count >= max_records:
                return
