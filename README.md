# hackerexperience-rust

An attempt to recreate some aspects of HackerExperience in Rust.

## What This Actually Is

This is a learning project with ~78,000 lines of Rust code across 258 files. It contains:

- 20 separate crates with various incomplete implementations
- Database schemas and migrations
- Some API endpoints that don't do much
- A Leptos frontend that barely works
- Lots of boilerplate and scaffolding

## Reality Check

Despite what previous commits may claim:
- This is NOT a complete port of HackerExperience
- It does NOT have "100% parity" with anything
- Most of the code is just structure without real functionality
- The game mechanics are mostly stubbed out
- It won't run as a playable game

## What Actually Works

Very little. You might be able to:
- Compile the project (if dependencies cooperate)
- Run a basic server that serves some endpoints
- See a basic frontend if you squint

## Project Structure

```
crates/
├── he-core/              # Basic types
├── he-db/                # Database stuff
├── he-api/               # Some API routes
├── he-game-mechanics/    # Mostly empty game logic
├── he-legacy-compat/     # Attempts at compatibility
├── he-leptos-frontend/   # Basic web UI
└── [14 other crates]     # Various incomplete pieces
```

## Don't Bother Running This

But if you insist:

```bash
# You'll need Rust and PostgreSQL
cargo build --workspace

# Set up a database somehow
export DATABASE_URL="postgresql://localhost/whatever"

# Try to run something
cargo run --bin he-api  # Might start a server
```

## What's Missing

Almost everything:
- Actual game mechanics
- Working hacking system
- Missions
- Multiplayer
- Banking
- Clans
- Research
- Any form of fun

## Contributing

This is a learning mess. Fork it if you want, but you're probably better off starting fresh.

## License

MIT - Take it, it's not worth much.

## Note

This was an educational exercise in Rust. The inflated claims in the git history were misguided. This is nowhere near a working game.