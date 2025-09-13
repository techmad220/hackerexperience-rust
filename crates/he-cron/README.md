# HackerExperience Cron System

A modern, async Rust replacement for the legacy PHP cron jobs. Built with tokio-cron-scheduler for precise timing and async/await for efficient execution.

## Overview

This crate provides a comprehensive cron system that replaces all the PHP cron jobs from the legacy HackerExperience system. Each job maintains the exact same business logic as the original PHP implementation while using modern Rust patterns.

## Features

- **Async/await**: All jobs run asynchronously for better performance
- **Proper scheduling**: Uses tokio-cron-scheduler for reliable cron execution
- **Error handling**: Comprehensive error handling with proper logging
- **Database integration**: Uses sqlx for safe, async database operations
- **AWS S3 integration**: Backup jobs automatically upload to S3
- **Modular design**: Each job is implemented as a separate module

## Ported Jobs

### Backup Jobs
- `backup_forum` - Backs up forum database to S3 (every 4 hours)
- `backup_game` - Backs up game database to S3 (every 2 hours)

### Game Maintenance
- `restore_software` - Restores NPC software from templates (every 30 minutes)
- `generate_missions` - Generates new missions for players (every hour)
- `update_premium` - Updates expired premium subscriptions (every 15 minutes)

### War Management
- `defcon` - Detects clan war conditions (every 5 minutes)
- `end_war` - Processes finished clan wars (every minute)

### Statistics
- `update_server_stats` - Updates round statistics (every 10 minutes)

### Cleanup
- `safenet_update` - Manages SafeNet system (every 30 minutes)
- `doom_updater` - Monitors doom virus countdowns (every minute)
- `finish_round` - Handles round completion (triggered by doom virus)

## Environment Variables

```bash
# Database
DATABASE_URL="mysql://user:password@localhost/game"

# AWS S3 (for backups)
AWS_ACCESS_KEY_ID="your-access-key"
AWS_SECRET_ACCESS_KEY="your-secret-key"
S3_BACKUP_BUCKET="your-backup-bucket"

# Database credentials (for mysqldump)
FORUM_DB_HOST="localhost"
FORUM_DB_USER="forum"
FORUM_DB_PASSWORD="password"
FORUM_DB_NAME="forum"

GAME_DB_HOST="localhost"
GAME_DB_USER="he"
GAME_DB_PASSWORD="password"
GAME_DB_NAME="game"
```

## Usage

### Running the Cron Daemon

```bash
# Start the cron scheduler
cargo run --bin he-cron

# Or install and run
cargo install --path .
he-cron
```

### Using in Code

```rust
use he_cron::{CronScheduler, start_cron_scheduler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Use the convenience function
    start_cron_scheduler().await?;
    
    // Option 2: Manual setup
    let scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    // Keep running...
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

### Individual Jobs

You can also run individual jobs if needed:

```rust
use he_cron::jobs::backup_game::GameBackupJob;
use std::sync::Arc;

let db_pool = Arc::new(/* your database pool */);
GameBackupJob::execute(db_pool).await?;
```

## Schedule Configuration

Each job has a predefined schedule based on the original PHP cron configuration:

- Backup jobs: Regular intervals for data safety
- Maintenance jobs: Frequent intervals to keep the game running smoothly
- War management: High frequency for real-time war detection
- Statistics: Regular intervals for accurate reporting
- Cleanup: As needed to maintain system health

## Architecture

```
he-cron/
├── src/
│   ├── lib.rs              # Main library interface
│   ├── scheduler.rs        # Tokio-based cron scheduler
│   ├── error.rs           # Error types and handling
│   ├── traits.rs          # Job trait definitions
│   ├── utils.rs           # Utility functions
│   ├── bin/
│   │   └── he-cron.rs     # CLI binary
│   └── jobs/
│       ├── mod.rs         # Job module exports
│       ├── backup_forum.rs
│       ├── backup_game.rs
│       ├── restore_software.rs
│       ├── update_server_stats.rs
│       ├── end_war.rs
│       ├── generate_missions.rs
│       ├── defcon.rs
│       ├── update_premium.rs
│       ├── safenet_update.rs
│       ├── doom_updater.rs
│       └── finish_round.rs
└── Cargo.toml
```

## Logging

The system uses `tracing` for structured logging. Set the log level with:

```bash
RUST_LOG=info he-cron
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific job
cargo test --lib jobs::backup_game
```

## Migration from PHP

The Rust implementation maintains 100% functional compatibility with the PHP versions:

1. **Same database schema**: All SQL queries use the existing tables
2. **Same business logic**: Algorithms and calculations are identical
3. **Same scheduling**: Cron expressions match the original timings
4. **Same side effects**: File creation, S3 uploads, etc. work the same way

## Performance Benefits

- **Memory efficiency**: Rust's zero-cost abstractions and memory safety
- **Async I/O**: Non-blocking database and network operations
- **Error handling**: Explicit error types prevent silent failures
- **Resource usage**: Lower CPU and memory usage compared to PHP processes
- **Reliability**: Strong typing and ownership prevent common runtime errors

## Monitoring

All jobs log their execution status:
- Start/completion messages
- Error details with context
- Performance metrics (execution time)
- Business logic results (records processed, etc.)

## Contributing

When adding new cron jobs:

1. Create a new module in `src/jobs/`
2. Implement the job logic with proper error handling
3. Add the job to `jobs/mod.rs` exports
4. Register the job in `scheduler.rs`
5. Update this README with the new job details
6. Add tests for the job logic