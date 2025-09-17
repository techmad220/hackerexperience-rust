# NetHeist API Documentation

## Overview

NetHeist provides a comprehensive RESTful API for all game operations. All endpoints return JSON responses and require JWT authentication (except public endpoints).

## Base URL

```
Production: https://api.netheist.com
Development: http://localhost:3005
```

## Authentication

### JWT Token

All protected endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <jwt_token>
```

### Public Endpoints

- `POST /api/login` - User login
- `POST /api/register` - User registration
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics

## Error Responses

All errors follow a consistent format:

```json
{
  "error": true,
  "code": "ERROR_CODE",
  "message": "Human readable message",
  "details": "Technical details (debug mode only)",
  "kind": "ErrorKind",
  "context": {
    "file": "source_file.rs",
    "line": 123,
    "request_id": "uuid",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| AUTH_001 | Unauthorized access |
| AUTH_002 | Invalid token |
| AUTH_003 | Session expired |
| AUTH_004 | Forbidden action |
| VAL_001 | Validation error |
| VAL_002 | Invalid input |
| VAL_003 | Missing required field |
| DB_001 | Database error |
| DB_002 | Query failed |
| GAME_001 | Insufficient resources |
| GAME_002 | Invalid game state |
| GAME_003 | Process already running |
| RATE_001 | Rate limit exceeded |

## Endpoints

### Authentication

#### POST /api/login
User login endpoint.

**Request:**
```json
{
  "username": "player123",
  "password": "secure_password"
}
```

**Response:**
```json
{
  "success": true,
  "token": "jwt_token",
  "refresh_token": "refresh_token",
  "user": {
    "id": 123,
    "username": "player123",
    "email": "player@example.com",
    "level": 42
  }
}
```

#### POST /api/register
Create a new user account.

**Request:**
```json
{
  "username": "newplayer",
  "email": "new@example.com",
  "password": "secure_password",
  "confirm_password": "secure_password"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Account created successfully",
  "user_id": 456
}
```

#### POST /api/refresh
Refresh access token using refresh token.

**Request:**
```json
{
  "refresh_token": "refresh_token_string"
}
```

**Response:**
```json
{
  "token": "new_jwt_token",
  "expires_in": 3600
}
```

### Game State

#### GET /api/state
Get current game state for authenticated user.

**Response:**
```json
{
  "user": {
    "id": 123,
    "username": "player123",
    "level": 42,
    "experience": 125000,
    "credits": 50000
  },
  "hardware": {
    "cpu": {"level": 5, "cores": 4},
    "ram": {"level": 4, "capacity": 8192},
    "hdd": {"level": 3, "capacity": 512000},
    "network": {"level": 6, "bandwidth": 1000}
  },
  "processes": [
    {
      "id": 789,
      "type": "crack",
      "target": "192.168.1.100",
      "progress": 65,
      "time_remaining": 120
    }
  ],
  "notifications": []
}
```

### Hacking

#### POST /api/hack/scan
Scan a target server to gather information.

**Request:**
```json
{
  "target_ip": "192.168.1.100"
}
```

**Response:**
```json
{
  "success": true,
  "server_info": {
    "ip_address": "192.168.1.100",
    "hostname": "corp-server-01",
    "owner": "MegaCorp",
    "server_type": "Corporate",
    "security_level": 5,
    "firewall_level": 3,
    "is_online": true
  },
  "process_id": 123
}
```

#### POST /api/hack/crack
Initiate hacking attempt on target server.

**Request:**
```json
{
  "target_ip": "192.168.1.100",
  "crack_method": "exploit"
}
```

**Response:**
```json
{
  "success": true,
  "access_granted": false,
  "process_id": 456,
  "estimated_time": 300,
  "message": "Hacking process initiated"
}
```

#### POST /api/hack/download
Download files from hacked server.

**Request:**
```json
{
  "server_ip": "192.168.1.100",
  "file_id": "file_789"
}
```

**Response:**
```json
{
  "success": true,
  "process_id": 789,
  "file_size": 102400,
  "download_time": 60
}
```

### Processes

#### GET /api/processes
Get all active processes for the user.

**Response:**
```json
{
  "processes": [
    {
      "pid": 123,
      "type": "scan",
      "target": "192.168.1.100",
      "start_time": "2024-01-01T00:00:00Z",
      "end_time": "2024-01-01T00:05:00Z",
      "progress": 45,
      "priority": 0,
      "can_cancel": true
    }
  ],
  "total": 3,
  "max_processes": 5
}
```

#### POST /api/processes/start
Start a new process.

**Request:**
```json
{
  "type": "scan",
  "target": "192.168.1.100",
  "priority": 1
}
```

**Response:**
```json
{
  "success": true,
  "process_id": 234,
  "estimated_completion": "2024-01-01T00:10:00Z"
}
```

#### POST /api/processes/cancel
Cancel a running process.

**Request:**
```json
{
  "process_id": 123
}
```

**Response:**
```json
{
  "success": true,
  "message": "Process cancelled"
}
```

### Hardware

#### GET /api/hardware
Get user's hardware configuration.

**Response:**
```json
{
  "cpu": {
    "model": "QuadCore X5",
    "level": 5,
    "cores": 4,
    "frequency": 3200,
    "processes_boost": 2
  },
  "ram": {
    "model": "HyperRAM DDR5",
    "level": 4,
    "capacity": 8192,
    "speed": 4800
  },
  "hdd": {
    "model": "QuantumDrive SSD",
    "level": 3,
    "capacity": 512000,
    "read_speed": 5000,
    "write_speed": 4000
  },
  "network": {
    "model": "FiberLink Pro",
    "level": 6,
    "bandwidth": 1000,
    "latency": 5
  }
}
```

#### POST /api/hardware/upgrade
Upgrade hardware component.

**Request:**
```json
{
  "component": "cpu",
  "target_level": 6
}
```

**Response:**
```json
{
  "success": true,
  "cost": 25000,
  "new_level": 6,
  "improvements": {
    "cores": "+1",
    "frequency": "+400MHz",
    "processes_boost": "+1"
  }
}
```

### Banking

#### GET /api/bank/accounts
Get user's bank accounts.

**Response:**
```json
{
  "accounts": [
    {
      "account_id": "acc_123",
      "bank_name": "CyberBank",
      "balance": 50000,
      "account_type": "checking"
    },
    {
      "account_id": "acc_456",
      "bank_name": "SecureVault",
      "balance": 150000,
      "account_type": "savings"
    }
  ],
  "total_balance": 200000
}
```

#### POST /api/bank/transfer
Transfer money between accounts.

**Request:**
```json
{
  "from_account": "acc_123",
  "to_account": "acc_456",
  "amount": 10000,
  "memo": "Savings deposit"
}
```

**Response:**
```json
{
  "success": true,
  "transaction_id": "txn_789",
  "new_balance": 40000
}
```

### Missions

#### GET /api/missions
Get available missions.

**Response:**
```json
{
  "active_missions": [
    {
      "mission_id": 1,
      "title": "First Steps",
      "description": "Complete the tutorial",
      "objective": "Hack your first server",
      "reward": {
        "experience": 500,
        "credits": 1000
      },
      "progress": 0,
      "status": "active"
    }
  ],
  "completed_missions": [2, 3, 4],
  "available_missions": [5, 6]
}
```

#### POST /api/missions/accept
Accept a new mission.

**Request:**
```json
{
  "mission_id": 5
}
```

**Response:**
```json
{
  "success": true,
  "mission": {
    "mission_id": 5,
    "title": "Corporate Espionage",
    "objectives": [
      "Hack into MegaCorp servers",
      "Download classified files",
      "Cover your tracks"
    ]
  }
}
```

### Progression

#### GET /api/progression
Get player progression data.

**Response:**
```json
{
  "level": 42,
  "experience": 125000,
  "next_level_exp": 130000,
  "skill_points": 5,
  "skills": {
    "hacking": 15,
    "defense": 10,
    "stealth": 8,
    "hardware": 12,
    "software": 9,
    "networking": 7
  },
  "achievements": [
    {
      "id": "first_hack",
      "name": "Script Kiddie",
      "description": "Complete your first hack",
      "unlocked_at": "2024-01-01T00:00:00Z"
    }
  ],
  "statistics": {
    "total_hacks": 234,
    "successful_hacks": 198,
    "servers_owned": 45,
    "money_earned": 2500000,
    "processes_completed": 567
  }
}
```

#### POST /api/progression/skills/invest
Invest skill points.

**Request:**
```json
{
  "skill": "hacking",
  "points": 2
}
```

**Response:**
```json
{
  "success": true,
  "skill": "hacking",
  "new_level": 17,
  "points_remaining": 3,
  "bonuses": [
    "+5% hack success rate",
    "+10% crack speed"
  ]
}
```

### Clan

#### GET /api/clan
Get clan information.

**Response:**
```json
{
  "clan": {
    "id": 123,
    "name": "Elite Hackers",
    "tag": "ELITE",
    "description": "Top tier hackers only",
    "leader": "masterhacker",
    "members": 25,
    "level": 10,
    "reputation": 5000
  },
  "member_info": {
    "role": "officer",
    "joined_at": "2024-01-01T00:00:00Z",
    "contribution": 50000
  }
}
```

#### POST /api/clan/create
Create a new clan.

**Request:**
```json
{
  "name": "Shadow Collective",
  "tag": "SHDW",
  "description": "Operating from the shadows"
}
```

**Response:**
```json
{
  "success": true,
  "clan_id": 456,
  "message": "Clan created successfully"
}
```

### Leaderboard

#### GET /api/leaderboard
Get global leaderboard.

**Query Parameters:**
- `type`: Type of leaderboard (player, clan, weekly, monthly)
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 50, max: 100)

**Response:**
```json
{
  "leaderboard": [
    {
      "rank": 1,
      "username": "elitehacker",
      "level": 100,
      "experience": 10000000,
      "reputation": 50000,
      "clan": "ELITE"
    }
  ],
  "page": 1,
  "total_pages": 20,
  "your_rank": 42
}
```

## WebSocket Events

### Connection
```javascript
ws://localhost:3005/ws
```

### Authentication
```json
{
  "type": "auth",
  "token": "jwt_token"
}
```

### Event Types

#### Process Updates
```json
{
  "type": "process_update",
  "data": {
    "process_id": 123,
    "progress": 75,
    "status": "running"
  }
}
```

#### Chat Messages
```json
{
  "type": "chat_message",
  "data": {
    "channel": "global",
    "username": "player123",
    "message": "Hello world!",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

#### System Notifications
```json
{
  "type": "notification",
  "data": {
    "title": "Hack Successful",
    "message": "You've gained access to server 192.168.1.100",
    "level": "success"
  }
}
```

## Rate Limiting

- Default: 100 requests per minute per user
- Authentication endpoints: 5 requests per minute
- Heavy operations (upgrades, transfers): 10 requests per minute

Rate limit headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

## Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 409 | Conflict |
| 422 | Unprocessable Entity |
| 429 | Too Many Requests |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

## Versioning

The API uses URL versioning. Current version: v1

Future versions will be available at:
- `/api/v2/*`
- `/api/v3/*`

## SDK Support

Official SDKs available:
- JavaScript/TypeScript
- Python
- Rust
- Go (coming soon)

## Support

For API support, please contact:
- Email: api@netheist.com
- Discord: discord.gg/netheist
- GitHub Issues: github.com/netheist/api-issues