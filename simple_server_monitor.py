#!/usr/bin/env python3
"""
Simple Server Monitoring via SSH
================================

This script monitors the HackerExperience server using SSH commands.
"""

import subprocess
import json
import time
import logging
from datetime import datetime

logger = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class SimpleServerMonitor:
    def __init__(self, host="172.104.215.73", user="root", password="Cl@ud31776!linode"):
        self.host = host
        self.user = user
        self.password = password

    def run_ssh_command(self, command):
        """Execute SSH command using sshpass"""
        try:
            result = subprocess.run([
                'sshpass', '-p', self.password,
                'ssh', '-o', 'StrictHostKeyChecking=no',
                f'{self.user}@{self.host}',
                command
            ], capture_output=True, text=True, timeout=10)

            return result.stdout.strip(), result.stderr.strip(), result.returncode
        except Exception as e:
            logger.error(f"SSH command failed: {e}")
            return "", str(e), 1

    def get_system_info(self):
        """Get system information"""
        logger.info("Gathering system information...")

        info = {}

        # System info
        stdout, _, _ = self.run_ssh_command("uname -a")
        if stdout:
            info['kernel'] = stdout

        # CPU info
        stdout, _, _ = self.run_ssh_command("nproc")
        if stdout.isdigit():
            info['cpu_cores'] = int(stdout)

        # Memory info
        stdout, _, _ = self.run_ssh_command("free -m | grep '^Mem:' | awk '{print $2, $3, $7}'")
        if stdout:
            parts = stdout.split()
            if len(parts) >= 3:
                info['total_memory_mb'] = int(parts[0])
                info['used_memory_mb'] = int(parts[1])
                info['available_memory_mb'] = int(parts[2])
                info['memory_usage_percent'] = (int(parts[1]) / int(parts[0])) * 100

        # Disk info
        stdout, _, _ = self.run_ssh_command("df -h / | tail -1 | awk '{print $2, $3, $4, $5}'")
        if stdout:
            parts = stdout.split()
            if len(parts) >= 4:
                info['disk_total'] = parts[0]
                info['disk_used'] = parts[1]
                info['disk_available'] = parts[2]
                info['disk_usage_percent'] = parts[3]

        # Load average
        stdout, _, _ = self.run_ssh_command("uptime | awk -F'load average:' '{print $2}'")
        if stdout:
            loads = [float(x.strip()) for x in stdout.split(',') if x.strip()]
            if loads:
                info['load_1min'] = loads[0]
                if len(loads) > 1:
                    info['load_5min'] = loads[1]
                if len(loads) > 2:
                    info['load_15min'] = loads[2]

        return info

    def get_process_info(self):
        """Get process information"""
        logger.info("Gathering process information...")

        process_info = {}

        # Top CPU processes
        stdout, _, _ = self.run_ssh_command("ps aux --sort=-%cpu | head -6 | tail -5")
        top_cpu = []
        for line in stdout.split('\n'):
            if line.strip():
                parts = line.split(None, 10)
                if len(parts) >= 11:
                    top_cpu.append({
                        'user': parts[0],
                        'pid': parts[1],
                        'cpu_percent': parts[2],
                        'memory_percent': parts[3],
                        'command': parts[10][:50]
                    })
        process_info['top_cpu_processes'] = top_cpu

        # Check for HackerExperience processes
        stdout, _, _ = self.run_ssh_command("ps aux | grep -E '(python|node|rust|he-|hacker)' | grep -v grep")
        he_processes = []
        for line in stdout.split('\n'):
            if line.strip():
                parts = line.split(None, 10)
                if len(parts) >= 11:
                    he_processes.append({
                        'user': parts[0],
                        'pid': parts[1],
                        'cpu_percent': parts[2],
                        'memory_percent': parts[3],
                        'command': parts[10]
                    })
        process_info['he_processes'] = he_processes

        # Total processes
        stdout, _, _ = self.run_ssh_command("ps aux | wc -l")
        if stdout.isdigit():
            process_info['total_processes'] = int(stdout) - 1

        return process_info

    def get_network_info(self):
        """Get network information"""
        logger.info("Gathering network information...")

        network_info = {}

        # Listening ports
        stdout, _, _ = self.run_ssh_command("netstat -tlnp | grep LISTEN")
        listening_ports = []
        for line in stdout.split('\n'):
            if line.strip():
                parts = line.split()
                if len(parts) >= 4:
                    port_info = parts[3].split(':')[-1]
                    listening_ports.append(port_info)

        network_info['listening_ports'] = listening_ports

        # Network connections
        stdout, _, _ = self.run_ssh_command("netstat -an | grep ESTABLISHED | wc -l")
        if stdout.isdigit():
            network_info['established_connections'] = int(stdout)

        return network_info

    def check_service_status(self):
        """Check critical services"""
        logger.info("Checking service status...")

        services = {}

        critical_services = ['sshd', 'systemd-resolved', 'systemd-networkd']

        for service in critical_services:
            stdout, _, _ = self.run_ssh_command(f"systemctl is-active {service}")
            services[service] = {
                'status': stdout,
                'active': stdout == 'active'
            }

        return services

    def monitor_performance_snapshot(self, duration=60, interval=5):
        """Take performance snapshots over time"""
        logger.info(f"Taking performance snapshots for {duration} seconds...")

        snapshots = []
        start_time = time.time()

        while time.time() - start_time < duration:
            snapshot = {
                'timestamp': datetime.now().isoformat()
            }

            # CPU usage
            stdout, _, _ = self.run_ssh_command("top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | sed 's/%us,//'")
            if stdout:
                try:
                    snapshot['cpu_usage_percent'] = float(stdout)
                except:
                    snapshot['cpu_usage_percent'] = 0.0

            # Memory usage
            stdout, _, _ = self.run_ssh_command("free | grep Mem | awk '{printf \"%.2f\", ($3/$2) * 100.0}'")
            if stdout:
                try:
                    snapshot['memory_usage_percent'] = float(stdout)
                except:
                    snapshot['memory_usage_percent'] = 0.0

            # Load average
            stdout, _, _ = self.run_ssh_command("uptime | awk -F'load average:' '{print $2}' | awk -F',' '{print $1}'")
            if stdout:
                try:
                    snapshot['load_1min'] = float(stdout.strip())
                except:
                    snapshot['load_1min'] = 0.0

            snapshots.append(snapshot)
            logger.info(f"Snapshot: CPU {snapshot.get('cpu_usage_percent', 0):.1f}%, Memory {snapshot.get('memory_usage_percent', 0):.1f}%, Load {snapshot.get('load_1min', 0):.2f}")

            if time.time() - start_time < duration:
                time.sleep(interval)

        return snapshots

    def analyze_performance(self, snapshots):
        """Analyze performance snapshots"""
        if not snapshots:
            return {}

        cpu_values = [s.get('cpu_usage_percent', 0) for s in snapshots if s.get('cpu_usage_percent') is not None]
        memory_values = [s.get('memory_usage_percent', 0) for s in snapshots if s.get('memory_usage_percent') is not None]
        load_values = [s.get('load_1min', 0) for s in snapshots if s.get('load_1min') is not None]

        analysis = {}

        if cpu_values:
            analysis['cpu_analysis'] = {
                'average': sum(cpu_values) / len(cpu_values),
                'minimum': min(cpu_values),
                'maximum': max(cpu_values),
                'samples': len(cpu_values)
            }

        if memory_values:
            analysis['memory_analysis'] = {
                'average': sum(memory_values) / len(memory_values),
                'minimum': min(memory_values),
                'maximum': max(memory_values),
                'samples': len(memory_values)
            }

        if load_values:
            analysis['load_analysis'] = {
                'average': sum(load_values) / len(load_values),
                'minimum': min(load_values),
                'maximum': max(load_values),
                'samples': len(load_values)
            }

        return analysis

    def generate_recommendations(self, system_info, performance_analysis, process_info):
        """Generate performance recommendations"""
        recommendations = []

        # CPU recommendations
        if 'cpu_analysis' in performance_analysis:
            cpu = performance_analysis['cpu_analysis']
            if cpu['average'] > 80:
                recommendations.append({
                    'category': 'CPU Performance',
                    'priority': 'HIGH',
                    'issue': f"High average CPU usage ({cpu['average']:.1f}%)",
                    'recommendation': 'Consider scaling resources or optimizing CPU-intensive processes'
                })

        # Memory recommendations
        if 'memory_usage_percent' in system_info:
            memory_usage = system_info['memory_usage_percent']
            if memory_usage > 85:
                recommendations.append({
                    'category': 'Memory Usage',
                    'priority': 'HIGH',
                    'issue': f"High memory usage ({memory_usage:.1f}%)",
                    'recommendation': 'Add more RAM or optimize memory usage'
                })

        # Load average recommendations
        cpu_cores = system_info.get('cpu_cores', 1)
        if 'load_1min' in system_info:
            load = system_info['load_1min']
            if load > cpu_cores * 1.5:
                recommendations.append({
                    'category': 'System Load',
                    'priority': 'MEDIUM',
                    'issue': f"High load average ({load:.2f}) for {cpu_cores} cores",
                    'recommendation': 'Investigate high-load processes and consider optimization'
                })

        # Process recommendations
        he_processes = process_info.get('he_processes', [])
        if not he_processes:
            recommendations.append({
                'category': 'Application',
                'priority': 'CRITICAL',
                'issue': 'No HackerExperience processes detected',
                'recommendation': 'Check if the application is running and properly deployed'
            })

        return recommendations

    def run_comprehensive_test(self):
        """Run comprehensive server monitoring"""
        logger.info("Starting comprehensive server monitoring...")

        results = {
            'test_timestamp': datetime.now().isoformat(),
            'server_host': self.host
        }

        try:
            # Get system information
            results['system_info'] = self.get_system_info()

            # Get process information
            results['process_info'] = self.get_process_info()

            # Get network information
            results['network_info'] = self.get_network_info()

            # Check services
            results['service_status'] = self.check_service_status()

            # Monitor performance over time (2 minutes for demo)
            snapshots = self.monitor_performance_snapshot(duration=120, interval=10)
            results['performance_snapshots'] = snapshots

            # Analyze performance
            results['performance_analysis'] = self.analyze_performance(snapshots)

            # Generate recommendations
            results['recommendations'] = self.generate_recommendations(
                results['system_info'],
                results['performance_analysis'],
                results['process_info']
            )

        except Exception as e:
            logger.error(f"Server monitoring failed: {e}")
            results['error'] = str(e)

        # Save results
        with open('server_monitoring_results.json', 'w') as f:
            json.dump(results, f, indent=2, default=str)

        return results


def main():
    logger.info("Starting Simple Server Monitoring")

    monitor = SimpleServerMonitor()
    results = monitor.run_comprehensive_test()

    print("\n" + "="*60)
    print("SERVER MONITORING RESULTS")
    print("="*60)

    if 'error' in results:
        print(f"❌ Monitoring failed: {results['error']}")
        return

    # System info
    if 'system_info' in results:
        sys_info = results['system_info']
        print(f"🖥️  System Information:")
        print(f"   CPU Cores: {sys_info.get('cpu_cores', 'Unknown')}")
        print(f"   Memory: {sys_info.get('used_memory_mb', 0):.0f}/{sys_info.get('total_memory_mb', 0):.0f} MB ({sys_info.get('memory_usage_percent', 0):.1f}%)")
        print(f"   Disk: {sys_info.get('disk_used', 'Unknown')}/{sys_info.get('disk_total', 'Unknown')} ({sys_info.get('disk_usage_percent', 'Unknown')})")
        print(f"   Load: {sys_info.get('load_1min', 0):.2f} (1min)")

    # Process info
    if 'process_info' in results:
        proc_info = results['process_info']
        print(f"📊 Process Information:")
        print(f"   Total processes: {proc_info.get('total_processes', 'Unknown')}")
        print(f"   HackerExperience processes: {len(proc_info.get('he_processes', []))}")

        # Show HE processes
        for proc in proc_info.get('he_processes', [])[:3]:
            print(f"     → PID {proc['pid']}: {proc['command'][:60]}...")

    # Network info
    if 'network_info' in results:
        net_info = results['network_info']
        print(f"🌐 Network Information:")
        print(f"   Listening ports: {', '.join(net_info.get('listening_ports', [])[:10])}")
        print(f"   Established connections: {net_info.get('established_connections', 'Unknown')}")

    # Performance analysis
    if 'performance_analysis' in results:
        perf = results['performance_analysis']

        if 'cpu_analysis' in perf:
            cpu = perf['cpu_analysis']
            print(f"⚡ CPU Performance:")
            print(f"   Average: {cpu['average']:.1f}%, Peak: {cpu['maximum']:.1f}%")

        if 'memory_analysis' in perf:
            mem = perf['memory_analysis']
            print(f"🧠 Memory Performance:")
            print(f"   Average: {mem['average']:.1f}%, Peak: {mem['maximum']:.1f}%")

    # Recommendations
    if 'recommendations' in results:
        recommendations = results['recommendations']
        print(f"\n📋 Recommendations ({len(recommendations)}):")
        for i, rec in enumerate(recommendations[:5], 1):
            priority_emoji = {"HIGH": "🔥", "MEDIUM": "⚠️", "LOW": "💡", "CRITICAL": "❌"}.get(rec['priority'], "📝")
            print(f"{i}. {priority_emoji} [{rec['priority']}] {rec['category']}: {rec['issue']}")
            print(f"   → {rec['recommendation']}")

    print(f"\n💾 Detailed results saved to server_monitoring_results.json")


if __name__ == "__main__":
    main()