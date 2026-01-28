import pytest


class TestUsers:
    def test_create_user_success(self, api_client, sample_user_data):
        """Test successful user creation"""
        response = api_client.create_user(sample_user_data)

        assert response.status_code == 200
        data = response.json()
        assert 'id' in data
        assert data['name'] == sample_user_data['name']
        assert data['email'] == sample_user_data['email']
        assert data['role'] == sample_user_data['role']
        assert data['blocked'] is False

        # Cleanup
        api_client.delete_user(data['id'])

    def test_create_user_duplicate_email(self, api_client, sample_user_data, created_user):
        """Test creating user with duplicate email fails"""
        response = api_client.create_user(sample_user_data)

        assert response.status_code == 409
        assert 'error' in response.json()

    def test_get_user_success(self, api_client, created_user):
        """Test successful user retrieval"""
        response = api_client.get_user(created_user)

        assert response.status_code == 200
        data = response.json()
        assert data['id'] == created_user
        assert 'name' in data
        assert 'email' in data

    def test_get_user_self_via_session(self, api_client, created_user_with_password):
        """Test user retrieval via session token"""
        token_response = api_client.create_session_token({
            "email": created_user_with_password["email"],
            "password": created_user_with_password["password"],
        })
        assert token_response.status_code == 200
        token = token_response.json()["token"]

        response = api_client.get_user_with_session(created_user_with_password["id"], token)

        assert response.status_code == 200
        data = response.json()
        assert data["id"] == created_user_with_password["id"]
        assert data["email"] == created_user_with_password["email"]

    def test_get_current_user_success(self, api_client, created_user_with_password):
        """Test current user retrieval via session token"""
        token_response = api_client.create_session_token({
            "email": created_user_with_password["email"],
            "password": created_user_with_password["password"],
        })
        assert token_response.status_code == 200
        token = token_response.json()["token"]

        response = api_client.get_current_user(token)

        assert response.status_code == 200
        data = response.json()
        assert data["id"] == created_user_with_password["id"]
        assert data["email"] == created_user_with_password["email"]

    def test_get_user_not_found(self, api_client, sample_uuid):
        """Test getting non-existent user returns 404"""
        response = api_client.get_user(sample_uuid)

        assert response.status_code == 404

    def test_delete_user_success(self, api_client, sample_user_data):
        """Test successful user deletion"""
        # Create user first
        create_response = api_client.create_user(sample_user_data)
        assert create_response.status_code == 200
        user_id = create_response.json()['id']

        # Delete user
        delete_response = api_client.delete_user(user_id)
        assert delete_response.status_code == 200
        delete_data = delete_response.json()
        assert delete_data["success"] is True
        assert str(user_id) in delete_data["message"]

        # Verify user is deleted
        get_response = api_client.get_user(user_id)
        assert get_response.status_code == 404

    def test_delete_user_not_found(self, api_client):
        """Test deleting non-existent user returns 404"""
        response = api_client.delete_user("07849650-088f-43ba-9062-757b85c000e1")

        assert response.status_code == 404

    def test_create_user_invalid_data(self, api_client):
        """Test creating user with invalid data fails"""
        invalid_data = {"email": ""}  # Missing required fields

        response = api_client.create_user(invalid_data)

        assert response.status_code == 422
