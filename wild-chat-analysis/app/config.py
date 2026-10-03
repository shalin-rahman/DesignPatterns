"""Load settings from config.yaml, the environment / .env file, and CLI overrides.

Order of precedence (highest first): CLI overrides, environment variables, config.yaml,
built-in defaults.
"""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import yaml
from dotenv import load_dotenv
from pydantic import BaseModel, Field, SecretStr, ValidationError

DEFAULT_DATASET_URL = (
    "https://huggingface.co/datasets/allenai/WildChat/resolve/main/data/"
    "train-00003-of-00006.parquet"
)

# Environment variable -> LLMSettings field
_LLM_ENV: dict[str, str] = {
    "LLM_API_KEY": "api_key",
    "LLM_BASE_URL": "base_url",
    "LLM_MODEL": "model",
    "LLM_TEMPERATURE": "temperature",
    "LLM_MAX_TOKENS": "max_tokens",
    "LLM_REQUEST_TIMEOUT": "request_timeout",
    "LLM_RETRY_ATTEMPTS": "retry_attempts",
    "LLM_ORGANIZATION": "organization",
    "LLM_PROJECT": "project",
    "LLM_JSON_MODE": "json_mode",
    "LLM_REQUESTS_PER_MINUTE": "requests_per_minute",
}

# Environment variable -> Settings field
_TOP_ENV: dict[str, str] = {
    "DATASET_URL": "dataset_url",
    "HF_TOKEN": "hf_token",
    "OUTPUT_FILE": "output_file",
}

# Keys the YAML file keeps at top level but which belong to the LLM settings.
_TOP_LEVEL_LLM_KEYS = ("request_timeout", "retry_attempts")


class ConfigError(Exception):
    """Raised when the configuration is missing or invalid."""


class LLMSettings(BaseModel):
    """API settings: endpoint, model, sampling, timeouts, retries and rate limit."""
    api_key: SecretStr | None = None
    base_url: str = "https://api.groq.com/openai/v1"
    model: str = "openai/gpt-oss-120b"
    temperature: float = Field(0.0, ge=0.0, le=2.0)
    max_tokens: int = Field(1000, gt=0)
    request_timeout: float = Field(60.0, gt=0)
    retry_attempts: int = Field(3, ge=0)
    organization: str | None = None
    project: str | None = None
    json_mode: bool = True
    requests_per_minute: float = Field(0.0, ge=0.0)
    backoff_base: float = Field(1.0, ge=0.0)
    backoff_max: float = Field(60.0, ge=0.0)


class Settings(BaseModel):
    """Run settings: dataset, sampling, output paths, concurrency and logging, plus `llm`."""
    dataset_url: str = DEFAULT_DATASET_URL
    cache_dir: Path = Path("data")
    hf_token: SecretStr | None = None
    output_file: Path = Path("output/prompt_smells.json")
    max_records: int | None = Field(None, gt=0)
    max_prompts: int | None = Field(None, gt=0)
    sample_fraction: float = Field(1.0, gt=0.0, le=1.0)
    sample_seed: str = "wildchat"
    language: str = "English"
    concurrency: int = Field(5, ge=1)
    read_batch_size: int = Field(1000, ge=1)
    max_prompt_chars: int = Field(12000, ge=100)
    parse_retries: int = Field(2, ge=0)
    resume: bool = True
    log_level: str = "INFO"
    log_file: Path | None = None
    progress_log_every: int = Field(1000, ge=1)
    llm: LLMSettings = Field(default_factory=LLMSettings)

    @property
    def checkpoint_file(self) -> Path:
        """The checkpoint path, made from the output path: `x.json` becomes `x.checkpoint.jsonl`."""
        return self.output_file.with_suffix(".checkpoint.jsonl")

    @property
    def failures_file(self) -> Path:
        """The failure log path, made from the output path: `x.json` becomes `x.failures.jsonl`."""
        return self.output_file.with_suffix(".failures.jsonl")


def _read_yaml(config_path: Path | None) -> dict[str, Any]:
    """Load the YAML config as a dict. No path gives {}; a missing or invalid file raises ConfigError."""
    if config_path is None:
        return {}
    if not config_path.exists():
        raise ConfigError(f"Config file not found: {config_path}")
    try:
        data = yaml.safe_load(config_path.read_text(encoding="utf-8")) or {}
    except yaml.YAMLError as exc:
        raise ConfigError(f"Invalid YAML in {config_path}: {exc}") from exc
    if not isinstance(data, dict):
        raise ConfigError(f"{config_path} must contain a mapping at top level")
    return data


def load_settings(
    config_path: Path | None = None,
    overrides: dict[str, Any] | None = None,
    env_file: Path | None = Path(".env"),
) -> Settings:
    """Build Settings. Overrides with a value of None are ignored."""
    if env_file is not None and env_file.exists():
        load_dotenv(env_file, override=False)

    data = _read_yaml(config_path)
    llm: dict[str, Any] = dict(data.pop("llm", None) or {})
    for key in _TOP_LEVEL_LLM_KEYS:
        if key in data:
            llm[key] = data.pop(key)

    for env_name, field in _LLM_ENV.items():
        value = os.getenv(env_name)
        if value not in (None, ""):
            llm[field] = value
    for env_name, field in _TOP_ENV.items():
        value = os.getenv(env_name)
        if value not in (None, ""):
            data[field] = value

    for key, value in (overrides or {}).items():
        if value is None:
            continue
        if key in LLMSettings.model_fields:
            llm[key] = value
        else:
            data[key] = value

    data["llm"] = llm
    try:
        return Settings.model_validate(data)
    except ValidationError as exc:
        raise ConfigError(f"Invalid configuration:\n{exc}") from exc
