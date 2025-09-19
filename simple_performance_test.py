#!/usr/bin/env python3
"""
Simplified HackerExperience Performance Testing
==============================================

This is a simplified version that works with available packages and tests the actual running server.
"""

import asyncio
import aiohttp
import time
import json
import statistics
from datetime import datetime
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class SimplePerformanceTester:
    def __init__(self, base_url: str = "http://172.104.215.73"):
        self.base_url = base_url.rstrip('/')
        self.session = None
        self.results = {}

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def test_endpoint(self, method: str, endpoint: str, data: dict = None) -> dict:
        """Test a single endpoint and return performance metrics"""
        start_time = time.time()

        try:
            if method == "GET":
                async with self.session.get(f"{self.base_url}{endpoint}") as response:
                    content = await response.text()
                    status_code = response.status
            else:
                async with self.session.request(method, f"{self.base_url}{endpoint}", json=data) as response:
                    content = await response.text()
                    status_code = response.status

            response_time = (time.time() - start_time) * 1000
            return {
                'endpoint': f"{method} {endpoint}",
                'response_time_ms': response_time,
                'status_code': status_code,
                'response_size': len(content),
                'success': 200 <= status_code < 300
            }

        except Exception as e:
            response_time = (time.time() - start_time) * 1000
            logger.error(f"Error testing {method} {endpoint}: {e}")
            return {
                'endpoint': f"{method} {endpoint}",
                'response_time_ms': response_time,
                'status_code': 0,
                'response_size': 0,
                'success': False,
                'error': str(e)
            }

    async def test_basic_endpoints(self):
        """Test basic endpoints that should be available"""
        logger.info("Testing basic endpoints...")

        endpoints = [
            ("GET", "/"),
            ("GET", "/health"),
            ("GET", "/api/"),
            ("GET", "/metrics"),
            ("OPTIONS", "/api/register"),
            ("OPTIONS", "/api/login"),
        ]

        results = []

        for method, endpoint in endpoints:
            # Test each endpoint multiple times
            endpoint_results = []

            for i in range(3):
                result = await self.test_endpoint(method, endpoint)
                endpoint_results.append(result)
                await asyncio.sleep(0.1)  # Small delay between requests

            # Calculate statistics for this endpoint
            response_times = [r['response_time_ms'] for r in endpoint_results if r['success']]
            success_rate = sum(1 for r in endpoint_results if r['success']) / len(endpoint_results)

            summary = {
                'endpoint': f"{method} {endpoint}",
                'tests_run': len(endpoint_results),
                'success_rate': success_rate,
                'avg_response_time_ms': statistics.mean(response_times) if response_times else 0,
                'min_response_time_ms': min(response_times) if response_times else 0,
                'max_response_time_ms': max(response_times) if response_times else 0,
                'all_results': endpoint_results
            }

            results.append(summary)
            logger.info(f"{method} {endpoint}: {summary['avg_response_time_ms']:.2f}ms avg, {summary['success_rate']:.1%} success")

        self.results['basic_endpoints'] = results
        return results

    async def test_concurrent_load(self, concurrent_requests: int = 20):
        """Test with concurrent requests"""
        logger.info(f"Testing concurrent load with {concurrent_requests} requests...")

        async def single_request():
            return await self.test_endpoint("GET", "/health")

        start_time = time.time()

        # Create concurrent tasks
        tasks = [single_request() for _ in range(concurrent_requests)]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        total_time = time.time() - start_time

        # Process results
        successful_results = [r for r in results if isinstance(r, dict) and r['success']]
        failed_results = [r for r in results if isinstance(r, Exception) or (isinstance(r, dict) and not r['success'])]

        response_times = [r['response_time_ms'] for r in successful_results]

        summary = {
            'concurrent_requests': concurrent_requests,
            'total_time_seconds': total_time,
            'successful_requests': len(successful_results),
            'failed_requests': len(failed_results),
            'requests_per_second': len(successful_results) / total_time if total_time > 0 else 0,
            'avg_response_time_ms': statistics.mean(response_times) if response_times else 0,
            'min_response_time_ms': min(response_times) if response_times else 0,
            'max_response_time_ms': max(response_times) if response_times else 0,
            'p95_response_time_ms': sorted(response_times)[int(0.95 * len(response_times))] if response_times else 0
        }

        self.results['concurrent_load'] = summary
        logger.info(f"Concurrent test: {summary['requests_per_second']:.2f} RPS, {summary['avg_response_time_ms']:.2f}ms avg")
        return summary

    async def test_server_discovery(self):
        """Discover what the server is actually running"""
        logger.info("Discovering server endpoints...")

        discovery_results = {}

        # Test common paths
        paths_to_test = [
            "/",
            "/health",
            "/api",
            "/api/v1",
            "/docs",
            "/swagger",
            "/metrics",
            "/status",
            "/ping",
            "/admin",
            "/secure-admin-ea9017d46e0fe963",  # From credentials
            "/api/register",
            "/api/login",
            "/api/users",
            "/api/game"
        ]

        for path in paths_to_test:
            result = await self.test_endpoint("GET", path)
            discovery_results[path] = {
                'status_code': result['status_code'],
                'response_size': result['response_size'],
                'response_time_ms': result['response_time_ms'],
                'accessible': result['success']
            }

            if result['success']:
                logger.info(f"✓ {path} - {result['status_code']} ({result['response_size']} bytes)")
            else:
                logger.debug(f"✗ {path} - {result['status_code']}")

        self.results['server_discovery'] = discovery_results
        return discovery_results

    def analyze_results(self):
        """Analyze all test results and generate recommendations"""
        logger.info("Analyzing performance results...")

        analysis = {
            'test_timestamp': datetime.now().isoformat(),
            'server_url': self.base_url,
            'summary': {},
            'recommendations': []
        }

        # Analyze basic endpoints
        if 'basic_endpoints' in self.results:
            endpoints = self.results['basic_endpoints']
            working_endpoints = [e for e in endpoints if e['success_rate'] > 0]

            avg_response_times = [e['avg_response_time_ms'] for e in working_endpoints]

            analysis['summary']['basic_endpoints'] = {
                'total_tested': len(endpoints),
                'working_endpoints': len(working_endpoints),
                'avg_response_time_ms': statistics.mean(avg_response_times) if avg_response_times else 0,
                'fastest_endpoint_ms': min(avg_response_times) if avg_response_times else 0,
                'slowest_endpoint_ms': max(avg_response_times) if avg_response_times else 0
            }

            # Generate recommendations for endpoint performance
            if avg_response_times:
                overall_avg = statistics.mean(avg_response_times)
                if overall_avg > 1000:
                    analysis['recommendations'].append({
                        'category': 'Response Time',
                        'priority': 'HIGH',
                        'issue': f'High average response time ({overall_avg:.1f}ms)',
                        'recommendation': 'Optimize server processing, check database queries, consider caching'
                    })

        # Analyze concurrent load
        if 'concurrent_load' in self.results:
            load = self.results['concurrent_load']

            analysis['summary']['concurrent_performance'] = {
                'requests_per_second': load['requests_per_second'],
                'success_rate': load['successful_requests'] / (load['successful_requests'] + load['failed_requests']),
                'avg_response_time_ms': load['avg_response_time_ms']
            }

            # Generate load recommendations
            if load['requests_per_second'] < 10:
                analysis['recommendations'].append({
                    'category': 'Throughput',
                    'priority': 'MEDIUM',
                    'issue': f'Low throughput ({load["requests_per_second"]:.1f} RPS)',
                    'recommendation': 'Scale server resources, optimize application code, implement connection pooling'
                })

            if load['failed_requests'] > 0:
                analysis['recommendations'].append({
                    'category': 'Reliability',
                    'priority': 'HIGH',
                    'issue': f'{load["failed_requests"]} failed requests under load',
                    'recommendation': 'Investigate error causes, increase server capacity, add error handling'
                })

        # Analyze server discovery
        if 'server_discovery' in self.results:
            discovery = self.results['server_discovery']
            accessible_endpoints = [path for path, data in discovery.items() if data['accessible']]

            analysis['summary']['server_discovery'] = {
                'total_paths_tested': len(discovery),
                'accessible_paths': len(accessible_endpoints),
                'accessible_endpoints': accessible_endpoints
            }

            # Check for security concerns
            security_sensitive_paths = ['/admin', '/secure-admin-ea9017d46e0fe963']
            accessible_sensitive = [path for path in security_sensitive_paths if path in accessible_endpoints]

            if accessible_sensitive:
                analysis['recommendations'].append({
                    'category': 'Security',
                    'priority': 'HIGH',
                    'issue': f'Sensitive admin paths accessible: {accessible_sensitive}',
                    'recommendation': 'Ensure admin interfaces are properly secured with authentication'
                })

        return analysis

    async def run_performance_test(self):
        """Run the complete performance test suite"""
        logger.info("Starting HackerExperience Performance Test")

        try:
            # Discover server endpoints
            await self.test_server_discovery()

            # Test basic endpoints
            await self.test_basic_endpoints()

            # Test concurrent load
            for concurrent_count in [5, 10, 20]:
                await self.test_concurrent_load(concurrent_count)
                await asyncio.sleep(1)  # Brief pause between load tests

            # Analyze results
            analysis = self.analyze_results()

            # Save detailed results
            with open('performance_test_results.json', 'w') as f:
                json.dump({
                    'raw_results': self.results,
                    'analysis': analysis
                }, f, indent=2, default=str)

            return analysis

        except Exception as e:
            logger.error(f"Performance test failed: {e}")
            return {'error': str(e)}


async def main():
    logger.info("Starting Simple HackerExperience Performance Test")

    async with SimplePerformanceTester() as tester:
        results = await tester.run_performance_test()

        print("\n" + "="*60)
        print("HACKEREXPERIENCE PERFORMANCE TEST RESULTS")
        print("="*60)

        if 'error' in results:
            print(f"❌ Test failed: {results['error']}")
            return

        # Print summary
        summary = results.get('summary', {})

        if 'server_discovery' in summary:
            discovery = summary['server_discovery']
            print(f"🔍 Server Discovery:")
            print(f"   Accessible endpoints: {discovery['accessible_paths']}/{discovery['total_paths_tested']}")
            print(f"   Working endpoints: {', '.join(discovery['accessible_endpoints'][:5])}...")

        if 'basic_endpoints' in summary:
            basic = summary['basic_endpoints']
            print(f"⚡ Endpoint Performance:")
            print(f"   Working endpoints: {basic['working_endpoints']}/{basic['total_tested']}")
            print(f"   Average response time: {basic['avg_response_time_ms']:.2f}ms")
            print(f"   Range: {basic['fastest_endpoint_ms']:.2f}ms - {basic['slowest_endpoint_ms']:.2f}ms")

        if 'concurrent_performance' in summary:
            concurrent = summary['concurrent_performance']
            print(f"🚀 Concurrent Performance:")
            print(f"   Throughput: {concurrent['requests_per_second']:.2f} RPS")
            print(f"   Success rate: {concurrent['success_rate']:.1%}")
            print(f"   Avg response time: {concurrent['avg_response_time_ms']:.2f}ms")

        # Print recommendations
        recommendations = results.get('recommendations', [])
        if recommendations:
            print(f"\n📋 Performance Recommendations ({len(recommendations)}):")
            for i, rec in enumerate(recommendations[:5], 1):
                priority_emoji = {"HIGH": "🔥", "MEDIUM": "⚠️", "LOW": "💡"}.get(rec['priority'], "📝")
                print(f"{i}. {priority_emoji} [{rec['priority']}] {rec['category']}: {rec['issue']}")
                print(f"   → {rec['recommendation']}")

        print(f"\n💾 Detailed results saved to performance_test_results.json")


if __name__ == "__main__":
    asyncio.run(main())