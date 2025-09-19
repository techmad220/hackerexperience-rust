#!/usr/bin/env python3
"""
Server System Monitoring and Performance Testing Module
=======================================================

This module monitors server system performance including:
- CPU, Memory, Disk, Network usage
- Process monitoring
- System load analysis
- Resource bottleneck detection

Author: Claude Code Performance Testing Bot
Date: 2025-09-19
"""

import asyncio
import paramiko
import time
import json
import logging
import statistics
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional
import re
import threading

logger = logging.getLogger(__name__)

class ServerMonitoringTester:
    def __init__(self, host: str = "172.104.215.73", username: str = "root", password: str = "Cl@ud31776!linode"):
        self.host = host
        self.username = username
        self.password = password
        self.ssh_client = None
        self.monitoring_data = []

    def __enter__(self):
        self.ssh_client = paramiko.SSHClient()
        self.ssh_client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        try:
            self.ssh_client.connect(self.host, username=self.username, password=self.password, timeout=10)
            logger.info(f"Successfully connected to server {self.host}")
            return self
        except Exception as e:
            logger.error(f"Failed to connect to server {self.host}: {e}")
            raise

    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.ssh_client:
            self.ssh_client.close()

    def execute_command(self, command: str, timeout: int = 30) -> tuple[str, str, int]:
        """Execute command on remote server"""
        try:
            stdin, stdout, stderr = self.ssh_client.exec_command(command, timeout=timeout)
            stdout_data = stdout.read().decode('utf-8')
            stderr_data = stderr.read().decode('utf-8')
            exit_code = stdout.channel.recv_exit_status()
            return stdout_data, stderr_data, exit_code
        except Exception as e:
            logger.error(f"Command execution failed: {e}")
            return "", str(e), 1

    def get_system_info(self) -> Dict[str, Any]:
        """Get basic system information"""
        logger.info("Gathering system information...")

        system_info = {}

        # CPU information
        stdout, _, _ = self.execute_command("nproc && cat /proc/cpuinfo | grep 'model name' | head -1")
        lines = stdout.strip().split('\n')
        if lines:
            system_info['cpu_cores'] = int(lines[0]) if lines[0].isdigit() else 1
            if len(lines) > 1 and 'model name' in lines[1]:
                system_info['cpu_model'] = lines[1].split(':')[1].strip()

        # Memory information
        stdout, _, _ = self.execute_command("free -m")
        for line in stdout.split('\n'):
            if line.startswith('Mem:'):
                parts = line.split()
                system_info['total_memory_mb'] = int(parts[1])
                break

        # Disk information
        stdout, _, _ = self.execute_command("df -h / | tail -1")
        if stdout.strip():
            parts = stdout.strip().split()
            if len(parts) >= 4:
                system_info['disk_total'] = parts[1]
                system_info['disk_used'] = parts[2]
                system_info['disk_available'] = parts[3]
                system_info['disk_use_percent'] = parts[4]

        # OS information
        stdout, _, _ = self.execute_command("uname -a && cat /etc/os-release | grep PRETTY_NAME")
        lines = stdout.strip().split('\n')
        if lines:
            system_info['kernel'] = lines[0]
            for line in lines:
                if 'PRETTY_NAME' in line:
                    system_info['os'] = line.split('=')[1].strip('"')
                    break

        # Network information
        stdout, _, _ = self.execute_command("hostname -I | awk '{print $1}'")
        if stdout.strip():
            system_info['internal_ip'] = stdout.strip()

        return system_info

    def get_current_performance_snapshot(self) -> Dict[str, Any]:
        """Get current system performance snapshot"""
        snapshot = {
            'timestamp': datetime.now().isoformat()
        }

        # CPU usage
        stdout, _, _ = self.execute_command("top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | sed 's/%us,//'")
        if stdout.strip():
            try:
                snapshot['cpu_usage_percent'] = float(stdout.strip())
            except:
                snapshot['cpu_usage_percent'] = 0.0

        # Memory usage
        stdout, _, _ = self.execute_command(
            "free | grep Mem | awk '{printf \"%.2f\", ($3/$2) * 100.0}'"
        )
        if stdout.strip():
            try:
                snapshot['memory_usage_percent'] = float(stdout.strip())
            except:
                snapshot['memory_usage_percent'] = 0.0

        # Load average
        stdout, _, _ = self.execute_command("uptime | awk -F'load average:' '{print $2}'")
        if stdout.strip():
            load_parts = stdout.strip().split(',')
            try:
                snapshot['load_1min'] = float(load_parts[0].strip())
                snapshot['load_5min'] = float(load_parts[1].strip()) if len(load_parts) > 1 else 0.0
                snapshot['load_15min'] = float(load_parts[2].strip()) if len(load_parts) > 2 else 0.0
            except:
                snapshot['load_1min'] = snapshot['load_5min'] = snapshot['load_15min'] = 0.0

        # Disk I/O
        stdout, _, _ = self.execute_command("iostat -d 1 2 | tail -n +4 | head -1")
        if stdout.strip():
            parts = stdout.strip().split()
            if len(parts) >= 4:
                try:
                    snapshot['disk_read_kb_s'] = float(parts[2])
                    snapshot['disk_write_kb_s'] = float(parts[3])
                except:
                    snapshot['disk_read_kb_s'] = snapshot['disk_write_kb_s'] = 0.0

        # Network I/O
        stdout, _, _ = self.execute_command(
            "cat /proc/net/dev | grep -E 'eth0|ens|enp' | head -1 | awk '{print $2, $10}'"
        )
        if stdout.strip():
            parts = stdout.strip().split()
            if len(parts) >= 2:
                try:
                    snapshot['network_rx_bytes'] = int(parts[0])
                    snapshot['network_tx_bytes'] = int(parts[1])
                except:
                    snapshot['network_rx_bytes'] = snapshot['network_tx_bytes'] = 0

        return snapshot

    def get_process_information(self) -> Dict[str, Any]:
        """Get information about running processes"""
        logger.info("Gathering process information...")

        process_info = {}

        # Get top CPU consuming processes
        stdout, _, _ = self.execute_command(
            "ps aux --sort=-%cpu | head -11 | tail -10"
        )

        top_cpu_processes = []
        for line in stdout.strip().split('\n'):
            if line.strip():
                parts = line.split()
                if len(parts) >= 11:
                    top_cpu_processes.append({
                        'user': parts[0],
                        'pid': parts[1],
                        'cpu_percent': parts[2],
                        'memory_percent': parts[3],
                        'command': ' '.join(parts[10:])[:50]
                    })

        process_info['top_cpu_processes'] = top_cpu_processes

        # Get top memory consuming processes
        stdout, _, _ = self.execute_command(
            "ps aux --sort=-%mem | head -11 | tail -10"
        )

        top_memory_processes = []
        for line in stdout.strip().split('\n'):
            if line.strip():
                parts = line.split()
                if len(parts) >= 11:
                    top_memory_processes.append({
                        'user': parts[0],
                        'pid': parts[1],
                        'cpu_percent': parts[2],
                        'memory_percent': parts[3],
                        'command': ' '.join(parts[10:])[:50]
                    })

        process_info['top_memory_processes'] = top_memory_processes

        # Check for specific HackerExperience processes
        stdout, _, _ = self.execute_command(
            "ps aux | grep -E '(he-|hacker|rust|node|pm2)' | grep -v grep"
        )

        he_processes = []
        for line in stdout.strip().split('\n'):
            if line.strip():
                parts = line.split()
                if len(parts) >= 11:
                    he_processes.append({
                        'user': parts[0],
                        'pid': parts[1],
                        'cpu_percent': parts[2],
                        'memory_percent': parts[3],
                        'command': ' '.join(parts[10:])
                    })

        process_info['hackerexperience_processes'] = he_processes

        # Get process count
        stdout, _, _ = self.execute_command("ps aux | wc -l")
        if stdout.strip().isdigit():
            process_info['total_processes'] = int(stdout.strip()) - 1  # Subtract header

        return process_info

    def monitor_system_over_time(self, duration_seconds: int = 300, interval_seconds: int = 5) -> List[Dict[str, Any]]:
        """Monitor system performance over time"""
        logger.info(f"Starting system monitoring for {duration_seconds} seconds (interval: {interval_seconds}s)")

        monitoring_data = []
        start_time = time.time()

        while time.time() - start_time < duration_seconds:
            snapshot = self.get_current_performance_snapshot()
            monitoring_data.append(snapshot)

            logger.info(f"CPU: {snapshot.get('cpu_usage_percent', 0):.1f}%, "
                       f"Memory: {snapshot.get('memory_usage_percent', 0):.1f}%, "
                       f"Load: {snapshot.get('load_1min', 0):.2f}")

            time.sleep(interval_seconds)

        self.monitoring_data.extend(monitoring_data)
        return monitoring_data

    def analyze_performance_patterns(self, monitoring_data: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Analyze performance patterns from monitoring data"""
        logger.info("Analyzing performance patterns...")

        if not monitoring_data:
            return {"error": "No monitoring data available"}

        analysis = {}

        # CPU analysis
        cpu_values = [d.get('cpu_usage_percent', 0) for d in monitoring_data]
        if cpu_values:
            analysis['cpu_analysis'] = {
                'average': statistics.mean(cpu_values),
                'minimum': min(cpu_values),
                'maximum': max(cpu_values),
                'median': statistics.median(cpu_values),
                'p95': sorted(cpu_values)[int(0.95 * len(cpu_values))] if cpu_values else 0,
                'high_usage_periods': len([v for v in cpu_values if v > 80]),
                'total_samples': len(cpu_values)
            }

        # Memory analysis
        memory_values = [d.get('memory_usage_percent', 0) for d in monitoring_data]
        if memory_values:
            analysis['memory_analysis'] = {
                'average': statistics.mean(memory_values),
                'minimum': min(memory_values),
                'maximum': max(memory_values),
                'median': statistics.median(memory_values),
                'p95': sorted(memory_values)[int(0.95 * len(memory_values))] if memory_values else 0,
                'high_usage_periods': len([v for v in memory_values if v > 85]),
                'total_samples': len(memory_values)
            }

        # Load average analysis
        load_1min_values = [d.get('load_1min', 0) for d in monitoring_data if d.get('load_1min') is not None]
        if load_1min_values:
            analysis['load_analysis'] = {
                'average_1min': statistics.mean(load_1min_values),
                'maximum_1min': max(load_1min_values),
                'median_1min': statistics.median(load_1min_values),
                'high_load_periods': len([v for v in load_1min_values if v > 2.0]),
                'total_samples': len(load_1min_values)
            }

        return analysis

    def check_system_health(self) -> Dict[str, Any]:
        """Check overall system health"""
        logger.info("Checking system health...")

        health_status = {
            'overall_status': 'HEALTHY',
            'checks': {},
            'warnings': [],
            'critical_issues': []
        }

        # Check disk space
        stdout, _, _ = self.execute_command("df / | tail -1 | awk '{print $5}' | sed 's/%//'")
        if stdout.strip().isdigit():
            disk_usage = int(stdout.strip())
            health_status['checks']['disk_usage_percent'] = disk_usage

            if disk_usage > 90:
                health_status['critical_issues'].append(f"Critical: Disk usage at {disk_usage}%")
                health_status['overall_status'] = 'CRITICAL'
            elif disk_usage > 80:
                health_status['warnings'].append(f"Warning: Disk usage at {disk_usage}%")
                if health_status['overall_status'] == 'HEALTHY':
                    health_status['overall_status'] = 'WARNING'

        # Check memory usage
        stdout, _, _ = self.execute_command("free | grep Mem | awk '{printf \"%.0f\", ($3/$2) * 100.0}'")
        if stdout.strip():
            try:
                memory_usage = float(stdout.strip())
                health_status['checks']['memory_usage_percent'] = memory_usage

                if memory_usage > 95:
                    health_status['critical_issues'].append(f"Critical: Memory usage at {memory_usage:.1f}%")
                    health_status['overall_status'] = 'CRITICAL'
                elif memory_usage > 85:
                    health_status['warnings'].append(f"Warning: Memory usage at {memory_usage:.1f}%")
                    if health_status['overall_status'] == 'HEALTHY':
                        health_status['overall_status'] = 'WARNING'
            except:
                pass

        # Check load average
        stdout, _, _ = self.execute_command("uptime | awk -F'load average:' '{print $2}' | awk -F',' '{print $1}'")
        if stdout.strip():
            try:
                load_1min = float(stdout.strip())
                health_status['checks']['load_1min'] = load_1min

                # Get number of CPU cores for comparison
                stdout_cores, _, _ = self.execute_command("nproc")
                cpu_cores = int(stdout_cores.strip()) if stdout_cores.strip().isdigit() else 1

                if load_1min > cpu_cores * 2:
                    health_status['critical_issues'].append(f"Critical: High load average {load_1min:.2f} (CPUs: {cpu_cores})")
                    health_status['overall_status'] = 'CRITICAL'
                elif load_1min > cpu_cores * 1.5:
                    health_status['warnings'].append(f"Warning: Elevated load average {load_1min:.2f} (CPUs: {cpu_cores})")
                    if health_status['overall_status'] == 'HEALTHY':
                        health_status['overall_status'] = 'WARNING'
            except:
                pass

        # Check for zombie processes
        stdout, _, _ = self.execute_command("ps aux | awk '{print $8}' | grep -c Z")
        if stdout.strip().isdigit():
            zombie_count = int(stdout.strip())
            health_status['checks']['zombie_processes'] = zombie_count

            if zombie_count > 10:
                health_status['warnings'].append(f"Warning: {zombie_count} zombie processes detected")
                if health_status['overall_status'] == 'HEALTHY':
                    health_status['overall_status'] = 'WARNING'

        # Check if critical services are running
        critical_services = ['sshd', 'systemd']
        running_services = []

        for service in critical_services:
            stdout, _, _ = self.execute_command(f"systemctl is-active {service}")
            status = stdout.strip()
            running_services.append({
                'service': service,
                'status': status,
                'running': status == 'active'
            })

            if status != 'active':
                health_status['critical_issues'].append(f"Critical: Service {service} is not active ({status})")
                health_status['overall_status'] = 'CRITICAL'

        health_status['services'] = running_services

        return health_status

    def run_comprehensive_server_test(self) -> Dict[str, Any]:
        """Run comprehensive server monitoring test"""
        logger.info("Starting comprehensive server monitoring test...")

        results = {
            'test_timestamp': datetime.now().isoformat(),
            'server_host': self.host
        }

        try:
            # Get system information
            results['system_info'] = self.get_system_info()

            # Get initial system health check
            results['initial_health'] = self.check_system_health()

            # Get process information
            results['process_info'] = self.get_process_information()

            # Monitor system over time (5 minutes with 10-second intervals)
            monitoring_data = self.monitor_system_over_time(duration_seconds=300, interval_seconds=10)
            results['monitoring_data'] = monitoring_data

            # Analyze performance patterns
            results['performance_analysis'] = self.analyze_performance_patterns(monitoring_data)

            # Final health check
            results['final_health'] = self.check_system_health()

            # Generate recommendations
            results['recommendations'] = self._generate_server_recommendations(results)

        except Exception as e:
            logger.error(f"Server monitoring test failed: {e}")
            results['error'] = str(e)

        # Save results
        with open('server_monitoring_report.json', 'w') as f:
            json.dump(results, f, indent=2, default=str)

        logger.info("Server monitoring report saved to server_monitoring_report.json")
        return results

    def _generate_server_recommendations(self, results: Dict[str, Any]) -> List[Dict[str, str]]:
        """Generate server optimization recommendations"""
        recommendations = []

        # CPU recommendations
        if 'performance_analysis' in results and 'cpu_analysis' in results['performance_analysis']:
            cpu_analysis = results['performance_analysis']['cpu_analysis']

            if cpu_analysis['average'] > 70:
                recommendations.append({
                    'category': 'CPU Performance',
                    'priority': 'HIGH',
                    'issue': f"High average CPU usage ({cpu_analysis['average']:.1f}%)",
                    'recommendation': 'Consider scaling server resources, optimizing applications, or load balancing'
                })

            if cpu_analysis['high_usage_periods'] > len(results.get('monitoring_data', [])) * 0.3:
                recommendations.append({
                    'category': 'CPU Stability',
                    'priority': 'MEDIUM',
                    'issue': f"Frequent high CPU usage periods ({cpu_analysis['high_usage_periods']} occurrences)",
                    'recommendation': 'Investigate CPU-intensive processes and optimize or redistribute workload'
                })

        # Memory recommendations
        if 'performance_analysis' in results and 'memory_analysis' in results['performance_analysis']:
            memory_analysis = results['performance_analysis']['memory_analysis']

            if memory_analysis['average'] > 80:
                recommendations.append({
                    'category': 'Memory Management',
                    'priority': 'HIGH',
                    'issue': f"High average memory usage ({memory_analysis['average']:.1f}%)",
                    'recommendation': 'Add more RAM, optimize memory-hungry applications, or implement memory caching strategies'
                })

        # Load average recommendations
        if 'performance_analysis' in results and 'load_analysis' in results['performance_analysis']:
            load_analysis = results['performance_analysis']['load_analysis']
            cpu_cores = results.get('system_info', {}).get('cpu_cores', 1)

            if load_analysis['average_1min'] > cpu_cores:
                recommendations.append({
                    'category': 'System Load',
                    'priority': 'MEDIUM',
                    'issue': f"High load average ({load_analysis['average_1min']:.2f}) for {cpu_cores} cores",
                    'recommendation': 'Optimize application performance, consider vertical scaling, or implement job queuing'
                })

        # Process recommendations
        if 'process_info' in results:
            he_processes = results['process_info'].get('hackerexperience_processes', [])

            if not he_processes:
                recommendations.append({
                    'category': 'Application Status',
                    'priority': 'CRITICAL',
                    'issue': 'No HackerExperience processes detected',
                    'recommendation': 'Verify application is running, check service status, review deployment'
                })

        # Health check recommendations
        if 'final_health' in results:
            health = results['final_health']

            if health['overall_status'] in ['WARNING', 'CRITICAL']:
                for warning in health.get('warnings', []):
                    recommendations.append({
                        'category': 'System Health',
                        'priority': 'MEDIUM',
                        'issue': warning,
                        'recommendation': 'Address the specific warning to maintain system stability'
                    })

                for critical in health.get('critical_issues', []):
                    recommendations.append({
                        'category': 'System Health',
                        'priority': 'CRITICAL',
                        'issue': critical,
                        'recommendation': 'Immediate attention required to prevent system failure'
                    })

        return recommendations


def main():
    """Run server monitoring tests"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler('server_monitoring.log'),
            logging.StreamHandler()
        ]
    )

    logger.info("Starting Server Monitoring Test Suite")

    try:
        with ServerMonitoringTester() as monitor:
            results = monitor.run_comprehensive_server_test()

            # Print summary
            print("\n" + "="*60)
            print("SERVER MONITORING TEST SUMMARY")
            print("="*60)

            if 'error' in results:
                print(f"❌ Test failed: {results['error']}")
                return

            # System info
            if 'system_info' in results:
                sys_info = results['system_info']
                print(f"🖥️  Server: {sys_info.get('os', 'Unknown OS')}")
                print(f"   CPU: {sys_info.get('cpu_cores', 'Unknown')} cores - {sys_info.get('cpu_model', 'Unknown')}")
                print(f"   Memory: {sys_info.get('total_memory_mb', 'Unknown')} MB")
                print(f"   Disk: {sys_info.get('disk_used', 'Unknown')}/{sys_info.get('disk_total', 'Unknown')} ({sys_info.get('disk_use_percent', 'Unknown')})")

            # Performance analysis
            if 'performance_analysis' in results:
                perf = results['performance_analysis']

                if 'cpu_analysis' in perf:
                    cpu = perf['cpu_analysis']
                    print(f"⚡ CPU Performance:")
                    print(f"   Average: {cpu['average']:.1f}%, Peak: {cpu['maximum']:.1f}%")
                    print(f"   High usage periods: {cpu['high_usage_periods']}")

                if 'memory_analysis' in perf:
                    mem = perf['memory_analysis']
                    print(f"🧠 Memory Performance:")
                    print(f"   Average: {mem['average']:.1f}%, Peak: {mem['maximum']:.1f}%")

                if 'load_analysis' in perf:
                    load = perf['load_analysis']
                    print(f"📊 Load Average:")
                    print(f"   Average: {load['average_1min']:.2f}, Peak: {load['maximum_1min']:.2f}")

            # Health status
            if 'final_health' in results:
                health = results['final_health']
                status_emoji = {"HEALTHY": "✅", "WARNING": "⚠️", "CRITICAL": "❌"}
                print(f"🏥 Health Status: {status_emoji.get(health['overall_status'], '❓')} {health['overall_status']}")

            # Recommendations
            if 'recommendations' in results:
                print(f"\n📋 Recommendations: {len(results['recommendations'])}")
                for i, rec in enumerate(results['recommendations'][:3], 1):
                    print(f"{i}. [{rec['priority']}] {rec['category']}: {rec['issue']}")
                    print(f"   → {rec['recommendation']}")

            print("\n📄 Full report saved to server_monitoring_report.json")

    except Exception as e:
        logger.error(f"Server monitoring test suite failed: {e}")
        print(f"❌ Test suite failed: {e}")


if __name__ == "__main__":
    main()