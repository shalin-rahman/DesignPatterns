"""Configuration for scent-llm.

Resolution order for every setting: CLI flag > environment variable
(including one loaded from a .env file in the cwd) > config file
(scent_llm.toml in cwd) > built-in default. No API key ever has a
hardcoded value in this codebase.
"""
from __future__ import annotations

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from dotenv import load_dotenv

try:
    import tomllib  # Python 3.11+
except ModuleNotFoundError:  # pragma: no cover - 3.10 fallback
    import tomli as tomllib  # type: ignore[no-redef]

DEFAULT_CONFIG_FILENAME = "scent_llm.toml"
DEFAULT_ENV_FILENAME = ".env"

DEFAULT_OLLAMA_HOST = "http://localhost:11434"
DEFAULT_OLLAMA_MODEL = "llama3.1:8b"
DEFAULT_GROQ_MODEL = "llama-3.3-70b-versatile"
DEFAULT_GROQ_BASE_URL = "https://api.groq.com/openai/v1"
DEFAULT_GEMINI_MODEL = "gemini-3-flash-preview"
DEFAULT_OPENROUTER_MODEL = "openai/gpt-oss-120b"
DEFAULT_OPENROUTER_BASE_URL = "https://openrouter.ai/api/v1"

DEFAULT_TEMPERATURE = 0.2
DEFAULT_MAX_TOKENS = 2048
DEFAULT_TIMEOUT_SECONDS = 60


@dataclass
class LLMConfig:
    """Everything needed to reach a configured LLM provider.

    `provider` picks the backend; every other field has a safe default so
    the tool works out of the box against a local Ollama install with no
    API key at all.
    """

    provider: str = "ollama"
    model: str = DEFAULT_OLLAMA_MODEL
    ollama_host: str = DEFAULT_OLLAMA_HOST
    groq_base_url: str = DEFAULT_GROQ_BASE_URL
    groq_api_key: str | None = None
    gemini_api_key: str | None = None
    openrouter_base_url: str = DEFAULT_OPENROUTER_BASE_URL
    openrouter_api_key: str | None = None
    temperature: float = DEFAULT_TEMPERATURE
    max_tokens: int = DEFAULT_MAX_TOKENS
    timeout_seconds: int = DEFAULT_TIMEOUT_SECONDS
    system_message: str = "You are a careful, precise senior software engineer."

    @staticmethod
    def load(config_path: Path | None = None, overrides: dict[str, Any] | None = None) -> "LLMConfig":
        cfg = LLMConfig()

        # Populates os.environ from a .env file in the cwd, if present. Never
        # overrides a variable already set in the real shell environment.
        load_dotenv(dotenv_path=Path.cwd() / DEFAULT_ENV_FILENAME, override=False)

        file_data = _read_config_file(config_path)
        llm_section = file_data.get("llm", {})
        _apply(cfg, llm_section)

        # A blank value in a .env file (e.g. "SCENT_LLM_MODEL=") loads as an
        # empty string, not None; treat that the same as unset for strings.
        env_strings = {
            "provider": os.environ.get("SCENT_LLM_PROVIDER"),
            "model": os.environ.get("SCENT_LLM_MODEL"),
            "ollama_host": os.environ.get("OLLAMA_HOST"),
            "groq_base_url": os.environ.get("GROQ_BASE_URL"),
            "groq_api_key": os.environ.get("GROQ_API_KEY"),
            "gemini_api_key": os.environ.get("GEMINI_API_KEY"),
            "openrouter_base_url": os.environ.get("OPENROUTER_BASE_URL"),
            "openrouter_api_key": os.environ.get("OPENROUTER_API_KEY"),
        }
        _apply(cfg, {k: v for k, v in env_strings.items() if v})

        env_numbers = {
            "temperature": _maybe_float(os.environ.get("SCENT_LLM_TEMPERATURE")),
            "max_tokens": _maybe_int(os.environ.get("SCENT_LLM_MAX_TOKENS")),
        }
        _apply(cfg, {k: v for k, v in env_numbers.items() if v is not None})

        if overrides:
            _apply(cfg, {k: v for k, v in overrides.items() if v is not None})

        # Provider-appropriate default model if the user only switched providers.
        if overrides and overrides.get("provider") and not overrides.get("model"):
            if cfg.provider == "groq" and cfg.model == DEFAULT_OLLAMA_MODEL:
                cfg.model = DEFAULT_GROQ_MODEL
            elif cfg.provider == "gemini" and cfg.model == DEFAULT_OLLAMA_MODEL:
                cfg.model = DEFAULT_GEMINI_MODEL
            elif cfg.provider == "openrouter" and cfg.model == DEFAULT_OLLAMA_MODEL:
                cfg.model = DEFAULT_OPENROUTER_MODEL
            elif cfg.provider == "ollama" and cfg.model == DEFAULT_GROQ_MODEL:
                cfg.model = DEFAULT_OLLAMA_MODEL
            elif cfg.provider == "ollama" and cfg.model == DEFAULT_GEMINI_MODEL:
                cfg.model = DEFAULT_OLLAMA_MODEL
            elif cfg.provider == "ollama" and cfg.model == DEFAULT_OPENROUTER_MODEL:
                cfg.model = DEFAULT_OLLAMA_MODEL

        return cfg


def _read_config_file(config_path: Path | None) -> dict[str, Any]:
    path = config_path or Path.cwd() / DEFAULT_CONFIG_FILENAME
    if not path.exists():
        return {}
    try:
        with path.open("rb") as fh:
            return tomllib.load(fh)
    except (OSError, tomllib.TOMLDecodeError):
        # A malformed config file must never crash the tool; fall back to defaults.
        return {}


def _apply(cfg: LLMConfig, data: dict[str, Any]) -> None:
    for key, value in data.items():
        if value is not None and hasattr(cfg, key):
            setattr(cfg, key, value)


def _maybe_float(value: str | None) -> float | None:
    try:
        return float(value) if value is not None else None
    except ValueError:
        return None


def _maybe_int(value: str | None) -> int | None:
    try:
        return int(value) if value is not None else None
    except ValueError:
        return None
