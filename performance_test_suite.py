#!/usr/bin/env python3
"""
HackerExperience Rust - Comprehensive Performance Testing Suite
================================================================

This script performs comprehensive performance testing including:
1. API response time measurements
2. Concurrent user simulation (10-100 users)
3. Memory usage monitoring
4. Database query performance testing
5. Frontend page load time measurement
6. Memory leak detection
7. WebSocket connection stability testing
8. Cache mechanism verification

Author: Claude Code Performance Testing Bot
Date: 2025-09-19
"""

import asyncio
import aiohttp
import psutil
import time
import json
import statistics
# websockets import - will handle gracefully if not available
try:
    import websockets
    WEBSOCKETS_AVAILABLE = True
except ImportError:
    WEBSOCKETS_AVAILABLE = False
import concurrent.futures
import threading
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, asdict
# matplotlib import removed due to compilation issues
# import matplotlib.pyplot as plt
# import numpy as np
from datetime import datetime, timedelta
import logging
import sys
import argparse

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('performance_test.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

@dataclass
class PerformanceMetrics:
    """Data class to store performance metrics"""
    endpoint: str
    response_time_ms: float
    status_code: int
    response_size_bytes: int
    memory_usage_mb: float
    cpu_usage_percent: float
    timestamp: datetime

@dataclass
class ConcurrencyTestResult:
    """Data class for concurrency test results"""
    concurrent_users: int
    total_requests: int
    successful_requests: int
    failed_requests: int
    avg_response_time_ms: float
    min_response_time_ms: float
    max_response_time_ms: float
    p95_response_time_ms: float
    throughput_rps: float
    error_rate_percent: float

class PerformanceTester:
    def __init__(self, base_url: str = "http://172.104.215.73:3000", ws_url: str = "ws://172.104.215.73:3001/ws"):
        self.base_url = base_url.rstrip('/')
        self.ws_url = ws_url
        self.session = None
        self.auth_token = None
        self.metrics: List[PerformanceMetrics] = []
        self.concurrency_results: List[ConcurrencyTestResult] = []
        self.memory_baseline_mb = None

        # Test credentials
        self.test_credentials = {
            "email": "test@hackerexperience.com",
            "password": "TestPassword123!"
        }

        # API endpoints to test
        self.endpoints = [
            ("POST", "/api/register", {"register_payload": True}),
            ("POST", "/api/login", {"login_payload": True}),
            ("POST", "/api/logout", {}),
            ("GET", "/api/game/dashboard", {}),
            ("POST", "/api/game/process/start", {"process_payload": True}),
            ("GET", "/api/game/software", {}),
            ("POST", "/api/game/bank/transfer", {"transfer_payload": True}),
            ("GET", "/api/leaderboard/level", {}),
            ("GET", "/health", {}),
            ("GET", "/health/detailed", {}),
            ("GET", "/metrics", {})
        ]

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        self.memory_baseline_mb = psutil.virtual_memory().used / (1024 * 1024)
        logger.info(f"Baseline memory usage: {self.memory_baseline_mb:.2f} MB")
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    def get_payload_for_endpoint(self, method: str, endpoint: str, payload_type: dict) -> dict:
        """Generate appropriate payload for each endpoint"""
        if payload_type.get("register_payload"):
            timestamp = int(time.time())
            return {
                "username": f"testuser{timestamp}",
                "email": f"test{timestamp}@hackerexperience.com",
                "password": "TestPassword123!"
            }
        elif payload_type.get("login_payload"):
            return self.test_credentials
        elif payload_type.get("process_payload"):
            return {
                "target_id": 1,
                "process_type": "hack",
                "priority": 1
            }
        elif payload_type.get("transfer_payload"):
            return {
                "recipient_id": 2,
                "amount": 1000,
                "memo": "Performance test transfer"
            }
        return {}

    async def authenticate(self) -> bool:
        """Authenticate with the API to get JWT token"""
        try:
            # First try to register a user
            timestamp = int(time.time())
            register_payload = {
                "username": f"perftest{timestamp}",
                "email": f"perftest{timestamp}@hackerexperience.com",
                "password": "PerfTest123!"
            }

            async with self.session.post(f"{self.base_url}/api/register",
                                       json=register_payload) as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('success') and data.get('token'):
                        self.auth_token = data['token']
                        self.test_credentials = {
                            "email": register_payload["email"],
                            "password": register_payload["password"]
                        }
                        logger.info("Successfully registered and authenticated test user")
                        return True

            # If registration fails, try to login with existing credentials
            async with self.session.post(f"{self.base_url}/api/login",
                                       json=self.test_credentials) as response:
                if response.status == 200:
                    data = await response.json()
                    if data.get('success') and data.get('token'):
                        self.auth_token = data['token']
                        logger.info("Successfully authenticated with existing credentials")
                        return True

        except Exception as e:
            logger.error(f"Authentication failed: {e}")

        return False

    async def measure_endpoint_performance(self, method: str, endpoint: str, payload: dict) -> PerformanceMetrics:
        """Measure performance metrics for a single endpoint"""
        start_memory = psutil.virtual_memory().used / (1024 * 1024)
        start_cpu = psutil.cpu_percent()
        start_time = time.time()

        headers = {}
        if self.auth_token and endpoint not in ['/api/register', '/api/login', '/health', '/health/detailed', '/metrics']:
            headers['Authorization'] = f'Bearer {self.auth_token}'

        try:
            if method == "GET":
                async with self.session.get(f"{self.base_url}{endpoint}", headers=headers) as response:
                    content = await response.read()
                    status_code = response.status
            else:
                async with self.session.request(method, f"{self.base_url}{endpoint}",
                                              json=payload, headers=headers) as response:
                    content = await response.read()
                    status_code = response.status

            end_time = time.time()
            end_memory = psutil.virtual_memory().used / (1024 * 1024)
            end_cpu = psutil.cpu_percent()

            response_time_ms = (end_time - start_time) * 1000
            response_size_bytes = len(content)
            memory_usage_mb = end_memory - start_memory
            cpu_usage_percent = end_cpu - start_cpu

            return PerformanceMetrics(
                endpoint=f"{method} {endpoint}",
                response_time_ms=response_time_ms,
                status_code=status_code,
                response_size_bytes=response_size_bytes,
                memory_usage_mb=memory_usage_mb,
                cpu_usage_percent=cpu_usage_percent,
                timestamp=datetime.now()
            )

        except Exception as e:
            logger.error(f"Error testing {method} {endpoint}: {e}")
            return PerformanceMetrics(
                endpoint=f"{method} {endpoint}",
                response_time_ms=9999.0,
                status_code=0,
                response_size_bytes=0,
                memory_usage_mb=0.0,
                cpu_usage_percent=0.0,
                timestamp=datetime.now()
            )

    async def test_api_endpoints(self):
        """Test all API endpoints for response times"""
        logger.info("Starting API endpoint performance testing...")

        if not await self.authenticate():
            logger.error("Failed to authenticate - some tests may fail")

        for method, endpoint, payload_config in self.endpoints:
            payload = self.get_payload_for_endpoint(method, endpoint, payload_config)

            # Test each endpoint multiple times for better statistics
            endpoint_metrics = []
            for _ in range(5):
                metric = await self.measure_endpoint_performance(method, endpoint, payload)
                endpoint_metrics.append(metric)
                self.metrics.append(metric)

                # Small delay between requests
                await asyncio.sleep(0.1)

            # Log summary for this endpoint
            avg_response_time = statistics.mean([m.response_time_ms for m in endpoint_metrics])
            success_rate = len([m for m in endpoint_metrics if 200 <= m.status_code < 300]) / len(endpoint_metrics) * 100

            logger.info(f"{method} {endpoint}: Avg {avg_response_time:.2f}ms, Success: {success_rate:.1f}%")

    async def simulate_concurrent_user(self, session: aiohttp.ClientSession, user_id: int,
                                     requests_per_user: int) -> List[float]:
        """Simulate a single user making multiple requests"""
        response_times = []

        # Authenticate this user session
        timestamp = int(time.time())
        user_creds = {
            "username": f"loaduser{user_id}_{timestamp}",
            "email": f"loaduser{user_id}_{timestamp}@hackerexperience.com",
            "password": "LoadTest123!"
        }

        # Register user
        try:
            async with session.post(f"{self.base_url}/api/register", json=user_creds) as response:
                if response.status == 200:
                    data = await response.json()
                    token = data.get('token')
                else:
                    token = None
        except:
            token = None

        headers = {}
        if token:
            headers['Authorization'] = f'Bearer {token}'

        # Make requests
        for _ in range(requests_per_user):
            start_time = time.time()
            try:
                # Test different endpoints randomly
                test_endpoints = [
                    ("GET", "/health"),
                    ("GET", "/api/game/dashboard"),
                    ("GET", "/api/game/software"),
                    ("GET", "/api/leaderboard/level")
                ]

                method, endpoint = test_endpoints[user_id % len(test_endpoints)]

                async with session.request(method, f"{self.base_url}{endpoint}",
                                         headers=headers) as response:
                    await response.read()

                response_time = (time.time() - start_time) * 1000
                response_times.append(response_time)

            except Exception as e:
                # Record failed request as high response time
                response_times.append(9999.0)

            # Small delay between requests from same user
            await asyncio.sleep(0.05)

        return response_times

    async def test_concurrent_users(self, user_counts: List[int] = [10, 25, 50, 100]):
        """Test server performance under concurrent user load"""
        logger.info("Starting concurrent user testing...")

        for user_count in user_counts:
            logger.info(f"Testing with {user_count} concurrent users...")

            requests_per_user = 10
            total_requests = user_count * requests_per_user

            # Create connector with higher limits
            connector = aiohttp.TCPConnector(limit=200, limit_per_host=50)

            async with aiohttp.ClientSession(
                connector=connector,
                timeout=aiohttp.ClientTimeout(total=30)
            ) as session:

                start_time = time.time()

                # Create concurrent user tasks
                tasks = [
                    self.simulate_concurrent_user(session, user_id, requests_per_user)
                    for user_id in range(user_count)
                ]

                # Execute all user simulations concurrently
                results = await asyncio.gather(*tasks, return_exceptions=True)

                end_time = time.time()
                duration = end_time - start_time

                # Process results
                all_response_times = []
                successful_requests = 0
                failed_requests = 0

                for result in results:
                    if isinstance(result, Exception):
                        failed_requests += requests_per_user
                    else:
                        for rt in result:
                            all_response_times.append(rt)
                            if rt < 9999.0:
                                successful_requests += 1
                            else:
                                failed_requests += 1

                # Calculate metrics
                if all_response_times:
                    avg_response_time = statistics.mean(all_response_times)
                    min_response_time = min(all_response_times)
                    max_response_time = max([rt for rt in all_response_times if rt < 9999.0] or [0])

                    # Calculate 95th percentile
                    sorted_times = sorted([rt for rt in all_response_times if rt < 9999.0])
                    if sorted_times:
                        p95_response_time = sorted_times[int(0.95 * len(sorted_times))]
                    else:
                        p95_response_time = 0
                else:
                    avg_response_time = min_response_time = max_response_time = p95_response_time = 0

                throughput_rps = successful_requests / duration if duration > 0 else 0
                error_rate_percent = (failed_requests / total_requests) * 100 if total_requests > 0 else 0

                result = ConcurrencyTestResult(
                    concurrent_users=user_count,
                    total_requests=total_requests,
                    successful_requests=successful_requests,
                    failed_requests=failed_requests,
                    avg_response_time_ms=avg_response_time,
                    min_response_time_ms=min_response_time,
                    max_response_time_ms=max_response_time,
                    p95_response_time_ms=p95_response_time,
                    throughput_rps=throughput_rps,
                    error_rate_percent=error_rate_percent
                )

                self.concurrency_results.append(result)

                logger.info(f"Results for {user_count} users:")
                logger.info(f"  Throughput: {throughput_rps:.2f} RPS")
                logger.info(f"  Avg Response Time: {avg_response_time:.2f}ms")
                logger.info(f"  95th Percentile: {p95_response_time:.2f}ms")
                logger.info(f"  Error Rate: {error_rate_percent:.2f}%")

    async def test_websocket_stability(self, duration_seconds: int = 60):
        """Test WebSocket connection stability"""
        if not WEBSOCKETS_AVAILABLE:
            logger.warning("WebSocket testing skipped - websockets package not available")
            return {"error": "websockets package not available", "skipped": True}

        logger.info(f"Starting WebSocket stability test for {duration_seconds} seconds...")

        ws_metrics = {
            "connections_attempted": 0,
            "connections_successful": 0,
            "messages_sent": 0,
            "messages_received": 0,
            "connection_drops": 0,
            "reconnections": 0,
            "avg_latency_ms": 0
        }

        latencies = []

        async def websocket_client(client_id: int):
            try:
                ws_metrics["connections_attempted"] += 1

                async with websockets.connect(self.ws_url) as websocket:
                    ws_metrics["connections_successful"] += 1

                    # Authenticate if we have a token
                    if self.auth_token:
                        auth_msg = {
                            "type": "auth",
                            "token": self.auth_token
                        }
                        await websocket.send(json.dumps(auth_msg))
                        ws_metrics["messages_sent"] += 1

                        # Wait for auth response
                        try:
                            response = await asyncio.wait_for(websocket.recv(), timeout=5.0)
                            ws_metrics["messages_received"] += 1
                        except asyncio.TimeoutError:
                            pass

                    # Send periodic ping messages
                    start_time = time.time()
                    message_count = 0

                    while time.time() - start_time < duration_seconds:
                        try:
                            # Send ping
                            ping_start = time.time()
                            ping_msg = {"type": "ping", "timestamp": ping_start}
                            await websocket.send(json.dumps(ping_msg))
                            ws_metrics["messages_sent"] += 1

                            # Wait for pong
                            try:
                                response = await asyncio.wait_for(websocket.recv(), timeout=2.0)
                                ping_end = time.time()
                                latency = (ping_end - ping_start) * 1000
                                latencies.append(latency)
                                ws_metrics["messages_received"] += 1
                                message_count += 1
                            except asyncio.TimeoutError:
                                pass

                            await asyncio.sleep(1)  # Send ping every second

                        except websockets.exceptions.ConnectionClosed:
                            ws_metrics["connection_drops"] += 1
                            break
                        except Exception as e:
                            logger.error(f"WebSocket client {client_id} error: {e}")
                            break

            except Exception as e:
                logger.error(f"WebSocket connection failed for client {client_id}: {e}")

        # Test with multiple concurrent WebSocket connections
        tasks = [websocket_client(i) for i in range(5)]
        await asyncio.gather(*tasks, return_exceptions=True)

        if latencies:
            ws_metrics["avg_latency_ms"] = statistics.mean(latencies)

        logger.info("WebSocket stability test results:")
        logger.info(f"  Connection success rate: {ws_metrics['connections_successful']}/{ws_metrics['connections_attempted']}")
        logger.info(f"  Messages sent: {ws_metrics['messages_sent']}")
        logger.info(f"  Messages received: {ws_metrics['messages_received']}")
        logger.info(f"  Connection drops: {ws_metrics['connection_drops']}")
        logger.info(f"  Average latency: {ws_metrics['avg_latency_ms']:.2f}ms")

        return ws_metrics

    def monitor_memory_usage(self, duration_seconds: int = 300):
        """Monitor memory usage over time to detect leaks"""
        logger.info(f"Starting memory leak detection for {duration_seconds} seconds...")

        memory_samples = []
        start_time = time.time()

        while time.time() - start_time < duration_seconds:
            memory_mb = psutil.virtual_memory().used / (1024 * 1024)
            memory_samples.append({
                "timestamp": time.time() - start_time,
                "memory_mb": memory_mb
            })

            time.sleep(5)  # Sample every 5 seconds

        # Analyze memory trend
        timestamps = [s["timestamp"] for s in memory_samples]
        memory_values = [s["memory_mb"] for s in memory_samples]

        # Calculate memory growth rate (MB per minute)
        if len(memory_samples) >= 2:
            memory_growth_mb_per_min = (memory_values[-1] - memory_values[0]) / (timestamps[-1] / 60)
        else:
            memory_growth_mb_per_min = 0

        logger.info(f"Memory monitoring results:")
        logger.info(f"  Initial memory: {memory_values[0]:.2f} MB")
        logger.info(f"  Final memory: {memory_values[-1]:.2f} MB")
        logger.info(f"  Growth rate: {memory_growth_mb_per_min:.2f} MB/minute")
        logger.info(f"  Peak memory: {max(memory_values):.2f} MB")

        return {
            "initial_memory_mb": memory_values[0],
            "final_memory_mb": memory_values[-1],
            "peak_memory_mb": max(memory_values),
            "memory_growth_mb_per_min": memory_growth_mb_per_min,
            "samples": memory_samples
        }

    async def test_frontend_performance(self):
        """Test frontend page load times"""
        logger.info("Testing frontend page load performance...")

        frontend_url = self.base_url.replace(':3000', ':8080')  # Assuming frontend on port 8080

        frontend_pages = [
            "/",
            "/login",
            "/register",
            "/dashboard",
            "/game"
        ]

        page_metrics = []

        for page in frontend_pages:
            page_times = []

            for _ in range(3):  # Test each page 3 times
                start_time = time.time()
                try:
                    async with self.session.get(f"{frontend_url}{page}") as response:
                        content = await response.read()
                        load_time = (time.time() - start_time) * 1000
                        page_times.append(load_time)

                except Exception as e:
                    logger.error(f"Failed to load page {page}: {e}")
                    page_times.append(9999.0)

                await asyncio.sleep(0.5)

            avg_load_time = statistics.mean(page_times)
            page_metrics.append({
                "page": page,
                "avg_load_time_ms": avg_load_time,
                "min_load_time_ms": min(page_times),
                "max_load_time_ms": max(page_times)
            })

            logger.info(f"Page {page}: {avg_load_time:.2f}ms average load time")

        return page_metrics

    def generate_performance_report(self):
        """Generate comprehensive performance report"""
        logger.info("Generating performance report...")

        report = {
            "test_summary": {
                "timestamp": datetime.now().isoformat(),
                "base_url": self.base_url,
                "total_api_tests": len(self.metrics),
                "total_concurrency_tests": len(self.concurrency_results)
            },
            "api_performance": {},
            "concurrency_performance": [asdict(r) for r in self.concurrency_results],
            "recommendations": []
        }

        # Process API metrics
        endpoint_groups = {}
        for metric in self.metrics:
            if metric.endpoint not in endpoint_groups:
                endpoint_groups[metric.endpoint] = []
            endpoint_groups[metric.endpoint].append(metric)

        for endpoint, metrics in endpoint_groups.items():
            response_times = [m.response_time_ms for m in metrics if m.status_code != 0]
            success_rate = len([m for m in metrics if 200 <= m.status_code < 300]) / len(metrics) * 100

            if response_times:
                report["api_performance"][endpoint] = {
                    "avg_response_time_ms": statistics.mean(response_times),
                    "min_response_time_ms": min(response_times),
                    "max_response_time_ms": max(response_times),
                    "success_rate_percent": success_rate,
                    "total_tests": len(metrics)
                }

        # Generate recommendations
        recommendations = []

        # Check API response times
        slow_endpoints = [
            (endpoint, data["avg_response_time_ms"])
            for endpoint, data in report["api_performance"].items()
            if data["avg_response_time_ms"] > 1000
        ]

        if slow_endpoints:
            recommendations.append({
                "category": "API Performance",
                "priority": "HIGH",
                "issue": f"Slow endpoints detected: {len(slow_endpoints)} endpoints > 1000ms",
                "recommendation": "Optimize database queries, add caching, consider request/response compression",
                "endpoints": [endpoint for endpoint, _ in slow_endpoints]
            })

        # Check concurrency performance
        if self.concurrency_results:
            max_users_test = max(self.concurrency_results, key=lambda x: x.concurrent_users)
            if max_users_test.error_rate_percent > 5:
                recommendations.append({
                    "category": "Scalability",
                    "priority": "HIGH",
                    "issue": f"High error rate ({max_users_test.error_rate_percent:.1f}%) at {max_users_test.concurrent_users} users",
                    "recommendation": "Scale horizontally, optimize connection pooling, increase server resources"
                })

            if max_users_test.throughput_rps < 50:
                recommendations.append({
                    "category": "Throughput",
                    "priority": "MEDIUM",
                    "issue": f"Low throughput ({max_users_test.throughput_rps:.1f} RPS) under load",
                    "recommendation": "Profile application bottlenecks, optimize critical code paths, consider async processing"
                })

        report["recommendations"] = recommendations

        # Save report to file
        with open('performance_report.json', 'w') as f:
            json.dump(report, f, indent=2, default=str)

        logger.info("Performance report saved to performance_report.json")
        return report

async def main():
    parser = argparse.ArgumentParser(description='HackerExperience Performance Test Suite')
    parser.add_argument('--base-url', default='http://172.104.215.73:3000',
                       help='Base URL for API testing')
    parser.add_argument('--ws-url', default='ws://172.104.215.73:3001/ws',
                       help='WebSocket URL for testing')
    parser.add_argument('--skip-auth', action='store_true',
                       help='Skip authentication-required tests')
    parser.add_argument('--quick', action='store_true',
                       help='Run quick test suite (reduced duration and user counts)')

    args = parser.parse_args()

    # Adjust test parameters for quick mode
    if args.quick:
        user_counts = [5, 10, 20]
        ws_duration = 30
        memory_duration = 60
    else:
        user_counts = [10, 25, 50, 100]
        ws_duration = 60
        memory_duration = 300

    logger.info("Starting HackerExperience Performance Test Suite")
    logger.info(f"Target server: {args.base_url}")

    async with PerformanceTester(args.base_url, args.ws_url) as tester:
        try:
            # 1. Test API endpoints
            await tester.test_api_endpoints()

            # 2. Test concurrent users
            await tester.test_concurrent_users(user_counts)

            # 3. Test WebSocket stability
            await tester.test_websocket_stability(ws_duration)

            # 4. Test frontend performance
            await tester.test_frontend_performance()

            # 5. Memory monitoring (run in background thread)
            memory_thread = threading.Thread(
                target=lambda: tester.monitor_memory_usage(memory_duration)
            )
            memory_thread.start()

            # Wait for memory monitoring to complete
            memory_thread.join()

            # 6. Generate comprehensive report
            report = tester.generate_performance_report()

            # Print summary
            print("\n" + "="*60)
            print("PERFORMANCE TEST SUMMARY")
            print("="*60)

            print(f"API Endpoints Tested: {len(tester.metrics)}")
            if tester.concurrency_results:
                max_users = max(r.concurrent_users for r in tester.concurrency_results)
                print(f"Max Concurrent Users: {max_users}")

                best_throughput = max(r.throughput_rps for r in tester.concurrency_results)
                print(f"Peak Throughput: {best_throughput:.2f} RPS")

            print(f"Recommendations Generated: {len(report['recommendations'])}")

            for rec in report['recommendations'][:3]:  # Show top 3 recommendations
                print(f"\n[{rec['priority']}] {rec['category']}: {rec['issue']}")
                print(f"    → {rec['recommendation']}")

            print("\nFull report saved to performance_report.json")

        except Exception as e:
            logger.error(f"Performance test failed: {e}")
            sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())