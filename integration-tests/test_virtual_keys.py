import pytest

class TestVirtualKeys:
    def test_create_virtual_key_success(self, api_client, created_project):
        """Test successful virtual_key creation"""
        virtual_key_data = {
            "project_id": created_project,
        }

        response = api_client.create_virtual_key(virtual_key_data)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['project_id'] == created_project
        assert 'key' in data
        assert data['alias'].startswith('sk-')
        assert data['blocked'] is False

        # Cleanup
        api_client.delete_virtual_key(data['id'])

    def test_get_virtual_key_success(self, api_client, created_virtual_key):
        """Test successful virtual_key retrieval"""
        response = api_client.get_virtual_key(created_virtual_key)

        assert response.status_code == 200
        data = response.json()
        assert data['id'] == created_virtual_key
        assert 'project_id' in data
        assert 'key' in data
        assert 'alias' in data
        assert 'blocked' in data

    def test_get_virtual_key_not_found(self, api_client, sample_uuid):
        """Test getting non-existent virtual_key returns 404"""
        response = api_client.get_virtual_key(sample_uuid)

        assert response.status_code == 404

    def test_delete_virtual_key_success(self, api_client, created_project):
        """Test successful virtual_key deletion"""
        virtual_key_data = {
            "project_id": created_project,
        }

        response = api_client.create_virtual_key(virtual_key_data)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        virtual_key_id = data['id']

        # Delete virtual_key
        delete_response = api_client.delete_virtual_key(virtual_key_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert delete_data["message"] is None

        # Verify virtual_key is deleted
        get_response = api_client.get_virtual_key(virtual_key_id)
        assert get_response.status_code == 404

    def test_delete_virtual_key_not_found(self, api_client, sample_uuid):
        """Test deleting non-existent virtual_key returns 404"""
        response = api_client.delete_virtual_key(sample_uuid)

        assert response.status_code == 404

    def test_create_virtual_key_invalid_data(self, api_client):
        """Test creating virtual_key with invalid data fails"""
        invalid_data = {"invalid": ""}  # Missing required fields

        response = api_client.create_virtual_key(invalid_data)

        assert response.status_code == 422

    def test_search_virtual_keys_by_project(self, api_client, created_project):
        """Test searching virtual keys by project returns expected entries"""
        virtual_key_data = {
            "project_id": created_project,
        }

        response = api_client.create_virtual_key(virtual_key_data)
        assert response.status_code == 200
        key_id = response.json()['id']

        search_response = api_client.search_virtual_keys(project_id=created_project)
        assert search_response.status_code == 200
        data = search_response.json()
        assert 'keys' in data
        assert any(item['id'] == key_id for item in data['keys'])

        api_client.delete_virtual_key(key_id)
