import pytest


class TestProjectInviteCodes:
    def test_create_project_invite_code_success(self, api_client, created_project):
        """Test successful invite code creation"""
        payload = {
            "project_id": created_project,
            "role": "guest",
        }

        response = api_client.create_project_invite_code(payload)

        assert response.status_code == 200
        data = response.json()
        assert "id" in data
        assert data["project_id"] == created_project
        assert "code" in data
        assert data["role"] == "guest"
        assert data["valid"] is True
        assert data["valid_until"] is None

        invite_id = data["id"]

        get_response = api_client.get_project_invite_code(invite_id)
        assert get_response.status_code == 200
        get_data = get_response.json()
        assert get_data["id"] == invite_id
        assert get_data["project_id"] == created_project
        assert get_data["code"] == data["code"]
        assert get_data["role"] == "guest"

        delete_response = api_client.delete_project_invite_code(invite_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert delete_data["message"] is None

    def test_create_project_invite_code_invalid_payload(self, api_client):
        """Test invite code creation with invalid payload fails"""
        response = api_client.create_project_invite_code({})

        assert response.status_code == 422

    def test_create_project_invite_code_missing_body(self, api_client):
        """Test invite code creation with no body fails"""
        response = api_client.create_project_invite_code(None)

        assert response.status_code == 400

    def test_delete_project_invite_code_not_found(self, api_client, sample_uuid):
        """Test deleting missing invite code returns not found"""
        response = api_client.delete_project_invite_code(sample_uuid)

        assert response.status_code == 404

    def test_get_project_invite_code_not_found(self, api_client, sample_uuid):
        """Test getting missing invite code returns not found"""
        response = api_client.get_project_invite_code(sample_uuid)

        assert response.status_code == 404
