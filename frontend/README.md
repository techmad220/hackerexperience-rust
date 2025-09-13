# Hacker Experience - Frontend Integration

This frontend application connects to the Rust backend and provides a complete web-based interface for the Hacker Experience game.

## Features Implemented

### 🔐 Authentication System
- **Login Page**: Modern login/register interface (`login.html`)
- **JWT Token Management**: Automatic token handling and refresh
- **Session Persistence**: Remembers login state across sessions
- **Auto-redirect**: Redirects unauthenticated users to login

### 🌐 API Integration
- **Complete API Client**: Comprehensive wrapper for all backend endpoints (`js/api.js`)
- **Error Handling**: Robust error handling with retry logic and user feedback
- **Offline Support**: Request queueing when offline
- **Safe Methods**: `safeGet()`, `safePost()` methods with built-in error handling

### ⚡ Real-time Updates
- **WebSocket Integration**: Real-time communication with backend (`js/websocket.js`)
- **Process Updates**: Live process progress and completion notifications
- **Chat System**: Real-time messaging between players
- **Player Updates**: Live money, experience, and level updates

### 🎮 Game Interface
- **Desktop Environment**: Terminal-style interface with command execution
- **Process Management**: Real-time process monitoring and control
- **Network Tools**: Server scanning, hacking, and connection management
- **Software Management**: View and manage installed software
- **Hardware Information**: Display player hardware specifications

### 📄 Game Pages
- **Processes**: Real-time process monitoring with progress bars
- **Software**: Installed software management
- **Hardware**: Hardware specifications display
- **Internet**: Network browsing and server interaction
- **Servers**: Owned server management
- **Clan**: Clan information and management
- **Missions**: Mission tracking and completion
- **Rankings**: Player leaderboards
- **Chat**: Real-time player communication
- **Logs**: System log viewing and management

### 🔧 State Management
- **Game Client Service**: Centralized state management (`js/game-client.js`)
- **Event System**: Reactive state updates
- **Local Storage**: Automatic state persistence
- **Sync System**: Keeps local state in sync with server

## File Structure

```
frontend/
├── index.html          # Main game interface
├── login.html          # Authentication page
├── css/
│   └── game.css        # Game styling
└── js/
    ├── api.js          # API client with error handling
    ├── game.js         # Main game logic and UI
    ├── websocket.js    # Real-time WebSocket communication
    └── game-client.js  # Centralized state management
```

## API Endpoints Used

### Authentication
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout
- `POST /auth/register` - User registration
- `GET /auth/me` - Get current user

### Player Management
- `GET /user/profile` - Get player profile
- `PUT /user/update` - Update player data
- `GET /user/stats` - Get player statistics

### Process Management
- `GET /processes/active` - Get active processes
- `POST /processes/start` - Start new process
- `POST /processes/{id}/kill` - Kill specific process
- `POST /processes/kill-all` - Kill all processes

### Software Management
- `GET /software/installed` - Get installed software
- `POST /software/{id}/start` - Start software
- `POST /software/{id}/stop` - Stop software

### Network Operations
- `POST /network/scan` - Scan network for servers
- `POST /servers/connect` - Connect to server
- `GET /servers/available` - Get available servers
- `GET /servers/owned` - Get owned servers

### Game Features
- `GET /clan/info` - Get clan information
- `GET /missions/active` - Get active missions
- `GET /rankings/top` - Get player rankings
- `GET /chat/history` - Get chat history
- `POST /chat/send` - Send chat message
- `GET /logs/recent` - Get recent logs

## WebSocket Events

### Incoming Events
- `process_start` - Process started
- `process_update` - Process progress update
- `process_complete` - Process completed
- `process_killed` - Process killed
- `player_update` - Player data changed
- `chat_message` - New chat message
- `notification` - System notification

### Outgoing Events
- `authenticate` - Authenticate WebSocket connection
- `chat` - Send chat message
- `ping` - Keep connection alive

## Usage

1. **Start the Backend**: Ensure the Rust backend is running
2. **Serve Frontend**: Serve the frontend files through a web server
3. **Navigate to Login**: Go to `login.html` if not authenticated
4. **Enter Game**: After login, automatically redirected to main game

## Key Features

### Terminal Commands
The game includes a functional terminal with commands:
- `help` - Show available commands
- `ps` - List running processes  
- `software` - List installed software
- `hardware` - Show hardware specs
- `scan <ip>` - Scan network address
- `connect <ip>` - Connect to server
- `hack <ip>` - Start hacking process
- `kill <id>` - Kill process by ID
- `killall` - Kill all processes
- `logout` - Logout from system

### Real-time Features
- Live process progress updates
- Real-time chat messages
- Instant notifications
- Auto-refreshing player stats
- WebSocket reconnection handling

### Error Handling
- Network timeout handling
- Automatic retry logic
- Offline request queueing
- User-friendly error messages
- Authentication error handling

## Development Notes

- All API calls use the centralized API client
- WebSocket connection requires authentication
- State is managed through the GameClient service
- UI updates are reactive to state changes
- Proper error handling throughout the application
- Mobile-responsive design considerations