import pytest


class TestConnections:
    def test_create_azure_openai_connection_success(self, api_client, sample_azure_openai_connection_data):
        """Test successful creation of Azure OpenAI connection"""
        response = api_client.create_connection(sample_azure_openai_connection_data)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert 'provider' in data

        # Cleanup
        api_client.delete_connection(data['id'])

    def test_create_openai_connection_success(self, api_client, sample_openai_connection_data):
        """Test successful creation of OpenAI connection"""
        response = api_client.create_connection(sample_openai_connection_data)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['provider'] == 'openai/v1'
        assert data['model'] == sample_openai_connection_data['model']

        # Cleanup
        api_client.delete_connection(data['id'])

    def test_create_gemini_connection_success(self, api_client, sample_gemini_connection_data):
        """Test successful creation of Gemini connection"""
        response = api_client.create_connection(sample_gemini_connection_data)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['provider'] == 'gemini'
        assert data['model'] == sample_gemini_connection_data['model']

        # Cleanup
        api_client.delete_connection(data['id'])

    def test_get_azure_openai_connection_success(self, api_client, created_azure_openai_connection):
        """Test successful connection retrieval"""
        response = api_client.get_connection(created_azure_openai_connection)

        assert response.status_code == 200
        data = response.json()
        assert data['id'] == created_azure_openai_connection
        assert 'provider' in data

    def test_get_connection_not_found(self, api_client, sample_uuid):
        """Test getting non-existent connection returns 404"""
        response = api_client.get_connection(sample_uuid)

        assert response.status_code == 404

    def test_delete_connection_success(self, api_client, sample_azure_openai_connection_data):
        """Test successful connection deletion"""
        # Create connection first
        create_response = api_client.create_connection(sample_azure_openai_connection_data)
        assert create_response.status_code == 200
        connection_id = create_response.json()['id']

        # Delete connection
        delete_response = api_client.delete_connection(connection_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert delete_data["message"] is None

        # Verify connection is deleted
        get_response = api_client.get_connection(connection_id)
        assert get_response.status_code == 404

    def test_delete_connection_not_found(self, api_client, sample_uuid):
        """Test deleting non-existent connection returns 404"""
        response = api_client.delete_connection(sample_uuid)

        assert response.status_code == 404

    def test_create_connection_invalid_provider(self, api_client):
        """Test creating connection with invalid data fails"""
        invalid_data = {"provider": "i/do/not/exist"}  # Missing required fields

        response = api_client.create_connection(invalid_data)

        assert response.status_code == 422

    def test_create_azure_openai_connection_invalid_data(self, api_client):
        """Test creating connection with invalid data fails"""
        invalid_data = {"provider": "azure/openai"}  # Missing required fields

        response = api_client.create_connection(invalid_data)

        assert response.status_code == 422

    def test_list_connections_success(self, api_client, created_azure_openai_connection):
        """Test listing connections returns entries"""
        response = api_client.list_connections()

        assert response.status_code == 200
        data = response.json()
        assert 'connections' in data
        assert any(item['id'] == created_azure_openai_connection for item in data['connections'])
