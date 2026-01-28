import pytest


class TestConnectionDeploymentMaps:
    def test_create_connection_deployment_map_success(self, api_client, created_azure_openai_connection, created_deployment):
        """Test successful creation of association between connection and a deployment"""
        payload = {
            'connection_id': created_azure_openai_connection,
            'deployment_id': created_deployment
        }

        response = api_client.create_connection_deployment_map(payload)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['connection_id'] == created_azure_openai_connection
        assert data['deployment_id'] == created_deployment

        # Cleanup
        api_client.delete_connection_deployment_map(data['id'])

    def test_get_connection_deployment_map_success(self, api_client, created_connection_deployment_map):
        """Test successful retrieval of association between connection and a deployment"""
        response = api_client.get_connection_deployment_map(created_connection_deployment_map)

        assert response.status_code == 200
        data = response.json()
        assert data['id'] == created_connection_deployment_map
        assert 'connection_id' in data
        assert 'deployment_id' in data

    def test_create_connection_deployment_map_duplicate(self, api_client, created_azure_openai_connection, created_deployment):
        """Test duplicate connection/deployment map returns conflict"""
        payload = {
            'connection_id': created_azure_openai_connection,
            'deployment_id': created_deployment
        }

        first = api_client.create_connection_deployment_map(payload)
        assert first.status_code == 200
        first_id = first.json()['id']

        second = api_client.create_connection_deployment_map(payload)
        assert second.status_code == 409
        assert 'error' in second.json()

        api_client.delete_connection_deployment_map(first_id)

    def test_create_connection_deployment_map_invalid_refs(self, api_client, sample_uuid, created_deployment):
        """Test invalid connection id returns not found"""
        payload = {
            'connection_id': sample_uuid,
            'deployment_id': created_deployment
        }

        response = api_client.create_connection_deployment_map(payload)
        assert response.status_code == 404

    def test_create_connection_deployment_map_invalid_deployment(self, api_client, created_azure_openai_connection, sample_uuid):
        """Test invalid deployment id returns not found"""
        payload = {
            'connection_id': created_azure_openai_connection,
            'deployment_id': sample_uuid
        }

        response = api_client.create_connection_deployment_map(payload)
        assert response.status_code == 404

    def test_get_connection_deployment_map_not_found(self, api_client, sample_uuid):
        """Test fetching missing connection/deployment map returns not found"""
        response = api_client.get_connection_deployment_map(sample_uuid)

        assert response.status_code == 404

    def test_delete_connection_deployment_map_not_found(self, api_client, sample_uuid):
        """Test deleting missing connection/deployment map returns not found"""
        response = api_client.delete_connection_deployment_map(sample_uuid)

        assert response.status_code == 404

    def test_delete_connection_deployment_map_success(self, api_client, created_azure_openai_connection, created_deployment):
        """Test deleting connection/deployment map returns success"""
        payload = {
            'connection_id': created_azure_openai_connection,
            'deployment_id': created_deployment
        }
        create_response = api_client.create_connection_deployment_map(payload)
        assert create_response.status_code == 200
        map_id = create_response.json()['id']

        delete_response = api_client.delete_connection_deployment_map(map_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert delete_data["message"] is None

        get_response = api_client.get_connection_deployment_map(map_id)
        assert get_response.status_code == 404
