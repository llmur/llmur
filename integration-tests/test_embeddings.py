import pytest
import requests
from config import Config


class TestEmbeddings:
    def test_embeddings_openai(self, api_client, openai_embeddings_provider_setup):
        """Test OpenAI provider embeddings"""
        payload = {
            "model": openai_embeddings_provider_setup["deployment_name"],
            "input": "Hello",
        }

        response = api_client.create_embeddings(payload, openai_embeddings_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert "data" in data
        assert len(data["data"]) > 0
        assert isinstance(data["data"][0]["embedding"], list)

    def test_embeddings_azure(self, api_client, azure_embeddings_provider_setup):
        """Test Azure OpenAI provider embeddings"""
        payload = {
            "model": azure_embeddings_provider_setup["deployment_name"],
            "input": "Hello",
        }

        response = api_client.create_embeddings(payload, azure_embeddings_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert "data" in data
        assert len(data["data"]) > 0

    def test_embeddings_azure_array_input(self, api_client, azure_embeddings_provider_setup):
        """Test Azure OpenAI embeddings with array input"""
        payload = {
            "model": azure_embeddings_provider_setup["deployment_name"],
            "input": ["Hello", "World"],
        }

        response = api_client.create_embeddings(payload, azure_embeddings_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert len(data["data"]) == 2

    def test_embeddings_gemini(self, api_client, gemini_embeddings_provider_setup):
        """Test Gemini provider embeddings"""
        payload = {
            "model": gemini_embeddings_provider_setup["deployment_name"],
            "input": "Hello",
        }

        response = api_client.create_embeddings(payload, gemini_embeddings_provider_setup["virtual_key"])

        assert response.status_code == 200
        data = response.json()
        assert "data" in data
        assert len(data["data"]) > 0

    def test_embeddings_token_array_non_openai(self, api_client, azure_embeddings_provider_setup):
        """Test token array input is rejected for non-OpenAI providers"""
        payload = {
            "model": azure_embeddings_provider_setup["deployment_name"],
            "input": [1, 2, 3],
        }

        response = api_client.create_embeddings(payload, azure_embeddings_provider_setup["virtual_key"])

        assert response.status_code == 400

    def test_embeddings_token_array_openai(self, api_client, openai_embeddings_provider_setup):
        """Test token array input is accepted for OpenAI providers"""
        payload = {
            "model": openai_embeddings_provider_setup["deployment_name"],
            "input": [1, 2, 3],
        }

        response = api_client.create_embeddings(payload, openai_embeddings_provider_setup["virtual_key"])

        assert response.status_code == 200

    def test_embeddings_invalid_payload(self, api_client, azure_embeddings_provider_setup):
        """Test invalid payload returns bad request"""
        response = api_client.create_embeddings({}, azure_embeddings_provider_setup["virtual_key"])

        assert response.status_code == 400

    def test_embeddings_missing_auth(self):
        """Test missing Authorization header returns unauthorized"""
        payload = {
            "model": "missing",
            "input": "Hello",
        }

        response = requests.post(
            f"{Config.BASE_URL}/v1/embeddings",
            headers={"Content-Type": "application/json"},
            json=payload,
            timeout=Config.TIMEOUT,
        )

        assert response.status_code == 401

    def test_embeddings_azure_invalid_deployment(self, api_client, azure_embeddings_invalid_provider_setup):
        """Test Azure provider errors propagate (invalid deployment)"""
        payload = {
            "model": azure_embeddings_invalid_provider_setup["deployment_name"],
            "input": "Hello",
        }

        response = api_client.create_embeddings(payload, azure_embeddings_invalid_provider_setup["virtual_key"])

        assert response.status_code == 404

    def test_embeddings_gemini_invalid_model(self, api_client, gemini_embeddings_invalid_provider_setup):
        """Test Gemini provider errors propagate (invalid model)"""
        payload = {
            "model": gemini_embeddings_invalid_provider_setup["deployment_name"],
            "input": "Hello",
        }

        response = api_client.create_embeddings(payload, gemini_embeddings_invalid_provider_setup["virtual_key"])

        assert response.status_code == 404
