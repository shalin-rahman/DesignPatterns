"""Client for any OpenAI-compatible Chat Completions API, with retry and rate limiting."""

from __future__ import annotations

import logging
import random
import threading
import time
from collections.abc import Callable
from typing import Any

import requests
from tenacity import RetryCallState, Retrying, retry_if_exception_type, stop_after_attempt

from app.config import ConfigError, LLMSettings

logger = logging.getLogger(__name__)

_RETRYABLE_STATUS = {408, 409, 425, 500, 502, 503, 504}
_FATAL_STATUS = {401, 403, 404}


class LLMError(Exception):
    """Base class for API errors. Messages never contain the API key or the prompt."""


class RetryableLLMError(LLMError):
    def __init__(self, message: str, retry_after: float | None = None) -> None:
        super().__init__(message)
        self.retry_after = retry_after


class RateLimitError(RetryableLLMError):
    pass


class NonRetryableLLMError(LLMError):
    pass


class FatalLLMError(NonRetryableLLMError):
    """Every later request would fail too (bad key, no access, wrong model), so the run stops."""


class InvalidOutputError(LLMError):
    """The provider rejected the model output as invalid JSON (for example Groq
    `json_validate_failed`). The analyzer may ask again."""


class RateLimiter:
    """Spaces requests evenly so that at most `requests_per_minute` are started."""

    def __init__(self, requests_per_minute: float) -> None:
        self._interval = 60.0 / requests_per_minute if requests_per_minute > 0 else 0.0
        self._lock = threading.Lock()
        self._next_slot = 0.0

    def acquire(self) -> None:
        if not self._interval:
            return
        with self._lock:
            now = time.monotonic()
            wait = self._next_slot - now
            self._next_slot = max(now, self._next_slot) + self._interval
        if wait > 0:
            time.sleep(wait)


def _parse_retry_after(value: str | None) -> float | None:
    if not value:
        return None
    try:
        return max(0.0, float(value))
    except ValueError:
        return None


def _error_code(response: Any) -> str:
    try:
        error = response.json().get("error") or {}
        return str(error.get("code") or error.get("type") or "unknown error")
    except (ValueError, AttributeError):
        return "unknown error"


def _short(text: str, limit: int = 200) -> str:
    text = " ".join(text.split())
    return text if len(text) <= limit else text[:limit] + "..."


class LLMClient:
    def __init__(
        self,
        settings: LLMSettings,
        session_factory: Callable[[], requests.Session] = requests.Session,
    ) -> None:
        if settings.api_key is None or not settings.api_key.get_secret_value().strip():
            raise ConfigError("LLM_API_KEY is not set. Add it to .env or the environment.")
        self._settings = settings
        self._url = settings.base_url.rstrip("/") + "/chat/completions"
        self._session_factory = session_factory
        self._local = threading.local()
        self._sessions: list[requests.Session] = []
        self._sessions_lock = threading.Lock()
        self._limiter = RateLimiter(settings.requests_per_minute)

    def __repr__(self) -> str:
        return f"LLMClient(url={self._url!r}, model={self._settings.model!r})"

    def _headers(self) -> dict[str, str]:
        assert self._settings.api_key is not None
        headers = {
            "Authorization": f"Bearer {self._settings.api_key.get_secret_value()}",
            "Content-Type": "application/json",
        }
        if self._settings.organization:
            headers["OpenAI-Organization"] = self._settings.organization
        if self._settings.project:
            headers["OpenAI-Project"] = self._settings.project
        return headers

    def _session(self) -> requests.Session:
        session = getattr(self._local, "session", None)
        if session is None:
            session = self._session_factory()
            self._local.session = session
            with self._sessions_lock:
                self._sessions.append(session)
        return session

    def close(self) -> None:
        with self._sessions_lock:
            for session in self._sessions:
                session.close()
            self._sessions.clear()

    def _wait(self, retry_state: RetryCallState) -> float:
        base = self._settings.backoff_base * (2 ** (retry_state.attempt_number - 1))
        delay = min(self._settings.backoff_max, base) + random.uniform(0, base * 0.25)
        exc = retry_state.outcome.exception() if retry_state.outcome else None
        retry_after = getattr(exc, "retry_after", None)
        if retry_after is not None:
            delay = max(delay, min(retry_after, self._settings.backoff_max))
        return delay

    @staticmethod
    def _log_retry(retry_state: RetryCallState) -> None:
        exc = retry_state.outcome.exception() if retry_state.outcome else None
        delay = retry_state.next_action.sleep if retry_state.next_action else 0.0
        if isinstance(exc, RateLimitError):
            logger.warning("API rate limit encountered. Retrying in %.1fs...", delay)
        else:
            logger.warning("Temporary API error (%s). Retrying in %.1fs...", exc, delay)

    def complete(self, messages: list[dict[str, str]]) -> str:
        """Send a chat request and return the assistant text."""
        retrying = Retrying(
            stop=stop_after_attempt(self._settings.retry_attempts + 1),
            wait=self._wait,
            retry=retry_if_exception_type(RetryableLLMError),
            before_sleep=self._log_retry,
            reraise=True,
        )
        return retrying(self._send, messages)

    def _send(self, messages: list[dict[str, str]]) -> str:
        self._limiter.acquire()
        payload: dict[str, Any] = {
            "model": self._settings.model,
            "messages": messages,
            "temperature": self._settings.temperature,
            "max_tokens": self._settings.max_tokens,
        }
        if self._settings.json_mode:
            payload["response_format"] = {"type": "json_object"}

        try:
            response = self._session().post(
                self._url,
                json=payload,
                headers=self._headers(),
                timeout=self._settings.request_timeout,
            )
        except requests.Timeout:
            raise RetryableLLMError("request timed out") from None
        except requests.ConnectionError:
            raise RetryableLLMError("connection error") from None
        except requests.RequestException as exc:
            raise RetryableLLMError(f"request failed: {type(exc).__name__}") from None

        status = response.status_code
        if status == 429:
            raise RateLimitError(
                "HTTP 429", retry_after=_parse_retry_after(response.headers.get("Retry-After"))
            )
        if status in _RETRYABLE_STATUS or status >= 500:
            raise RetryableLLMError(f"HTTP {status}")
        if status == 400 and "json_validate_failed" in response.text:
            raise InvalidOutputError("provider rejected output as invalid JSON")
        if status >= 400:
            # The body can echo the prompt, so only the provider's error code leaves this method.
            logger.debug("HTTP %d body: %s", status, _short(response.text))
            message = f"HTTP {status}: {_error_code(response)}"
            if status in _FATAL_STATUS:
                raise FatalLLMError(message)
            raise NonRetryableLLMError(message)

        try:
            content = response.json()["choices"][0]["message"]["content"]
        except (ValueError, KeyError, IndexError, TypeError):
            raise RetryableLLMError("malformed API response") from None
        if not isinstance(content, str) or not content.strip():
            raise InvalidOutputError("empty model output")
        return content
