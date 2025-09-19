#!/usr/bin/env python3
"""
Enhanced Test Server for HackerExperience
Includes all missing endpoints
"""

from flask import Flask, jsonify, request, send_from_directory
from flask_cors import CORS
import os
import json
from datetime import datetime

app = Flask(__name__)
import os
FRONTEND_ORIGIN = os.environ.get('FRONTEND_ORIGIN', 'http://localhost:8080')
CORS(app, resources={r"/*": {"origins": [FRONTEND_ORIGIN]}}, supports_credentials=True)

# Mock data
game_state = {
    "user": {
        "id": 1,
        "username": "player123",
        "level": 42,
        "experience": 125000,
        "reputation": 850,
        "money": 50000
    },
    "hardware": {
        "cpu": 500,
        "ram": 1024,
        "hdd": 5000,
        "net": 100
    },
    "processes": [
        {
            "id": 1,
            "type": "crack",
            "target": "192.168.1.1",
            "progress": 65,
            "end_time": "2025-09-19T12:00:00Z"
        }
    ]
}

# Health check
@app.route('/health')
def health():
    return jsonify({"status": "healthy", "timestamp": datetime.now().isoformat()})

# API State endpoint
@app.route('/api/state')
def api_state():
    return jsonify({
        "success": True,
        "state": game_state,
        "timestamp": datetime.now().isoformat()
    })

# Processes endpoint
@app.route('/api/processes')
def api_processes():
    return jsonify({
        "success": True,
        "processes": game_state["processes"],
        "active_count": len(game_state["processes"])
    })

# Hardware endpoint
@app.route('/api/hardware')
def api_hardware():
    return jsonify({
        "success": True,
        "hardware": game_state["hardware"],
        "usage": {
            "cpu": 45,
            "ram": 67,
            "hdd": 23,
            "net": 12
        }
    })

# Process management
@app.route('/api/processes/start', methods=['POST'])
def start_process():
    data = request.get_json()
    new_process = {
        "id": len(game_state["processes"]) + 1,
        "type": data.get("process_type", "hack"),
        "target": data.get("target", "unknown"),
        "progress": 0,
        "end_time": "2025-09-19T13:00:00Z"
    }
    game_state["processes"].append(new_process)
    return jsonify({
        "success": True,
        "process": new_process,
        "message": "Process started successfully"
    })

# Missions endpoint
@app.route('/api/missions')
def api_missions():
    return jsonify({
        "success": True,
        "missions": [
            {
                "id": 1,
                "mission_type": "tutorial",
                "status": "active",
                "reward_money": 1000,
                "reward_xp": 50,
                "progress": 2,
                "total_steps": 3
            }
        ]
    })

# Software endpoint
@app.route('/api/software/list')
def software_list():
    return jsonify({
        "success": True,
        "software": [
            {
                "id": 1,
                "name": "Cracker v3.0",
                "type": "cracker",
                "version": 3.0,
                "size": 150
            },
            {
                "id": 2,
                "name": "Firewall Pro",
                "type": "firewall",
                "version": 5.2,
                "size": 200
            }
        ]
    })

# Hacking endpoints
@app.route('/api/hacking/scan', methods=['POST'])
def scan_server():
    data = request.get_json()
    return jsonify({
        "success": True,
        "server_info": {
            "ip_address": data.get("target_ip", "1.2.3.4"),
            "hostname": "target.server.com",
            "owner": "Corporation",
            "security_level": 50,
            "firewall_level": 30,
            "is_online": True
        }
    })

@app.route('/api/hacking/internet')
def internet():
    return jsonify({
        "success": True,
        "servers": [
            {
                "ip": "1.2.3.4",
                "hostname": "whois.first.org",
                "type": "Web Server"
            },
            {
                "ip": "5.6.7.8",
                "hostname": "bank.secure.com",
                "type": "Bank Server"
            }
        ]
    })

# WebSocket endpoint (mock)
@app.route('/ws')
def websocket():
    return jsonify({
        "message": "WebSocket endpoint active",
        "connect_url": "ws://localhost:3000/ws",
        "protocols": ["game", "chat"]
    })

# Login endpoint
@app.route('/api/login', methods=['POST'])
def login():
    data = request.get_json()
    return jsonify({
        "success": True,
        "user": game_state["user"],
        "token": "mock_jwt_token_" + str(datetime.now().timestamp()),
        "refresh_token": "mock_refresh_token"
    })

# Register endpoint
@app.route('/api/register', methods=['POST'])
def register():
    data = request.get_json()
    return jsonify({
        "success": True,
        "user": {
            "id": 2,
            "username": data.get("username", "newuser"),
            "email": data.get("email", "user@example.com")
        },
        "token": "mock_jwt_token_new",
        "message": "Registration successful"
    })

# Serve frontend files
@app.route('/')
def index():
    return jsonify({
        "message": "HackerExperience Test Server",
        "version": "1.0.0",
        "endpoints": [
            "/health",
            "/api/state",
            "/api/processes",
            "/api/hardware",
            "/api/missions",
            "/api/software/list",
            "/api/hacking/scan",
            "/api/hacking/internet",
            "/api/login",
            "/api/register",
            "/ws"
        ]
    })

if __name__ == '__main__':
    print("🚀 Starting Enhanced HackerExperience Test Server")
    print("📡 API Server: http://localhost:3000")
    print("🌐 Frontend: http://localhost:8080")
    print("\nAvailable endpoints:")
    print("  GET  /health")
    print("  GET  /api/state")
    print("  GET  /api/processes")
    print("  GET  /api/hardware")
    print("  POST /api/processes/start")
    print("  GET  /api/missions")
    print("  GET  /api/software/list")
    print("  POST /api/hacking/scan")
    print("  GET  /api/hacking/internet")
    print("  POST /api/login")
    print("  POST /api/register")
    print("  GET  /ws (WebSocket mock)")

    app.run(host='0.0.0.0', port=3000, debug=False)
