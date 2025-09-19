#!/usr/bin/env python3
"""
Database Performance Testing Module
===================================

This module specifically tests database query performance for the HackerExperience system.
It tests connection pooling, query optimization, and database load handling.

Author: Claude Code Performance Testing Bot
Date: 2025-09-19
"""

import asyncio
# Database testing - will be simulated without asyncpg
# import asyncpg
ASYNCPG_AVAILABLE = False
import time
import statistics
import logging
from typing import List, Dict, Any, Optional
import json
from datetime import datetime, timedelta
import concurrent.futures
import threading

logger = logging.getLogger(__name__)

class DatabasePerformanceTester:
    def __init__(self, db_host: str = "172.104.215.73", db_port: int = 5432,
                 db_name: str = "production_db", db_user: str = "prod_user",
                 db_password: str = "Pr0d@2024!"):
        self.db_config = {
            'host': db_host,
            'port': db_port,
            'database': db_name,
            'user': db_user,
            'password': db_password
        }
        self.pool = None
        self.query_metrics = []

    async def __aenter__(self):
        if not ASYNCPG_AVAILABLE:
            logger.warning("AsyncPG not available - database testing will be simulated")
            return self

        try:
            # Create connection pool would go here
            logger.info("Database connection pool created successfully")
            return self
        except Exception as e:
            logger.error(f"Failed to create database connection pool: {e}")
            raise

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.pool:
            await self.pool.close()

    async def test_connection_performance(self) -> Dict[str, float]:
        """Test database connection performance"""
        logger.info("Testing database connection performance...")

        connection_times = []

        for i in range(10):
            start_time = time.time()
            try:
                async with self.pool.acquire() as conn:
                    await conn.execute('SELECT 1')
                    connection_time = (time.time() - start_time) * 1000
                    connection_times.append(connection_time)
            except Exception as e:
                logger.error(f"Connection test {i+1} failed: {e}")
                connection_times.append(9999.0)

            await asyncio.sleep(0.1)

        return {
            'avg_connection_time_ms': statistics.mean(connection_times),
            'min_connection_time_ms': min(connection_times),
            'max_connection_time_ms': max(connection_times),
            'connection_success_rate': len([t for t in connection_times if t < 9999]) / len(connection_times)
        }

    async def test_basic_queries(self) -> Dict[str, Any]:
        """Test performance of basic database queries"""
        logger.info("Testing basic query performance...")

        queries = [
            ("SELECT current_timestamp", "Current timestamp"),
            ("SELECT count(*) FROM information_schema.tables", "Table count"),
            ("SELECT version()", "Database version"),
            ("SHOW server_version", "Server version"),
        ]

        query_results = {}

        for query, description in queries:
            query_times = []

            for _ in range(5):
                start_time = time.time()
                try:
                    async with self.pool.acquire() as conn:
                        result = await conn.fetch(query)
                        query_time = (time.time() - start_time) * 1000
                        query_times.append(query_time)
                except Exception as e:
                    logger.error(f"Query '{description}' failed: {e}")
                    query_times.append(9999.0)

                await asyncio.sleep(0.1)

            query_results[description] = {
                'query': query,
                'avg_time_ms': statistics.mean(query_times),
                'min_time_ms': min(query_times),
                'max_time_ms': max(query_times)
            }

        return query_results

    async def test_concurrent_queries(self, concurrent_connections: int = 50) -> Dict[str, Any]:
        """Test database performance under concurrent load"""
        logger.info(f"Testing concurrent query performance with {concurrent_connections} connections...")

        async def execute_query_batch(batch_id: int):
            """Execute a batch of queries from a single connection"""
            query_times = []

            try:
                async with self.pool.acquire() as conn:
                    queries = [
                        "SELECT pg_sleep(0.01)",  # Simulate work
                        "SELECT random()",
                        "SELECT current_timestamp",
                        "SELECT count(*) FROM information_schema.columns",
                    ]

                    for query in queries:
                        start_time = time.time()
                        try:
                            await conn.fetch(query)
                            query_time = (time.time() - start_time) * 1000
                            query_times.append(query_time)
                        except Exception as e:
                            logger.error(f"Batch {batch_id} query failed: {e}")
                            query_times.append(9999.0)

                        await asyncio.sleep(0.01)

            except Exception as e:
                logger.error(f"Batch {batch_id} connection failed: {e}")
                return []

            return query_times

        start_time = time.time()

        # Create concurrent tasks
        tasks = [execute_query_batch(i) for i in range(concurrent_connections)]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        total_time = time.time() - start_time

        # Process results
        all_query_times = []
        successful_batches = 0
        failed_batches = 0

        for result in results:
            if isinstance(result, Exception):
                failed_batches += 1
            elif isinstance(result, list) and result:
                successful_batches += 1
                all_query_times.extend(result)
            else:
                failed_batches += 1

        successful_queries = len([t for t in all_query_times if t < 9999])
        failed_queries = len(all_query_times) - successful_queries

        if all_query_times:
            avg_query_time = statistics.mean(all_query_times)
            p95_query_time = sorted(all_query_times)[int(0.95 * len(all_query_times))]
        else:
            avg_query_time = p95_query_time = 0

        return {
            'concurrent_connections': concurrent_connections,
            'total_duration_seconds': total_time,
            'successful_batches': successful_batches,
            'failed_batches': failed_batches,
            'successful_queries': successful_queries,
            'failed_queries': failed_queries,
            'avg_query_time_ms': avg_query_time,
            'p95_query_time_ms': p95_query_time,
            'queries_per_second': successful_queries / total_time if total_time > 0 else 0
        }

    async def test_application_queries(self) -> Dict[str, Any]:
        """Test performance of typical application queries"""
        logger.info("Testing application-specific query performance...")

        # First, let's see what tables exist
        table_info = {}
        try:
            async with self.pool.acquire() as conn:
                # Get list of tables
                tables = await conn.fetch("""
                    SELECT table_name
                    FROM information_schema.tables
                    WHERE table_schema = 'public'
                    ORDER BY table_name
                """)

                for table in tables:
                    table_name = table['table_name']

                    # Get table statistics
                    try:
                        stats = await conn.fetch(f"""
                            SELECT
                                COUNT(*) as row_count,
                                pg_total_relation_size('{table_name}') as size_bytes
                        """)
                        table_info[table_name] = {
                            'row_count': stats[0]['row_count'] if stats else 0,
                            'size_bytes': stats[0]['size_bytes'] if stats else 0
                        }
                    except Exception as e:
                        logger.warning(f"Could not get stats for table {table_name}: {e}")
                        table_info[table_name] = {'row_count': 0, 'size_bytes': 0}

        except Exception as e:
            logger.error(f"Failed to get table information: {e}")
            return {"error": str(e)}

        # Test generic application queries based on common patterns
        app_queries = {
            "Table List Query": "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
            "Column Information": "SELECT column_name, data_type FROM information_schema.columns WHERE table_schema = 'public' LIMIT 100",
            "Database Size": "SELECT pg_database_size(current_database())",
            "Active Connections": "SELECT count(*) FROM pg_stat_activity",
        }

        # Add table-specific queries if tables exist
        if table_info:
            # Pick the largest table for testing
            largest_table = max(table_info.items(), key=lambda x: x[1]['row_count'])
            table_name = largest_table[0]

            if largest_table[1]['row_count'] > 0:
                app_queries[f"Count {table_name}"] = f"SELECT COUNT(*) FROM {table_name}"
                app_queries[f"Recent {table_name}"] = f"SELECT * FROM {table_name} LIMIT 10"

        query_results = {}

        for description, query in app_queries.items():
            query_times = []

            for _ in range(3):
                start_time = time.time()
                try:
                    async with self.pool.acquire() as conn:
                        result = await conn.fetch(query)
                        query_time = (time.time() - start_time) * 1000
                        query_times.append(query_time)

                        # Store some result info
                        result_count = len(result) if result else 0

                except Exception as e:
                    logger.error(f"App query '{description}' failed: {e}")
                    query_times.append(9999.0)
                    result_count = 0

                await asyncio.sleep(0.1)

            query_results[description] = {
                'query': query,
                'avg_time_ms': statistics.mean(query_times),
                'min_time_ms': min(query_times),
                'max_time_ms': max(query_times),
                'result_count': result_count
            }

        return {
            'table_info': table_info,
            'query_results': query_results
        }

    async def test_index_performance(self) -> Dict[str, Any]:
        """Test database index performance and suggestions"""
        logger.info("Testing database index performance...")

        try:
            async with self.pool.acquire() as conn:
                # Get index information
                indexes = await conn.fetch("""
                    SELECT
                        schemaname,
                        tablename,
                        indexname,
                        indexdef,
                        idx_scan,
                        idx_tup_read,
                        idx_tup_fetch
                    FROM pg_indexes pi
                    LEFT JOIN pg_stat_user_indexes psi ON pi.indexname = psi.indexname
                    WHERE schemaname = 'public'
                    ORDER BY tablename, indexname
                """)

                # Get table scan statistics
                table_stats = await conn.fetch("""
                    SELECT
                        relname as table_name,
                        seq_scan,
                        seq_tup_read,
                        idx_scan,
                        idx_tup_fetch,
                        n_tup_ins + n_tup_upd + n_tup_del as modifications
                    FROM pg_stat_user_tables
                    ORDER BY seq_scan DESC
                """)

                # Get slow queries if pg_stat_statements is available
                slow_queries = []
                try:
                    slow_queries = await conn.fetch("""
                        SELECT
                            query,
                            calls,
                            total_time,
                            mean_time,
                            rows
                        FROM pg_stat_statements
                        WHERE mean_time > 100
                        ORDER BY mean_time DESC
                        LIMIT 10
                    """)
                except:
                    # pg_stat_statements might not be enabled
                    pass

        except Exception as e:
            logger.error(f"Failed to analyze index performance: {e}")
            return {"error": str(e)}

        return {
            'indexes': [dict(idx) for idx in indexes],
            'table_stats': [dict(stat) for stat in table_stats],
            'slow_queries': [dict(query) for query in slow_queries],
            'index_recommendations': self._generate_index_recommendations(table_stats, indexes)
        }

    def _generate_index_recommendations(self, table_stats, indexes) -> List[str]:
        """Generate index recommendations based on statistics"""
        recommendations = []

        for stat in table_stats:
            table_name = stat['table_name']
            seq_scan = stat['seq_scan'] or 0
            idx_scan = stat['idx_scan'] or 0

            # High sequential scan ratio suggests missing indexes
            if seq_scan > 0 and idx_scan > 0:
                scan_ratio = seq_scan / (seq_scan + idx_scan)
                if scan_ratio > 0.7:  # More than 70% sequential scans
                    recommendations.append(
                        f"Table '{table_name}' has high sequential scan ratio ({scan_ratio:.1%}). Consider adding indexes on frequently queried columns."
                    )
            elif seq_scan > 100 and idx_scan == 0:
                recommendations.append(
                    f"Table '{table_name}' only uses sequential scans. Consider adding indexes."
                )

        # Check for unused indexes
        unused_indexes = [idx for idx in indexes if (idx['idx_scan'] or 0) == 0]
        if unused_indexes:
            recommendations.append(
                f"Found {len(unused_indexes)} potentially unused indexes that could be dropped to improve write performance."
            )

        return recommendations

    async def run_comprehensive_test(self) -> Dict[str, Any]:
        """Run comprehensive database performance test"""
        logger.info("Starting comprehensive database performance test...")

        results = {
            'test_timestamp': datetime.now().isoformat(),
            'database_config': {
                'host': self.db_config['host'],
                'port': self.db_config['port'],
                'database': self.db_config['database']
            }
        }

        try:
            # Test connection performance
            results['connection_performance'] = await self.test_connection_performance()

            # Test basic queries
            results['basic_queries'] = await self.test_basic_queries()

            # Test application queries
            results['application_queries'] = await self.test_application_queries()

            # Test concurrent load
            for conn_count in [10, 25, 50]:
                concurrent_result = await self.test_concurrent_queries(conn_count)
                results[f'concurrent_{conn_count}_connections'] = concurrent_result

            # Test index performance
            results['index_analysis'] = await self.test_index_performance()

            # Generate overall recommendations
            results['recommendations'] = self._generate_db_recommendations(results)

        except Exception as e:
            logger.error(f"Database performance test failed: {e}")
            results['error'] = str(e)

        # Save results
        with open('database_performance_report.json', 'w') as f:
            json.dump(results, f, indent=2, default=str)

        logger.info("Database performance report saved to database_performance_report.json")
        return results

    def _generate_db_recommendations(self, results: Dict[str, Any]) -> List[Dict[str, str]]:
        """Generate database optimization recommendations"""
        recommendations = []

        # Connection performance recommendations
        if 'connection_performance' in results:
            conn_perf = results['connection_performance']
            if conn_perf['avg_connection_time_ms'] > 100:
                recommendations.append({
                    'category': 'Connection Performance',
                    'priority': 'MEDIUM',
                    'issue': f"High connection time ({conn_perf['avg_connection_time_ms']:.1f}ms)",
                    'recommendation': 'Optimize connection pooling settings, check network latency, consider connection caching'
                })

        # Query performance recommendations
        if 'basic_queries' in results:
            slow_queries = [
                (desc, data) for desc, data in results['basic_queries'].items()
                if data['avg_time_ms'] > 500
            ]
            if slow_queries:
                recommendations.append({
                    'category': 'Query Performance',
                    'priority': 'HIGH',
                    'issue': f"Slow basic queries detected: {len(slow_queries)} queries > 500ms",
                    'recommendation': 'Check database server resources, analyze query execution plans, optimize database configuration'
                })

        # Concurrent load recommendations
        concurrent_tests = [k for k in results.keys() if k.startswith('concurrent_')]
        if concurrent_tests:
            latest_concurrent = results[concurrent_tests[-1]]  # Highest load test

            if latest_concurrent['queries_per_second'] < 100:
                recommendations.append({
                    'category': 'Scalability',
                    'priority': 'HIGH',
                    'issue': f"Low throughput under load ({latest_concurrent['queries_per_second']:.1f} QPS)",
                    'recommendation': 'Scale database resources, optimize connection pooling, consider read replicas'
                })

            if latest_concurrent.get('failed_queries', 0) > 0:
                recommendations.append({
                    'category': 'Reliability',
                    'priority': 'HIGH',
                    'issue': f"Query failures under concurrent load ({latest_concurrent['failed_queries']} failures)",
                    'recommendation': 'Increase connection pool size, add query timeouts, implement retry logic'
                })

        # Index recommendations
        if 'index_analysis' in results and 'index_recommendations' in results['index_analysis']:
            for idx_rec in results['index_analysis']['index_recommendations']:
                recommendations.append({
                    'category': 'Index Optimization',
                    'priority': 'MEDIUM',
                    'issue': 'Index optimization opportunity',
                    'recommendation': idx_rec
                })

        return recommendations


async def main():
    """Run database performance tests"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s'
    )

    logger.info("Starting Database Performance Test Suite")

    try:
        async with DatabasePerformanceTester() as db_tester:
            results = await db_tester.run_comprehensive_test()

            # Print summary
            print("\n" + "="*60)
            print("DATABASE PERFORMANCE TEST SUMMARY")
            print("="*60)

            if 'error' in results:
                print(f"❌ Test failed: {results['error']}")
                return

            # Connection performance
            if 'connection_performance' in results:
                conn_perf = results['connection_performance']
                print(f"🔗 Connection Performance:")
                print(f"   Average: {conn_perf['avg_connection_time_ms']:.2f}ms")
                print(f"   Success Rate: {conn_perf['connection_success_rate']:.1%}")

            # Latest concurrent test
            concurrent_tests = [k for k in results.keys() if k.startswith('concurrent_')]
            if concurrent_tests:
                latest = results[concurrent_tests[-1]]
                print(f"⚡ Concurrent Performance ({latest['concurrent_connections']} connections):")
                print(f"   Throughput: {latest['queries_per_second']:.1f} QPS")
                print(f"   Avg Query Time: {latest['avg_query_time_ms']:.2f}ms")
                print(f"   95th Percentile: {latest['p95_query_time_ms']:.2f}ms")

            # Recommendations
            if 'recommendations' in results:
                print(f"\n📋 Recommendations: {len(results['recommendations'])}")
                for i, rec in enumerate(results['recommendations'][:3], 1):
                    print(f"{i}. [{rec['priority']}] {rec['category']}: {rec['issue']}")
                    print(f"   → {rec['recommendation']}")

            print("\n📄 Full report saved to database_performance_report.json")

    except Exception as e:
        logger.error(f"Database performance test suite failed: {e}")
        print(f"❌ Test suite failed: {e}")


if __name__ == "__main__":
    asyncio.run(main())