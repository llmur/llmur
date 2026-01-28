import pytest


class TestMemberships:
    def test_create_membership_success(self, api_client, created_user, created_project):
        """Test successful membership creation"""
        membership_data = {
            "user_id": created_user,
            "project_id": created_project,
            "role": "admin"
        }

        response = api_client.create_membership(membership_data)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['user_id'] == created_user
        assert data['project_id'] == created_project
        assert data['role'] == 'admin'

        # Cleanup
        api_client.delete_membership(data['id'])

    def test_get_membership_success(self, api_client, created_user, created_project):
        """Test successful membership retrieval"""
        # Create membership first
        membership_data = {
            "user_id": created_user,
            "project_id": created_project,
            "role": "admin"
        }
        create_response = api_client.create_membership(membership_data)
        assert create_response.status_code == 200
        membership_id = create_response.json()['id']

        # Get membership
        response = api_client.get_membership(membership_id)

        assert response.status_code == 200
        data = response.json()
        assert data['id'] == membership_id
        assert data['user_id'] == created_user
        assert data['project_id'] == created_project
        assert data['role'] == 'admin'

        # Cleanup
        api_client.delete_membership(membership_id)

    def test_get_membership_not_found(self, api_client, sample_uuid):
        """Test getting non-existent membership returns 404"""
        response = api_client.get_membership(sample_uuid)

        assert response.status_code == 404

    def test_delete_membership_success(self, api_client, created_user, created_project):
        """Test successful membership deletion"""
        # Create membership first
        membership_data = {
            "user_id": created_user,
            "project_id": created_project,
            "role": "admin"
        }
        create_response = api_client.create_membership(membership_data)
        assert create_response.status_code == 200
        membership_id = create_response.json()['id']

        # Delete membership
        delete_response = api_client.delete_membership(membership_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert delete_data["message"] is None

        # Verify membership is deleted
        get_response = api_client.get_membership(membership_id)
        assert get_response.status_code == 404

    def test_delete_membership_not_found(self, api_client, sample_uuid):
        """Test deleting non-existent membership returns 404"""
        response = api_client.delete_membership(sample_uuid)

        assert response.status_code == 404

    def test_create_membership_invalid_user(self, api_client, created_project, sample_uuid):
        """Test creating membership with invalid user fails"""
        membership_data = {
            "user_id": sample_uuid,  # Non-existent user
            "project_id": created_project,
            "role": "admin"
        }

        response = api_client.create_membership(membership_data)

        assert response.status_code == 404

    def test_create_membership_invalid_project(self, api_client, created_user, sample_uuid):
        """Test creating membership with invalid project fails"""
        membership_data = {
            "user_id": created_user,
            "project_id": sample_uuid,  # Non-existent project
            "role": "admin"
        }

        response = api_client.create_membership(membership_data)

        assert response.status_code == 404

    def test_create_duplicate_membership(self, api_client, created_user, created_project):
        """Test creating duplicate membership fails"""
        membership_data = {
            "user_id": created_user,
            "project_id": created_project,
            "role": "admin"
        }

        # Create first membership
        first_response = api_client.create_membership(membership_data)
        assert first_response.status_code == 200
        first_membership_id = first_response.json()['id']

        # Try to create duplicate
        second_response = api_client.create_membership(membership_data)
        assert second_response.status_code == 409
        assert 'error' in second_response.json()

        # Cleanup
        api_client.delete_membership(first_membership_id)

    def test_search_memberships_by_project(self, api_client, created_user, created_project):
        """Test searching memberships by project returns expected entries"""
        membership_data = {
            "user_id": created_user,
            "project_id": created_project,
            "role": "admin"
        }

        create_response = api_client.create_membership(membership_data)
        assert create_response.status_code == 200
        membership_id = create_response.json()['id']

        response = api_client.search_memberships(project_id=created_project)
        assert response.status_code == 200
        data = response.json()
        assert 'memberships' in data
        assert any(item['id'] == membership_id for item in data['memberships'])

        api_client.delete_membership(membership_id)
