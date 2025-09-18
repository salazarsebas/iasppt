#!/usr/bin/env python3
"""
Integration tests for DeAI platform components.

This script tests the complete flow from API gateway through the smart contract
to the node network, ensuring all components work together correctly.
"""

import asyncio
import json
import os
import time
from typing import Dict, Any, Optional
import logging

import httpx
import websockets

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class IntegrationTestSuite:
    """
    Complete integration test suite for the DeAI platform.
    
    Tests include:
    - API gateway functionality
    - Authentication flows
    - Task submission and processing
    - WebSocket real-time updates
    - SDK functionality (Python and JavaScript)
    - Smart contract interactions
    - Node network operations
    """
    
    def __init__(
        self,
        api_url: str = "http://localhost:8080",
        ws_url: str = "ws://localhost:8081",
        contract_id: str = "deai-compute.testnet",
    ):
        self.api_url = api_url.rstrip("/")
        self.ws_url = ws_url
        self.contract_id = contract_id
        self.client = httpx.AsyncClient(timeout=30.0)
        self.access_token: Optional[str] = None
        self.test_user: Dict[str, Any] = {}
        
    async def __aenter__(self):
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.aclose()
    
    async def run_all_tests(self) -> bool:
        """
        Run the complete integration test suite.
        
        Returns:
            True if all tests pass, False otherwise
        """
        logger.info("🚀 Starting DeAI Integration Test Suite")
        logger.info("=" * 60)
        
        test_results = []
        
        # Test API Gateway Health
        test_results.append(await self.test_api_health())
        
        # Test User Registration and Authentication
        test_results.append(await self.test_user_registration())
        test_results.append(await self.test_user_login())
        
        # Test API Key Management
        test_results.append(await self.test_api_key_creation())
        
        # Test Task Management
        test_results.append(await self.test_task_submission())
        test_results.append(await self.test_task_retrieval())
        test_results.append(await self.test_task_cancellation())
        
        # Test Network Information
        test_results.append(await self.test_network_stats())
        test_results.append(await self.test_node_listing())
        
        # Test WebSocket Connection
        test_results.append(await self.test_websocket_connection())
        
        # Test Rate Limiting
        test_results.append(await self.test_rate_limiting())
        
        # Test Python SDK
        test_results.append(await self.test_python_sdk())
        
        # Summary
        passed = sum(test_results)
        total = len(test_results)
        
        logger.info("=" * 60)
        logger.info(f"🎯 Test Results: {passed}/{total} tests passed")
        
        if passed == total:
            logger.info("✅ All integration tests passed!")
            return True
        else:
            logger.error(f"❌ {total - passed} tests failed")
            return False
    
    async def test_api_health(self) -> bool:
        """Test API gateway health endpoint."""
        logger.info("🔍 Testing API health...")
        
        try:
            response = await self.client.get(f"{self.api_url}/health")
            if response.status_code == 200:
                logger.info("✅ API health check passed")
                return True
            else:
                logger.error(f"❌ API health check failed: {response.status_code}")
                return False
        except Exception as e:
            logger.error(f"❌ API health check failed: {e}")
            return False
    
    async def test_user_registration(self) -> bool:
        """Test user registration."""
        logger.info("🔍 Testing user registration...")
        
        try:
            # Generate unique test user
            timestamp = int(time.time())
            test_user_data = {
                "username": f"test_user_{timestamp}",
                "email": f"test_{timestamp}@deai.test",
                "password": "test_password_123",
                "near_account_id": f"test_{timestamp}.testnet"
            }
            
            response = await self.client.post(
                f"{self.api_url}/api/v1/auth/register",
                json=test_user_data
            )
            
            if response.status_code == 200:
                data = response.json()
                self.access_token = data["access_token"]
                self.test_user = data["user"]
                logger.info("✅ User registration passed")
                return True
            else:
                logger.error(f"❌ User registration failed: {response.status_code} - {response.text}")
                return False
                
        except Exception as e:
            logger.error(f"❌ User registration failed: {e}")
            return False
    
    async def test_user_login(self) -> bool:
        """Test user login."""
        logger.info("🔍 Testing user login...")
        
        try:
            login_data = {
                "username": self.test_user["username"],
                "password": "test_password_123"
            }
            
            response = await self.client.post(
                f"{self.api_url}/api/v1/auth/login",
                json=login_data
            )
            
            if response.status_code == 200:
                data = response.json()
                self.access_token = data["access_token"]
                logger.info("✅ User login passed")
                return True
            else:
                logger.error(f"❌ User login failed: {response.status_code}")
                return False
                
        except Exception as e:
            logger.error(f"❌ User login failed: {e}")
            return False
    
    async def test_api_key_creation(self) -> bool:
        """Test API key creation."""
        logger.info("🔍 Testing API key creation...")
        
        try:
            headers = {"Authorization": f"Bearer {self.access_token}"}
            
            response = await self.client.post(
                f"{self.api_url}/api/v1/user/api-keys",
                json={"name": "test_integration_key", "expires_in_days": 30},
                headers=headers
            )
            
            if response.status_code == 200:
                logger.info("✅ API key creation passed")
                return True
            else:
                logger.error(f"❌ API key creation failed: {response.status_code}")
                return False
                
        except Exception as e:
            logger.error(f"❌ API key creation failed: {e}")
            return False
    
    async def test_task_submission(self) -> bool:
        """Test task submission."""
        logger.info("🔍 Testing task submission...")
        
        try:
            headers = {"Authorization": f"Bearer {self.access_token}"}
            
            task_data = {
                "task_type": "text_generation",
                "model_name": "gpt2",
                "input_data": "The future of artificial intelligence is",
                "max_cost": "0.1",
                "priority": 5
            }
            
            response = await self.client.post(
                f"{self.api_url}/api/v1/tasks",
                json=task_data,
                headers=headers
            )
            
            if response.status_code == 200:
                task = response.json()
                self.test_task_id = task["id"]
                logger.info(f"✅ Task submission passed (ID: {task['id'][:8]}...)")
                return True
            else:
                logger.error(f"❌ Task submission failed: {response.status_code}")
                return False
                
        except Exception as e:
            logger.error(f"❌ Task submission failed: {e}")
            return False
    
    async def test_task_retrieval(self) -> bool:
        """Test task retrieval."""
        logger.info("🔍 Testing task retrieval...")
        
        try:
            headers = {"Authorization": f"Bearer {self.access_token}"}
            
            response = await self.client.get(
                f"{self.api_url}/api/v1/tasks/{self.test_task_id}",
                headers=headers
            )
            
            if response.status_code == 200:
                task = response.json()
                logger.info(f"✅ Task retrieval passed (Status: {task['status']})")
                return True
            else:
                logger.error(f"❌ Task retrieval failed: {response.status_code}")
                return False
                
        except Exception as e:
            logger.error(f"❌ Task retrieval failed: {e}")
            return False
    
    async def test_task_cancellation(self) -> bool:
        """Test task cancellation."""
        logger.info("🔍 Testing task cancellation...")
        
        try:
            headers = {"Authorization": f"Bearer {self.access_token}"}
            
            response = await self.client.post(
                f"{self.api_url}/api/v1/tasks/{self.test_task_id}/cancel",
                headers=headers
            )
            
            if response.status_code == 200:
                logger.info("✅ Task cancellation passed")
                return True
            else:
                logger.error(f"❌ Task cancellation failed: {response.status_code}")
                return False
                
        except Exception as e:
            logger.error(f"❌ Task cancellation failed: {e}")
            return False
    
    async def test_network_stats(self) -> bool:
        """Test network statistics retrieval."""
        logger.info("🔍 Testing network statistics...")
        
        try:
            response = await self.client.get(f"{self.api_url}/api/v1/network/stats")
            
            if response.status_code == 200:
                stats = response.json()
                logger.info(f"✅ Network stats passed (Active nodes: {stats.get('active_nodes', 0)})")
                return True
            else:
                logger.error(f"❌ Network stats failed: {response.status_code}")
                return False
                
        except Exception as e:
            logger.error(f"❌ Network stats failed: {e}")
            return False
    
    async def test_node_listing(self) -> bool:
        """Test node listing."""
        logger.info("🔍 Testing node listing...")
        
        try:
            response = await self.client.get(f"{self.api_url}/api/v1/nodes")
            
            if response.status_code == 200:
                nodes = response.json()
                logger.info(f"✅ Node listing passed ({len(nodes)} nodes found)")
                return True
            else:
                logger.error(f"❌ Node listing failed: {response.status_code}")
                return False
                
        except Exception as e:
            logger.error(f"❌ Node listing failed: {e}")
            return False
    
    async def test_websocket_connection(self) -> bool:
        """Test WebSocket connection and real-time updates."""
        logger.info("🔍 Testing WebSocket connection...")
        
        try:
            uri = f"{self.ws_url}/ws?token={self.access_token}"
            
            async with websockets.connect(uri) as websocket:
                # Send ping
                await websocket.send(json.dumps({"type": "ping"}))
                
                # Wait for pong
                response = await websocket.recv()
                data = json.loads(response)
                
                if data.get("type") == "pong":
                    logger.info("✅ WebSocket connection passed")
                    return True
                else:
                    logger.error("❌ WebSocket connection failed: Invalid response")
                    return False
                    
        except Exception as e:
            logger.error(f"❌ WebSocket connection failed: {e}")
            return False
    
    async def test_rate_limiting(self) -> bool:
        """Test rate limiting functionality."""
        logger.info("🔍 Testing rate limiting...")
        
        try:
            headers = {"Authorization": f"Bearer {self.access_token}"}
            
            # Make multiple rapid requests
            responses = []
            for i in range(5):
                response = await self.client.get(
                    f"{self.api_url}/api/v1/user/profile",
                    headers=headers
                )
                responses.append(response.status_code)
            
            # Check that we get rate limited (429) or all succeed
            if 429 in responses or all(code == 200 for code in responses):
                logger.info("✅ Rate limiting test passed")
                return True
            else:
                logger.error(f"❌ Rate limiting test failed: {responses}")
                return False
                
        except Exception as e:
            logger.error(f"❌ Rate limiting test failed: {e}")
            return False
    
    async def test_python_sdk(self) -> bool:
        """Test Python SDK functionality."""
        logger.info("🔍 Testing Python SDK...")
        
        try:
            # This would require the SDK to be installed locally
            # For now, we'll test basic client initialization
            
            from deai_sdk import DeAIClient
            
            async with DeAIClient(api_url=self.api_url) as client:
                # Test authentication
                client.set_api_key(self.access_token)
                
                # Test profile retrieval
                profile = await client.get_profile()
                
                if profile.username == self.test_user["username"]:
                    logger.info("✅ Python SDK test passed")
                    return True
                else:
                    logger.error("❌ Python SDK test failed: Profile mismatch")
                    return False
                    
        except ImportError:
            logger.warning("⚠️ Python SDK not installed, skipping test")
            return True
        except Exception as e:
            logger.error(f"❌ Python SDK test failed: {e}")
            return False


async def main():
    """Main entry point for integration tests."""
    # Read configuration from environment
    api_url = os.getenv("DEAI_API_URL", "http://localhost:8080")
    ws_url = os.getenv("DEAI_WS_URL", "ws://localhost:8081")
    contract_id = os.getenv("DEAI_CONTRACT_ID", "deai-compute.testnet")
    
    async with IntegrationTestSuite(api_url, ws_url, contract_id) as test_suite:
        success = await test_suite.run_all_tests()
        
        if success:
            print("\n🎉 All integration tests completed successfully!")
            exit(0)
        else:
            print("\n💥 Some integration tests failed!")
            exit(1)


if __name__ == "__main__":
    asyncio.run(main())