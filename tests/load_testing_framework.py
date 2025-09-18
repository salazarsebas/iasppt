#!/usr/bin/env python3
"""
Load Testing Framework for DeAI Platform

Specifically designed to test:
- 100 concurrent tasks across 10 nodes
- Performance under sustained load
- System scalability and bottleneck identification
- Resource utilization monitoring
"""

import asyncio
import json
import time
import statistics
from typing import Dict, Any, List, Optional, Tuple
from dataclasses import dataclass, field
from concurrent.futures import ThreadPoolExecutor
import logging
import psutil
import threading
from datetime import datetime, timedelta

import httpx
import numpy as np
import matplotlib.pyplot as plt

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class LoadTestMetrics:
    """Comprehensive metrics for load testing."""
    # Task metrics
    tasks_submitted: int = 0
    tasks_completed: int = 0
    tasks_failed: int = 0
    
    # Timing metrics
    submission_times: List[float] = field(default_factory=list)
    processing_times: List[float] = field(default_factory=list)
    total_test_duration: float = 0.0
    
    # Throughput metrics
    submissions_per_second: float = 0.0
    completions_per_second: float = 0.0
    
    # Latency metrics
    min_latency: float = 0.0
    max_latency: float = 0.0
    avg_latency: float = 0.0
    p95_latency: float = 0.0
    p99_latency: float = 0.0
    
    # Resource metrics
    cpu_usage_samples: List[float] = field(default_factory=list)
    memory_usage_samples: List[float] = field(default_factory=list)
    
    # Error tracking
    error_types: Dict[str, int] = field(default_factory=dict)
    
    # Node distribution
    node_task_distribution: Dict[str, int] = field(default_factory=dict)

@dataclass
class LoadTestConfig:
    """Configuration for load testing."""
    total_tasks: int = 100
    concurrent_users: int = 10
    target_nodes: int = 10
    ramp_up_duration: float = 10.0  # seconds
    test_duration: float = 300.0    # 5 minutes
    task_types: List[str] = field(default_factory=lambda: [
        "text_generation", "image_classification", "sentiment_analysis", 
        "linear_regression", "neural_network_inference"
    ])
    max_cost_per_task: float = 0.1
    think_time: float = 1.0  # seconds between user actions
    
class LoadTestingFramework:
    """
    Advanced load testing framework for DeAI platform.
    
    Features:
    - Realistic user simulation
    - Concurrent task execution
    - Real-time monitoring
    - Performance bottleneck identification
    - Resource utilization tracking
    """
    
    def __init__(
        self,
        api_url: str = "http://localhost:8080",
        config: Optional[LoadTestConfig] = None
    ):
        self.api_url = api_url.rstrip("/")
        self.config = config or LoadTestConfig()
        self.metrics = LoadTestMetrics()
        self.client_pool: List[httpx.AsyncClient] = []
        self.test_users: List[Dict[str, Any]] = []
        self.is_running = False
        self.start_time = 0.0
        
        # Resource monitoring
        self.resource_monitor_task: Optional[asyncio.Task] = None
        self.stop_monitoring = False
    
    async def setup_test_environment(self) -> bool:
        """Setup the test environment with users and clients."""
        logger.info("🔧 Setting up load test environment...")
        
        try:
            # Create HTTP clients
            for i in range(self.config.concurrent_users):
                client = httpx.AsyncClient(timeout=60.0)
                self.client_pool.append(client)
            
            # Create test users
            for i in range(self.config.concurrent_users):
                user_data = await self.create_test_user(i)
                if user_data:
                    self.test_users.append(user_data)
                else:
                    logger.error(f"Failed to create test user {i}")
                    return False
            
            logger.info(f"✅ Created {len(self.test_users)} test users")
            
            # Verify node availability
            nodes_available = await self.check_node_availability()
            if nodes_available < self.config.target_nodes:
                logger.warning(f"⚠️ Only {nodes_available} nodes available (target: {self.config.target_nodes})")
            
            return len(self.test_users) >= self.config.concurrent_users
            
        except Exception as e:
            logger.error(f"❌ Failed to setup test environment: {e}")
            return False
    
    async def run_load_test(self) -> LoadTestMetrics:
        """
        Execute the comprehensive load test.
        
        Returns:
            LoadTestMetrics with all collected data
        """
        logger.info("🚀 Starting Load Test")
        logger.info("=" * 60)
        logger.info(f"Configuration:")
        logger.info(f"  - Total tasks: {self.config.total_tasks}")
        logger.info(f"  - Concurrent users: {self.config.concurrent_users}")
        logger.info(f"  - Target nodes: {self.config.target_nodes}")
        logger.info(f"  - Test duration: {self.config.test_duration}s")
        logger.info("=" * 60)
        
        if not await self.setup_test_environment():
            logger.error("❌ Failed to setup test environment")
            return self.metrics
        
        self.is_running = True
        self.start_time = time.time()
        
        # Start resource monitoring
        self.resource_monitor_task = asyncio.create_task(self.monitor_resources())
        
        try:
            # Execute load test phases
            await self.ramp_up_phase()
            await self.sustained_load_phase()
            await self.ramp_down_phase()
            
        except Exception as e:
            logger.error(f"❌ Load test failed: {e}")
        finally:
            self.is_running = False
            self.stop_monitoring = True
            
            if self.resource_monitor_task:
                await self.resource_monitor_task
            
            await self.cleanup()
        
        # Calculate final metrics
        self.calculate_final_metrics()
        
        # Generate report
        await self.generate_load_test_report()
        
        return self.metrics
    
    async def ramp_up_phase(self) -> None:
        """Gradually ramp up the load to target level."""
        logger.info("📈 Starting ramp-up phase...")
        
        ramp_up_start = time.time()
        users_per_second = self.config.concurrent_users / self.config.ramp_up_duration
        
        # Start users gradually
        user_tasks = []
        for i in range(self.config.concurrent_users):
            delay = i / users_per_second
            user_task = asyncio.create_task(
                self.simulate_user_delayed(i, delay)
            )
            user_tasks.append(user_task)
        
        # Wait for ramp-up to complete
        await asyncio.sleep(self.config.ramp_up_duration)
        logger.info(f"✅ Ramp-up phase completed in {time.time() - ramp_up_start:.1f}s")
    
    async def sustained_load_phase(self) -> None:
        """Execute sustained load for the specified duration."""
        logger.info("🔥 Starting sustained load phase...")
        
        sustained_start = time.time()
        sustained_duration = self.config.test_duration - self.config.ramp_up_duration
        
        # Continue user simulation for sustained load
        await asyncio.sleep(sustained_duration)
        
        logger.info(f"✅ Sustained load phase completed in {time.time() - sustained_start:.1f}s")
    
    async def ramp_down_phase(self) -> None:
        """Gradually reduce load and complete remaining tasks."""
        logger.info("📉 Starting ramp-down phase...")
        
        # Allow time for remaining tasks to complete
        await asyncio.sleep(30)  # 30 seconds for cleanup
        
        logger.info("✅ Ramp-down phase completed")
    
    async def simulate_user_delayed(self, user_id: int, delay: float) -> None:
        """Simulate a user with initial delay for ramp-up."""
        await asyncio.sleep(delay)
        await self.simulate_user(user_id)
    
    async def simulate_user(self, user_id: int) -> None:
        """
        Simulate a realistic user submitting tasks.
        
        Args:
            user_id: Index of the user in test_users list
        """
        if user_id >= len(self.test_users):
            logger.error(f"❌ Invalid user_id: {user_id}")
            return
        
        user_data = self.test_users[user_id]
        client = self.client_pool[user_id]
        headers = {"Authorization": f"Bearer {user_data['access_token']}"}
        
        user_tasks_submitted = 0
        tasks_per_user = self.config.total_tasks // self.config.concurrent_users
        
        try:
            while (self.is_running and 
                   user_tasks_submitted < tasks_per_user and
                   time.time() - self.start_time < self.config.test_duration):
                
                # Submit a task
                task_submitted = await self.submit_user_task(
                    client, headers, user_id, user_tasks_submitted
                )
                
                if task_submitted:
                    user_tasks_submitted += 1
                    self.metrics.tasks_submitted += 1
                
                # Simulate think time
                await asyncio.sleep(self.config.think_time)
                
        except Exception as e:
            logger.error(f"❌ User {user_id} simulation failed: {e}")
    
    async def submit_user_task(
        self, 
        client: httpx.AsyncClient, 
        headers: Dict[str, str], 
        user_id: int, 
        task_num: int
    ) -> bool:
        """Submit a single task and track metrics."""
        
        try:
            # Select random task type
            task_type = np.random.choice(self.config.task_types)
            
            task_data = {
                "task_type": task_type,
                "model_name": self.get_model_for_task_type(task_type),
                "input_data": self.generate_task_input(task_type, user_id, task_num),
                "max_cost": str(self.config.max_cost_per_task),
                "priority": np.random.randint(1, 10)
            }
            
            # Track submission time
            submission_start = time.time()
            
            response = await client.post(
                f"{self.api_url}/api/v1/tasks",
                json=task_data,
                headers=headers
            )
            
            submission_time = time.time() - submission_start
            self.metrics.submission_times.append(submission_time)
            
            if response.status_code == 200:
                task_response = response.json()
                task_id = task_response["id"]
                
                # Track task for completion monitoring
                asyncio.create_task(
                    self.monitor_task_completion(client, headers, task_id, submission_start)
                )
                
                return True
            else:
                error_type = f"HTTP_{response.status_code}"
                self.metrics.error_types[error_type] = self.metrics.error_types.get(error_type, 0) + 1
                self.metrics.tasks_failed += 1
                return False
                
        except Exception as e:
            error_type = type(e).__name__
            self.metrics.error_types[error_type] = self.metrics.error_types.get(error_type, 0) + 1
            self.metrics.tasks_failed += 1
            logger.error(f"❌ Task submission failed: {e}")
            return False
    
    async def monitor_task_completion(
        self, 
        client: httpx.AsyncClient, 
        headers: Dict[str, str], 
        task_id: str, 
        submission_start: float
    ) -> None:
        """Monitor a task until completion and record metrics."""
        
        try:
            max_wait_time = 120  # 2 minutes max
            check_interval = 2   # Check every 2 seconds
            
            start_monitoring = time.time()
            
            while time.time() - start_monitoring < max_wait_time:
                response = await client.get(
                    f"{self.api_url}/api/v1/tasks/{task_id}",
                    headers=headers
                )
                
                if response.status_code == 200:
                    task_status = response.json()
                    status = task_status.get("status")
                    
                    if status == "completed":
                        processing_time = time.time() - submission_start
                        self.metrics.processing_times.append(processing_time)
                        self.metrics.tasks_completed += 1
                        
                        # Track node assignment if available
                        assigned_node = task_status.get("assigned_node")
                        if assigned_node:
                            self.metrics.node_task_distribution[assigned_node] = \
                                self.metrics.node_task_distribution.get(assigned_node, 0) + 1
                        
                        return
                    
                    elif status == "failed":
                        self.metrics.tasks_failed += 1
                        return
                
                await asyncio.sleep(check_interval)
            
            # Task timed out
            self.metrics.tasks_failed += 1
            self.metrics.error_types["timeout"] = self.metrics.error_types.get("timeout", 0) + 1
            
        except Exception as e:
            self.metrics.tasks_failed += 1
            logger.error(f"❌ Task monitoring failed: {e}")
    
    async def monitor_resources(self) -> None:
        """Monitor system resource usage during the test."""
        logger.info("📊 Starting resource monitoring...")
        
        while not self.stop_monitoring:
            try:
                # CPU usage
                cpu_percent = psutil.cpu_percent(interval=1)
                self.metrics.cpu_usage_samples.append(cpu_percent)
                
                # Memory usage
                memory = psutil.virtual_memory()
                memory_percent = memory.percent
                self.metrics.memory_usage_samples.append(memory_percent)
                
                await asyncio.sleep(5)  # Sample every 5 seconds
                
            except Exception as e:
                logger.error(f"❌ Resource monitoring error: {e}")
                await asyncio.sleep(5)
    
    def calculate_final_metrics(self) -> None:
        """Calculate final performance metrics."""
        self.metrics.total_test_duration = time.time() - self.start_time
        
        # Throughput calculations
        if self.metrics.total_test_duration > 0:
            self.metrics.submissions_per_second = self.metrics.tasks_submitted / self.metrics.total_test_duration
            self.metrics.completions_per_second = self.metrics.tasks_completed / self.metrics.total_test_duration
        
        # Latency calculations
        if self.metrics.submission_times:
            self.metrics.min_latency = min(self.metrics.submission_times)
            self.metrics.max_latency = max(self.metrics.submission_times)
            self.metrics.avg_latency = statistics.mean(self.metrics.submission_times)
            self.metrics.p95_latency = np.percentile(self.metrics.submission_times, 95)
            self.metrics.p99_latency = np.percentile(self.metrics.submission_times, 99)
    
    async def generate_load_test_report(self) -> None:
        """Generate comprehensive load test report."""
        logger.info("📋 Generating load test report...")
        
        # Text report
        report_lines = [
            "=" * 80,
            "DeAI Platform Load Test Report",
            "=" * 80,
            f"Test Duration: {self.metrics.total_test_duration:.1f} seconds",
            f"Configuration:",
            f"  - Target Tasks: {self.config.total_tasks}",
            f"  - Concurrent Users: {self.config.concurrent_users}",
            f"  - Target Nodes: {self.config.target_nodes}",
            "",
            "Task Results:",
            f"  - Tasks Submitted: {self.metrics.tasks_submitted}",
            f"  - Tasks Completed: {self.metrics.tasks_completed}",
            f"  - Tasks Failed: {self.metrics.tasks_failed}",
            f"  - Success Rate: {(self.metrics.tasks_completed / max(self.metrics.tasks_submitted, 1)) * 100:.1f}%",
            "",
            "Throughput:",
            f"  - Submissions/sec: {self.metrics.submissions_per_second:.2f}",
            f"  - Completions/sec: {self.metrics.completions_per_second:.2f}",
            "",
            "Latency (Task Submission):",
            f"  - Min: {self.metrics.min_latency:.3f}s",
            f"  - Avg: {self.metrics.avg_latency:.3f}s",
            f"  - Max: {self.metrics.max_latency:.3f}s",
            f"  - P95: {self.metrics.p95_latency:.3f}s",
            f"  - P99: {self.metrics.p99_latency:.3f}s",
            "",
            "Processing Time:",
        ]
        
        if self.metrics.processing_times:
            avg_processing = statistics.mean(self.metrics.processing_times)
            max_processing = max(self.metrics.processing_times)
            report_lines.extend([
                f"  - Avg: {avg_processing:.1f}s",
                f"  - Max: {max_processing:.1f}s",
            ])
        
        report_lines.extend([
            "",
            "Resource Usage:",
            f"  - Avg CPU: {statistics.mean(self.metrics.cpu_usage_samples):.1f}%" if self.metrics.cpu_usage_samples else "  - CPU: N/A",
            f"  - Max CPU: {max(self.metrics.cpu_usage_samples):.1f}%" if self.metrics.cpu_usage_samples else "",
            f"  - Avg Memory: {statistics.mean(self.metrics.memory_usage_samples):.1f}%" if self.metrics.memory_usage_samples else "  - Memory: N/A",
            f"  - Max Memory: {max(self.metrics.memory_usage_samples):.1f}%" if self.metrics.memory_usage_samples else "",
            "",
            "Node Distribution:",
        ])
        
        for node_id, task_count in self.metrics.node_task_distribution.items():
            report_lines.append(f"  - {node_id}: {task_count} tasks")
        
        if self.metrics.error_types:
            report_lines.extend(["", "Error Types:"])
            for error_type, count in self.metrics.error_types.items():
                report_lines.append(f"  - {error_type}: {count}")
        
        report_lines.extend([
            "",
            "Performance Assessment:",
            f"  - Meets 100 concurrent tasks: {'✅' if self.metrics.tasks_submitted >= 100 else '❌'}",
            f"  - Target throughput (>50 TPS): {'✅' if self.metrics.submissions_per_second >= 50 else '❌'}",
            f"  - Low error rate (<5%): {'✅' if (self.metrics.tasks_failed / max(self.metrics.tasks_submitted, 1)) < 0.05 else '❌'}",
            f"  - Response time (<5s): {'✅' if self.metrics.avg_latency < 5.0 else '❌'}",
            "=" * 80
        ])
        
        # Save report
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        report_path = f"/tmp/deai_load_test_report_{timestamp}.txt"
        
        with open(report_path, 'w') as f:
            f.write('\n'.join(report_lines))
        
        # Save JSON metrics
        json_path = f"/tmp/deai_load_test_metrics_{timestamp}.json"
        metrics_dict = {
            "tasks_submitted": self.metrics.tasks_submitted,
            "tasks_completed": self.metrics.tasks_completed,
            "tasks_failed": self.metrics.tasks_failed,
            "total_test_duration": self.metrics.total_test_duration,
            "submissions_per_second": self.metrics.submissions_per_second,
            "completions_per_second": self.metrics.completions_per_second,
            "avg_latency": self.metrics.avg_latency,
            "p95_latency": self.metrics.p95_latency,
            "p99_latency": self.metrics.p99_latency,
            "node_task_distribution": self.metrics.node_task_distribution,
            "error_types": self.metrics.error_types
        }
        
        with open(json_path, 'w') as f:
            json.dump(metrics_dict, f, indent=2)
        
        logger.info(f"📊 Reports generated:")
        logger.info(f"  - Text report: {report_path}")
        logger.info(f"  - JSON metrics: {json_path}")
        
        # Print summary to console
        for line in report_lines:
            logger.info(line)
    
    async def cleanup(self) -> None:
        """Clean up resources after testing."""
        logger.info("🧹 Cleaning up resources...")
        
        for client in self.client_pool:
            await client.aclose()
        
        logger.info("✅ Cleanup completed")
    
    # Helper methods
    async def create_test_user(self, user_id: int) -> Optional[Dict[str, Any]]:
        """Create a test user for load testing."""
        try:
            timestamp = int(time.time() * 1000) + user_id
            user_data = {
                "username": f"load_test_user_{timestamp}",
                "email": f"load_test_{timestamp}@deai.test",
                "password": "load_test_password_123",
                "near_account_id": f"load_test_{timestamp}.testnet"
            }
            
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(
                    f"{self.api_url}/api/v1/auth/register",
                    json=user_data
                )
                
                if response.status_code == 200:
                    return response.json()
                else:
                    logger.error(f"Failed to create load test user {user_id}: {response.status_code}")
                    return None
                    
        except Exception as e:
            logger.error(f"Failed to create load test user {user_id}: {e}")
            return None
    
    async def check_node_availability(self) -> int:
        """Check how many nodes are available for testing."""
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(f"{self.api_url}/api/v1/nodes")
                
                if response.status_code == 200:
                    nodes = response.json()
                    active_nodes = [node for node in nodes if node.get("is_active", False)]
                    return len(active_nodes)
                else:
                    return 0
                    
        except Exception as e:
            logger.error(f"Failed to check node availability: {e}")
            return 0
    
    def get_model_for_task_type(self, task_type: str) -> str:
        """Get appropriate model for task type."""
        model_mapping = {
            "text_generation": "gpt2-small",
            "image_classification": "resnet18",
            "sentiment_analysis": "bert-base",
            "linear_regression": "sklearn_linear",
            "neural_network_inference": "pytorch_mlp"
        }
        return model_mapping.get(task_type, "default_model")
    
    def generate_task_input(self, task_type: str, user_id: int, task_num: int) -> str:
        """Generate realistic input data for different task types."""
        input_templates = {
            "text_generation": f"User {user_id} task {task_num}: Generate text about AI",
            "image_classification": f"image_data_placeholder_{user_id}_{task_num}",
            "sentiment_analysis": f"This is test text for sentiment analysis from user {user_id}",
            "linear_regression": f"[[1, 2, 3], [4, 5, 6], [7, 8, 9]]",  # Sample data
            "neural_network_inference": f"input_vector_{user_id}_{task_num}"
        }
        return input_templates.get(task_type, f"default_input_{user_id}_{task_num}")


async def main():
    """Main entry point for load testing."""
    import argparse
    
    parser = argparse.ArgumentParser(description="DeAI Platform Load Testing Framework")
    parser.add_argument("--api-url", default="http://localhost:8080", help="API URL")
    parser.add_argument("--tasks", type=int, default=100, help="Total number of tasks")
    parser.add_argument("--users", type=int, default=10, help="Concurrent users")
    parser.add_argument("--nodes", type=int, default=10, help="Target nodes")
    parser.add_argument("--duration", type=int, default=300, help="Test duration in seconds")
    
    args = parser.parse_args()
    
    config = LoadTestConfig(
        total_tasks=args.tasks,
        concurrent_users=args.users,
        target_nodes=args.nodes,
        test_duration=args.duration
    )
    
    logger.info("🚀 DeAI Platform Load Testing Framework")
    logger.info(f"Target: {args.tasks} tasks across {args.nodes} nodes with {args.users} users")
    
    framework = LoadTestingFramework(api_url=args.api_url, config=config)
    
    try:
        metrics = await framework.run_load_test()
        
        # Determine if test passed
        success_rate = metrics.tasks_completed / max(metrics.tasks_submitted, 1)
        throughput_met = metrics.submissions_per_second >= 50
        error_rate_ok = (metrics.tasks_failed / max(metrics.tasks_submitted, 1)) < 0.05
        latency_ok = metrics.avg_latency < 5.0
        
        if (success_rate >= 0.95 and throughput_met and error_rate_ok and latency_ok):
            logger.info("\n🎉 Load test PASSED! System meets performance requirements.")
            exit(0)
        else:
            logger.error("\n💥 Load test FAILED! System does not meet performance requirements.")
            exit(1)
            
    except KeyboardInterrupt:
        logger.info("\n⏹️ Load test interrupted by user")
        exit(130)
    except Exception as e:
        logger.error(f"\n💥 Load test failed with exception: {e}")
        exit(1)


if __name__ == "__main__":
    asyncio.run(main())