#!/usr/bin/env python3
"""
Cache Performance Testing Module
===============================

This module tests caching mechanisms and performance.
Since we can't connect to Redis directly, we'll test HTTP-level caching.

Author: Claude Code Performance Testing Bot
Date: 2025-09-19
"""

import asyncio
import aiohttp
import time
import hashlib
import logging
from datetime import datetime

logger = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class CachePerformanceTester:
    def __init__(self, base_url: str = "http://172.104.215.73"):
        self.base_url = base_url.rstrip('/')
        self.session = None
        self.cache_results = {}

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def test_response_consistency(self, endpoint: str, num_requests: int = 10):
        """Test if responses are consistent (indicating caching)"""
        logger.info(f"Testing response consistency for {endpoint}")

        responses = []
        response_times = []

        for i in range(num_requests):
            start_time = time.time()
            try:
                async with self.session.get(f"{self.base_url}{endpoint}") as response:
                    content = await response.text()
                    headers = dict(response.headers)

                response_time = (time.time() - start_time) * 1000
                response_times.append(response_time)

                # Calculate content hash
                content_hash = hashlib.md5(content.encode()).hexdigest()

                responses.append({
                    'request_num': i + 1,
                    'response_time_ms': response_time,
                    'status_code': response.status,
                    'content_hash': content_hash,
                    'content_length': len(content),
                    'cache_headers': {
                        'cache_control': headers.get('cache-control'),
                        'etag': headers.get('etag'),
                        'last_modified': headers.get('last-modified'),
                        'expires': headers.get('expires')
                    }
                })

            except Exception as e:
                logger.error(f"Request {i+1} failed: {e}")
                responses.append({
                    'request_num': i + 1,
                    'error': str(e)
                })

            # Small delay between requests
            await asyncio.sleep(0.1)

        # Analyze consistency
        content_hashes = [r.get('content_hash') for r in responses if 'content_hash' in r]
        unique_hashes = set(content_hashes)

        analysis = {
            'endpoint': endpoint,
            'total_requests': num_requests,
            'successful_requests': len([r for r in responses if 'content_hash' in r]),
            'unique_content_hashes': len(unique_hashes),
            'content_consistent': len(unique_hashes) <= 1,
            'avg_response_time_ms': sum(response_times) / len(response_times) if response_times else 0,
            'min_response_time_ms': min(response_times) if response_times else 0,
            'max_response_time_ms': max(response_times) if response_times else 0,
            'cache_headers_found': any(
                any(h.values()) for r in responses
                for h in [r.get('cache_headers', {})] if h
            ),
            'responses': responses
        }

        return analysis

    async def test_cache_performance_improvement(self, endpoint: str):
        """Test if repeated requests show performance improvement (cache hits)"""
        logger.info(f"Testing cache performance improvement for {endpoint}")

        # First request (cold cache)
        cold_times = []
        for _ in range(3):
            start_time = time.time()
            try:
                async with self.session.get(f"{self.base_url}{endpoint}") as response:
                    await response.text()
                    cold_time = (time.time() - start_time) * 1000
                    cold_times.append(cold_time)
            except:
                cold_times.append(9999.0)
            await asyncio.sleep(0.1)

        # Wait a moment
        await asyncio.sleep(1)

        # Subsequent requests (warm cache)
        warm_times = []
        for _ in range(10):
            start_time = time.time()
            try:
                async with self.session.get(f"{self.base_url}{endpoint}") as response:
                    await response.text()
                    warm_time = (time.time() - start_time) * 1000
                    warm_times.append(warm_time)
            except:
                warm_times.append(9999.0)
            await asyncio.sleep(0.05)

        cold_avg = sum(cold_times) / len(cold_times) if cold_times else 0
        warm_avg = sum(warm_times) / len(warm_times) if warm_times else 0

        performance_improvement = ((cold_avg - warm_avg) / cold_avg * 100) if cold_avg > 0 else 0

        return {
            'endpoint': endpoint,
            'cold_cache_avg_ms': cold_avg,
            'warm_cache_avg_ms': warm_avg,
            'performance_improvement_percent': performance_improvement,
            'cache_likely_present': performance_improvement > 10,  # 10% improvement suggests caching
            'cold_times': cold_times,
            'warm_times': warm_times
        }

    async def test_concurrent_cache_behavior(self, endpoint: str, concurrent_requests: int = 20):
        """Test cache behavior under concurrent load"""
        logger.info(f"Testing concurrent cache behavior for {endpoint}")

        async def single_request():
            start_time = time.time()
            try:
                async with self.session.get(f"{self.base_url}{endpoint}") as response:
                    content = await response.text()
                    response_time = (time.time() - start_time) * 1000
                    return {
                        'success': True,
                        'response_time_ms': response_time,
                        'status_code': response.status,
                        'content_hash': hashlib.md5(content.encode()).hexdigest()
                    }
            except Exception as e:
                return {
                    'success': False,
                    'response_time_ms': (time.time() - start_time) * 1000,
                    'error': str(e)
                }

        # Execute concurrent requests
        start_time = time.time()
        tasks = [single_request() for _ in range(concurrent_requests)]
        results = await asyncio.gather(*tasks)
        total_time = time.time() - start_time

        successful_results = [r for r in results if r['success']]
        failed_results = [r for r in results if not r['success']]

        # Analyze content consistency
        content_hashes = [r.get('content_hash') for r in successful_results if 'content_hash' in r]
        unique_hashes = set(content_hashes)

        response_times = [r['response_time_ms'] for r in successful_results]

        return {
            'endpoint': endpoint,
            'concurrent_requests': concurrent_requests,
            'total_time_seconds': total_time,
            'successful_requests': len(successful_results),
            'failed_requests': len(failed_results),
            'requests_per_second': len(successful_results) / total_time if total_time > 0 else 0,
            'content_consistent': len(unique_hashes) <= 1,
            'unique_content_versions': len(unique_hashes),
            'avg_response_time_ms': sum(response_times) / len(response_times) if response_times else 0,
            'response_time_consistency': max(response_times) - min(response_times) if response_times else 0
        }

    async def run_cache_tests(self):
        """Run comprehensive cache testing"""
        logger.info("Starting comprehensive cache performance testing...")

        results = {
            'test_timestamp': datetime.now().isoformat(),
            'base_url': self.base_url,
            'tests': {}
        }

        # Test endpoints
        test_endpoints = [
            '/',
            '/health',
            '/api',
            '/metrics'
        ]

        for endpoint in test_endpoints:
            logger.info(f"Testing caching for endpoint: {endpoint}")

            endpoint_results = {}

            try:
                # Test response consistency
                endpoint_results['consistency'] = await self.test_response_consistency(endpoint)

                # Test performance improvement
                endpoint_results['performance'] = await self.test_cache_performance_improvement(endpoint)

                # Test concurrent behavior
                endpoint_results['concurrent'] = await self.test_concurrent_cache_behavior(endpoint)

            except Exception as e:
                logger.error(f"Cache testing failed for {endpoint}: {e}")
                endpoint_results['error'] = str(e)

            results['tests'][endpoint] = endpoint_results

            # Small delay between endpoint tests
            await asyncio.sleep(1)

        # Analyze results
        results['analysis'] = self.analyze_cache_results(results['tests'])

        # Save results
        with open('cache_performance_results.json', 'w') as f:
            import json
            json.dump(results, f, indent=2, default=str)

        return results

    def analyze_cache_results(self, test_results):
        """Analyze cache test results"""
        analysis = {
            'cache_indicators': [],
            'performance_summary': {},
            'recommendations': []
        }

        working_endpoints = []
        cache_evidence = []

        for endpoint, tests in test_results.items():
            if 'error' in tests:
                continue

            working_endpoints.append(endpoint)

            # Check for cache evidence
            evidence_score = 0

            # Evidence from consistency test
            if 'consistency' in tests:
                consistency = tests['consistency']
                if consistency.get('content_consistent', False):
                    evidence_score += 2
                if consistency.get('cache_headers_found', False):
                    evidence_score += 3

            # Evidence from performance test
            if 'performance' in tests:
                performance = tests['performance']
                if performance.get('cache_likely_present', False):
                    evidence_score += 4
                improvement = performance.get('performance_improvement_percent', 0)
                if improvement > 20:  # Significant improvement
                    evidence_score += 2

            # Evidence from concurrent test
            if 'concurrent' in tests:
                concurrent = tests['concurrent']
                if concurrent.get('content_consistent', False):
                    evidence_score += 1
                # Low response time variation suggests caching
                time_variation = concurrent.get('response_time_consistency', 0)
                if time_variation < 50:  # Less than 50ms variation
                    evidence_score += 1

            cache_evidence.append({
                'endpoint': endpoint,
                'evidence_score': evidence_score,
                'likely_cached': evidence_score >= 5
            })

        analysis['cache_indicators'] = cache_evidence

        # Performance summary
        if working_endpoints:
            avg_response_times = []
            for endpoint, tests in test_results.items():
                if 'performance' in tests:
                    avg_time = tests['performance'].get('warm_cache_avg_ms', 0)
                    if avg_time > 0:
                        avg_response_times.append(avg_time)

            analysis['performance_summary'] = {
                'endpoints_tested': len(working_endpoints),
                'endpoints_with_cache_evidence': len([e for e in cache_evidence if e['likely_cached']]),
                'avg_response_time_ms': sum(avg_response_times) / len(avg_response_times) if avg_response_times else 0
            }

        # Recommendations
        cached_endpoints = len([e for e in cache_evidence if e['likely_cached']])
        total_endpoints = len(working_endpoints)

        if cached_endpoints == 0 and total_endpoints > 0:
            analysis['recommendations'].append({
                'category': 'Caching',
                'priority': 'HIGH',
                'issue': 'No cache evidence detected on any endpoints',
                'recommendation': 'Implement HTTP caching headers and consider reverse proxy caching (nginx, Varnish)'
            })
        elif cached_endpoints < total_endpoints * 0.5:
            analysis['recommendations'].append({
                'category': 'Caching',
                'priority': 'MEDIUM',
                'issue': f'Only {cached_endpoints}/{total_endpoints} endpoints show cache evidence',
                'recommendation': 'Extend caching to more endpoints, especially static content and API responses'
            })

        # Check response times
        if analysis['performance_summary'].get('avg_response_time_ms', 0) > 500:
            analysis['recommendations'].append({
                'category': 'Performance',
                'priority': 'MEDIUM',
                'issue': f"Slow average response time ({analysis['performance_summary']['avg_response_time_ms']:.1f}ms)",
                'recommendation': 'Implement aggressive caching and optimize backend processing'
            })

        return analysis


async def main():
    logger.info("Starting Cache Performance Testing")

    async with CachePerformanceTester() as tester:
        results = await tester.run_cache_tests()

        print("\n" + "="*60)
        print("CACHE PERFORMANCE TEST RESULTS")
        print("="*60)

        if 'error' in results:
            print(f"❌ Cache testing failed: {results['error']}")
            return

        analysis = results.get('analysis', {})

        # Cache indicators
        cache_indicators = analysis.get('cache_indicators', [])
        if cache_indicators:
            print("🗄️  Cache Evidence by Endpoint:")
            for indicator in cache_indicators:
                cache_emoji = "✅" if indicator['likely_cached'] else "❌"
                print(f"   {cache_emoji} {indicator['endpoint']}: Score {indicator['evidence_score']}/10")

        # Performance summary
        perf_summary = analysis.get('performance_summary', {})
        if perf_summary:
            print(f"📊 Performance Summary:")
            print(f"   Endpoints tested: {perf_summary['endpoints_tested']}")
            print(f"   Endpoints with cache evidence: {perf_summary['endpoints_with_cache_evidence']}")
            print(f"   Average response time: {perf_summary['avg_response_time_ms']:.2f}ms")

        # Recommendations
        recommendations = analysis.get('recommendations', [])
        if recommendations:
            print(f"\n📋 Cache Recommendations ({len(recommendations)}):")
            for i, rec in enumerate(recommendations, 1):
                priority_emoji = {"HIGH": "🔥", "MEDIUM": "⚠️", "LOW": "💡"}.get(rec['priority'], "📝")
                print(f"{i}. {priority_emoji} [{rec['priority']}] {rec['category']}: {rec['issue']}")
                print(f"   → {rec['recommendation']}")

        print(f"\n💾 Detailed results saved to cache_performance_results.json")


if __name__ == "__main__":
    asyncio.run(main())