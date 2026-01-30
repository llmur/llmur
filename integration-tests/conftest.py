import uuid
import pytest
from api_client import APIClient
from config import Config


@pytest.fixture(scope="session")
def api_client():
    """Shared API client for all tests"""
    return APIClient()

@pytest.fixture
def sample_uuid():
    """Sample uuid string. Used for invalid id validations"""
    return "07849650-088f-43ba-9062-757b85c000e1"


@pytest.fixture
def sample_user_data():
    """Sample user data for testing"""
    return {
        "name": "Test User",
        "email": "test@example.com",
        "password": "Hello1234",
        "role": "admin"
    }


@pytest.fixture
def sample_project_data():
    """Sample project data for testing"""
    return {
        "name": "Test Project"
    }


@pytest.fixture
def sample_membership_data():
    """Sample membership data for testing"""
    return {
        "user_id": 1,
        "project_id": 1,
        "role": "admin"
    }


@pytest.fixture
def sample_azure_openai_connection_data():
    """Sample connection data for testing"""
    return {
        "provider": "azure/openai",
        "deployment_name": "gpt-4o",
        "api_endpoint": "https://aoairesource.openai.azure.com",
        "api_key": "dummy-key",
        "api_version": "v1"
    }

@pytest.fixture
def sample_openai_connection_data():
    """Sample OpenAI connection data for testing"""
    return {
        "provider": "openai/v1",
        "model": "gpt-4o-mini",
        "api_endpoint": "https://api.openai.com",
        "api_key": "sk-test-openai",
    }

@pytest.fixture
def sample_gemini_connection_data():
    """Sample Gemini connection data for testing"""
    return {
        "provider": "gemini",
        "model": "gemini-1.5-flash",
        "api_endpoint": "https://generativelanguage.googleapis.com",
        "api_key": "test-gemini-key",
        "api_version": "v1beta",
    }


@pytest.fixture
def sample_deployment_data():
    """Sample deployment data for testing"""
    return {
        "name": "my-deployment",
        "access": "public"
    }

@pytest.fixture
def sample_virtual_key_data():
    """Sample deployment data for testing"""
    return {
        "alias": "my-super-key",
        "project_id": 1
    }

@pytest.fixture
def sample_connection_deployment_map_data():
    """Sample connection <-> deployment map data for testing"""
    return {
        "connection_id": 1,
        "deployment_id": 1,
    }


@pytest.fixture
def sample_virtual_key_deployment_map_data():
    """Sample virtual_key <-> deployment map data for testing"""
    return {
        "virtual_key_id": 1,
        "deployment_id": 1,
    }


@pytest.fixture
def created_user(api_client, sample_user_data):
    """Create a user for testing and clean up after"""
    response = api_client.create_user(sample_user_data)
    assert response.status_code == 200
    user_id = response.json()['id']

    yield user_id

    # Cleanup
    api_client.delete_user(user_id)


@pytest.fixture
def created_user_with_password(api_client):
    """Create a user with a known password for auth tests and clean up after"""
    password = "Hello1234"
    email = f"test-{uuid.uuid4().hex[:8]}@example.com"
    payload = {
        "name": "Auth User",
        "email": email,
        "password": password,
        "role": "admin",
    }
    response = api_client.create_user(payload)
    assert response.status_code == 200
    user_id = response.json()['id']

    yield {
        "id": user_id,
        "email": email,
        "password": password,
    }

    api_client.delete_user(user_id)


@pytest.fixture
def created_project(api_client, sample_project_data):
    """Create a project for testing and clean up after"""
    response = api_client.create_project(sample_project_data)
    assert response.status_code == 200
    project_id = response.json()['id']

    yield project_id

    # Cleanup
    api_client.delete_project(project_id)

@pytest.fixture
def created_azure_openai_connection(api_client, sample_azure_openai_connection_data):
    """Create a connection for testing and clean up after"""
    response = api_client.create_connection(sample_azure_openai_connection_data)
    assert response.status_code == 200
    connection_id = response.json()['id']

    yield connection_id

    # Cleanup
    api_client.delete_connection(connection_id)


@pytest.fixture
def created_deployment(api_client, sample_deployment_data):
    """Create a deployment for testing and clean up after"""
    response = api_client.create_deployment(sample_deployment_data)
    assert response.status_code == 200
    deployment_id = response.json()['id']

    yield deployment_id

    # Cleanup
    api_client.delete_deployment(deployment_id)

@pytest.fixture
def created_connection_deployment_map(api_client, created_azure_openai_connection, created_deployment):
    """Create a connection <-> deployment map for testing and clean up after"""
    payload = {
        'connection_id': created_azure_openai_connection,
        'deployment_id': created_deployment
    }

    cd_map = api_client.create_connection_deployment_map(payload)
    assert cd_map.status_code == 200
    cd_map_id = cd_map.json()['id']

    yield cd_map_id

    # Cleanup
    api_client.delete_connection_deployment_map(cd_map_id)



@pytest.fixture
def created_virtual_key(api_client, sample_virtual_key_data, created_project):
    """Create a deployment for testing and clean up after"""
    payload = sample_virtual_key_data
    sample_virtual_key_data['project_id'] = created_project

    response = api_client.create_virtual_key(payload)
    assert response.status_code == 200
    key_id = response.json()['id']

    yield key_id

    # Cleanup
    api_client.delete_virtual_key(key_id)


@pytest.fixture
def created_virtual_key_deployment_map(api_client, created_virtual_key, created_deployment):
    """Create a Virtual Key <-> Deployment map for testing and clean up after"""
    payload = {
        'virtual_key_id': created_virtual_key,
        'deployment_id': created_deployment
    }

    vkd_map = api_client.create_virtual_key_deployment_map(payload)
    assert vkd_map.status_code == 200
    vkd_map_id = vkd_map.json()['id']

    yield vkd_map_id

    # Cleanup
    api_client.delete_virtual_key_deployment_map(vkd_map_id)


def _provider_ready(values):
    return all(value is not None and str(value).strip() for value in values)


def _create_provider_setup(api_client, project_id, deployment_name, connection_payload):
    deployment_resp = api_client.create_deployment({
        "name": deployment_name,
        "access": "public",
    })
    assert deployment_resp.status_code == 200
    deployment_id = deployment_resp.json()['id']

    connection_resp = api_client.create_connection(connection_payload)
    assert connection_resp.status_code == 200
    connection_id = connection_resp.json()['id']

    map_resp = api_client.create_connection_deployment_map({
        "connection_id": connection_id,
        "deployment_id": deployment_id,
    })
    assert map_resp.status_code == 200
    connection_deployment_id = map_resp.json()['id']

    key_resp = api_client.create_virtual_key({
        "project_id": project_id,
    })
    assert key_resp.status_code == 200
    key_payload = key_resp.json()
    virtual_key_id = key_payload['id']
    virtual_key = key_payload['key']

    vkd_resp = api_client.create_virtual_key_deployment_map({
        "virtual_key_id": virtual_key_id,
        "deployment_id": deployment_id,
    })
    assert vkd_resp.status_code == 200
    virtual_key_deployment_id = vkd_resp.json()['id']

    return {
        "deployment_id": deployment_id,
        "deployment_name": deployment_name,
        "connection_id": connection_id,
        "connection_deployment_id": connection_deployment_id,
        "virtual_key_id": virtual_key_id,
        "virtual_key": virtual_key,
        "virtual_key_deployment_id": virtual_key_deployment_id,
    }


def _cleanup_provider_setup(api_client, setup):
    api_client.delete_virtual_key_deployment_map(setup["virtual_key_deployment_id"])
    api_client.delete_virtual_key(setup["virtual_key_id"])
    api_client.delete_connection_deployment_map(setup["connection_deployment_id"])
    api_client.delete_deployment(setup["deployment_id"])
    api_client.delete_connection(setup["connection_id"])


@pytest.fixture
def openai_chat_provider_setup(api_client, created_project):
    if not _provider_ready([Config.OPENAI_API_KEY, Config.OPENAI_CHAT_COMPLETIONS_MODEL]):
        pytest.skip("OPENAI_API_KEY/OPENAI_CHAT_COMPLETIONS_MODEL not configured")

    deployment_name = f"openai-chat-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "openai/v1",
        "model": Config.OPENAI_CHAT_COMPLETIONS_MODEL,
        "api_endpoint": Config.OPENAI_BASE_URL,
        "api_key": Config.OPENAI_API_KEY,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def openai_embeddings_provider_setup(api_client, created_project):
    if not _provider_ready([Config.OPENAI_API_KEY, Config.OPENAI_EMBEDDINGS_MODEL]):
        pytest.skip("OPENAI_API_KEY/OPENAI_EMBEDDINGS_MODEL not configured")

    deployment_name = f"openai-emb-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "openai/v1",
        "model": Config.OPENAI_EMBEDDINGS_MODEL,
        "api_endpoint": Config.OPENAI_BASE_URL,
        "api_key": Config.OPENAI_API_KEY,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def azure_chat_provider_setup(api_client, created_project):
    if not _provider_ready([
        Config.AZURE_OPENAI_API_KEY,
        Config.AZURE_OPENAI_ENDPOINT,
        Config.AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT,
    ]):
        pytest.skip("Azure OpenAI chat config not fully set")

    deployment_name = f"azure-chat-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "azure/openai",
        "deployment_name": Config.AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT,
        "api_endpoint": Config.AZURE_OPENAI_ENDPOINT,
        "api_key": Config.AZURE_OPENAI_API_KEY,
        "api_version": Config.AZURE_OPENAI_API_VERSION,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def azure_embeddings_provider_setup(api_client, created_project):
    if not _provider_ready([
        Config.AZURE_OPENAI_API_KEY,
        Config.AZURE_OPENAI_ENDPOINT,
        Config.AZURE_OPENAI_EMBEDDINGS_DEPLOYMENT,
    ]):
        pytest.skip("Azure OpenAI embeddings config not fully set")

    deployment_name = f"azure-emb-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "azure/openai",
        "deployment_name": Config.AZURE_OPENAI_EMBEDDINGS_DEPLOYMENT,
        "api_endpoint": Config.AZURE_OPENAI_ENDPOINT,
        "api_key": Config.AZURE_OPENAI_API_KEY,
        "api_version": Config.AZURE_OPENAI_API_VERSION,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def gemini_chat_provider_setup(api_client, created_project):
    if not _provider_ready([
        Config.GEMINI_API_KEY,
        Config.GEMINI_CHAT_COMPLETIONS_MODEL,
    ]):
        pytest.skip("GEMINI_API_KEY/GEMINI_CHAT_COMPLETIONS_MODEL not configured")

    deployment_name = f"gemini-chat-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "gemini",
        "model": Config.GEMINI_CHAT_COMPLETIONS_MODEL,
        "api_endpoint": Config.GEMINI_BASE_URL,
        "api_key": Config.GEMINI_API_KEY,
        "api_version": Config.GEMINI_API_VERSION,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def gemini_embeddings_provider_setup(api_client, created_project):
    if not _provider_ready([
        Config.GEMINI_API_KEY,
        Config.GEMINI_EMBEDDINGS_MODEL,
    ]):
        pytest.skip("GEMINI_API_KEY/GEMINI_EMBEDDINGS_MODEL not configured")

    deployment_name = f"gemini-emb-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "gemini",
        "model": Config.GEMINI_EMBEDDINGS_MODEL,
        "api_endpoint": Config.GEMINI_BASE_URL,
        "api_key": Config.GEMINI_API_KEY,
        "api_version": Config.GEMINI_API_VERSION,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def azure_chat_invalid_provider_setup(api_client, created_project):
    if not _provider_ready([
        Config.AZURE_OPENAI_API_KEY,
        Config.AZURE_OPENAI_ENDPOINT,
        Config.AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT,
    ]):
        pytest.skip("Azure OpenAI chat config not fully set")

    invalid_deployment = f"{Config.AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT}-invalid-{uuid.uuid4().hex[:6]}"
    deployment_name = f"azure-chat-invalid-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "azure/openai",
        "deployment_name": invalid_deployment,
        "api_endpoint": Config.AZURE_OPENAI_ENDPOINT,
        "api_key": Config.AZURE_OPENAI_API_KEY,
        "api_version": Config.AZURE_OPENAI_API_VERSION,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def azure_embeddings_invalid_provider_setup(api_client, created_project):
    if not _provider_ready([
        Config.AZURE_OPENAI_API_KEY,
        Config.AZURE_OPENAI_ENDPOINT,
        Config.AZURE_OPENAI_EMBEDDINGS_DEPLOYMENT,
    ]):
        pytest.skip("Azure OpenAI embeddings config not fully set")

    invalid_deployment = f"{Config.AZURE_OPENAI_EMBEDDINGS_DEPLOYMENT}-invalid-{uuid.uuid4().hex[:6]}"
    deployment_name = f"azure-emb-invalid-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "azure/openai",
        "deployment_name": invalid_deployment,
        "api_endpoint": Config.AZURE_OPENAI_ENDPOINT,
        "api_key": Config.AZURE_OPENAI_API_KEY,
        "api_version": Config.AZURE_OPENAI_API_VERSION,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def gemini_chat_invalid_provider_setup(api_client, created_project):
    if not _provider_ready([
        Config.GEMINI_API_KEY,
        Config.GEMINI_CHAT_COMPLETIONS_MODEL,
    ]):
        pytest.skip("GEMINI_API_KEY/GEMINI_CHAT_COMPLETIONS_MODEL not configured")

    invalid_model = f"{Config.GEMINI_CHAT_COMPLETIONS_MODEL}-invalid-{uuid.uuid4().hex[:6]}"
    deployment_name = f"gemini-chat-invalid-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "gemini",
        "model": invalid_model,
        "api_endpoint": Config.GEMINI_BASE_URL,
        "api_key": Config.GEMINI_API_KEY,
        "api_version": Config.GEMINI_API_VERSION,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def gemini_embeddings_invalid_provider_setup(api_client, created_project):
    if not _provider_ready([
        Config.GEMINI_API_KEY,
        Config.GEMINI_EMBEDDINGS_MODEL,
    ]):
        pytest.skip("GEMINI_API_KEY/GEMINI_EMBEDDINGS_MODEL not configured")

    invalid_model = f"{Config.GEMINI_EMBEDDINGS_MODEL}-invalid-{uuid.uuid4().hex[:6]}"
    deployment_name = f"gemini-emb-invalid-{uuid.uuid4().hex[:8]}"
    setup = _create_provider_setup(api_client, created_project, deployment_name, {
        "provider": "gemini",
        "model": invalid_model,
        "api_endpoint": Config.GEMINI_BASE_URL,
        "api_key": Config.GEMINI_API_KEY,
        "api_version": Config.GEMINI_API_VERSION,
    })

    yield setup

    _cleanup_provider_setup(api_client, setup)


@pytest.fixture
def azure_chat_rate_limited_setup(api_client):
    if not _provider_ready([
        Config.AZURE_OPENAI_API_KEY,
        Config.AZURE_OPENAI_ENDPOINT,
        Config.AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT,
    ]):
        pytest.skip("Azure OpenAI chat config not fully set")

    project_resp = api_client.create_project({
        "name": f"rl-project-{uuid.uuid4().hex[:8]}",
        "request_limits": {"requests_per_day": 1},
    })
    assert project_resp.status_code == 200
    project_id = project_resp.json()["id"]

    deployment_name = f"rl-deployment-{uuid.uuid4().hex[:8]}"
    deployment_resp = api_client.create_deployment({
        "name": deployment_name,
        "access": "public",
        "request_limits": {"requests_per_day": 1},
    })
    assert deployment_resp.status_code == 200
    deployment_id = deployment_resp.json()["id"]

    connection_resp = api_client.create_connection({
        "provider": "azure/openai",
        "deployment_name": Config.AZURE_OPENAI_CHAT_COMPLETIONS_DEPLOYMENT,
        "api_endpoint": Config.AZURE_OPENAI_ENDPOINT,
        "api_key": Config.AZURE_OPENAI_API_KEY,
        "api_version": Config.AZURE_OPENAI_API_VERSION,
    })
    assert connection_resp.status_code == 200
    connection_id = connection_resp.json()["id"]

    map_resp = api_client.create_connection_deployment_map({
        "connection_id": connection_id,
        "deployment_id": deployment_id,
    })
    assert map_resp.status_code == 200
    connection_deployment_id = map_resp.json()["id"]

    key_resp = api_client.create_virtual_key({
        "project_id": project_id,
        "request_limits": {"requests_per_day": 1},
    })
    assert key_resp.status_code == 200
    key_payload = key_resp.json()

    vkd_resp = api_client.create_virtual_key_deployment_map({
        "virtual_key_id": key_payload["id"],
        "deployment_id": deployment_id,
    })
    assert vkd_resp.status_code == 200
    vkd_id = vkd_resp.json()["id"]

    yield {
        "project_id": project_id,
        "deployment_id": deployment_id,
        "deployment_name": deployment_name,
        "connection_id": connection_id,
        "connection_deployment_id": connection_deployment_id,
        "virtual_key_id": key_payload["id"],
        "virtual_key": key_payload["key"],
        "virtual_key_deployment_id": vkd_id,
    }

    api_client.delete_virtual_key_deployment_map(vkd_id)
    api_client.delete_virtual_key(key_payload["id"])
    api_client.delete_connection_deployment_map(connection_deployment_id)
    api_client.delete_deployment(deployment_id)
    api_client.delete_connection(connection_id)
    api_client.delete_project(project_id)
