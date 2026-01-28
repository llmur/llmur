import pytest


class TestProjects:
    def test_create_project_success(self, api_client, sample_project_data):
        """Test successful project creation"""
        response = api_client.create_project(sample_project_data)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['name'] == sample_project_data['name']

        # Cleanup
        api_client.delete_project(data['id'])

    def test_get_project_success(self, api_client, created_project):
        """Test successful project retrieval"""
        response = api_client.get_project(created_project)

        assert response.status_code == 200
        data = response.json()
        assert data['id'] == created_project
        assert 'name' in data

    def test_get_project_member_via_session(self, api_client, created_project, created_user_with_password):
        """Test project retrieval as a project member via session token"""
        membership_resp = api_client.create_membership({
            "user_id": created_user_with_password["id"],
            "project_id": created_project,
            "role": "guest",
        })
        assert membership_resp.status_code == 200
        membership_id = membership_resp.json()["id"]

        try:
            token_response = api_client.create_session_token({
                "email": created_user_with_password["email"],
                "password": created_user_with_password["password"],
            })
            assert token_response.status_code == 200
            token = token_response.json()["token"]

            response = api_client.get_project_with_session(created_project, token)

            assert response.status_code == 200
            data = response.json()
            assert data["id"] == created_project
        finally:
            api_client.delete_membership(membership_id)

    def test_get_project_not_found(self, api_client, sample_uuid):
        """Test getting non-existent project returns 404"""
        response = api_client.get_project(sample_uuid)

        assert response.status_code == 404

    def test_delete_project_success(self, api_client, sample_project_data):
        """Test successful project deletion"""
        # Create project first
        create_response = api_client.create_project(sample_project_data)
        assert create_response.status_code == 200
        project_id = create_response.json()['id']

        # Delete project
        delete_response = api_client.delete_project(project_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert delete_data["message"] is None

        # Verify project is deleted
        get_response = api_client.get_project(project_id)
        assert get_response.status_code == 404

    def test_delete_project_not_found(self, api_client, sample_uuid):
        """Test deleting non-existent project returns 404"""
        response = api_client.delete_project(sample_uuid)

        assert response.status_code == 404

    def test_create_project_invalid_data(self, api_client):
        """Test creating project with invalid data fails"""
        invalid_data = {"invalid": ""}  # Missing required fields

        response = api_client.create_project(invalid_data)

        assert response.status_code == 422
