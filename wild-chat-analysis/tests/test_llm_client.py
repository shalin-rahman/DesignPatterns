from __future__ import annotations

import logging
from typing import Any

import pytest
import requests

from app.config import ConfigError, LLMSettings
from app.llm_client import (
    FatalLLMError,
    InvalidOutputError,
    LLMClient,
    NonRetryableLLMError,
    RetryableLLMError,
)

SECRET = "sk-test-secret-123"


class FakeResponse:
    def __init__(self, status: int, body: Any = None, headers: dict[str, str] | None = None) -> None:
        self.status_code = status
        self._body = body
        self.headers = headers or {}
        self.text = body if isinstance(body, str) else str(body)

    def json(self) -> Any:
        if isinstance(self._body, (dict, list)):
            return self._body
        raise ValueError("not json")


class FakeSession:
    def __init__(self, responses: list[Any]) -> None:
        self.responses = responses
        self.requests: list[dict[str, Any]] = []

    def post(self, url: str, **kwargs: Any) -> FakeResponse:
        self.requests.append({"url": url, **kwargs})
        item = self.responses.pop(0)
        if isinstance(item, Exception):
            raise item
        return item

    def close(self) -> None:
        pass


def ok(content: str = '{"smells": []}') -> FakeResponse:
    return FakeResponse(200, {"choices": [{"message": {"content": content}}]})


def make_client(responses: list[Any], **overrides: Any) -> tuple[LLMClient, FakeSession]:
    session = FakeSession(responses)
    settings = LLMSettings(
        api_key=SECRET, base_url="https://api.example.com/v1/", model="m",
        backoff_base=0, retry_attempts=2, **overrides,
    )
    return LLMClient(settings, session_factory=lambda: session), session


def test_missing_api_key_raises() -> None:
    with pytest.raises(ConfigError):
        LLMClient(LLMSettings(api_key=None))


def test_request_shape() -> None:
    client, session = make_client([ok()], organization="org", project="proj")
    assert client.complete([{"role": "user", "content": "x"}]) == '{"smells": []}'
    sent = session.requests[0]
    assert sent["url"] == "https://api.example.com/v1/chat/completions"
    assert sent["headers"]["Authorization"] == f"Bearer {SECRET}"
    assert sent["headers"]["OpenAI-Organization"] == "org"
    assert sent["json"]["response_format"] == {"type": "json_object"}
    assert sent["json"]["model"] == "m"


def test_retries_on_rate_limit_then_succeeds(caplog: pytest.LogCaptureFixture) -> None:
    client, session = make_client([FakeResponse(429, "slow down", {"Retry-After": "0"}), ok()])
    with caplog.at_level(logging.WARNING):
        assert client.complete([]) == '{"smells": []}'
    assert len(session.requests) == 2
    assert "rate limit" in caplog.text
    assert SECRET not in caplog.text


def test_retries_on_timeout_and_server_error() -> None:
    client, session = make_client([requests.Timeout(), FakeResponse(503, "busy"), ok()])
    client.complete([])
    assert len(session.requests) == 3


def test_gives_up_after_retry_attempts() -> None:
    client, session = make_client([FakeResponse(500, "x")] * 3)
    with pytest.raises(RetryableLLMError):
        client.complete([])
    assert len(session.requests) == 3


def test_auth_error_is_fatal_and_not_retried() -> None:
    client, session = make_client([FakeResponse(401, "bad key")])
    with pytest.raises(FatalLLMError) as info:
        client.complete([])
    assert len(session.requests) == 1
    assert SECRET not in str(info.value)


def test_client_error_hides_body() -> None:
    body = {"error": {"code": "context_length_exceeded", "message": "your prompt: secret text"}}
    client, session = make_client([FakeResponse(413, body)])
    with pytest.raises(NonRetryableLLMError) as info:
        client.complete([])
    assert not isinstance(info.value, FatalLLMError)
    assert len(session.requests) == 1
    assert "context_length_exceeded" in str(info.value)
    assert "secret text" not in str(info.value)


def test_provider_json_validation_error() -> None:
    client, _ = make_client([FakeResponse(400, '{"error": {"code": "json_validate_failed"}}')])
    with pytest.raises(InvalidOutputError):
        client.complete([])


def test_malformed_api_response_is_retried() -> None:
    client, session = make_client([FakeResponse(200, {"unexpected": True}), ok()])
    client.complete([])
    assert len(session.requests) == 2


def test_key_not_in_repr() -> None:
    client, _ = make_client([])
    assert SECRET not in repr(client)
    assert SECRET not in repr(LLMSettings(api_key=SECRET))
