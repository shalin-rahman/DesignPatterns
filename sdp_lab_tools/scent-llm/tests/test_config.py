import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scent_llm.config import (
    DEFAULT_GROQ_MODEL,
    DEFAULT_GEMINI_MODEL,
    DEFAULT_OLLAMA_MODEL,
    LLMConfig,
)


class TestLLMConfig(unittest.TestCase):
    def test_builtin_defaults_use_local_ollama(self):
        with patch.dict(os.environ, {}, clear=True):
            config = LLMConfig.load(config_path=Path("missing-scent-llm.toml"))

        self.assertEqual(config.provider, "ollama")
        self.assertEqual(config.model, DEFAULT_OLLAMA_MODEL)
        self.assertEqual(config.temperature, 0.2)
        self.assertEqual(config.max_tokens, 2048)

    def test_cli_overrides_environment_and_toml(self):
        with tempfile.TemporaryDirectory() as directory:
            config_path = Path(directory) / "scent_llm.toml"
            config_path.write_text(
                '[llm]\nprovider = "ollama"\nmodel = "file-model"\ntemperature = 0.4\n',
                encoding="utf-8",
            )
            with patch.dict(
                os.environ,
                {
                    "SCENT_LLM_PROVIDER": "groq",
                    "SCENT_LLM_MODEL": "env-model",
                    "SCENT_LLM_TEMPERATURE": "0.6",
                },
                clear=True,
            ):
                config = LLMConfig.load(
                    config_path=config_path,
                    overrides={
                        "provider": "ollama",
                        "model": "cli-model",
                        "temperature": 0.1,
                    },
                )

        self.assertEqual(config.provider, "ollama")
        self.assertEqual(config.model, "cli-model")
        self.assertEqual(config.temperature, 0.1)

    def test_switching_provider_selects_provider_default_model(self):
        with patch.dict(os.environ, {}, clear=True):
            config = LLMConfig.load(overrides={"provider": "groq"})

        self.assertEqual(config.provider, "groq")
        self.assertEqual(config.model, DEFAULT_GROQ_MODEL)

    def test_switching_to_gemini_selects_provider_default_model(self):
        with patch.dict(os.environ, {}, clear=True):
            config = LLMConfig.load(overrides={"provider": "gemini"})

        self.assertEqual(config.provider, "gemini")
        self.assertEqual(config.model, DEFAULT_GEMINI_MODEL)

    def test_invalid_numeric_environment_values_keep_defaults(self):
        with patch.dict(
            os.environ,
            {"SCENT_LLM_TEMPERATURE": "not-a-number", "SCENT_LLM_MAX_TOKENS": "invalid"},
            clear=True,
        ):
            config = LLMConfig.load(config_path=Path("missing-scent-llm.toml"))

        self.assertEqual(config.temperature, 0.2)
        self.assertEqual(config.max_tokens, 2048)


if __name__ == "__main__":
    unittest.main()
