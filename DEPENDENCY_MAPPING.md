# Helix Elixir to Rust Dependency Mapping

## Current Elixir Dependencies → Rust Equivalents

### Core Framework Dependencies

#### Phoenix Framework → Axum + Tower
```elixir
{:phoenix, "~> 1.3.0"}           → axum = "0.7"
                                   tower = "0.4"
                                   tower-http = "0.5"
```

#### CORS Support
```elixir
{:corsica, "~> 1.1.0"}           → tower-http = { version = "0.5", features = ["cors"] }
```

### Database Dependencies

#### Ecto ORM → SQLx
```elixir
{:ecto, "~> 2.2.8"}              → sqlx = { version = "0.7", features = [
{:postgrex, "~> 0.13.4"}           "postgres", "runtime-tokio-rustls", 
{:ecto_enum, "~> 1.0"}             "uuid", "chrono", "json"
                                 ] }
```

#### Database Connection Pooling
```elixir
# Built into Ecto                → r2d2 = "0.8" (or SQLx built-in pool)
```

### Serialization & JSON

#### JSON Handling
```elixir
{:poison, "~> 3.1"}              → serde_json = "1.0"
                                   serde = { version = "1.0", features = ["derive"] }
```

### Authentication & Security

#### Password Hashing
```elixir
{:comeonin, "~> 4.0.3"}          → bcrypt = "0.15"
{:bcrypt_elixir, "~> 1.0.5"}
```

### Logging & Monitoring

#### Structured Logging
```elixir
{:timber, "~> 2.5"}              → tracing = "0.1"
{:logger_file_backend, "~>0.0.10"} tracing-subscriber = { version = "0.3", features = [
                                     "json", "env-filter", "local-time"
                                   ] }
                                   tracing-appender = "0.2"
```

### Utilities

#### Random String Generation
```elixir
{:entropy_string, "~> 1.3"}     → rand = "0.8"
                                   uuid = { version = "1.0", features = ["v4"] }
```

#### Custom Helix Libraries
```elixir
{:helf, "~> 0.0.3"}              → Custom helix-hell crate
{:burette, git: "..."}           → Custom Rust implementation
```

### Release & Deployment

#### Application Packaging
```elixir
{:distillery, "~>1.5.2"}        → cargo = "1.0" (built-in)
                                   Docker multi-stage builds
```

### Development & Testing Dependencies

#### Testing Framework
```elixir
{:ex_machina, "~> 2.1"}          → fake = "2.9" (for test data generation)
                                   tokio-test = "0.4"
                                   assert_matches = "1.5"
```

#### Code Quality
```elixir
{:credo, "~> 0.8.10"}           → clippy (built into rustc)
{:excoveralls, "~> 0.8.1"}      → cargo-tarpaulin = "0.27" (coverage)
{:inch_ex, "~> 0.5.6"}          → cargo-doc (built-in documentation)
```

#### Documentation
```elixir
{:ex_doc, "~> 0.18.1"}          → Built into cargo doc
{:earmark, "~> 1.2.4"}          → Built into rustdoc
```

## Complete Rust Cargo.toml Dependencies

### Workspace Root Cargo.toml
```toml
[workspace]
resolver = "2"
members = [
    "crates/helix-core",
    "crates/helix-config",
    "crates/helix-database",
    "crates/helix-events",
    "crates/helix-account",
    "crates/helix-cache",
    "crates/helix-client",
    "crates/helix-entity",
    "crates/helix-log",
    "crates/helix-network",
    "crates/helix-notification",
    "crates/helix-process",
    "crates/helix-server",
    "crates/helix-software",
    "crates/helix-story",
    "crates/helix-universe",
    "crates/helix-websocket",
    "crates/helix-api",
    "helix-main"
]

[workspace.dependencies]
# Async Runtime
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"

# Web Framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "compression-gzip"] }

# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono", "json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
config = { version = "0.14", features = ["toml"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Time & UUID
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter", "local-time"] }
tracing-appender = "0.2"

# Security
bcrypt = "0.15"

# WebSocket
tokio-tungstenite = "0.21"

# Utilities
rand = "0.8"
once_cell = "1.19"

# Testing (dev-dependencies)
tokio-test = "0.4"
fake = { version = "2.9", features = ["derive", "chrono", "uuid"] }
assert_matches = "1.5"
```

### Core Domain Crate Dependencies

#### helix-core/Cargo.toml
```toml
[package]
name = "helix-core"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async
tokio = { workspace = true }
futures = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Time & ID
chrono = { workspace = true }
uuid = { workspace = true }

# Logging
tracing = { workspace = true }

# Utilities
once_cell = { workspace = true }

[dev-dependencies]
tokio-test = { workspace = true }
```

#### helix-database/Cargo.toml
```toml
[package]
name = "helix-database"
version = "0.1.0"
edition = "2021"

[dependencies]
helix-core = { path = "../helix-core" }

# Database
sqlx = { workspace = true }

# Async
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Time & ID
chrono = { workspace = true }
uuid = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }

[dev-dependencies]
tokio-test = { workspace = true }
```

#### helix-events/Cargo.toml
```toml
[package]
name = "helix-events"
version = "0.1.0"
edition = "2021"

[dependencies]
helix-core = { path = "../helix-core" }

# Async
tokio = { workspace = true }
futures = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Time & ID
chrono = { workspace = true }
uuid = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }

# Traits
async-trait = "0.1"

[dev-dependencies]
tokio-test = { workspace = true }
```

#### helix-websocket/Cargo.toml
```toml
[package]
name = "helix-websocket"
version = "0.1.0"
edition = "2021"

[dependencies]
helix-core = { path = "../helix-core" }
helix-events = { path = "../helix-events" }

# WebSocket
tokio-tungstenite = { workspace = true }

# Async
tokio = { workspace = true }
futures = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Time & ID
chrono = { workspace = true }
uuid = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }

# HTTP for WebSocket upgrade
hyper = { version = "1.0", features = ["server", "http1", "http2"] }

[dev-dependencies]
tokio-test = { workspace = true }
```

#### helix-api/Cargo.toml
```toml
[package]
name = "helix-api"
version = "0.1.0"
edition = "2021"

[dependencies]
helix-core = { path = "../helix-core" }
helix-database = { path = "../helix-database" }
helix-events = { path = "../helix-events" }
# All domain crates...
helix-account = { path = "../helix-account" }
helix-server = { path = "../helix-server" }
# ... etc

# Web framework
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }

# Async
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Time & ID
chrono = { workspace = true }
uuid = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }

# Security
bcrypt = { workspace = true }

[dev-dependencies]
tokio-test = { workspace = true }
axum-test = "14.0"
```

### Main Application Dependencies

#### helix-main/Cargo.toml
```toml
[package]
name = "helix-main"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "helix"
path = "src/main.rs"

[dependencies]
# All Helix crates
helix-core = { path = "../crates/helix-core" }
helix-config = { path = "../crates/helix-config" }
helix-database = { path = "../crates/helix-database" }
helix-events = { path = "../crates/helix-events" }
helix-api = { path = "../crates/helix-api" }
helix-websocket = { path = "../crates/helix-websocket" }

# All domain crates
helix-account = { path = "../crates/helix-account" }
helix-cache = { path = "../crates/helix-cache" }
helix-client = { path = "../crates/helix-client" }
helix-entity = { path = "../crates/helix-entity" }
helix-log = { path = "../crates/helix-log" }
helix-network = { path = "../crates/helix-network" }
helix-notification = { path = "../crates/helix-notification" }
helix-process = { path = "../crates/helix-process" }
helix-server = { path = "../crates/helix-server" }
helix-software = { path = "../crates/helix-software" }
helix-story = { path = "../crates/helix-story" }
helix-universe = { path = "../crates/helix-universe" }

# Runtime
tokio = { workspace = true }
futures = { workspace = true }

# Configuration
config = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
tracing-appender = { workspace = true }

# Time
chrono = { workspace = true }

# Signal handling
tokio-signal = "0.2"
signal-hook = "0.3"

[dev-dependencies]
tokio-test = { workspace = true }
```

## Additional Rust Ecosystem Tools

### Development Tools
```bash
# Code formatting
rustfmt

# Linting
clippy

# Documentation
cargo doc

# Testing
cargo test

# Coverage
cargo tarpaulin

# Security audit
cargo audit

# Dependency checking
cargo outdated
```

### Performance & Monitoring

#### Profiling
```toml
[dev-dependencies]
pprof = { version = "0.13", features = ["flamegraph", "protobuf-codec"] }
```

#### Metrics
```toml
[dependencies]
metrics = "0.22"
metrics-exporter-prometheus = "0.13"
```

#### Health checks
```toml
[dependencies]
tower-http = { version = "0.5", features = ["trace", "compression-gzip", "timeout"] }
```

## Environment-Specific Features

### Development Features
```toml
[features]
default = []
dev = ["fake", "tokio-console"]

[dependencies]
# Dev-only dependencies
tokio-console = { version = "0.1", optional = true }
fake = { version = "2.9", optional = true, features = ["derive"] }
```

### Production Optimizations
```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

This mapping provides a comprehensive translation of all Elixir dependencies to their Rust equivalents, ensuring feature parity while taking advantage of Rust's performance and type safety benefits.