# HackerExperience Rust Port

A Rust implementation of the classic browser-based hacking game HackerExperience.

## 🚀 Project Status: ACTIVE DEVELOPMENT

This project is under active development to create a complete, playable port of HackerExperience using modern Rust technologies.

### Current Progress
- **Core Infrastructure**: ✅ Complete (database, API framework, WebSocket)
- **Game Engine**: 🚧 30% (process system, hardware, software mechanics)
- **Frontend**: 🚧 15% (basic Leptos/WASM interface)
- **Game Features**: 🚧 20% (partial implementations)
- **Overall**: **~25% Complete**

📋 **[See Full Development Roadmap](./ROADMAP.md)** - Targeting June 2026 for production release!

## What's Working Now

### ✅ Implemented
- Database layer with PostgreSQL/SQLx
- Basic REST API structure
- WebSocket real-time communication
- Leptos frontend framework with WASM
- Core entity models (User, Hardware, Process, Software)
- Session management and authentication
- Basic process scheduling system

### 🚧 In Progress
- Complete process execution engine
- Full hacking mechanics
- Network topology system
- Mission framework
- Banking system

### ⏳ Planned
- Clan system
- Complete UI/UX
- Production deployment
- Comprehensive testing

## Tech Stack

- **Backend**: Rust with Actix-Web
- **Database**: PostgreSQL with SQLx (compile-time checked queries)
- **Frontend**: Leptos (Rust/WASM reactive framework)
- **Real-time**: WebSockets via Actix
- **Architecture**: Modular crate system for scalability

## Project Structure

```
hackerexperience-rust/
├── crates/
│   ├── he-core/           # Core game types and entities
│   ├── he-db/             # Database layer
│   ├── he-api/            # REST/GraphQL API
│   ├── he-game-mechanics/ # Game logic implementation
│   ├── he-leptos-frontend/# WebAssembly frontend
│   ├── he-helix-*/        # Game subsystems (process, network, etc.)
│   └── he-legacy-compat/  # Compatibility with original game
├── migrations/            # Database schema
├── frontend/             # Static assets
└── tests/               # Integration tests
```

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PostgreSQL
sudo apt install postgresql postgresql-contrib

# Install Trunk for frontend
cargo install trunk

# Install SQLx CLI
cargo install sqlx-cli
```

### Development Setup

```bash
# Clone repository
git clone https://github.com/techmad220/hackerexperience-rust
cd hackerexperience-rust

# Setup database
createdb hackerexperience
export DATABASE_URL="postgresql://localhost/hackerexperience"

# Run migrations
sqlx migrate run

# Build the project
cargo build --workspace

# Run the API server
cargo run --bin he-api

# In another terminal, run the frontend
cd crates/he-leptos-frontend
trunk serve --open
```

### Docker Setup (Coming Soon)

```bash
docker-compose up -d
```

## Contributing

We're actively looking for contributors! This is a community project to revive and modernize HackerExperience.

### How to Help

1. **Check the [Roadmap](./ROADMAP.md)** for current priorities
2. **Pick an unchecked task** from Phase 1 or 2
3. **Open an issue** to discuss your approach
4. **Submit a PR** with tests

### Needed Skills

- **Rust Developers** - Core game logic
- **Frontend Developers** - Leptos/WASM UI
- **Game Designers** - Balance and mechanics
- **DevOps Engineers** - Infrastructure setup
- **Testers** - QA and bug hunting

### Development Guidelines

- Write tests for new features
- Follow Rust best practices
- Document public APIs
- Keep commits focused and atomic
- Update relevant documentation

## Why Rust?

- **Performance**: 10-100x faster than the original PHP
- **Memory Safety**: No crashes, no memory leaks
- **Concurrency**: Handle thousands of players efficiently
- **Type Safety**: Catch bugs at compile time
- **WASM Support**: Run in browsers at near-native speed

## Goals

1. **Faithful Recreation**: Preserve the original gameplay
2. **Modern Architecture**: Scalable and maintainable
3. **Open Source**: Community-driven development
4. **Cross-Platform**: Browser, desktop, and mobile
5. **Production Ready**: Handle real player load

## License

MIT License - This is a community project, free and open source.

## Disclaimer

This is an independent recreation project and is not affiliated with the original HackerExperience or its creators. It's a community effort to preserve and modernize a beloved game.

## Contact

- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: General questions and ideas
- **Discord**: Coming soon for real-time chat

---

**🌟 Star this repo to support active development!**

**📊 [View Development Progress](./ROADMAP.md) | 🐛 [Report Issues](https://github.com/techmad220/hackerexperience-rust/issues)**