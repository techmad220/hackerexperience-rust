# HackerExperience Rust Implementation

A Rust-based implementation inspired by the classic HackerExperience browser game, focusing on modern architecture and performance.

## Current Status

This project is under active development. It represents a ground-up reimplementation using Rust, with modern web technologies and architectural patterns.

### What's Implemented

#### Core Systems
- **Player Management** - Basic player entities, authentication, and session handling
- **Database Layer** - PostgreSQL integration with SQLx for type-safe queries
- **REST API** - Partial implementation of game endpoints
- **WebSocket Support** - Real-time communication infrastructure
- **Actor System** - Message-passing concurrency for game processes

#### Game Mechanics (Partial)
- **Process System** - Basic process scheduling and execution
- **Hardware Components** - CPU, RAM, HDD management
- **Software System** - Program installation and dependencies
- **Network Topology** - Simple server connections
- **Banking System** - Basic financial transactions

#### Frontend
- **Leptos Framework** - Modern reactive UI with WebAssembly
- **Terminal Interface** - Basic command-line simulation
- **Modal System** - UI components for game interactions

### What's NOT Implemented Yet

- Complete mission system
- Full hacking mechanics
- Clan/corporation features
- Research tree
- Marketplace
- PvP combat system
- Many original game features

## Technical Stack

- **Backend**: Rust with Actix-Web
- **Database**: PostgreSQL with SQLx
- **Frontend**: Leptos (Rust/WASM)
- **Real-time**: WebSockets via Actix
- **Async Runtime**: Tokio

## Project Structure

```
hackerexperience-rust/
├── crates/
│   ├── he-core/           # Core game types and entities
│   ├── he-db/             # Database layer
│   ├── he-api/            # REST API endpoints
│   ├── he-game-mechanics/ # Game logic implementation
│   ├── he-leptos-frontend/# Web UI
│   └── he-*               # Various game subsystems
├── migrations/            # Database schema
└── scripts/              # Development utilities
```

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PostgreSQL
sudo apt install postgresql postgresql-contrib

# Install Trunk (for frontend)
cargo install trunk
```

### Setup

```bash
# Clone repository
git clone https://github.com/yourusername/hackerexperience-rust
cd hackerexperience-rust

# Setup database
createdb hackerexperience
export DATABASE_URL="postgresql://localhost/hackerexperience"

# Run migrations
sqlx migrate run

# Build project
cargo build --workspace

# Run backend server
cargo run --bin he-api

# In another terminal, run frontend
cd crates/he-leptos-frontend
trunk serve
```

### Development

```bash
# Run tests
cargo test --workspace

# Watch for changes
cargo watch -x "run --bin he-api"

# Check code
cargo check --workspace
cargo clippy --workspace
```

## Architecture Notes

This implementation takes a different approach from the original:

- **Actor-based concurrency** instead of traditional threading
- **Type-safe SQL** queries compiled at build time
- **WebAssembly frontend** for better performance
- **Event-sourced** game state for consistency
- **Microservice-ready** architecture

## Contributing

This is an educational project exploring game development with Rust. Contributions are welcome, but please note:

1. This is NOT a complete recreation of the original game
2. Many features are simplified or reimagined
3. Focus is on learning and experimentation with Rust

## Disclaimer

This is an independent project inspired by HackerExperience. It is not affiliated with, endorsed by, or connected to the original game or its creators. This implementation is for educational purposes and explores modern web game architecture using Rust.

## License

MIT - See LICENSE file for details

## Acknowledgments

- Inspired by the original HackerExperience game concept
- Built with the Rust community's excellent libraries and tools
- Thanks to all contributors and testers