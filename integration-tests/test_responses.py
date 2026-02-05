import json
import requests
from config import Config


class TestResponses:
    def _collect_sse_events(self, response, max_events=200):
        events = []
        terminal_type = None

        for line in response.iter_lines(decode_unicode=True):
            if not line:
                continue
            if not line.startswith("data:"):
                continue

            data = line[len("data:"):].strip()
            if data == "[DONE]":
                terminal_type = "done"
                break

            try:
                event = json.loads(data)
            except json.JSONDecodeError:
                continue

            events.append(event)
            event_type = event.get("type")
            if event_type in {"response.completed", "response.failed", "response.incomplete"}:
                terminal_type = event_type
                break

            if len(events) >= max_events:
                break

        response.close()
        return events, terminal_type

    def test_responses_openai(self, api_client, openai_chat_provider_setup):
        """Test OpenAI provider responses (non-stream)"""
        payload = {
            "model": openai_chat_provider_setup["deployment_name"],
            "input": "Hello",
        }

        response = api_client.create_responses(payload, openai_chat_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert data.get("id")
        assert data.get("object")
        assert isinstance(data.get("output"), list)
        assert data.get("model") == openai_chat_provider_setup["deployment_name"]

    def test_responses_openai_stream(self, openai_chat_provider_setup):
        """Test OpenAI provider responses (stream)"""
        payload = {
            "model": openai_chat_provider_setup["deployment_name"],
            "input": "Hello",
            "stream": True,
        }
        headers = {
            "Authorization": f"Bearer {openai_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/responses",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events, terminal_type = self._collect_sse_events(response)
        assert events
        assert terminal_type is not None

    def test_responses_azure(self, api_client, azure_chat_provider_setup):
        """Test Azure OpenAI provider responses"""
        payload = {
            "model": azure_chat_provider_setup["deployment_name"],
            "input": "Hello",
        }

        response = api_client.create_responses(payload, azure_chat_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert data.get("id")
        assert data.get("object")
        assert isinstance(data.get("output"), list)

    def test_responses_azure_stream(self, azure_chat_provider_setup):
        """Test Azure OpenAI provider responses (stream)"""
        payload = {
            "model": azure_chat_provider_setup["deployment_name"],
            "input": "Hello",
            "stream": True,
        }
        headers = {
            "Authorization": f"Bearer {azure_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/responses",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events, terminal_type = self._collect_sse_events(response)
        assert events
        assert terminal_type is not None

    def test_responses_gemini(self, api_client, gemini_chat_provider_setup):
        """Test Gemini provider responses"""
        payload = {
            "model": gemini_chat_provider_setup["deployment_name"],
            "input": "Hello",
        }

        response = api_client.create_responses(payload, gemini_chat_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert data.get("id")
        assert data.get("object")
        assert isinstance(data.get("output"), list)
        assert data.get("model") == Config.GEMINI_CHAT_COMPLETIONS_MODEL

    def test_responses_gemini_stream(self, gemini_chat_provider_setup):
        """Test Gemini provider responses (stream)"""
        payload = {
            "model": gemini_chat_provider_setup["deployment_name"],
            "input": "Hello",
            "stream": True,
        }
        headers = {
            "Authorization": f"Bearer {gemini_chat_provider_setup['virtual_key']}",
            "Content-Type": "application/json",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/responses",
            headers=headers,
            json=payload,
            stream=True,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events, terminal_type = self._collect_sse_events(response)
        assert events
        assert terminal_type is not None

    def test_responses_invalid_payload(self, api_client, azure_chat_provider_setup):
        """Test invalid payload returns bad request"""
        response = api_client.create_responses({}, azure_chat_provider_setup["virtual_key"])

        assert response.status_code == 400

    def test_responses_invalid_auth(self, api_client):
        """Test invalid Authorization header returns unauthorized"""
        payload = {
            "model": "missing",
            "input": "Hello",
        }

        response = api_client.create_responses(payload, "invalid-key")

        assert response.status_code == 401

    def test_responses_missing_auth(self):
        """Test missing Authorization header returns unauthorized"""
        payload = {
            "model": "missing",
            "input": "Hello",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/responses",
            headers={"Content-Type": "application/json"},
            json=payload,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 401
