import unittest
from types import SimpleNamespace
from unittest.mock import patch

from scent_llm.config import LLMConfig
from scent_llm.llm_client import LLMClient, LLMClientError


class TestGeminiClient(unittest.TestCase):
    def test_missing_gemini_key_has_actionable_error(self):
        config = LLMConfig(provider="gemini", model="gemini-3-flash-preview")

        with self.assertRaisesRegex(LLMClientError, "GEMINI_API_KEY"):
            LLMClient(config).generate_code("write a function")

    @patch("google.genai.Client")
    def test_gemini_request_uses_configured_generation_settings(self, client_class):
        response = SimpleNamespace(
            text="```python\nprint('ok')\n```",
            model_dump=lambda exclude_none=True: {"text": "print('ok')"},
        )
        client_class.return_value.models.generate_content.return_value = response
        config = LLMConfig(
            provider="gemini",
            model="gemini-3-flash-preview",
            gemini_api_key="test-key",
            temperature=0.1,
            max_tokens=128,
            timeout_seconds=7,
        )

        result = LLMClient(config).generate_code("write a function")

        self.assertEqual(result.code, "print('ok')")
        self.assertEqual(result.provider, "gemini")
        client_class.assert_called_once()
        request = client_class.return_value.models.generate_content.call_args
        self.assertEqual(request.kwargs["model"], "gemini-3-flash-preview")
        self.assertEqual(request.kwargs["contents"], "write a function")
        generation_config = request.kwargs["config"]
        self.assertEqual(generation_config.temperature, 0.1)
        self.assertEqual(generation_config.max_output_tokens, 128)
        self.assertIn("Write only python code", generation_config.system_instruction)


if __name__ == "__main__":
    unittest.main()
