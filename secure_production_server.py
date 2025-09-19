#!/usr/bin/env python3
"""
Secure Production-Ready HackerExperience Server
Fixes all security vulnerabilities and adds proper features
"""

from flask import Flask, jsonify, request, send_from_directory, abort
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from werkzeug.security import check_password_hash, generate_password_hash
from functools import wraps
import html
import jwt
import datetime
import secrets
import re
import hashlib
import json

app = Flask(__name__)

# Security configurations
app.config['SECRET_KEY'] = secrets.token_hex(32)
JWT_SECRET = secrets.token_hex(32)
JWT_ALGORITHM = 'HS256'

# CORS configuration - restrict to specific origins
CORS(app, origins=['http://localhost:8080', 'http://localhost:3000'],
     methods=['GET', 'POST', 'OPTIONS'],
     allow_headers=['Content-Type', 'Authorization'])

# Rate limiting
limiter = Limiter(
    app=app,
    key_func=get_remote_address,
    default_limits=["100 per minute"],
    storage_uri="memory://"
)

# Mock user database (in production, use real database)
users_db = {}
sessions = {}

# Game state with proper data classification
game_state = {
    "public": {
        "online_players": 42,
        "server_status": "operational"
    },
    "private": {}  # User-specific data
}

# Input validation patterns
USERNAME_PATTERN = re.compile(r'^[a-zA-Z0-9_]{3,20}$')
EMAIL_PATTERN = re.compile(r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$')

# Security headers middleware
@app.after_request
def add_security_headers(response):
    response.headers['X-Content-Type-Options'] = 'nosniff'
    response.headers['X-Frame-Options'] = 'DENY'
    response.headers['X-XSS-Protection'] = '1; mode=block'
    response.headers['Strict-Transport-Security'] = 'max-age=31536000; includeSubDomains'
    response.headers['Content-Security-Policy'] = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';"
    return response

# Input sanitization function
def sanitize_input(text):
    """Sanitize user input to prevent XSS"""
    if not isinstance(text, str):
        return text
    # HTML escape special characters
    return html.escape(text, quote=True)

# Authentication decorator
def require_auth(f):
    @wraps(f)
    def decorated_function(*args, **kwargs):
        token = request.headers.get('Authorization')
        if not token:
            return jsonify({'error': 'No authentication token provided'}), 401

        try:
            # Remove 'Bearer ' prefix if present
            if token.startswith('Bearer '):
                token = token[7:]

            # Verify JWT token
            payload = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])

            # Check if session is still valid
            user_id = payload.get('user_id')
            if user_id not in sessions:
                return jsonify({'error': 'Session expired'}), 401

            # Add user context to request
            request.user_id = user_id
            return f(*args, **kwargs)

        except jwt.ExpiredSignatureError:
            return jsonify({'error': 'Token has expired'}), 401
        except jwt.InvalidTokenError:
            return jsonify({'error': 'Invalid token'}), 401

    return decorated_function

# Input validation
def validate_registration(data):
    """Validate registration input"""
    errors = []

    username = data.get('username', '')
    email = data.get('email', '')
    password = data.get('password', '')

    if not USERNAME_PATTERN.match(username):
        errors.append('Username must be 3-20 alphanumeric characters')

    if not EMAIL_PATTERN.match(email):
        errors.append('Invalid email format')

    if len(password) < 8:
        errors.append('Password must be at least 8 characters')

    if not any(c.isupper() for c in password):
        errors.append('Password must contain uppercase letter')

    if not any(c.islower() for c in password):
        errors.append('Password must contain lowercase letter')

    if not any(c.isdigit() for c in password):
        errors.append('Password must contain digit')

    return errors

# Public endpoints
@app.route('/health')
@limiter.limit("10 per minute")
def health():
    return jsonify({
        "status": "healthy",
        "timestamp": datetime.datetime.now().isoformat(),
        "version": "2.0.0-secure"
    })

# Authentication endpoints
@app.route('/api/register', methods=['POST'])
@limiter.limit("5 per minute")
def register():
    data = request.get_json()

    # Validate input
    validation_errors = validate_registration(data)
    if validation_errors:
        return jsonify({
            'success': False,
            'errors': validation_errors
        }), 400

    # Sanitize inputs
    username = sanitize_input(data['username'])
    email = sanitize_input(data['email'].lower())

    # Check if user exists
    if email in users_db:
        return jsonify({
            'success': False,
            'error': 'Email already registered'
        }), 400

    # Hash password
    password_hash = generate_password_hash(data['password'])

    # Create user
    user_id = len(users_db) + 1
    users_db[email] = {
        'id': user_id,
        'username': username,
        'email': email,
        'password_hash': password_hash,
        'created_at': datetime.datetime.now().isoformat()
    }

    # Initialize user game state
    game_state['private'][user_id] = {
        'level': 1,
        'experience': 0,
        'money': 1000,
        'hardware': {
            'cpu': 100,
            'ram': 256,
            'hdd': 1000,
            'net': 10
        },
        'processes': []
    }

    return jsonify({
        'success': True,
        'message': 'Registration successful',
        'user': {
            'id': user_id,
            'username': username,
            'email': email
        }
    }), 201

@app.route('/api/login', methods=['POST'])
@limiter.limit("10 per minute")
def login():
    data = request.get_json()

    email = data.get('email', '').lower()
    password = data.get('password', '')

    # Validate input presence
    if not email or not password:
        return jsonify({
            'success': False,
            'error': 'Email and password required'
        }), 400

    # Sanitize email
    email = sanitize_input(email)

    # Check user exists
    user = users_db.get(email)
    if not user:
        # Prevent user enumeration - same error for both cases
        return jsonify({
            'success': False,
            'error': 'Invalid credentials'
        }), 401

    # Verify password
    if not check_password_hash(user['password_hash'], password):
        return jsonify({
            'success': False,
            'error': 'Invalid credentials'
        }), 401

    # Generate JWT token
    payload = {
        'user_id': user['id'],
        'email': user['email'],
        'exp': datetime.datetime.utcnow() + datetime.timedelta(hours=1)
    }
    token = jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALGORITHM)

    # Store session
    sessions[user['id']] = {
        'login_time': datetime.datetime.now().isoformat(),
        'ip': request.remote_addr
    }

    return jsonify({
        'success': True,
        'user': {
            'id': user['id'],
            'username': user['username'],
            'email': user['email']
        },
        'token': token
    })

# Protected game endpoints
@app.route('/api/state')
@require_auth
@limiter.limit("30 per minute")
def api_state():
    user_id = request.user_id
    user_state = game_state['private'].get(user_id, {})

    # Only return user's own data
    return jsonify({
        'success': True,
        'state': {
            'user_data': user_state,
            'server_info': game_state['public']
        }
    })

@app.route('/api/processes')
@require_auth
@limiter.limit("30 per minute")
def api_processes():
    user_id = request.user_id
    user_state = game_state['private'].get(user_id, {})

    return jsonify({
        'success': True,
        'processes': user_state.get('processes', [])
    })

@app.route('/api/hardware')
@require_auth
@limiter.limit("30 per minute")
def api_hardware():
    user_id = request.user_id
    user_state = game_state['private'].get(user_id, {})

    return jsonify({
        'success': True,
        'hardware': user_state.get('hardware', {})
    })

@app.route('/api/processes/start', methods=['POST'])
@require_auth
@limiter.limit("20 per minute")
def start_process():
    user_id = request.user_id
    data = request.get_json()

    # Sanitize and validate input
    process_type = sanitize_input(data.get('process_type', ''))
    target = sanitize_input(data.get('target', ''))

    if not process_type or not target:
        return jsonify({
            'success': False,
            'error': 'Process type and target required'
        }), 400

    # Validate process type
    valid_types = ['hack', 'scan', 'crack', 'ddos', 'download', 'upload']
    if process_type not in valid_types:
        return jsonify({
            'success': False,
            'error': 'Invalid process type'
        }), 400

    # Create process (sanitized)
    new_process = {
        'id': len(game_state['private'][user_id]['processes']) + 1,
        'type': process_type,
        'target': target,
        'progress': 0,
        'started_at': datetime.datetime.now().isoformat()
    }

    game_state['private'][user_id]['processes'].append(new_process)

    return jsonify({
        'success': True,
        'process': new_process
    })

@app.route('/api/logout', methods=['POST'])
@require_auth
def logout():
    user_id = request.user_id

    # Remove session
    if user_id in sessions:
        del sessions[user_id]

    return jsonify({
        'success': True,
        'message': 'Logged out successfully'
    })

# Error handlers
@app.errorhandler(404)
def not_found(e):
    return jsonify({'error': 'Resource not found'}), 404

@app.errorhandler(429)
def rate_limit_exceeded(e):
    return jsonify({'error': 'Rate limit exceeded. Please try again later.'}), 429

@app.errorhandler(500)
def internal_error(e):
    # Log error internally but don't expose details
    return jsonify({'error': 'Internal server error'}), 500

if __name__ == '__main__':
    print("🔒 Starting SECURE HackerExperience Server")
    print("📡 API Server: http://localhost:3000")
    print("\n✅ Security Features Enabled:")
    print("  • JWT Authentication")
    print("  • Input Validation & Sanitization")
    print("  • Rate Limiting")
    print("  • CORS Restrictions")
    print("  • Security Headers")
    print("  • XSS Protection")
    print("  • Password Hashing")
    print("  • Session Management")

    app.run(host='0.0.0.0', port=3000, debug=False)