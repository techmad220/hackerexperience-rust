#!/usr/bin/env python3
"""
HackerExperience Game Spider - Simple Version
Checks for 404s, broken links, and functionality issues
"""

import requests
import re
import json
import time
from datetime import datetime
from urllib.parse import urljoin

class GameSpider:
    def __init__(self):
        self.frontend_base = "http://localhost:8080"
        self.api_base = "http://localhost:3000"  # Updated to test server port
        self.visited = set()
        self.issues = []
        self.timeout = 5

    def log(self, message, level="INFO"):
        colors = {
            "INFO": "\033[94m",
            "PASS": "\033[92m",
            "FAIL": "\033[91m",
            "WARN": "\033[93m"
        }
        color = colors.get(level, "")
        reset = "\033[0m"
        timestamp = datetime.now().strftime("%H:%M:%S")
        print(f"[{timestamp}] {color}[{level}]{reset} {message}")

    def check_url(self, url, description=""):
        """Check if a URL is accessible"""
        if url in self.visited:
            return True
        self.visited.add(url)

        try:
            response = requests.get(url, timeout=self.timeout)
            if response.status_code == 404:
                self.log(f"404 NOT FOUND: {url} - {description}", "FAIL")
                self.issues.append(f"404: {url}")
                return False
            elif response.status_code >= 500:
                self.log(f"SERVER ERROR {response.status_code}: {url}", "FAIL")
                self.issues.append(f"Server Error {response.status_code}: {url}")
                return False
            elif response.status_code != 200:
                self.log(f"HTTP {response.status_code}: {url}", "WARN")
                self.issues.append(f"HTTP {response.status_code}: {url}")
                return False
            else:
                self.log(f"OK: {url}", "PASS")
                return response
        except requests.exceptions.Timeout:
            self.log(f"TIMEOUT: {url}", "FAIL")
            self.issues.append(f"Timeout: {url}")
            return False
        except Exception as e:
            self.log(f"ERROR: {url} - {e}", "FAIL")
            self.issues.append(f"Error: {url} - {e}")
            return False

    def extract_links(self, html, base_url):
        """Extract links from HTML using regex"""
        links = set()

        # Find href links
        href_pattern = r'href=[\'"]?([^\'" >]+)'
        for match in re.finditer(href_pattern, html):
            link = match.group(1)
            if not link.startswith(('#', 'javascript:', 'mailto:')):
                full_link = urljoin(base_url, link)
                links.add(full_link)

        # Find src links (scripts, images, etc.)
        src_pattern = r'src=[\'"]?([^\'" >]+)'
        for match in re.finditer(src_pattern, html):
            link = match.group(1)
            if not link.startswith('data:'):
                full_link = urljoin(base_url, link)
                links.add(full_link)

        return links

    def test_frontend_pages(self):
        """Test all frontend pages"""
        self.log("\n" + "="*60, "INFO")
        self.log("TESTING FRONTEND PAGES", "INFO")
        self.log("="*60, "INFO")

        pages = [
            "/index.html",
            "/login.html",
            "/game.html",
            "/internet.html",
            "/software.html",
            "/hardware.html",
            "/log.html",
            "/finances.html",
            "/missions.html",
            "/profile.html",
            "/settings.html",
            "/clan.html",
            "/fame.html",
            "/ranking.html",
            "/task_manager.html",
            "/university.html",
            "/utilities.html",
            "/mail.html",
            "/hacked_database.html",
            "/he_game.html",
            "/he_landing.html",
            "/he_authentic.html",
            "/he_exact.html",
            "/he_matrix.html"
        ]

        working_pages = 0
        for page in pages:
            url = self.frontend_base + page
            response = self.check_url(url, f"Frontend page {page}")
            if response:
                working_pages += 1

                # Check for linked resources
                if hasattr(response, 'text'):
                    links = self.extract_links(response.text, url)
                    for link in links:
                        # Only check local resources
                        if link.startswith(self.frontend_base):
                            self.check_url(link, f"Resource from {page}")

        self.log(f"\nFrontend Pages: {working_pages}/{len(pages)} working",
                "PASS" if working_pages == len(pages) else "WARN")

    def test_css_files(self):
        """Test CSS files"""
        self.log("\n" + "="*60, "INFO")
        self.log("TESTING CSS FILES", "INFO")
        self.log("="*60, "INFO")

        css_files = [
            "/css/he-matrix.css",
            "/css/style.css",
            "/css/game.css"
        ]

        for css in css_files:
            url = self.frontend_base + css
            self.check_url(url, "CSS file")

    def test_javascript_files(self):
        """Test JavaScript files"""
        self.log("\n" + "="*60, "INFO")
        self.log("TESTING JAVASCRIPT FILES", "INFO")
        self.log("="*60, "INFO")

        js_files = [
            "/js/game.js",
            "/js/api.js",
            "/js/websocket.js",
            "/js/game-client.js"
        ]

        for js in js_files:
            url = self.frontend_base + js
            self.check_url(url, "JavaScript file")

    def test_api_endpoints(self):
        """Test API endpoints"""
        self.log("\n" + "="*60, "INFO")
        self.log("TESTING API ENDPOINTS", "INFO")
        self.log("="*60, "INFO")

        # Test GET endpoints
        get_endpoints = [
            "/health",
            "/api/state",
            "/api/processes",
            "/api/hardware"
        ]

        for endpoint in get_endpoints:
            url = self.api_base + endpoint
            response = self.check_url(url, f"API GET {endpoint}")
            if response and hasattr(response, 'text'):
                try:
                    data = json.loads(response.text)
                    if 'success' in data:
                        if data['success']:
                            self.log(f"  API Response: Success", "PASS")
                        else:
                            self.log(f"  API Response: Success=False", "WARN")
                except:
                    self.log(f"  Invalid JSON response", "WARN")

        # Test POST endpoints
        self.log("\nTesting POST endpoints...", "INFO")

        # Test process start
        try:
            response = requests.post(
                f"{self.api_base}/api/processes/start",
                json={"process_type": "Scan", "priority": "Normal", "target": "192.168.1.1"},
                timeout=5
            )
            if response.status_code == 200:
                data = response.json()
                if data.get('success'):
                    self.log("POST /api/processes/start: OK", "PASS")
                    process_id = data.get('process_id')

                    # Test process cancel
                    if process_id:
                        response = requests.post(
                            f"{self.api_base}/api/processes/cancel",
                            json={"process_id": process_id},
                            timeout=5
                        )
                        if response.status_code == 200:
                            self.log("POST /api/processes/cancel: OK", "PASS")
                        else:
                            self.log(f"POST /api/processes/cancel: {response.status_code}", "FAIL")
                else:
                    self.log(f"POST /api/processes/start: {data.get('error', 'Failed')}", "WARN")
            else:
                self.log(f"POST /api/processes/start: {response.status_code}", "FAIL")
        except Exception as e:
            self.log(f"POST endpoints error: {e}", "FAIL")

    def test_websocket(self):
        """Test WebSocket endpoint"""
        self.log("\n" + "="*60, "INFO")
        self.log("TESTING WEBSOCKET", "INFO")
        self.log("="*60, "INFO")

        # WebSocket upgrade will fail with regular HTTP, but we can check if endpoint exists
        try:
            response = requests.get(f"{self.api_base}/ws", timeout=5)
            # Expected to fail upgrade, but endpoint should exist
            if response.status_code in [426, 400, 101]:
                self.log("WebSocket endpoint exists", "PASS")
            else:
                self.log(f"WebSocket endpoint returned {response.status_code}", "WARN")
        except:
            self.log("WebSocket endpoint not accessible", "WARN")

    def check_for_common_issues(self):
        """Check for common web app issues"""
        self.log("\n" + "="*60, "INFO")
        self.log("CHECKING COMMON ISSUES", "INFO")
        self.log("="*60, "INFO")

        # Check for common missing files
        common_files = [
            "/favicon.ico",
            "/robots.txt",
            "/sitemap.xml"
        ]

        for file in common_files:
            url = self.frontend_base + file
            # These are optional, so we don't add to issues if missing
            response = requests.get(url, timeout=2)
            if response.status_code == 200:
                self.log(f"Found optional file: {file}", "INFO")
            else:
                self.log(f"Missing optional file: {file} (not critical)", "INFO")

    def generate_report(self):
        """Generate final report"""
        self.log("\n" + "="*70, "INFO")
        self.log("SPIDER REPORT SUMMARY", "INFO")
        self.log("="*70, "INFO")

        self.log(f"\nTotal URLs checked: {len(self.visited)}", "INFO")
        self.log(f"Total issues found: {len(self.issues)}",
                "FAIL" if len(self.issues) > 10 else "WARN" if len(self.issues) > 0 else "PASS")

        if self.issues:
            self.log("\nISSUES FOUND:", "WARN")
            for issue in self.issues:
                self.log(f"  • {issue}", "FAIL")

            # Categorize issues
            four_oh_fours = [i for i in self.issues if i.startswith("404")]
            server_errors = [i for i in self.issues if "Server Error" in i]
            timeouts = [i for i in self.issues if "Timeout" in i]

            self.log("\nISSUE SUMMARY:", "INFO")
            if four_oh_fours:
                self.log(f"  404 Errors: {len(four_oh_fours)}", "FAIL")
            if server_errors:
                self.log(f"  Server Errors: {len(server_errors)}", "FAIL")
            if timeouts:
                self.log(f"  Timeouts: {len(timeouts)}", "WARN")

            self.log("\nRECOMMENDATIONS:", "INFO")
            if four_oh_fours:
                self.log("  🔴 Fix missing pages/resources (404 errors)", "FAIL")
            if server_errors:
                self.log("  🔴 Fix server-side errors", "FAIL")
            if timeouts:
                self.log("  🟡 Optimize slow endpoints", "WARN")
        else:
            self.log("\n✅ NO ISSUES FOUND! Game is fully functional!", "PASS")

        # Save to file
        with open('spider_issues.txt', 'w') as f:
            f.write(f"Spider Report - {datetime.now()}\n")
            f.write("="*50 + "\n")
            f.write(f"URLs Checked: {len(self.visited)}\n")
            f.write(f"Issues Found: {len(self.issues)}\n\n")
            for issue in self.issues:
                f.write(f"{issue}\n")

        self.log("\n📄 Issues saved to spider_issues.txt", "INFO")

    def run(self):
        """Run the complete spider"""
        self.log("="*70, "INFO")
        self.log("HACKEREXPERIENCE GAME SPIDER", "INFO")
        self.log("="*70, "INFO")

        # Run all tests
        self.test_frontend_pages()
        self.test_css_files()
        self.test_javascript_files()
        self.test_api_endpoints()
        self.test_websocket()
        self.check_for_common_issues()

        # Generate report
        self.generate_report()

        return len(self.issues) == 0

if __name__ == "__main__":
    spider = GameSpider()
    success = spider.run()
    exit(0 if success else 1)