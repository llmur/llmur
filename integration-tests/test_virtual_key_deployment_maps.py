import pytest


class TestVirtualKeyDeploymentMaps:
    def test_create_virtual_key_deployment_map_success(self, api_client, created_virtual_key, created_deployment):
        """Test successful creation of association between key and a deployment"""
        payload = {
            'virtual_key_id': created_virtual_key,
            'deployment_id': created_deployment,
            'budget_limits': {"cost_per_day": 2.25},
            'request_limits': {"requests_per_day": 7},
            'token_limits': {"tokens_per_day": 13},
        }

        response = api_client.create_virtual_key_deployment_map(payload)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['virtual_key_id'] == created_virtual_key
        assert data['deployment_id'] == created_deployment
        assert data['budget_limits']['cost_per_day'] == pytest.approx(2.25)
        assert data['request_limits']['requests_per_day'] == 7
        assert data['token_limits']['tokens_per_day'] == 13

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
        assert 'budget_limits' in data
        assert 'request_limits' in data
        assert 'token_limits' in data

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

    def test_update_virtual_key_deployment_map_limits(
        self,
        api_client,
        created_virtual_key,
        created_deployment,
    ):
        """Test updating virtual key deployment map limits and clearing them"""
        payload = {
            'virtual_key_id': created_virtual_key,
            'deployment_id': created_deployment,
            'budget_limits': {"cost_per_day": 1.0},
        }

        create_response = api_client.create_virtual_key_deployment_map(payload)
        assert create_response.status_code == 200
        map_id = create_response.json()['id']

        update_response = api_client.update_virtual_key_deployment_map(map_id, {
            "budget_limits": {"cost_per_day": 2.5},
            "request_limits": {"requests_per_day": 3},
        })
        assert update_response.status_code == 200
        data = update_response.json()
        assert data['budget_limits']['cost_per_day'] == pytest.approx(2.5)
        assert data['request_limits']['requests_per_day'] == 3

        clear_response = api_client.update_virtual_key_deployment_map(map_id, {
            "budget_limits": {},
        })
        assert clear_response.status_code == 200
        cleared = clear_response.json()
        assert cleared['budget_limits']['cost_per_day'] is None

        api_client.delete_virtual_key_deployment_map(map_id)
