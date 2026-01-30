import requests
import json
from typing import Dict, Any, Optional
from config import Config


class APIClient:
    def __init__(self):
        self.base_url = Config.BASE_URL
        self.headers = Config.get_headers()
        self.timeout = Config.TIMEOUT

    def _make_request(
        self,
        method: str,
        endpoint: str,
        data: Optional[Dict] = None,
        headers: Optional[Dict[str, str]] = None,
        params: Optional[Dict[str, Any]] = None,
        use_default_headers: bool = True,
    ) -> requests.Response:
        """Make HTTP request to the API"""
        url = f"{self.base_url}{endpoint}"

        kwargs = {
            'headers': self.headers.copy() if use_default_headers else {},
            'timeout': self.timeout
        }

        if headers:
            kwargs['headers'].update(headers)

        if params:
            kwargs['params'] = params

        if data is not None:
            kwargs['json'] = data

        response = requests.request(method, url, **kwargs)
        return response

    # User endpoints
    def get_user(self, user_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/user/{user_id}')

    def create_user(self, user_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/user', user_data)

    def delete_user(self, user_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/user/{user_id}')

    def get_user_with_session(self, user_id: int, session_token: str) -> requests.Response:
        headers = {"X-LLMur-Session": session_token}
        return self._make_request('GET', f'/admin/user/{user_id}', headers=headers, use_default_headers=False)

    def get_current_user(self, session_token: str) -> requests.Response:
        headers = {"X-LLMur-Session": session_token}
        return self._make_request('GET', '/admin/user/me', headers=headers, use_default_headers=False)

    # Project endpoints
    def get_project(self, project_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/project/{project_id}')

    def get_project_with_session(self, project_id: int, session_token: str) -> requests.Response:
        headers = {"X-LLMur-Session": session_token}
        return self._make_request('GET', f'/admin/project/{project_id}', headers=headers, use_default_headers=False)

    def create_project(self, project_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/project', project_data)

    def delete_project(self, project_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/project/{project_id}')

    # Session token endpoints
    def create_session_token(self, session_token_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/session-token', session_token_data)

    # Project invite code endpoints
    def create_project_invite_code(self, invite_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/project-invite-code', invite_data)

    def get_project_invite_code(self, invite_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/project-invite-code/{invite_id}')

    def delete_project_invite_code(self, invite_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/project-invite-code/{invite_id}')

    # Membership endpoints
    def get_membership(self, membership_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/membership/{membership_id}')

    def create_membership(self, membership_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/membership', membership_data)

    def delete_membership(self, membership_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/membership/{membership_id}')

    def search_memberships(
        self,
        user_id: Optional[str] = None,
        project_id: Optional[str] = None,
    ) -> requests.Response:
        params = {}
        if user_id:
            params["user_id"] = user_id
        if project_id:
            params["project_id"] = project_id
        return self._make_request('GET', '/admin/membership', params=params if params else None)

    # Connections endpoints
    def get_connection(self, connection_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/connection/{connection_id}')

    def create_connection(self, connection_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/connection', connection_data)

    def delete_connection(self, connection_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/connection/{connection_id}')

    def list_connections(self) -> requests.Response:
        return self._make_request('GET', '/admin/connection')

    # Deployments endpoints
    def get_deployment(self, deployment_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/deployment/{deployment_id}')

    def get_deployment_with_session(self, deployment_id: int, session_token: str) -> requests.Response:
        headers = {"X-LLMur-Session": session_token}
        return self._make_request('GET', f'/admin/deployment/{deployment_id}', headers=headers, use_default_headers=False)

    def create_deployment(self, deployment_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/deployment', deployment_data)

    def delete_deployment(self, deployment_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/deployment/{deployment_id}')

    # Virtual keys endpoints
    def get_virtual_key(self, virtual_key_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/virtual-key/{virtual_key_id}')

    def create_virtual_key(self, virtual_key_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/virtual-key', virtual_key_data)

    def delete_virtual_key(self, virtual_key_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/virtual-key/{virtual_key_id}')

    def search_virtual_keys(self, project_id: Optional[str] = None) -> requests.Response:
        params = {"project_id": project_id} if project_id else None
        return self._make_request('GET', '/admin/virtual-key', params=params)

    # Connection <-> Deployment maps endpoint
    def get_connection_deployment_map(self, map_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/connection-deployment/{map_id}')

    def create_connection_deployment_map(self, map_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/connection-deployment', map_data)

    def delete_connection_deployment_map(self, map_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/connection-deployment/{map_id}')

    # Virtual Key <-> Deployment maps endpoint
    def get_virtual_key_deployment_map(self, map_id: int) -> requests.Response:
        return self._make_request('GET', f'/admin/virtual-key-deployment/{map_id}')

    def create_virtual_key_deployment_map(self, map_data: Dict[str, Any]) -> requests.Response:
        return self._make_request('POST', '/admin/virtual-key-deployment', map_data)

    def delete_virtual_key_deployment_map(self, map_id: int) -> requests.Response:
        return self._make_request('DELETE', f'/admin/virtual-key-deployment/{map_id}')

    def search_virtual_key_deployment_maps(
        self,
        virtual_key_id: Optional[str] = None,
        deployment_id: Optional[str] = None,
    ) -> requests.Response:
        params = {}
        if virtual_key_id:
            params["virtual_key_id"] = virtual_key_id
        if deployment_id:
            params["deployment_id"] = deployment_id
        return self._make_request('GET', '/admin/virtual-key-deployment', params=params if params else None)

    # Graph - Debug only
    def get_graph(self, key: str, deployment: str) -> requests.Response:
        return self._make_request('GET', f'/admin/graph/{key}/{deployment}')

    # OpenAI-compatible endpoints
    def create_chat_completion(self, payload: Dict[str, Any], bearer_token: str) -> requests.Response:
        headers = {"Authorization": f"Bearer {bearer_token}"}
        return self._make_request('POST', '/v1/chat/completions', payload, headers=headers)

    def create_embeddings(self, payload: Dict[str, Any], bearer_token: str) -> requests.Response:
        headers = {"Authorization": f"Bearer {bearer_token}"}
        return self._make_request('POST', '/v1/embeddings', payload, headers=headers)

    def create_responses(self, payload: Dict[str, Any], bearer_token: str) -> requests.Response:
        headers = {"Authorization": f"Bearer {bearer_token}"}
        return self._make_request('POST', '/v1/responses', payload, headers=headers)
