"""Thin, provider-agnostic client for generating code from a local or cloud LLM.

Supports four backends out of the box:
  - Ollama (local, no API key, default backend)
  - Groq (cloud, OpenAI-compatible chat completions API)
  - Gemini (cloud, Google Gen AI SDK)
  - OpenRouter (cloud, OpenAI-compatible chat completions API)

Both call paths deliberately set every field that scent_llm's own smell
detectors look for (system message, explicit temperature, bounded max
tokens, a pinned model string) so the generator never trips its own rules.
"""
from __future__ import annotations

import json
import urllib.error
import urllib.request
from dataclasses import dataclass

from scent_llm.config import LLMConfig


class LLMClientError(RuntimeError):
    """Raised when the configured LLM backend cannot be reached or errors out."""


@dataclass
class GenerationResult:
    code: str
    raw_response: dict
    provider: str
    model: str


class LLMClient:
    """Provider-agnostic facade. Callers only ever see `generate_code`."""

    def __init__(self, config: LLMConfig):
        self.config = config

    def generate_code(self, task_description: str, language: str = "python") -> GenerationResult:
        system_message = (
            f"{self.config.system_message} "
            f"Write only {language} code that solves the user's request. "
            "Respond with a single fenced code block and nothing else."
        )
        user_message = task_description

        if self.config.provider == "groq":
            return self._generate_groq(system_message, user_message)
        if self.config.provider == "ollama":
            return self._generate_ollama(system_message, user_message)
        if self.config.provider == "gemini":
            return self._generate_gemini(system_message, user_message)
        if self.config.provider == "openrouter":
            return self._generate_openrouter(system_message, user_message)
        raise LLMClientError(
            f"Unknown provider '{self.config.provider}'. Supported providers: "
            "'ollama', 'groq', 'gemini', 'openrouter'."
        )

    # -- Ollama ---------------------------------------------------------

    def _generate_ollama(self, system_message: str, user_message: str) -> GenerationResult:
        url = f"{self.config.ollama_host.rstrip('/')}/api/chat"
        payload = {
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system_message},
                {"role": "user", "content": user_message},
            ],
            "stream": False,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
            },
        }
        data = self._post_json(url, payload)
        try:
            content = data["message"]["content"]
        except (KeyError, TypeError) as exc:
            raise LLMClientError(f"Unexpected Ollama response shape: {data}") from exc
        return GenerationResult(
            code=_extract_code_block(content),
            raw_response=data,
            provider="ollama",
            model=self.config.model,
        )

    # -- Groq -------------------------------------------------------------

    def _generate_groq(self, system_message: str, user_message: str) -> GenerationResult:
        if not self.config.groq_api_key:
            raise LLMClientError(
                "Groq provider selected but no API key was found. "
                "Set the GROQ_API_KEY environment variable, or switch to "
                "provider='ollama' to run fully locally."
            )
        url = f"{self.config.groq_base_url.rstrip('/')}/chat/completions"
        payload = {
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system_message},
                {"role": "user", "content": user_message},
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        }
        headers = {
            "Authorization": f"Bearer {self.config.groq_api_key}",
            "Content-Type": "application/json",
        }
        data = self._post_json(url, payload, headers=headers)
        try:
            content = data["choices"][0]["message"]["content"]
        except (KeyError, IndexError, TypeError) as exc:
            raise LLMClientError(f"Unexpected Groq response shape: {data}") from exc
        return GenerationResult(
            code=_extract_code_block(content),
            raw_response=data,
            provider="groq",
            model=self.config.model,
        )

    # -- Gemini -------------------------------------------------------------

    def _generate_gemini(self, system_message: str, user_message: str) -> GenerationResult:
        if not self.config.gemini_api_key:
            raise LLMClientError(
                "Gemini provider selected but no API key was found. "
                "Set the GEMINI_API_KEY environment variable, or switch to "
                "provider='ollama' to run fully locally."
            )
        try:
            from google import genai
            from google.genai import errors
            from google.genai import types
        except ImportError as exc:
            raise LLMClientError(
                "Gemini support requires the google-genai package. "
                "Install project dependencies with `python -m pip install -e .`."
            ) from exc

        try:
            client = genai.Client(
                api_key=self.config.gemini_api_key,
                http_options=types.HttpOptions(timeout=self.config.timeout_seconds * 1000),
            )
            response = client.models.generate_content(
                model=self.config.model,
                contents=user_message,
                config=types.GenerateContentConfig(
                    system_instruction=system_message,
                    temperature=self.config.temperature,
                    max_output_tokens=self.config.max_tokens,
                ),
            )
            content = getattr(response, "text", None)
            if not content:
                raise LLMClientError("Gemini returned an empty response.")
        except LLMClientError:
            raise
        except errors.APIError as exc:
            raise LLMClientError(f"Gemini request failed: {exc}") from exc

        raw_response = (
            response.model_dump(exclude_none=True)
            if hasattr(response, "model_dump")
            else {"text": content}
        )
        return GenerationResult(
            code=_extract_code_block(content),
            raw_response=raw_response,
            provider="gemini",
            model=self.config.model,
        )

    # -- OpenRouter ---------------------------------------------------------

    def _generate_openrouter(self, system_message: str, user_message: str) -> GenerationResult:
        if not self.config.openrouter_api_key:
            raise LLMClientError(
                "OpenRouter provider selected but no API key was found. "
                "Set the OPENROUTER_API_KEY environment variable, or switch to "
                "provider='ollama' to run fully locally."
            )
        url = f"{self.config.openrouter_base_url.rstrip('/')}/chat/completions"
        payload = {
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system_message},
                {"role": "user", "content": user_message},
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        }
        headers = {
            "Authorization": f"Bearer {self.config.openrouter_api_key}",
            "Content-Type": "application/json",
        }
        data = self._post_json(url, payload, headers=headers)
        try:
            content = data["choices"][0]["message"]["content"]
        except (KeyError, IndexError, TypeError) as exc:
            raise LLMClientError(f"Unexpected OpenRouter response shape: {data}") from exc
        return GenerationResult(
            code=_extract_code_block(content),
            raw_response=data,
            provider="openrouter",
            model=self.config.model,
        )

    # -- shared HTTP helper ------------------------------------------------

    def _post_json(self, url: str, payload: dict, headers: dict | None = None) -> dict:
        body = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            url,
            data=body,
            headers={"Content-Type": "application/json", **(headers or {})},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=self.config.timeout_seconds) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as exc:
            detail = exc.read().decode("utf-8", errors="replace")
            raise LLMClientError(f"{self.config.provider} API returned {exc.code}: {detail}") from exc
        except urllib.error.URLError as exc:
            raise LLMClientError(
                f"Could not reach {self.config.provider} at {url}: {exc.reason}. "
                "If using Ollama, confirm it is running (`ollama serve`)."
            ) from exc
        except TimeoutError as exc:
            raise LLMClientError(f"{self.config.provider} request timed out after "
                                  f"{self.config.timeout_seconds}s.") from exc


def _extract_code_block(content: str) -> str:
    """Pull the first fenced code block out of a model response, if present."""
    if "```" not in content:
        return content.strip()
    parts = content.split("```")
    if len(parts) < 2:
        return content.strip()
    block = parts[1]
    lines = block.splitlines()
    if lines and lines[0].strip().isalpha():
        lines = lines[1:]
    return "\n".join(lines).strip()
