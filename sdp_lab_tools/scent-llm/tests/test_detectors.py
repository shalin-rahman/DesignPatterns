import unittest

from scent_llm.analysis.ast_analyzer import find_llm_call_sites
from scent_llm.smells.base import SmellKind
from scent_llm.smells.detectors import run_all_detectors

SMELLY = """
from openai import OpenAI
client = OpenAI()
client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "hi"}],
)
"""

CLEAN = """
from openai import OpenAI
client = OpenAI()
client.chat.completions.create(
    model="gpt-4o-2024-08-06",
    messages=[
        {"role": "system", "content": "Be precise."},
        {"role": "user", "content": "hi"},
    ],
    temperature=0.0,
    max_tokens=128,
    response_format={"type": "json_object"},
)
"""

OLLAMA_TAGGED = """
import ollama
ollama.chat(
    model="llama3.1:8b",
    messages=[
        {"role": "system", "content": "Be terse."},
        {"role": "user", "content": "hi"},
    ],
    options={"temperature": 0.1, "num_predict": 200},
)
"""


class TestDetectors(unittest.TestCase):
    def test_smelly_call_triggers_all_five_smells(self):
        call_sites = find_llm_call_sites(SMELLY, "smelly.py")
        self.assertEqual(len(call_sites), 1)
        findings = run_all_detectors(call_sites[0])
        found_kinds = {f.smell for f in findings}
        self.assertEqual(
            found_kinds,
            {SmellKind.NSO, SmellKind.UMM, SmellKind.TNES, SmellKind.NMVP, SmellKind.NSM},
        )

    def test_clean_call_triggers_nothing(self):
        call_sites = find_llm_call_sites(CLEAN, "clean.py")
        self.assertEqual(len(call_sites), 1)
        findings = run_all_detectors(call_sites[0])
        self.assertEqual(findings, [])

    def test_ollama_tagged_model_and_system_message_are_not_flagged(self):
        call_sites = find_llm_call_sites(OLLAMA_TAGGED, "ollama_call.py")
        self.assertEqual(len(call_sites), 1)
        findings = run_all_detectors(call_sites[0])
        found_kinds = {f.smell for f in findings}
        self.assertNotIn(SmellKind.NMVP, found_kinds)
        self.assertNotIn(SmellKind.NSM, found_kinds)

    def test_non_llm_call_is_ignored(self):
        source = "requests.get('https://example.com')\n"
        self.assertEqual(find_llm_call_sites(source, "irrelevant.py"), [])

    def test_syntax_error_returns_empty_list(self):
        self.assertEqual(find_llm_call_sites("def broken(:\n", "broken.py"), [])


if __name__ == "__main__":
    unittest.main()
