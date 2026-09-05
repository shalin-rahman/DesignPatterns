"""Example file with every LLM code smell present, for smoke-testing the detectors."""
from openai import OpenAI

client = OpenAI()


def ask_bad(question: str) -> str:
    response = client.chat.completions.create(
        model="gpt-4",  # NMVP: floating alias, not a pinned snapshot
        messages=[
            {"role": "user", "content": question},  # NSM: no system message
        ],
        # TNES: no temperature
        # UMM: no max_tokens
        # NSO: no response_format
    )
    return response.choices[0].message.content


def ask_good(question: str) -> str:
    response = client.chat.completions.create(
        model="gpt-4o-2024-08-06",
        messages=[
            {"role": "system", "content": "You are a precise assistant. Reply with JSON only."},
            {"role": "user", "content": question},
        ],
        temperature=0.0,
        max_tokens=256,
        response_format={"type": "json_object"},
    )
    return response.choices[0].message.content
