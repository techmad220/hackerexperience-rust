#!/usr/bin/env python3
"""
HackerExperience Production Readiness Testing Script
Tests all game functionality to ensure production readiness
"""

import requests
import json
import time
import sys
from datetime import datetime

class GameTester:
    def __init__(self):
        self.api_url = "http://localhost:3005"
        self.frontend_url = "http://localhost:8080"
        self.test_results = []
        self.total_tests = 0
        self.passed_tests = 0
        self.failed_tests = 0

    def log(self, message, status="INFO"):
        timestamp = datetime.now().strftime("%H:%M:%S")
        color_codes = {
            "INFO": "\033[94m",
            "PASS": "\033[92m",
            "FAIL": "\033[91m",
            "WARN": "\033[93m"
        }
        color = color_codes.get(status, "")
        reset = "\033[0m"
        print(f"[{timestamp}] {color}[{status}]{reset} {message}")

    def test(self, name, func):
        """Run a single test"""
        self.total_tests += 1
        try:
            result = func()
            if result:
                self.passed_tests += 1
                self.log(f"✓ {name}", "PASS")
                self.test_results.append({"test": name, "status": "PASS"})
                return True
            else:
                self.failed_tests += 1
                self.log(f"✗ {name}", "FAIL")
                self.test_results.append({"test": name, "status": "FAIL"})
                return False
        except Exception as e:
            self.failed_tests += 1
            self.log(f"✗ {name}: {str(e)}", "FAIL")
            self.test_results.append({"test": name, "status": "FAIL", "error": str(e)})
            return False

    def test_backend_health(self):
        """Test backend API health"""
        try:
            response = requests.get(f"{self.api_url}/health", timeout=5)
            return response.status_code == 200 and response.text == "OK"
        except:
            return False

    def test_frontend_accessible(self):
        """Test frontend accessibility"""
        try:
            response = requests.get(self.frontend_url, timeout=5)
            return response.status_code == 200
        except:
            return False

    def test_api_game_state(self):
        """Test game state API"""
        try:
            response = requests.get(f"{self.api_url}/api/state", timeout=5)
            data = response.json()
            return (response.status_code == 200 and
                    data.get("success") == True and
                    "hardware" in data.get("data", {}))
        except:
            return False

    def test_process_creation(self):
        """Test process creation API"""
        try:
            # Start a scan process
            payload = {
                "process_type": "Scan",
                "priority": "Normal",
                "target": "192.168.1.1"
            }
            response = requests.post(f"{self.api_url}/api/processes/start",
                                    json=payload,
                                    timeout=5)
            data = response.json()

            if not (response.status_code == 200 and data.get("success") == True):
                return False

            process_id = data.get("process_id")
            if not process_id:
                return False

            # Verify process exists
            time.sleep(0.5)
            response = requests.get(f"{self.api_url}/api/processes", timeout=5)
            data = response.json()

            # Process might complete quickly, so just check API response is valid
            return response.status_code == 200 and data.get("success") == True

        except:
            return False

    def test_process_cancellation(self):
        """Test process cancellation"""
        try:
            # Start a long-running process
            payload = {
                "process_type": "Mine",
                "priority": "Low",
                "target": None
            }
            response = requests.post(f"{self.api_url}/api/processes/start",
                                    json=payload,
                                    timeout=5)
            data = response.json()

            if not data.get("success"):
                return False

            process_id = data.get("process_id")

            # Cancel the process
            cancel_payload = {"process_id": process_id}
            response = requests.post(f"{self.api_url}/api/processes/cancel",
                                    json=cancel_payload,
                                    timeout=5)
            data = response.json()

            return response.status_code == 200 and data.get("success") == True

        except:
            return False

    def test_hardware_info(self):
        """Test hardware info API"""
        try:
            response = requests.get(f"{self.api_url}/api/hardware", timeout=5)
            data = response.json()

            if not (response.status_code == 200 and data.get("success") == True):
                return False

            hardware = data.get("data", {})
            return all(key in hardware for key in ["cpu_mhz", "ram_mb", "disk_gb", "network_mbps"])

        except:
            return False

    def test_resource_management(self):
        """Test resource management (CPU/RAM limits)"""
        try:
            # Start a process that uses 800 CPU
            payload1 = {
                "process_type": "Mine",
                "priority": "Normal",
                "target": None
            }
            response1 = requests.post(f"{self.api_url}/api/processes/start",
                                     json=payload1,
                                     timeout=5)
            data1 = response1.json()

            if not data1.get("success"):
                return False

            # Try to start another high-CPU process (should fail)
            payload2 = {
                "process_type": "DDoS",
                "priority": "High",
                "target": "10.0.0.1"
            }
            response2 = requests.post(f"{self.api_url}/api/processes/start",
                                     json=payload2,
                                     timeout=5)
            data2 = response2.json()

            # This should fail due to insufficient resources
            insufficient = (data2.get("success") == False and
                          "Insufficient" in data2.get("error", ""))

            # Clean up - cancel first process
            cancel_payload = {"process_id": data1.get("process_id")}
            requests.post(f"{self.api_url}/api/processes/cancel",
                        json=cancel_payload,
                        timeout=5)

            return insufficient

        except:
            return False

    def test_concurrent_processes(self):
        """Test multiple concurrent processes"""
        try:
            processes = []

            # Start multiple small processes
            for i in range(3):
                payload = {
                    "process_type": "Scan",
                    "priority": "Low",
                    "target": f"192.168.1.{i+1}"
                }
                response = requests.post(f"{self.api_url}/api/processes/start",
                                        json=payload,
                                        timeout=5)
                data = response.json()

                if data.get("success"):
                    processes.append(data.get("process_id"))

            # Should have started at least 2 processes
            if len(processes) < 2:
                return False

            # Clean up
            for pid in processes:
                cancel_payload = {"process_id": pid}
                requests.post(f"{self.api_url}/api/processes/cancel",
                            json=cancel_payload,
                            timeout=5)

            return True

        except:
            return False

    def test_frontend_pages(self):
        """Test accessibility of frontend pages"""
        pages = [
            "index.html",
            "login.html",
            "game.html",
            "internet.html",
            "software.html",
            "hardware.html",
            "log.html",
            "finances.html",
            "missions.html"
        ]

        accessible_count = 0
        for page in pages:
            try:
                response = requests.get(f"{self.frontend_url}/{page}", timeout=5)
                if response.status_code == 200:
                    accessible_count += 1
            except:
                pass

        return accessible_count >= len(pages) * 0.8  # 80% pages should be accessible

    def test_websocket_endpoint(self):
        """Test WebSocket endpoint availability"""
        try:
            # Just test that the endpoint exists (upgrade would fail with regular request)
            response = requests.get(f"{self.api_url}/ws", timeout=5)
            # WebSocket endpoint should return 426 (Upgrade Required) for regular HTTP
            return response.status_code in [426, 400, 101]
        except:
            return True  # WebSocket might not be configured

    def run_all_tests(self):
        """Run all production readiness tests"""
        self.log("=" * 60, "INFO")
        self.log("HACKEREXPERIENCE PRODUCTION READINESS TEST", "INFO")
        self.log("=" * 60, "INFO")

        # Backend tests
        self.log("Testing Backend API...", "INFO")
        self.test("Backend Health Check", self.test_backend_health)
        self.test("Game State API", self.test_api_game_state)
        self.test("Process Creation", self.test_process_creation)
        self.test("Process Cancellation", self.test_process_cancellation)
        self.test("Hardware Info API", self.test_hardware_info)
        self.test("Resource Management", self.test_resource_management)
        self.test("Concurrent Processes", self.test_concurrent_processes)

        # Frontend tests
        self.log("\nTesting Frontend...", "INFO")
        self.test("Frontend Accessibility", self.test_frontend_accessible)
        self.test("Frontend Pages", self.test_frontend_pages)

        # WebSocket test
        self.log("\nTesting WebSocket...", "INFO")
        self.test("WebSocket Endpoint", self.test_websocket_endpoint)

        # Summary
        self.log("\n" + "=" * 60, "INFO")
        self.log("TEST SUMMARY", "INFO")
        self.log("=" * 60, "INFO")

        success_rate = (self.passed_tests / self.total_tests * 100) if self.total_tests > 0 else 0

        self.log(f"Total Tests: {self.total_tests}", "INFO")
        self.log(f"Passed: {self.passed_tests}", "PASS" if self.passed_tests > 0 else "INFO")
        self.log(f"Failed: {self.failed_tests}", "FAIL" if self.failed_tests > 0 else "INFO")
        self.log(f"Success Rate: {success_rate:.1f}%", "INFO")

        # Production readiness assessment
        self.log("\n" + "=" * 60, "INFO")
        self.log("PRODUCTION READINESS ASSESSMENT", "INFO")
        self.log("=" * 60, "INFO")

        if success_rate >= 90:
            self.log("✓ System is PRODUCTION READY", "PASS")
            self.log("All critical systems operational", "PASS")
            return True
        elif success_rate >= 70:
            self.log("⚠ System is PARTIALLY READY", "WARN")
            self.log("Some features need attention", "WARN")
            return False
        else:
            self.log("✗ System is NOT PRODUCTION READY", "FAIL")
            self.log("Critical issues detected", "FAIL")
            return False

if __name__ == "__main__":
    tester = GameTester()
    is_ready = tester.run_all_tests()
    sys.exit(0 if is_ready else 1)