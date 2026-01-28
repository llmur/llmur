import json
import time
import pytest
import requests
from config import Config


class TestChatCompletions:
    def _collect_sse_events(self, response, max_events=200):
        events = []
        done = False

        for line in response.iter_lines(decode_unicode=True):
            if not line:
                continue
            if not line.startswith("data:"):
                continue

            data = line[len("data:"):].strip()
            if data == "[DONE]":
                done = True
                break

            try:
                events.append(json.loads(data))
            except json.JSONDecodeError:
                continue

            if len(events) >= max_events:
                break

        response.close()
        return events, done

    def test_chat_completions_openai(self, api_client, openai_chat_provider_setup):
        """Test OpenAI provider chat completions (non-stream)"""
        payload = {
            "model": openai_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
        }

        response = api_client.create_chat_completion(payload, openai_chat_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert "choices" in data
        assert data["choices"]
        assert data.get("object")
        assert data["model"] == openai_chat_provider_setup["deployment_name"]

    def test_chat_completions_openai_stream(self, openai_chat_provider_setup):
        """Test OpenAI provider chat completions (stream)"""
        payload = {
            "model": openai_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": True,
            "stream_options": {"include_usage": True},
        }
        headers = {
            "Authorization": f"Bearer {openai_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/chat/completions",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")
        response.close()

    def test_chat_completions_azure(self, api_client, azure_chat_provider_setup):
        """Test Azure OpenAI provider chat completions"""
        payload = {
            "model": azure_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
        }

        response = api_client.create_chat_completion(payload, azure_chat_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert "choices" in data
        assert data["choices"]
        assert data.get("object")
        assert data["model"] == azure_chat_provider_setup["deployment_name"]

    def test_chat_completions_gemini(self, api_client, gemini_chat_provider_setup):
        """Test Gemini provider chat completions"""
        payload = {
            "model": gemini_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
        }

        response = api_client.create_chat_completion(payload, gemini_chat_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert "choices" in data
        assert data["choices"]
        assert data.get("object")
        assert data["model"] == Config.GEMINI_CHAT_COMPLETIONS_MODEL

    def test_chat_completions_complex_payload(self, api_client, azure_chat_provider_setup):
        """Test complex payload without response_format"""
        payload = {
            "model": azure_chat_provider_setup["deployment_name"],
            "messages": [
                {"role": "system", "content": "You are a JSON generator."},
                {"role": "user", "content": "Return a JSON object with keys foo and bar."},
            ],
            "max_completion_tokens": 50,
            "user": "integration-test-user",
        }

        response = api_client.create_chat_completion(payload, azure_chat_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert data["choices"]

    def test_chat_completions_response_format_json_object(self, api_client, openai_chat_provider_setup):
        """Test response_format json_object returns valid JSON"""
        payload = {
            "model": openai_chat_provider_setup["deployment_name"],
            "messages": [
                {"role": "system", "content": "You are a JSON generator."},
                {"role": "user", "content": "Return a JSON object with keys foo and bar."},
            ],
            "temperature": 0.2,
            "top_p": 0.9,
            "max_tokens": 50,
            "user": "integration-test-user",
            "response_format": {"type": "json_object"},
        }

        response = api_client.create_chat_completion(payload, openai_chat_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        content = data["choices"][0]["message"]["content"]
        assert content
        json.loads(content)

    def test_chat_completions_stream_no_usage(self, azure_chat_provider_setup):
        """Test streaming without usage requested"""
        payload = {
            "model": azure_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": True,
        }
        headers = {
            "Authorization": f"Bearer {azure_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/chat/completions",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events, done = self._collect_sse_events(response)
        assert done is True
        assert any(event.get("choices") for event in events)
        assert not any(event.get("usage") for event in events if event.get("usage") is not None)

    def test_chat_completions_stream_with_usage(self, azure_chat_provider_setup):
        """Test streaming with usage requested"""
        payload = {
            "model": azure_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": True,
            "stream_options": {"include_usage": True},
        }
        headers = {
            "Authorization": f"Bearer {azure_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/chat/completions",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events, done = self._collect_sse_events(response)
        assert done is True
        assert any(event.get("choices") for event in events)
        assert any(event.get("usage") for event in events if event.get("usage") is not None)

    def test_chat_completions_stream_cancel_openai(self, openai_chat_provider_setup):
        """Test canceling OpenAI stream early"""
        payload = {
            "model": openai_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": True,
        }
        headers = {
            "Authorization": f"Bearer {openai_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/chat/completions",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events = []
        for line in response.iter_lines(decode_unicode=True):
            if not line or not line.startswith("data:"):
                continue
            data = line[len("data:"):].strip()
            if data == "[DONE]":
                break
            try:
                events.append(json.loads(data))
            except json.JSONDecodeError:
                continue
            if len(events) >= 2:
                break

        response.close()
        assert events

    def test_chat_completions_stream_cancel_azure(self, azure_chat_provider_setup):
        """Test canceling Azure stream early"""
        payload = {
            "model": azure_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": True,
        }
        headers = {
            "Authorization": f"Bearer {azure_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/chat/completions",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events = []
        for line in response.iter_lines(decode_unicode=True):
            if not line or not line.startswith("data:"):
                continue
            data = line[len("data:"):].strip()
            if data == "[DONE]":
                break
            try:
                events.append(json.loads(data))
            except json.JSONDecodeError:
                continue
            if len(events) >= 2:
                break

        response.close()
        assert events

    def test_chat_completions_stream_cancel_gemini(self, gemini_chat_provider_setup):
        """Test canceling Gemini stream early"""
        payload = {
            "model": gemini_chat_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": True,
        }
        headers = {
            "Authorization": f"Bearer {gemini_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/chat/completions",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events = []
        for line in response.iter_lines(decode_unicode=True):
            if not line or not line.startswith("data:"):
                continue
            data = line[len("data:"):].strip()
            if data == "[DONE]":
                break
            try:
                events.append(json.loads(data))
            except json.JSONDecodeError:
                continue
            if len(events) >= 2:
                break

        response.close()
        assert events

    def test_chat_completions_request_limit(self, api_client, azure_chat_rate_limited_setup):
        """Test request limit returns 429 after limit is reached"""
        payload = {
            "model": azure_chat_rate_limited_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
        }

        first = api_client.create_chat_completion(payload, azure_chat_rate_limited_setup["virtual_key"])
        assert first.status_code == 200

        last_response = None
        for _ in range(6):
            resp = api_client.create_chat_completion(payload, azure_chat_rate_limited_setup["virtual_key"])
            last_response = resp
            if resp.status_code == 429:
                break
            assert resp.status_code == 200
            time.sleep(0.3)

        assert last_response is not None
        assert last_response.status_code == 429

    def test_chat_completions_invalid_payload(self, api_client, azure_chat_provider_setup):
        """Test invalid payload returns bad request"""
        response = api_client.create_chat_completion({}, azure_chat_provider_setup["virtual_key"])

        assert response.status_code == 400

    def test_chat_completions_invalid_auth(self, api_client):
        """Test invalid Authorization header returns unauthorized"""
        payload = {
            "model": "missing",
            "messages": [{"role": "user", "content": "Hello"}],
        }

        response = api_client.create_chat_completion(payload, "invalid-key")

        assert response.status_code == 401

    def test_chat_completions_missing_auth(self):
        """Test missing Authorization header returns unauthorized"""
        payload = {
            "model": "missing",
            "messages": [{"role": "user", "content": "Hello"}],
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/chat/completions",
            headers={"Content-Type": "application/json"},
            json=payload,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 401

    def test_chat_completions_azure_invalid_deployment(self, api_client, azure_chat_invalid_provider_setup):
        """Test Azure provider errors propagate (invalid deployment)"""
        payload = {
            "model": azure_chat_invalid_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
        }

        response = api_client.create_chat_completion(payload, azure_chat_invalid_provider_setup["virtual_key"])

        assert response.status_code == 404

    def test_chat_completions_gemini_invalid_model(self, api_client, gemini_chat_invalid_provider_setup):
        """Test Gemini provider errors propagate (invalid model)"""
        payload = {
            "model": gemini_chat_invalid_provider_setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
        }

        response = api_client.create_chat_completion(payload, gemini_chat_invalid_provider_setup["virtual_key"])

        assert response.status_code == 404
