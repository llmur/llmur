import uuid
import pytest
from config import Config
from conftest import _provider_ready


def _setup_load_balanced_deployment(api_client, project_id, strategy, azure_weight, gemini_weight):
    if not _provider_ready([
        Config.AZURE_OPENAI_API_KEY,
        Config.AZURE_OPENAI_ENDPOINT,
        Config.AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT,
        Config.GEMINI_API_KEY,
        Config.GEMINI_CHAT_COMPLETIONS_MODEL,
    ]):
        pytest.skip("Azure/Gemini chat config not fully set")

    deployment_resp = api_client.create_deployment({
        "name": f"lb-{strategy}-{uuid.uuid4().hex[:8]}",
        "access": "public",
        "strategy": strategy,
    })
    assert deployment_resp.status_code == 200
    deployment = deployment_resp.json()

    azure_conn_resp = api_client.create_connection({
        "provider": "azure/openai",
        "deployment_name": Config.AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT,
        "api_endpoint": Config.AZURE_OPENAI_ENDPOINT,
        "api_key": Config.AZURE_OPENAI_API_KEY,
        "api_version": Config.AZURE_OPENAI_API_VERSION,
    })
    assert azure_conn_resp.status_code == 200
    azure_conn_id = azure_conn_resp.json()["id"]

    gemini_conn_resp = api_client.create_connection({
        "provider": "gemini",
        "model": Config.GEMINI_CHAT_COMPLETIONS_MODEL,
        "api_endpoint": Config.GEMINI_BASE_URL,
        "api_key": Config.GEMINI_API_KEY,
        "api_version": Config.GEMINI_API_VERSION,
    })
    assert gemini_conn_resp.status_code == 200
    gemini_conn_id = gemini_conn_resp.json()["id"]

    azure_map_resp = api_client.create_connection_deployment_map({
        "connection_id": azure_conn_id,
        "deployment_id": deployment["id"],
        "weight": azure_weight,
    })
    assert azure_map_resp.status_code == 200
    azure_map_id = azure_map_resp.json()["id"]

    gemini_map_resp = api_client.create_connection_deployment_map({
        "connection_id": gemini_conn_id,
        "deployment_id": deployment["id"],
        "weight": gemini_weight,
    })
    assert gemini_map_resp.status_code == 200
    gemini_map_id = gemini_map_resp.json()["id"]

    key_resp = api_client.create_virtual_key({
        "project_id": project_id,
    })
    assert key_resp.status_code == 200
    key_payload = key_resp.json()

    vkd_resp = api_client.create_virtual_key_deployment_map({
        "virtual_key_id": key_payload["id"],
        "deployment_id": deployment["id"],
    })
    assert vkd_resp.status_code == 200
    vkd_id = vkd_resp.json()["id"]

    return {
        "deployment_id": deployment["id"],
        "deployment_name": deployment["name"],
        "azure_connection_id": azure_conn_id,
        "gemini_connection_id": gemini_conn_id,
        "azure_map_id": azure_map_id,
        "gemini_map_id": gemini_map_id,
        "virtual_key_id": key_payload["id"],
        "virtual_key": key_payload["key"],
        "virtual_key_deployment_id": vkd_id,
        "azure_weight": azure_weight,
        "gemini_weight": gemini_weight,
    }


def _cleanup_load_balanced_deployment(api_client, setup):
    api_client.delete_virtual_key_deployment_map(setup["virtual_key_deployment_id"])
    api_client.delete_virtual_key(setup["virtual_key_id"])
    api_client.delete_connection_deployment_map(setup["azure_map_id"])
    api_client.delete_connection_deployment_map(setup["gemini_map_id"])
    api_client.delete_deployment(setup["deployment_id"])
    api_client.delete_connection(setup["azure_connection_id"])
    api_client.delete_connection(setup["gemini_connection_id"])


def _count_provider_responses(api_client, setup, num_requests):
    counts = {"azure": 0, "gemini": 0}
    for _ in range(num_requests):
        payload = {
            "model": setup["deployment_name"],
            "messages": [{"role": "user", "content": "Hello"}],
        }
        response = api_client.create_chat_completion(payload, setup["virtual_key"])
        assert response.status_code == 200
        data = response.json()
        model = data.get("model")
        if model == setup["deployment_name"]:
            counts["azure"] += 1
        elif model == Config.GEMINI_CHAT_COMPLETIONS_MODEL:
            counts["gemini"] += 1
        else:
            raise AssertionError(f"Unexpected model value: {model}")

    return counts


class TestLoadBalancing:
    def test_round_robin_distribution(self, api_client, created_project):
        """Test round-robin distribution across Azure and Gemini connections"""
        setup = _setup_load_balanced_deployment(
            api_client,
            created_project,
            "round_robin",
            azure_weight=1,
            gemini_weight=1,
        )
        try:
            counts = _count_provider_responses(api_client, setup, num_requests=6)
            assert counts["azure"] == 3
            assert counts["gemini"] == 3
        finally:
            _cleanup_load_balanced_deployment(api_client, setup)

    def test_weighted_round_robin_distribution(self, api_client, created_project):
        """Test weighted round-robin distribution across Azure and Gemini connections"""
        setup = _setup_load_balanced_deployment(
            api_client,
            created_project,
            "weighted_round_robin",
            azure_weight=2,
            gemini_weight=1,
        )
        try:
            counts = _count_provider_responses(api_client, setup, num_requests=6)
            assert counts["azure"] == 4
            assert counts["gemini"] == 2
        finally:
            _cleanup_load_balanced_deployment(api_client, setup)
