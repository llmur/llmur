import pytest

class TestDeployments:
    def test_create_deployment_success(self, api_client, sample_deployment_data, created_azure_openai_connection):
        """Test successful deployment creation"""
        payload = dict(sample_deployment_data)
        payload["connection_id"] = created_azure_openai_connection
        response = api_client.create_deployment(payload)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['name'] == payload['name']
        assert data['access'] == payload['access']
        assert data['connection_id'] == created_azure_openai_connection

        # Cleanup
        api_client.delete_deployment(data['id'])

    def test_get_deployment_success(self, api_client, created_deployment):
        """Test successful deployment retrieval"""
        response = api_client.get_deployment(created_deployment)

        assert response.status_code == 200
        data = response.json()
        assert data['id'] == created_deployment
        assert 'name' in data
        assert 'access' in data
        assert 'connection_id' in data

    def test_get_deployment_via_session(self, api_client, created_deployment, created_user_with_password):
        """Test deployment retrieval via session token"""
        token_response = api_client.create_session_token({
            "email": created_user_with_password["email"],
            "password": created_user_with_password["password"],
        })
        assert token_response.status_code == 200
        token = token_response.json()["token"]

        response = api_client.get_deployment_with_session(created_deployment, token)

        assert response.status_code == 200
        data = response.json()
        assert data["id"] == created_deployment

    def test_get_deployment_not_found(self, api_client, sample_uuid):
        """Test getting non-existent deployment returns 404"""
        response = api_client.get_deployment(sample_uuid)

        assert response.status_code == 404

    def test_update_deployment_success(
        self,
        api_client,
        created_deployment,
        sample_openai_connection_data,
    ):
        """Test updating deployment access and connection"""
        connection_resp = api_client.create_connection(sample_openai_connection_data)
        assert connection_resp.status_code == 200
        connection_id = connection_resp.json()["id"]

        try:
            update_response = api_client.update_deployment(created_deployment, {
                "access": "public",
                "connection_id": connection_id,
            })
            assert update_response.status_code == 200
            data = update_response.json()
            assert data["id"] == created_deployment
            assert data["access"] == "public"
            assert data["connection_id"] == connection_id
        finally:
            api_client.delete_connection(connection_id)

    def test_delete_deployment_success(self, api_client, sample_deployment_data, created_azure_openai_connection):
        """Test successful deployment deletion"""
        # Create deployment first
        payload = dict(sample_deployment_data)
        payload["connection_id"] = created_azure_openai_connection
        create_response = api_client.create_deployment(payload)
        assert create_response.status_code == 200
        deployment_id = create_response.json()['id']

        # Delete deployment
        delete_response = api_client.delete_deployment(deployment_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert delete_data["message"] is None

        # Verify deployment is deleted
        get_response = api_client.get_deployment(deployment_id)
        assert get_response.status_code == 404

    def test_delete_deployment_not_found(self, api_client, sample_uuid):
        """Test deleting non-existent deployment returns 404"""
        response = api_client.delete_deployment(sample_uuid)

        assert response.status_code == 404

    def test_create_deployment_invalid_data(self, api_client):
        """Test creating deployment with invalid data fails"""
        invalid_data = {"invalid": ""}  # Missing required fields

        response = api_client.create_deployment(invalid_data)

        assert response.status_code == 422
