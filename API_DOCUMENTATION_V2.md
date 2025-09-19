# HackerExperience Rust - Production API Documentation v2.0

## Base URL
- **Production**: `https://api.hackerexperience.com`
- **Development**: `http://localhost:3005`

## Authentication
All protected endpoints require a JWT Bearer token in the Authorization header:
```
Authorization: Bearer <jwt_token>
```

## Endpoints

### Authentication & User Management

#### POST /api/register
Register a new user account.

**Request Body:**
```json
{
  "username": "string (3-20 chars, alphanumeric)",
  "email": "valid@email.com",
  "password": "string (min 8 chars, must include uppercase, lowercase, digit)"
}
```

**Response:**
```json
{
  "success": true,
  "user": {
    "id": 12345,
    "username": "player123",
    "email": "player@example.com"
  },
  "token": "jwt_token_here",
  "refresh_token": "refresh_token_here"
}
```

**Validation Rules:**
- Username: 3-20 characters, alphanumeric with underscores/hyphens
- Email: Valid email format
- Password: Minimum 8 characters, must include uppercase, lowercase, and digit
- Passwords 12+ characters must include special character

---

#### POST /api/login
Authenticate user and receive JWT token.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**Response:**
```json
{
  "success": true,
  "user": {
    "id": 12345,
    "username": "player123",
    "email": "player@example.com",
    "last_login": "2025-09-18T10:30:00Z"
  },
  "token": "jwt_token_here",
  "refresh_token": "refresh_token_here",
  "expires_in": 3600
}
```

---

#### POST /api/refresh
Refresh an expired JWT token.

**Request Body:**
```json
{
  "refresh_token": "refresh_token_here"
}
```

**Response:**
```json
{
  "success": true,
  "token": "new_jwt_token",
  "refresh_token": "new_refresh_token",
  "expires_in": 3600
}
```

---

#### POST /api/logout
Invalidate user session.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "success": true,
  "message": "Logged out successfully"
}
```

---

### Game Mechanics

#### GET /api/game/dashboard
Get comprehensive dashboard data for the authenticated user.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "user": {
    "id": 12345,
    "username": "player123",
    "level": 42,
    "experience": 125000,
    "reputation": 850
  },
  "hardware": {
    "cpu": 500,
    "ram": 1024,
    "hdd": 5000,
    "net": 100
  },
  "active_processes": [
    {
      "id": 1,
      "type": "crack",
      "target": "192.168.1.1",
      "progress": 65,
      "end_time": "2025-09-18T12:00:00Z"
    }
  ],
  "bank_account": {
    "balance": 50000,
    "bitcoin": 0.05
  },
  "notifications": [
    {
      "id": 1,
      "message": "Process completed successfully",
      "timestamp": "2025-09-18T11:00:00Z"
    }
  ]
}
```

---

#### POST /api/game/process/start
Start a new game process (hack, crack, ddos, etc).

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "target_id": 67890,
  "process_type": "crack",
  "priority": 1
}
```

**Response:**
```json
{
  "success": true,
  "process": {
    "id": 123,
    "type": "crack",
    "target_id": 67890,
    "start_time": "2025-09-18T11:30:00Z",
    "end_time": "2025-09-18T12:00:00Z",
    "cpu_usage": 50,
    "ram_usage": 256
  }
}
```

**Process Types:**
- `crack` - Crack a server's password
- `hack` - Hack into a server
- `ddos` - DDoS attack
- `download` - Download software
- `upload` - Upload software
- `delete` - Delete files
- `hide` - Hide logs
- `seek` - Seek logs

---

#### DELETE /api/game/process/{process_id}
Cancel an active process.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "success": true,
  "message": "Process cancelled successfully",
  "refund": {
    "cpu_time": 150,
    "ram_time": 75
  }
}
```

---

#### GET /api/game/software
List user's software inventory.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "software": [
    {
      "id": 1,
      "name": "Cracker v3.0",
      "type": "cracker",
      "version": 3.0,
      "size": 150,
      "installed": true
    },
    {
      "id": 2,
      "name": "Firewall Pro",
      "type": "firewall",
      "version": 5.2,
      "size": 200,
      "installed": true
    }
  ],
  "storage_used": 350,
  "storage_total": 5000
}
```

---

#### POST /api/game/bank/transfer
Transfer money between accounts.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "recipient_id": 98765,
  "amount": 10000,
  "memo": "Payment for services"
}
```

**Response:**
```json
{
  "success": true,
  "transaction": {
    "id": "txn_abc123",
    "from": 12345,
    "to": 98765,
    "amount": 10000,
    "fee": 100,
    "timestamp": "2025-09-18T11:45:00Z"
  },
  "new_balance": 39900
}
```

---

### Clan System

#### GET /api/clan/{clan_id}
Get clan information.

**Response:**
```json
{
  "clan": {
    "id": 100,
    "name": "Elite Hackers",
    "description": "Top tier hacking group",
    "member_count": 25,
    "reputation": 5000,
    "leader": {
      "id": 12345,
      "username": "leader123"
    },
    "created_at": "2025-01-01T00:00:00Z"
  }
}
```

---

#### POST /api/clan/create
Create a new clan.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "name": "New Clan",
  "description": "A new hacking clan"
}
```

**Response:**
```json
{
  "success": true,
  "clan": {
    "id": 101,
    "name": "New Clan",
    "description": "A new hacking clan",
    "leader_id": 12345
  }
}
```

---

### Leaderboards

#### GET /api/leaderboard/{category}
Get leaderboard for specific category.

**Categories:** `level`, `reputation`, `money`, `clan`

**Query Parameters:**
- `limit`: Number of entries (default: 50, max: 100)
- `offset`: Pagination offset (default: 0)

**Response:**
```json
{
  "category": "level",
  "entries": [
    {
      "rank": 1,
      "user_id": 11111,
      "username": "pro_hacker",
      "value": 100,
      "clan": "Elite"
    }
  ],
  "user_rank": {
    "rank": 42,
    "value": 45
  }
}
```

---

### WebSocket Connection

#### WS /ws
Real-time WebSocket connection for game events.

**Authentication Flow:**
1. Connect to WebSocket endpoint
2. Send authentication message:
```json
{
  "type": "auth",
  "token": "jwt_token_here"
}
```

3. Receive authentication confirmation:
```json
{
  "type": "authenticated",
  "user_id": 12345,
  "username": "player123"
}
```

**Message Types:**
- `auth` - Authentication request
- `ping` - Keepalive ping
- `pong` - Keepalive response
- `game_action` - Game action request
- `notification` - Server notification
- `error` - Error message

**Example Game Action:**
```json
{
  "type": "game_action",
  "action": "start_process",
  "data": {
    "target_id": 67890,
    "process_type": "hack"
  }
}
```

---

### Health & Monitoring

#### GET /health
Basic health check.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-09-18T12:00:00Z"
}
```

---

#### GET /health/detailed
Detailed health status.

**Response:**
```json
{
  "status": "healthy",
  "checks": {
    "database": "connected",
    "redis": "connected",
    "disk_space": "ok",
    "memory": "ok"
  },
  "version": "1.0.0",
  "uptime": 86400
}
```

---

#### GET /metrics
Prometheus-compatible metrics endpoint.

**Response:** Prometheus text format metrics

---

## Rate Limiting

API endpoints are rate limited to prevent abuse:
- **Authentication endpoints**: 5 requests per minute
- **Game actions**: 30 requests per minute
- **General API**: 100 requests per minute

Rate limit headers:
- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Requests remaining
- `X-RateLimit-Reset`: Unix timestamp when limit resets

## Error Responses

All errors follow this format:
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {}
  }
}
```

### Common Error Codes
- `UNAUTHORIZED` - Invalid or missing authentication
- `FORBIDDEN` - Insufficient permissions
- `NOT_FOUND` - Resource not found
- `VALIDATION_ERROR` - Input validation failed
- `RATE_LIMITED` - Too many requests
- `INTERNAL_ERROR` - Server error

## Security Features

- **Password Security**: Argon2id hashing with secure parameters
- **JWT Tokens**: HS256 signed, 1-hour expiry
- **Input Validation**: Comprehensive validation on all inputs
- **SQL Injection Prevention**: Parameterized queries throughout
- **XSS Prevention**: HTML sanitization on user inputs
- **CSRF Protection**: Token validation on state-changing operations
- **Rate Limiting**: Per-endpoint and per-user limits

## SDK Examples

### JavaScript/TypeScript
```typescript
const API_BASE = 'https://api.hackerexperience.com';

class HackerExperienceAPI {
  private token: string;

  async login(email: string, password: string) {
    const response = await fetch(`${API_BASE}/api/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password })
    });

    const data = await response.json();
    if (data.success) {
      this.token = data.token;
    }
    return data;
  }

  async startProcess(targetId: number, processType: string) {
    const response = await fetch(`${API_BASE}/api/game/process/start`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.token}`
      },
      body: JSON.stringify({ target_id: targetId, process_type: processType })
    });

    return response.json();
  }
}
```

### Python
```python
import requests

class HackerExperienceAPI:
    def __init__(self):
        self.base_url = 'https://api.hackerexperience.com'
        self.token = None

    def login(self, email, password):
        response = requests.post(
            f'{self.base_url}/api/login',
            json={'email': email, 'password': password}
        )
        data = response.json()
        if data['success']:
            self.token = data['token']
        return data

    def start_process(self, target_id, process_type):
        headers = {'Authorization': f'Bearer {self.token}'}
        response = requests.post(
            f'{self.base_url}/api/game/process/start',
            headers=headers,
            json={'target_id': target_id, 'process_type': process_type}
        )
        return response.json()
```

## Changelog

### v2.0 (2025-09-18)
- Added comprehensive input validation
- Implemented L1 cache for performance
- Fixed authentication middleware
- Added WebSocket support
- Improved error handling
- Removed AWS SDK dependencies
- Consolidated to single web framework (axum)

### v1.0 (2025-09-15)
- Initial API release