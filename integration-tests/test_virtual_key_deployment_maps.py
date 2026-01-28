import pytest


class TestVirtualKeyDeploymentMaps:
    def test_create_virtual_key_deployment_map_success(self, api_client, created_virtual_key, created_deployment):
        """Test successful creation of association between key and a deployment"""
        payload = {
            'virtual_key_id': created_virtual_key,
            'deployment_id': created_deployment
        }

        response = api_client.create_virtual_key_deployment_map(payload)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['virtual_key_id'] == created_virtual_key
        assert data['deployment_id'] == created_deployment

        # Cleanup
        api_client.delete_virtual_key_deployment_map(data['id'])

    def test_get_virtual_key_deployment_map_success(self, api_client, created_virtual_key_deployment_map):
        """Test successful retrieval of association between virtual key and a deployment"""
        response = api_client.get_virtual_key_deployment_map(created_virtual_key_deployment_map)

        assert response.status_code == 200
        data = response.json()
        assert data['id'] == created_virtual_key_deployment_map
        assert 'virtual_key_id' in data
        assert 'deployment_id' in data

    def test_create_virtual_key_deployment_map_duplicate(self, api_client, created_virtual_key, created_deployment):
        """Test duplicate virtual key/deployment map returns conflict"""
        payload = {
            'virtual_key_id': created_virtual_key,
            'deployment_id': created_deployment
        }

        first = api_client.create_virtual_key_deployment_map(payload)
        assert first.status_code == 200
        first_id = first.json()['id']

        second = api_client.create_virtual_key_deployment_map(payload)
        assert second.status_code == 409
        assert 'error' in second.json()

        api_client.delete_virtual_key_deployment_map(first_id)

    def test_create_virtual_key_deployment_map_invalid_refs(self, api_client, sample_uuid, created_deployment):
        """Test invalid virtual key id returns not found"""
        payload = {
            'virtual_key_id': sample_uuid,
            'deployment_id': created_deployment
        }

        response = api_client.create_virtual_key_deployment_map(payload)
        assert response.status_code == 404

    def test_create_virtual_key_deployment_map_invalid_deployment(self, api_client, created_virtual_key, sample_uuid):
        """Test invalid deployment id returns not found"""
        payload = {
            'virtual_key_id': created_virtual_key,
            'deployment_id': sample_uuid
        }

        response = api_client.create_virtual_key_deployment_map(payload)
        assert response.status_code == 404

    def test_search_virtual_key_deployment_maps_by_key(self, api_client, created_virtual_key, created_deployment):
        """Test searching virtual key deployment maps by key returns expected entries"""
        payload = {
            'virtual_key_id': created_virtual_key,
            'deployment_id': created_deployment
        }

        create_response = api_client.create_virtual_key_deployment_map(payload)
        assert create_response.status_code == 200
        map_id = create_response.json()['id']

        response = api_client.search_virtual_key_deployment_maps(virtual_key_id=created_virtual_key)
        assert response.status_code == 200
        data = response.json()
        assert 'maps' in data
        assert any(item['id'] == map_id for item in data['maps'])

        api_client.delete_virtual_key_deployment_map(map_id)

    def test_get_virtual_key_deployment_map_not_found(self, api_client, sample_uuid):
        """Test fetching missing virtual key deployment map returns not found"""
        response = api_client.get_virtual_key_deployment_map(sample_uuid)

        assert response.status_code == 404

    def test_delete_virtual_key_deployment_map_not_found(self, api_client, sample_uuid):
        """Test deleting missing virtual key deployment map returns not found"""
        response = api_client.delete_virtual_key_deployment_map(sample_uuid)

        assert response.status_code == 404

    def test_delete_virtual_key_deployment_map_success(self, api_client, created_virtual_key, created_deployment):
        """Test deleting virtual key deployment map returns success"""
        payload = {
            'virtual_key_id': created_virtual_key,
            'deployment_id': created_deployment
        }

        create_response = api_client.create_virtual_key_deployment_map(payload)
        assert create_response.status_code == 200
        map_id = create_response.json()['id']

        delete_response = api_client.delete_virtual_key_deployment_map(map_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert delete_data["message"] is None

        get_response = api_client.get_virtual_key_deployment_map(map_id)
        assert get_response.status_code == 404
