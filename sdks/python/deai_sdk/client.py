"""
Main DeAI SDK client for interacting with the DeAI computation network.
"""

import asyncio
import time
from typing import Optional, Dict, Any, List, Union
from urllib.parse import urljoin

import httpx
from pydantic import ValidationError as PydanticValidationError

from .exceptions import (
    DeAIError,
    AuthenticationError,
    TaskError,
    NetworkError,
    ValidationError,
)
from .types import (
    TaskSubmissionRequest,
    TaskResponse,
    TaskResult,
    TaskStatus,
    NetworkStats,
    NodeInfo,
    UserProfile,
    ApiKey,
    PaginatedResponse,
    AuthResponse,
)
from .utils import validate_task_request, format_near_amount


class DeAIClient:
    """
    Main DeAI SDK client for interacting with the DeAI computation network.
    
    This client provides methods for authentication, task submission and management,
    network information retrieval, and user account management.
    
    Example:
        ```python
        from deai_sdk import DeAIClient
        
        # Initialize client
        client = DeAIClient(api_url="https://api.deai.org")
        
        # Authenticate
        await client.login("username", "password")
        
        # Submit a task
        task = await client.submit_task({
            "task_type": "text_generation",
            "model_name": "gpt2",
            "input_data": "The future of AI is",
            "max_cost": "0.1"
        })
        
        # Wait for completion
        result = await client.wait_for_completion(task.id)
        print(result.output)
        ```
    """
    
    def __init__(
        self,
        api_url: str = "https://api.deai.org",
        timeout: float = 30.0,
        retries: int = 3,
        retry_delay: float = 1.0,
    ):
        """
        Initialize the DeAI client.
        
        Args:
            api_url: Base URL for the DeAI API
            timeout: Request timeout in seconds
            retries: Number of retries for failed requests
            retry_delay: Delay between retries in seconds
        """
        self.api_url = api_url.rstrip("/")
        self.timeout = timeout
        self.retries = retries
        self.retry_delay = retry_delay
        self._access_token: Optional[str] = None
        
        # Create HTTP client
        self._client = httpx.AsyncClient(
            base_url=self.api_url,
            timeout=self.timeout,
            headers={
                "Content-Type": "application/json",
                "User-Agent": "DeAI-SDK-Python/0.1.0",
            }
        )
    
    async def __aenter__(self):
        """Async context manager entry."""
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()
    
    async def close(self):
        """Close the HTTP client."""
        await self._client.aclose()
    
    def _get_headers(self) -> Dict[str, str]:
        """Get headers for authenticated requests."""
        headers = {}
        if self._access_token:
            headers["Authorization"] = f"Bearer {self._access_token}"
        return headers
    
    async def _request(
        self,
        method: str,
        endpoint: str,
        data: Optional[Dict[str, Any]] = None,
        params: Optional[Dict[str, Any]] = None,
        authenticated: bool = True,
    ) -> Any:
        """
        Make an HTTP request with retry logic.
        
        Args:
            method: HTTP method
            endpoint: API endpoint
            data: Request body data
            params: Query parameters
            authenticated: Whether to include authentication headers
            
        Returns:
            Response data
            
        Raises:
            DeAIError: For various API errors
        """
        url = endpoint if endpoint.startswith("http") else f"/api/v1{endpoint}"
        headers = self._get_headers() if authenticated else {}
        
        for attempt in range(self.retries + 1):
            try:
                if method.upper() == "GET":
                    response = await self._client.get(
                        url, params=params, headers=headers
                    )
                elif method.upper() == "POST":
                    response = await self._client.post(
                        url, json=data, params=params, headers=headers
                    )
                elif method.upper() == "PUT":
                    response = await self._client.put(
                        url, json=data, params=params, headers=headers
                    )
                elif method.upper() == "DELETE":
                    response = await self._client.delete(
                        url, params=params, headers=headers
                    )
                else:
                    raise DeAIError(f"Unsupported HTTP method: {method}")
                
                # Handle response
                if response.status_code == 200:
                    return response.json()
                elif response.status_code == 201:
                    return response.json()
                elif response.status_code == 204:
                    return None
                elif response.status_code == 401:
                    raise AuthenticationError("Authentication failed")
                elif response.status_code == 403:
                    raise AuthenticationError("Access forbidden")
                elif response.status_code == 404:
                    raise DeAIError("Resource not found")
                elif response.status_code == 429:
                    raise DeAIError("Rate limited")
                elif response.status_code >= 500:
                    raise NetworkError("Server error")
                else:
                    error_data = response.json() if response.content else {}
                    message = error_data.get("message", f"HTTP {response.status_code}")
                    raise DeAIError(message)
                    
            except httpx.TimeoutException:
                if attempt == self.retries:
                    raise NetworkError("Request timeout")
                await asyncio.sleep(self.retry_delay * (2 ** attempt))
                
            except httpx.RequestError as e:
                if attempt == self.retries:
                    raise NetworkError(f"Request failed: {e}")
                await asyncio.sleep(self.retry_delay * (2 ** attempt))
    
    # Authentication Methods
    
    async def login(self, username: str, password: str) -> UserProfile:
        """
        Login with username and password.
        
        Args:
            username: Username
            password: Password
            
        Returns:
            User profile information
            
        Raises:
            AuthenticationError: If login fails
        """
        try:
            response_data = await self._request(
                "POST", 
                "/auth/login",
                data={"username": username, "password": password},
                authenticated=False
            )
            
            auth_response = AuthResponse(**response_data)
            self._access_token = auth_response.access_token
            return auth_response.user
            
        except Exception as e:
            raise AuthenticationError(f"Login failed: {e}")
    
    async def login_with_near(
        self,
        account_id: str,
        public_key: str,
        signature: str,
        message: str,
    ) -> UserProfile:
        """
        Login with Near wallet signature.
        
        Args:
            account_id: Near account ID
            public_key: Public key used for signing
            signature: Signature of the message
            message: Original message that was signed
            
        Returns:
            User profile information
            
        Raises:
            AuthenticationError: If login fails
        """
        try:
            response_data = await self._request(
                "POST",
                "/auth/near-login",
                data={
                    "account_id": account_id,
                    "public_key": public_key,
                    "signature": signature,
                    "message": message,
                },
                authenticated=False
            )
            
            auth_response = AuthResponse(**response_data)
            self._access_token = auth_response.access_token
            return auth_response.user
            
        except Exception as e:
            raise AuthenticationError(f"Near wallet login failed: {e}")
    
    def set_api_key(self, api_key: str) -> None:
        """
        Set API key for authentication.
        
        Args:
            api_key: API key string
        """
        self._access_token = api_key
    
    def logout(self) -> None:
        """Logout and clear authentication."""
        self._access_token = None
    
    def is_authenticated(self) -> bool:
        """Check if client is authenticated."""
        return self._access_token is not None
    
    # Task Management Methods
    
    async def submit_task(self, request: Dict[str, Any]) -> TaskResponse:
        """
        Submit a new AI task.
        
        Args:
            request: Task submission request data
            
        Returns:
            Task response with task information
            
        Raises:
            TaskError: If task submission fails
            ValidationError: If request data is invalid
        """
        try:
            # Validate request
            task_request = TaskSubmissionRequest(**request)
            validate_task_request(task_request)
            
            response_data = await self._request(
                "POST", "/tasks", data=task_request.dict()
            )
            
            return TaskResponse(**response_data)
            
        except PydanticValidationError as e:
            raise ValidationError(f"Invalid task request: {e}")
        except Exception as e:
            raise TaskError(f"Failed to submit task: {e}")
    
    async def get_task(self, task_id: str) -> TaskResponse:
        """
        Get task information.
        
        Args:
            task_id: Task ID
            
        Returns:
            Task information
            
        Raises:
            TaskError: If task retrieval fails
        """
        try:
            response_data = await self._request("GET", f"/tasks/{task_id}")
            return TaskResponse(**response_data)
        except Exception as e:
            raise TaskError(f"Failed to get task {task_id}: {e}")
    
    async def get_task_result(self, task_id: str) -> TaskResult:
        """
        Get task result.
        
        Args:
            task_id: Task ID
            
        Returns:
            Task result
            
        Raises:
            TaskError: If result retrieval fails
        """
        try:
            response_data = await self._request("GET", f"/tasks/{task_id}/result")
            return TaskResult(**response_data)
        except Exception as e:
            raise TaskError(f"Failed to get task result {task_id}: {e}")
    
    async def list_tasks(
        self,
        page: int = 1,
        limit: int = 20,
        status: Optional[TaskStatus] = None,
    ) -> PaginatedResponse[TaskResponse]:
        """
        List user's tasks with pagination.
        
        Args:
            page: Page number (1-based)
            limit: Number of items per page
            status: Filter by task status
            
        Returns:
            Paginated list of tasks
            
        Raises:
            TaskError: If task listing fails
        """
        try:
            params = {"page": page, "limit": limit}
            if status:
                params["status"] = status.value
                
            response_data = await self._request("GET", "/tasks", params=params)
            return PaginatedResponse[TaskResponse](**response_data)
            
        except Exception as e:
            raise TaskError(f"Failed to list tasks: {e}")
    
    async def cancel_task(self, task_id: str) -> TaskResponse:
        """
        Cancel a task.
        
        Args:
            task_id: Task ID
            
        Returns:
            Updated task information
            
        Raises:
            TaskError: If task cancellation fails
        """
        try:
            response_data = await self._request("POST", f"/tasks/{task_id}/cancel")
            return TaskResponse(**response_data)
        except Exception as e:
            raise TaskError(f"Failed to cancel task {task_id}: {e}")
    
    async def wait_for_completion(
        self,
        task_id: str,
        timeout: float = 300.0,
        poll_interval: float = 5.0,
    ) -> TaskResult:
        """
        Wait for task completion.
        
        Args:
            task_id: Task ID
            timeout: Maximum time to wait in seconds
            poll_interval: Polling interval in seconds
            
        Returns:
            Task result when completed
            
        Raises:
            TaskError: If task fails or times out
        """
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            task = await self.get_task(task_id)
            
            if task.status == TaskStatus.COMPLETED:
                return await self.get_task_result(task_id)
            elif task.status in [TaskStatus.FAILED, TaskStatus.CANCELLED]:
                raise TaskError(f"Task {task_id} {task.status.value}")
            
            await asyncio.sleep(poll_interval)
        
        raise TaskError(f"Task {task_id} timed out")
    
    # Network Information Methods
    
    async def get_network_stats(self) -> NetworkStats:
        """
        Get network statistics.
        
        Returns:
            Network statistics
            
        Raises:
            NetworkError: If stats retrieval fails
        """
        try:
            response_data = await self._request("GET", "/network/stats", authenticated=False)
            return NetworkStats(**response_data)
        except Exception as e:
            raise NetworkError(f"Failed to get network stats: {e}")
    
    async def list_nodes(self) -> List[NodeInfo]:
        """
        List active nodes.
        
        Returns:
            List of active nodes
            
        Raises:
            NetworkError: If node listing fails
        """
        try:
            response_data = await self._request("GET", "/nodes", authenticated=False)
            return [NodeInfo(**node) for node in response_data]
        except Exception as e:
            raise NetworkError(f"Failed to list nodes: {e}")
    
    async def get_node(self, node_id: str) -> NodeInfo:
        """
        Get specific node information.
        
        Args:
            node_id: Node ID
            
        Returns:
            Node information
            
        Raises:
            NetworkError: If node retrieval fails
        """
        try:
            response_data = await self._request("GET", f"/nodes/{node_id}", authenticated=False)
            return NodeInfo(**response_data)
        except Exception as e:
            raise NetworkError(f"Failed to get node {node_id}: {e}")
    
    # User Management Methods
    
    async def get_profile(self) -> UserProfile:
        """
        Get user profile.
        
        Returns:
            User profile information
            
        Raises:
            AuthenticationError: If profile retrieval fails
        """
        try:
            response_data = await self._request("GET", "/user/profile")
            return UserProfile(**response_data)
        except Exception as e:
            raise AuthenticationError(f"Failed to get profile: {e}")
    
    async def create_api_key(
        self, name: str, expires_in_days: Optional[int] = None
    ) -> ApiKey:
        """
        Create API key.
        
        Args:
            name: API key name
            expires_in_days: Expiration in days (optional)
            
        Returns:
            Created API key
            
        Raises:
            AuthenticationError: If API key creation fails
        """
        try:
            data = {"name": name}
            if expires_in_days is not None:
                data["expires_in_days"] = expires_in_days
                
            response_data = await self._request("POST", "/user/api-keys", data=data)
            return ApiKey(**response_data)
            
        except Exception as e:
            raise AuthenticationError(f"Failed to create API key: {e}")
    
    async def list_api_keys(self) -> List[ApiKey]:
        """
        List API keys.
        
        Returns:
            List of API keys
            
        Raises:
            AuthenticationError: If API key listing fails
        """
        try:
            response_data = await self._request("GET", "/user/api-keys")
            return [ApiKey(**key) for key in response_data]
        except Exception as e:
            raise AuthenticationError(f"Failed to list API keys: {e}")
    
    async def revoke_api_key(self, key_id: str) -> None:
        """
        Revoke API key.
        
        Args:
            key_id: API key ID
            
        Raises:
            AuthenticationError: If API key revocation fails
        """
        try:
            await self._request("POST", f"/user/api-keys/{key_id}/revoke")
        except Exception as e:
            raise AuthenticationError(f"Failed to revoke API key {key_id}: {e}")