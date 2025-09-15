# hackerexperience-rust

An incomplete attempt to recreate aspects of HackerExperience in Rust.

## What This Is

This is a learning project that attempted to port HackerExperience to Rust. It contains:

- ~78,000 lines of Rust code across 258 files
- 20 separate crates with various incomplete implementations
- Database schemas and migrations
- Some API endpoints
- A basic Leptos frontend
- Lots of scaffolding and boilerplate

## Reality Check

**This project is NOT:**
- A complete port of HackerExperience
- A playable game
- Production ready
- Actively maintained

**What it actually is:**
- An educational exercise in Rust
- Mostly structure without real functionality
- Incomplete implementations of game mechanics
- A collection of stubs and placeholders

## Project Structure

```
crates/
├── he-core/              # Basic types and entities
├── he-db/                # Database layer
├── he-api/               # API routes (mostly stubs)
├── he-game-mechanics/    # Incomplete game logic
├── he-legacy-compat/     # Attempted PHP compatibility
├── he-leptos-frontend/   # Basic web UI
└── [14 other crates]     # Various incomplete modules
```

## Technical Details

- **Language**: Rust
- **Web Framework**: Actix-Web
- **Database**: PostgreSQL with SQLx
- **Frontend**: Leptos (Rust/WASM)
- **Lines of Code**: ~78,000
- **Files**: 258 .rs files
- **Actual Functionality**: <10%

## Running It (Not Recommended)

If you want to see how incomplete it is:

```bash
# Prerequisites
cargo --version  # Need Rust installed
psql --version   # Need PostgreSQL

# Setup
export DATABASE_URL="postgresql://localhost/hackerexperience"
cargo build --workspace

# Try to run (will likely fail or do nothing useful)
cargo run --bin he-api
```

## What's Missing

Almost everything needed for a game:
- Complete game mechanics
- Working hacking simulation
- Mission system
- Multiplayer functionality
- Banking system
- Clans/corporations
- Research tree
- Any actual gameplay

## Contributing

This project is abandoned. You're better off starting fresh if you want to build a hacking game.

## License

MIT - Do whatever you want with it.

## Note

The inflated claims in older commits were incorrect. This is nowhere near a complete port of HackerExperience. It's a learning project that got out of hand with exaggerated documentation.

If you're looking to play HackerExperience or build something similar, this codebase won't help much. Consider it a cautionary tale about scope creep and honest documentation.