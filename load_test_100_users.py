#!/usr/bin/env python3
"""
Load test for HackerExperience with 100+ concurrent users
Tests the production-hardened server under real load
"""

import asyncio
import aiohttp
import json
import time
import random
from datetime import datetime
import statistics

class LoadTester:
    def __init__(self, base_url="http://localhost:3005", num_users=100):
        self.base_url = base_url
        self.num_users = num_users
        self.tokens = []  # Store auth tokens
        self.results = {
            "requests": 0,
            "successes": 0,
            "failures": 0,
            "response_times": [],
            "errors": [],
            "start_time": None,
            "end_time": None
        }

    async def register_and_login(self, session, user_id):
        """Register a user and get auth token"""
        username = f"testuser_{user_id}"
        password = f"password_{user_id}"

        # Try to login first (in case user exists)
        try:
            async with session.post(
                f"{self.base_url}/api/login",
                json={"username": username, "password": password}
            ) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    if data.get("success"):
                        return data.get("token")
        except:
            pass

        # Register new user
        try:
            async with session.post(
                f"{self.base_url}/api/register",
                json={
                    "username": username,
                    "password": password,
                    "email": f"{username}@test.com"
                }
            ) as resp:
                if resp.status == 200:
                    # Login after registration
                    async with session.post(
                        f"{self.base_url}/api/login",
                        json={"username": username, "password": password}
                    ) as login_resp:
                        if login_resp.status == 200:
                            data = await login_resp.json()
                            return data.get("token")
        except Exception as e:
            print(f"Failed to register user {user_id}: {e}")
            return None

    async def simulate_user(self, session, user_id, token):
        """Simulate a single user's activity"""
        headers = {"Authorization": f"Bearer {token}"}

        # User actions to perform
        actions = [
            ("GET", "/api/state"),
            ("GET", "/api/hardware"),
            ("GET", "/api/processes"),
            ("POST", "/api/processes/start", {
                "process_type": random.choice(["Scan", "Download", "Crack", "Mine"]),
                "priority": "normal",
                "target": f"192.168.1.{random.randint(1, 254)}"
            }),
            ("POST", "/api/processes/cancel", {"process_id": random.randint(1, 1000)}),
        ]

        for _ in range(10):  # Each user makes 10 requests
            method, endpoint, *data = random.choice(actions)
            url = f"{self.base_url}{endpoint}"

            start_time = time.time()
            try:
                if method == "GET":
                    async with session.get(url, headers=headers) as resp:
                        await resp.text()
                        response_time = time.time() - start_time
                        self.results["response_times"].append(response_time)

                        if resp.status < 400:
                            self.results["successes"] += 1
                        else:
                            self.results["failures"] += 1

                elif method == "POST":
                    payload = data[0] if data else {}
                    async with session.post(url, json=payload, headers=headers) as resp:
                        await resp.text()
                        response_time = time.time() - start_time
                        self.results["response_times"].append(response_time)

                        if resp.status < 400:
                            self.results["successes"] += 1
                        else:
                            self.results["failures"] += 1

                self.results["requests"] += 1

            except Exception as e:
                self.results["failures"] += 1
                self.results["errors"].append(str(e))
                self.results["requests"] += 1

            # Small delay between requests
            await asyncio.sleep(random.uniform(0.1, 0.5))

    async def run_load_test(self):
        """Run the load test with concurrent users"""
        print(f"🚀 Starting load test with {self.num_users} concurrent users")
        print(f"Target: {self.base_url}")
        print("-" * 60)

        self.results["start_time"] = datetime.now()

        # Create session with connection pool
        connector = aiohttp.TCPConnector(limit=100, limit_per_host=100)
        timeout = aiohttp.ClientTimeout(total=30)

        async with aiohttp.ClientSession(connector=connector, timeout=timeout) as session:
            # Phase 1: Register/login all users
            print("Phase 1: Registering users...")
            auth_tasks = []
            for i in range(self.num_users):
                auth_tasks.append(self.register_and_login(session, i))

            tokens = await asyncio.gather(*auth_tasks)
            self.tokens = [t for t in tokens if t is not None]
            print(f"✓ Registered {len(self.tokens)} users")

            if not self.tokens:
                print("❌ Failed to register any users. Is the server running?")
                return

            # Phase 2: Simulate concurrent user activity
            print(f"\nPhase 2: Simulating {len(self.tokens)} concurrent users...")
            user_tasks = []
            for i, token in enumerate(self.tokens):
                user_tasks.append(self.simulate_user(session, i, token))

            await asyncio.gather(*user_tasks, return_exceptions=True)

        self.results["end_time"] = datetime.now()
        self.print_results()

    def print_results(self):
        """Print load test results"""
        duration = (self.results["end_time"] - self.results["start_time"]).total_seconds()

        print("\n" + "=" * 60)
        print("LOAD TEST RESULTS")
        print("=" * 60)

        print(f"\n📊 Summary:")
        print(f"  Duration: {duration:.2f} seconds")
        print(f"  Total Requests: {self.results['requests']}")
        print(f"  Successful: {self.results['successes']} ({self.results['successes']/max(1, self.results['requests'])*100:.1f}%)")
        print(f"  Failed: {self.results['failures']} ({self.results['failures']/max(1, self.results['requests'])*100:.1f}%)")
        print(f"  Requests/sec: {self.results['requests']/max(1, duration):.2f}")

        if self.results["response_times"]:
            print(f"\n⏱️ Response Times:")
            print(f"  Mean: {statistics.mean(self.results['response_times'])*1000:.2f}ms")
            print(f"  Median: {statistics.median(self.results['response_times'])*1000:.2f}ms")
            print(f"  Min: {min(self.results['response_times'])*1000:.2f}ms")
            print(f"  Max: {max(self.results['response_times'])*1000:.2f}ms")

            # Calculate percentiles
            sorted_times = sorted(self.results['response_times'])
            p95 = sorted_times[int(len(sorted_times) * 0.95)]
            p99 = sorted_times[int(len(sorted_times) * 0.99)]
            print(f"  P95: {p95*1000:.2f}ms")
            print(f"  P99: {p99*1000:.2f}ms")

        if self.results["errors"]:
            print(f"\n❌ Unique Errors ({len(set(self.results['errors']))}):")
            for error in set(self.results["errors"])[:5]:
                print(f"  - {error}")

        # Performance assessment
        print(f"\n🎯 Assessment:")
        success_rate = self.results['successes'] / max(1, self.results['requests']) * 100
        avg_response = statistics.mean(self.results['response_times']) * 1000 if self.results['response_times'] else 0

        if success_rate > 99 and avg_response < 100:
            print("  ✅ EXCELLENT: Server handled load perfectly")
        elif success_rate > 95 and avg_response < 500:
            print("  ✅ GOOD: Server handled load well")
        elif success_rate > 90:
            print("  ⚠️ ACCEPTABLE: Server struggled but survived")
        else:
            print("  ❌ POOR: Server failed under load")

async def main():
    # Test with different user counts
    for num_users in [10, 50, 100, 200]:
        print(f"\n{'='*60}")
        print(f"Testing with {num_users} users")
        print('='*60)

        tester = LoadTester(num_users=num_users)
        await tester.run_load_test()

        if num_users < 200:
            print(f"\n⏳ Cooling down for 5 seconds...")
            await asyncio.sleep(5)

if __name__ == "__main__":
    asyncio.run(main())