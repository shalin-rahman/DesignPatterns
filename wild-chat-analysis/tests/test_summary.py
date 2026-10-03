from __future__ import annotations

from app.summary import group_prompts, summarize


def row(cid: str, content: str, smell: str | None, model: str = "gpt-4") -> dict:
    return {"conversation_id": cid, "model": model, "timestamp": None, "content": content,
            "smell_type": smell, "smell_reason": "r" if smell else None}


RECORDS = [
    row("c1", "fix this", "Vague / Missing Context"),
    row("c1", "fix this", "Ambiguous References"),
    row("c1", "x" * 300, None),
    row("c2", "Write a poem about the sea", None, model="gpt-3.5-turbo"),
    row("c3", "do it all", "Unsafe Request"),
]


def test_records_are_grouped_per_prompt() -> None:
    prompts = group_prompts(RECORDS)
    assert len(prompts) == 4
    assert prompts[0]["smells"] == ["Vague / Missing Context", "Ambiguous References"]


def test_summary_numbers() -> None:
    s = summarize(RECORDS)
    assert s["prompts"] == 4 and s["conversations"] == 3
    assert s["with_smell_pct"] == 50.0
    assert s["mean_smells"] == 0.75
    assert s["smells_per_prompt"] == {"0": 2, "1": 1, "2": 1, "3+": 0}
    assert {"smell": "Unsafe Request", "prompts": 1, "pct": 25.0, "listed": False} in s["smell_frequency"]
    assert s["by_model"]["gpt-3.5-turbo"] == {"prompts": 1, "with_smell_pct": 0.0, "mean_smells": 0.0}
    assert s["by_length"]["200-999"]["prompts"] == 1
    assert s["top_pairs"] == [{"pair": ["Ambiguous References", "Vague / Missing Context"], "prompts": 1}]


def test_empty_input() -> None:
    s = summarize([])
    assert s["prompts"] == 0 and s["with_smell_pct"] == 0.0 and s["smell_frequency"] == []
