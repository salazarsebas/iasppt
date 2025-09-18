"""
DeAI SDK - Python client for the DeAI distributed AI computation network.

This SDK provides a simple interface to submit AI tasks, manage results,
and interact with the DeAI network built on Near Protocol.
"""

__version__ = "0.1.0"
__author__ = "DeAI Team"
__email__ = "sdk@deai.org"

from .client import DeAIClient
from .websocket_client import DeAIWebSocketClient
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
)

__all__ = [
    "DeAIClient",
    "DeAIWebSocketClient",
    "DeAIError",
    "AuthenticationError", 
    "TaskError",
    "NetworkError",
    "ValidationError",
    "TaskSubmissionRequest",
    "TaskResponse",
    "TaskResult", 
    "TaskStatus",
    "NetworkStats",
    "NodeInfo",
    "UserProfile",
    "ApiKey",
]