#!/usr/bin/env python3
"""
Real WebSocket Server for HackerExperience
Provides real-time game updates and chat
"""

import asyncio
import websockets
import json
import jwt
import html
from datetime import datetime

# Configuration
WS_PORT = 3001
JWT_SECRET = 'your_jwt_secret_here'  # Should match main server
JWT_ALGORITHM = 'HS256'

# Connected clients
connected_clients = {}
game_rooms = {}

def sanitize_message(text):
    """Sanitize messages to prevent XSS in chat"""
    if not isinstance(text, str):
        return str(text)
    return html.escape(text, quote=True)

async def authenticate_client(websocket, message):
    """Authenticate WebSocket client"""
    try:
        token = message.get('token')
        if not token:
            await websocket.send(json.dumps({
                'type': 'error',
                'message': 'Authentication required'
            }))
            return None

        # Verify JWT token
        payload = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])
        user_id = payload.get('user_id')

        # Store authenticated connection
        connected_clients[user_id] = {
            'websocket': websocket,
            'authenticated_at': datetime.now().isoformat(),
            'username': payload.get('username', f'User{user_id}')
        }

        # Send authentication success
        await websocket.send(json.dumps({
            'type': 'authenticated',
            'user_id': user_id,
            'message': 'WebSocket authentication successful'
        }))

        # Broadcast user joined
        await broadcast_to_all({
            'type': 'user_joined',
            'username': connected_clients[user_id]['username'],
            'online_count': len(connected_clients)
        }, exclude=user_id)

        return user_id

    except jwt.InvalidTokenError as e:
        await websocket.send(json.dumps({
            'type': 'error',
            'message': 'Invalid authentication token'
        }))
        return None

async def broadcast_to_all(message, exclude=None):
    """Broadcast message to all connected clients"""
    disconnected = []

    for user_id, client in connected_clients.items():
        if user_id != exclude:
            try:
                await client['websocket'].send(json.dumps(message))
            except:
                disconnected.append(user_id)

    # Clean up disconnected clients
    for user_id in disconnected:
        del connected_clients[user_id]

async def broadcast_to_user(user_id, message):
    """Send message to specific user"""
    if user_id in connected_clients:
        try:
            await connected_clients[user_id]['websocket'].send(json.dumps(message))
        except:
            del connected_clients[user_id]

async def handle_game_action(user_id, action, data):
    """Handle game-specific actions"""
    sanitized_data = {k: sanitize_message(v) for k, v in data.items()}

    if action == 'start_process':
        # Notify user of process start
        await broadcast_to_user(user_id, {
            'type': 'process_started',
            'process': {
                'type': sanitized_data.get('process_type'),
                'target': sanitized_data.get('target'),
                'started_at': datetime.now().isoformat()
            }
        })

    elif action == 'hack_complete':
        # Broadcast hack completion to relevant users
        await broadcast_to_all({
            'type': 'system_message',
            'message': f"System breached at {sanitized_data.get('target', 'unknown')}"
        })

    elif action == 'chat':
        # Handle chat messages with sanitization
        message_text = sanitized_data.get('message', '')
        if len(message_text) > 500:  # Limit message length
            message_text = message_text[:500]

        await broadcast_to_all({
            'type': 'chat_message',
            'username': connected_clients[user_id]['username'],
            'message': message_text,
            'timestamp': datetime.now().isoformat()
        })

    elif action == 'join_room':
        # Handle room/channel joining
        room_name = sanitized_data.get('room', 'general')
        if room_name not in game_rooms:
            game_rooms[room_name] = []

        game_rooms[room_name].append(user_id)

        await broadcast_to_user(user_id, {
            'type': 'room_joined',
            'room': room_name,
            'members': len(game_rooms[room_name])
        })

async def handle_client(websocket, path):
    """Handle individual WebSocket client"""
    user_id = None
    try:
        # Wait for authentication
        async for message in websocket:
            try:
                data = json.loads(message)
                msg_type = data.get('type')

                if msg_type == 'auth' and user_id is None:
                    # Authenticate client
                    user_id = await authenticate_client(websocket, data)

                elif msg_type == 'ping' and user_id:
                    # Handle keepalive
                    await websocket.send(json.dumps({'type': 'pong'}))

                elif msg_type == 'game_action' and user_id:
                    # Handle game actions
                    action = data.get('action')
                    action_data = data.get('data', {})
                    await handle_game_action(user_id, action, action_data)

                elif not user_id:
                    # Not authenticated
                    await websocket.send(json.dumps({
                        'type': 'error',
                        'message': 'Please authenticate first'
                    }))

            except json.JSONDecodeError:
                await websocket.send(json.dumps({
                    'type': 'error',
                    'message': 'Invalid message format'
                }))
            except Exception as e:
                print(f"Error handling message: {e}")

    except websockets.exceptions.ConnectionClosed:
        pass
    finally:
        # Clean up on disconnect
        if user_id and user_id in connected_clients:
            username = connected_clients[user_id]['username']
            del connected_clients[user_id]

            # Notify others of disconnect
            await broadcast_to_all({
                'type': 'user_left',
                'username': username,
                'online_count': len(connected_clients)
            })

            # Remove from rooms
            for room in game_rooms.values():
                if user_id in room:
                    room.remove(user_id)

async def periodic_updates():
    """Send periodic updates to all clients"""
    while True:
        await asyncio.sleep(30)  # Every 30 seconds

        # Send online count update
        await broadcast_to_all({
            'type': 'status_update',
            'online_count': len(connected_clients),
            'timestamp': datetime.now().isoformat()
        })

async def main():
    """Start WebSocket server"""
    print(f"🔌 Starting WebSocket Server on port {WS_PORT}")
    print("✅ Features:")
    print("  • JWT Authentication")
    print("  • Real-time messaging")
    print("  • XSS Protection")
    print("  • Room support")
    print("  • Automatic reconnection handling")

    # Start periodic updates task
    asyncio.create_task(periodic_updates())

    # Start WebSocket server
    async with websockets.serve(handle_client, '0.0.0.0', WS_PORT):
        print(f"WebSocket server listening on ws://localhost:{WS_PORT}")
        await asyncio.Future()  # Run forever

if __name__ == '__main__':
    asyncio.run(main())