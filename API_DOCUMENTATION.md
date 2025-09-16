# HackerExperience API Documentation

## Base URL
```
Production: https://api.hackerexperience.com
Development: http://localhost:3000
```

## Authentication
All API requests require JWT authentication except for registration and login endpoints.

### Headers
```http
Authorization: Bearer <jwt_token>
Content-Type: application/json
X-CSRF-Token: <csrf_token> (for state-changing operations)
```

## Rate Limiting
- 100 requests per minute per IP
- 1000 requests per hour per user
- Headers returned: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

---

## Endpoints

### Authentication

#### POST /api/auth/register
Register new user account.

**Request:**
```json
{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

**Response (201):**
```json
{
  "user_id": 123,
  "username": "string",
  "token": "jwt_token",
  "refresh_token": "refresh_token"
}
```

#### POST /api/auth/login
Authenticate user and receive tokens.

**Request:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response (200):**
```json
{
  "token": "jwt_token",
  "refresh_token": "refresh_token",
  "user": {
    "id": 123,
    "username": "string",
    "level": 1,
    "experience": 0
  }
}
```

#### POST /api/auth/refresh
Refresh access token using refresh token.

**Request:**
```json
{
  "refresh_token": "string"
}
```

**Response (200):**
```json
{
  "token": "new_jwt_token"
}
```

#### POST /api/auth/logout
Invalidate current session.

**Response (200):**
```json
{
  "message": "Logged out successfully"
}
```

---

### Game Processes

#### GET /api/processes
List user's active processes.

**Query Parameters:**
- `status`: filter by status (running, completed, failed)
- `limit`: max results (default 50)
- `offset`: pagination offset

**Response (200):**
```json
{
  "processes": [
    {
      "id": 1,
      "type": "hack",
      "target_ip": "192.168.1.1",
      "progress": 75.5,
      "time_remaining": 120,
      "status": "running",
      "priority": "high"
    }
  ],
  "total": 10
}
```

#### POST /api/processes
Start new process.

**Request:**
```json
{
  "type": "hack|crack|download|upload|install",
  "target_ip": "192.168.1.1",
  "software_id": 123,
  "priority": "low|normal|high"
}
```

**Response (201):**
```json
{
  "process_id": 456,
  "estimated_time": 300,
  "resources_used": {
    "cpu": 50,
    "ram": 512,
    "bandwidth": 10
  }
}
```

#### DELETE /api/processes/{id}
Cancel running process.

**Response (200):**
```json
{
  "message": "Process cancelled"
}
```

---

### Servers

#### GET /api/servers
List owned servers.

**Response (200):**
```json
{
  "servers": [
    {
      "id": 1,
      "name": "Main Server",
      "ip": "192.168.1.100",
      "type": "dedicated",
      "hardware": {
        "cpu": "Intel i9",
        "ram": 32768,
        "hdd": 2048000
      },
      "status": "online"
    }
  ]
}
```

#### GET /api/servers/{ip}
Get server information.

**Response (200):**
```json
{
  "server": {
    "ip": "192.168.1.1",
    "hostname": "target.server",
    "os": "Linux",
    "open_ports": [22, 80, 443],
    "services": ["ssh", "http", "https"],
    "firewall": true,
    "access_level": "none|user|root"
  }
}
```

#### POST /api/servers/scan
Scan target server for information.

**Request:**
```json
{
  "target_ip": "192.168.1.1",
  "scan_type": "quick|deep|stealth"
}
```

**Response (200):**
```json
{
  "scan_id": 789,
  "process_id": 456,
  "estimated_time": 60
}
```

---

### Software

#### GET /api/software
List user's software collection.

**Query Parameters:**
- `type`: filter by type (cracker, firewall, antivirus, etc.)
- `installed`: filter by installation status

**Response (200):**
```json
{
  "software": [
    {
      "id": 1,
      "name": "Elite Cracker",
      "version": "3.0",
      "type": "cracker",
      "size": 1024,
      "installed_on": ["192.168.1.100"],
      "research_required": false
    }
  ]
}
```

#### POST /api/software/install
Install software on server.

**Request:**
```json
{
  "software_id": 123,
  "target_server": "192.168.1.100"
}
```

**Response (200):**
```json
{
  "process_id": 456,
  "installation_time": 120
}
```

#### POST /api/software/uninstall
Remove software from server.

**Request:**
```json
{
  "software_id": 123,
  "server_ip": "192.168.1.100"
}
```

---

### Hacking

#### POST /api/hack/attempt
Initiate hack attempt on target.

**Request:**
```json
{
  "target_ip": "192.168.1.1",
  "method": "exploit|bruteforce|backdoor",
  "port": 22,
  "bounce_through": ["192.168.1.50", "192.168.1.60"]
}
```

**Response (200):**
```json
{
  "hack_id": 999,
  "process_id": 456,
  "success_probability": 0.75,
  "estimated_time": 300
}
```

#### POST /api/hack/upload
Upload virus/malware to compromised server.

**Request:**
```json
{
  "target_ip": "192.168.1.1",
  "virus_id": 123
}
```

#### GET /api/hack/logs/{server_ip}
View server logs.

**Response (200):**
```json
{
  "logs": [
    {
      "id": 1,
      "timestamp": "2024-01-15T10:30:00Z",
      "source_ip": "192.168.1.100",
      "action": "login",
      "message": "SSH login successful"
    }
  ]
}
```

#### DELETE /api/hack/logs
Delete log entries.

**Request:**
```json
{
  "server_ip": "192.168.1.1",
  "log_ids": [1, 2, 3]
}
```

---

### Banking

#### GET /api/bank/accounts
List bank accounts.

**Response (200):**
```json
{
  "accounts": [
    {
      "account_number": "12345678",
      "bank_ip": "bank.secure.com",
      "balance": 50000,
      "currency": "USD"
    }
  ]
}
```

#### POST /api/bank/transfer
Transfer money between accounts.

**Request:**
```json
{
  "from_account": "12345678",
  "to_account": "87654321",
  "amount": 1000,
  "routing_servers": ["192.168.1.50"]
}
```

**Response (200):**
```json
{
  "transaction_id": "tx_abc123",
  "status": "completed",
  "trace_time": 3600
}
```

#### GET /api/bitcoin/wallet
Get Bitcoin wallet information.

**Response (200):**
```json
{
  "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "balance": 0.5,
  "transactions": []
}
```

---

### Missions

#### GET /api/missions
List available missions.

**Query Parameters:**
- `type`: filter by type (tutorial, main, side, daily)
- `status`: filter by status (available, active, completed)

**Response (200):**
```json
{
  "missions": [
    {
      "id": 1,
      "name": "First Steps",
      "description": "Complete the tutorial",
      "type": "tutorial",
      "objectives": [
        {
          "id": 1,
          "description": "Hack into tutorial server",
          "completed": false
        }
      ],
      "rewards": {
        "money": 1000,
        "experience": 100,
        "items": ["basic_cracker"]
      },
      "time_limit": null
    }
  ]
}
```

#### POST /api/missions/{id}/accept
Accept mission.

**Response (200):**
```json
{
  "mission_id": 1,
  "status": "active",
  "expires_at": null
}
```

#### POST /api/missions/{id}/complete
Submit mission for completion.

**Response (200):**
```json
{
  "mission_id": 1,
  "status": "completed",
  "rewards_claimed": true
}
```

---

### Clans

#### GET /api/clans
List all clans.

**Query Parameters:**
- `sort`: name|members|reputation
- `limit`: max results
- `offset`: pagination

**Response (200):**
```json
{
  "clans": [
    {
      "id": 1,
      "name": "Elite Hackers",
      "tag": "ELITE",
      "members": 50,
      "reputation": 10000,
      "recruiting": true
    }
  ]
}
```

#### POST /api/clans
Create new clan.

**Request:**
```json
{
  "name": "My Clan",
  "tag": "MYCLN",
  "description": "Best hackers",
  "requirements": {
    "min_level": 10
  }
}
```

#### POST /api/clans/{id}/join
Request to join clan.

**Response (200):**
```json
{
  "status": "pending|accepted",
  "message": "Join request sent"
}
```

#### POST /api/clans/{id}/war
Declare war on another clan.

**Request:**
```json
{
  "target_clan_id": 2,
  "war_type": "capture_the_flag|total_war"
}
```

---

### Hardware

#### GET /api/hardware/shop
List available hardware for purchase.

**Response (200):**
```json
{
  "components": [
    {
      "id": 1,
      "name": "Quantum CPU X1",
      "type": "cpu",
      "level": 10,
      "price": 50000,
      "specs": {
        "cores": 16,
        "frequency": 5.0
      }
    }
  ]
}
```

#### POST /api/hardware/purchase
Buy hardware component.

**Request:**
```json
{
  "component_id": 1,
  "quantity": 1,
  "install_on": "192.168.1.100"
}
```

---

### Research

#### GET /api/research/tree
Get research tree.

**Response (200):**
```json
{
  "research": [
    {
      "id": 1,
      "name": "Advanced Cryptography",
      "description": "Unlock better encryption",
      "cost": 1000,
      "time": 3600,
      "prerequisites": [],
      "unlocks": ["elite_hasher_v2"]
    }
  ]
}
```

#### POST /api/research/start
Begin research.

**Request:**
```json
{
  "research_id": 1,
  "boost": false
}
```

---

### Network

#### GET /api/network/topology
Get network map around player.

**Response (200):**
```json
{
  "nodes": [
    {
      "ip": "192.168.1.1",
      "type": "server|router|firewall",
      "connections": ["192.168.1.2", "192.168.1.3"],
      "distance": 1
    }
  ]
}
```

#### POST /api/network/traceroute
Trace route to target.

**Request:**
```json
{
  "target_ip": "192.168.1.100"
}
```

**Response (200):**
```json
{
  "hops": [
    {
      "hop": 1,
      "ip": "192.168.1.1",
      "hostname": "router.local",
      "time": 10
    }
  ]
}
```

---

## WebSocket Events

Connect to `wss://api.hackerexperience.com/ws` with JWT token.

### Client Events
```javascript
// Subscribe to updates
{
  "type": "subscribe",
  "channels": ["process", "chat", "notifications"]
}

// Send chat message
{
  "type": "chat",
  "channel": "global",
  "message": "Hello world"
}
```

### Server Events
```javascript
// Process update
{
  "type": "process_update",
  "data": {
    "process_id": 123,
    "progress": 50.0,
    "status": "running"
  }
}

// Process completed
{
  "type": "process_complete",
  "data": {
    "process_id": 123,
    "result": "success",
    "rewards": {}
  }
}

// New notification
{
  "type": "notification",
  "data": {
    "title": "Hack successful",
    "message": "You gained root access",
    "type": "success"
  }
}

// Chat message
{
  "type": "chat_message",
  "data": {
    "channel": "global",
    "user": "player123",
    "message": "Hello",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}

// Server attack alert
{
  "type": "attack_alert",
  "data": {
    "attacker_ip": "192.168.1.50",
    "target_server": "192.168.1.100",
    "attack_type": "ddos"
  }
}
```

---

## Error Responses

All errors follow this format:

```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Detailed error message",
    "field": "username" // Optional, for validation errors
  }
}
```

### Common Error Codes
- `UNAUTHORIZED` - Invalid or expired token
- `FORBIDDEN` - Insufficient permissions
- `NOT_FOUND` - Resource not found
- `VALIDATION_ERROR` - Input validation failed
- `RATE_LIMITED` - Too many requests
- `INSUFFICIENT_RESOURCES` - Not enough CPU/RAM/etc
- `INSUFFICIENT_FUNDS` - Not enough money
- `PROCESS_ALREADY_RUNNING` - Conflicting process
- `SERVER_OFFLINE` - Target server is offline
- `ACCESS_DENIED` - No access to target

---

## Status Codes

- `200 OK` - Success
- `201 Created` - Resource created
- `204 No Content` - Success with no response body
- `400 Bad Request` - Invalid request
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Access denied
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict
- `429 Too Many Requests` - Rate limited
- `500 Internal Server Error` - Server error
- `503 Service Unavailable` - Maintenance mode