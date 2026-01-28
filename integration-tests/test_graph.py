import uuid
import pytest


def _setup_graph(api_client, project_id, connection_payload):
    deployment_name = f"graph-{uuid.uuid4().hex[:8]}"
    deployment_resp = api_client.create_deployment({
        "name": deployment_name,
        "access": "public",
    })
    assert deployment_resp.status_code == 200
    deployment = deployment_resp.json()

    connection_resp = api_client.create_connection(connection_payload)
    assert connection_resp.status_code == 200
    connection_id = connection_resp.json()['id']

    map_resp = api_client.create_connection_deployment_map({
        "connection_id": connection_id,
        "deployment_id": deployment['id'],
    })
    assert map_resp.status_code == 200
    map_id = map_resp.json()['id']

    key_resp = api_client.create_virtual_key({
        "project_id": project_id,
    })
    assert key_resp.status_code == 200
    key_payload = key_resp.json()

    vkd_resp = api_client.create_virtual_key_deployment_map({
        "virtual_key_id": key_payload['id'],
        "deployment_id": deployment['id'],
    })
    assert vkd_resp.status_code == 200
    vkd_id = vkd_resp.json()['id']

    return {
        "deployment_id": deployment['id'],
        "deployment_name": deployment['name'],
        "connection_id": connection_id,
        "connection_deployment_id": map_id,
        "virtual_key_id": key_payload['id'],
        "virtual_key": key_payload['key'],
        "virtual_key_deployment_id": vkd_id,
    }


def _cleanup_graph(api_client, setup):
    api_client.delete_virtual_key_deployment_map(setup["virtual_key_deployment_id"])
    api_client.delete_virtual_key(setup["virtual_key_id"])
    if setup.get("connection_deployment_id"):
        api_client.delete_connection_deployment_map(setup["connection_deployment_id"])
    api_client.delete_deployment(setup["deployment_id"])
    if setup.get("connection_id"):
        api_client.delete_connection(setup["connection_id"])


class TestGraph:
    def test_get_graph_success(self, api_client, created_project, sample_azure_openai_connection_data):
        """Test retrieving graph for a valid key and deployment"""
        setup = _setup_graph(api_client, created_project, sample_azure_openai_connection_data)
        try:
            response = api_client.get_graph(setup["virtual_key"], setup["deployment_name"])

            assert response.status_code == 200
            data = response.json()
            assert "virtual_key" in data
            assert "deployment" in data
            assert "project" in data
            assert "connections" in data
        finally:
            _cleanup_graph(api_client, setup)

    def test_get_graph_invalid_key(self, api_client):
        """Test graph retrieval with invalid key returns unauthorized"""
        response = api_client.get_graph("invalid-key", "missing-deployment")

        assert response.status_code == 401

    def test_get_graph_invalid_deployment(self, api_client, created_project, sample_azure_openai_connection_data):
        """Test graph retrieval with invalid deployment returns not found"""
        setup = _setup_graph(api_client, created_project, sample_azure_openai_connection_data)
        try:
            response = api_client.get_graph(setup["virtual_key"], "missing-deployment")

            assert response.status_code == 404
        finally:
            _cleanup_graph(api_client, setup)

    def test_get_graph_missing_connection(self, api_client, created_project):
        """Test graph retrieval when deployment has no connections returns 503"""
        deployment_resp = api_client.create_deployment({
            "name": f"graph-nc-{uuid.uuid4().hex[:8]}",
            "access": "public",
        })
        assert deployment_resp.status_code == 200
        deployment = deployment_resp.json()

        key_resp = api_client.create_virtual_key({
            "project_id": created_project,
        })
        assert key_resp.status_code == 200
        key_payload = key_resp.json()

        vkd_resp = api_client.create_virtual_key_deployment_map({
            "virtual_key_id": key_payload['id'],
            "deployment_id": deployment['id'],
        })
        assert vkd_resp.status_code == 200
        vkd_id = vkd_resp.json()['id']

        try:
            response = api_client.get_graph(key_payload["key"], deployment["name"])

            assert response.status_code == 503
        finally:
            api_client.delete_virtual_key_deployment_map(vkd_id)
            api_client.delete_virtual_key(key_payload['id'])
            api_client.delete_deployment(deployment['id'])
