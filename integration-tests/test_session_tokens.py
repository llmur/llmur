import pytest


class TestSessionTokens:
    def test_create_session_token_success(self, api_client, created_user_with_password):
        """Test creating a session token with valid credentials"""
        payload = {
            "email": created_user_with_password["email"],
            "password": created_user_with_password["password"],
        }

        response = api_client.create_session_token(payload)

        assert response.status_code == 200
        data = response.json()
        assert "token" in data
        assert "info" in data
        assert data["token"]
        assert "id" in data["info"]
        assert data["info"]["revoked"] is False
        assert data["info"]["user_id"] == created_user_with_password["id"]

    def test_create_session_token_invalid_password(self, api_client, created_user_with_password):
        """Test creating a session token with invalid password"""
        payload = {
            "email": created_user_with_password["email"],
            "password": "wrong-password",
        }

        response = api_client.create_session_token(payload)

        assert response.status_code == 401

    def test_create_session_token_unknown_email(self, api_client):
        """Test creating a session token with unknown email"""
        payload = {
            "email": "missing@example.com",
            "password": "Hello1234",
        }

        response = api_client.create_session_token(payload)

        assert response.status_code == 401

    def test_create_session_token_invalid_payload(self, api_client):
        """Test creating a session token with invalid payload"""
        response = api_client.create_session_token({"email": ""})

        assert response.status_code == 422
