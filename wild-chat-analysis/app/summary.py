"""Turns the per-smell output records into the numbers used in the report."""

from __future__ import annotations

from collections import Counter
from itertools import combinations
from typing import Any

from app.prompt_analyzer import SMELL_CATEGORIES

LENGTH_BUCKETS: tuple[tuple[str, int, int | None], ...] = (
    ("< 50 chars", 0, 50),
    ("50-199", 50, 200),
    ("200-999", 200, 1000),
    (">= 1000", 1000, None),
)


def group_prompts(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Collapse one-record-per-smell rows back into one entry per prompt."""
    prompts: dict[tuple[str, str], dict[str, Any]] = {}
    for record in records:
        key = (record["conversation_id"], record["content"])
        entry = prompts.setdefault(
            key,
            {"conversation_id": record["conversation_id"], "model": record.get("model"),
             "content": record["content"], "smells": []},
        )
        if record.get("smell_type") and record["smell_type"] not in entry["smells"]:
            entry["smells"].append(record["smell_type"])
    return list(prompts.values())


def _bucket(length: int) -> str:
    """Return the name of the length bucket a prompt of `length` characters falls in."""
    for name, low, high in LENGTH_BUCKETS:
        if length >= low and (high is None or length < high):
            return name
    raise ValueError(length)


def _rate(group: list[dict[str, Any]]) -> dict[str, Any]:
    """For a group of prompts: how many there are, the % with any smell, and mean smells per prompt."""
    n = len(group)
    smelly = sum(1 for p in group if p["smells"])
    return {
        "prompts": n,
        "with_smell_pct": round(100 * smelly / n, 1) if n else 0.0,
        "mean_smells": round(sum(len(p["smells"]) for p in group) / n, 2) if n else 0.0,
    }


def summarize(records: list[dict[str, Any]]) -> dict[str, Any]:
    """Turn output records into the numbers used in the paper.

    Gives overall smell rates, smells per prompt, how often each smell appears, rates by
    ChatGPT model and by prompt length, and the five most common smell pairs.
    """
    prompts = group_prompts(records)
    n = len(prompts)
    counts = Counter(s for p in prompts for s in p["smells"])
    per_prompt = Counter(min(len(p["smells"]), 3) for p in prompts)
    pairs = Counter(pair for p in prompts for pair in combinations(sorted(p["smells"]), 2))
    lengths = sorted(len(p["content"]) for p in prompts)

    by_model: dict[str, list[dict[str, Any]]] = {}
    by_length: dict[str, list[dict[str, Any]]] = {name: [] for name, _, _ in LENGTH_BUCKETS}
    for p in prompts:
        by_model.setdefault(p["model"] or "unknown", []).append(p)
        by_length[_bucket(len(p["content"]))].append(p)

    return {
        "prompts": n,
        "conversations": len({p["conversation_id"] for p in prompts}),
        "median_length_chars": lengths[len(lengths) // 2] if lengths else 0,
        **{k: v for k, v in _rate(prompts).items() if k != "prompts"},
        "smells_per_prompt": {("3+" if k == 3 else str(k)): per_prompt.get(k, 0) for k in range(4)},
        "smell_frequency": [
            {"smell": s, "prompts": c, "pct": round(100 * c / n, 1), "listed": s in SMELL_CATEGORIES}
            for s, c in counts.most_common()
        ],
        "by_model": {m: _rate(g) for m, g in sorted(by_model.items())},
        "by_length": {name: _rate(g) for name, g in by_length.items()},
        "top_pairs": [{"pair": list(pair), "prompts": c} for pair, c in pairs.most_common(5)],
    }
