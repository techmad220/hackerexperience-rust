#!/bin/bash

# Script to fix dependency conflicts across all crates
# Priority: sqlx and bcrypt versions

echo "🔧 Fixing dependency conflicts in HackerExperience Rust project..."

# Update all Cargo.toml files to use sqlx 0.8
echo "📦 Updating SQLx to 0.8 across all crates..."
find . -name "Cargo.toml" -type f | while read -r file; do
    # Update sqlx from 0.7 to 0.8
    sed -i 's/sqlx = { version = "0\.7/sqlx = { version = "0.8/g' "$file"
    sed -i 's/sqlx = "0\.7/sqlx = "0.8/g' "$file"
done

# Update all bcrypt to 0.15
echo "🔐 Updating bcrypt to 0.15 across all crates..."
find . -name "Cargo.toml" -type f | while read -r file; do
    sed -i 's/bcrypt = "0\.14/bcrypt = "0.15/g' "$file"
    sed -i 's/bcrypt = { version = "0\.14/bcrypt = { version = "0.15/g' "$file"
done

# Use workspace dependencies where possible
echo "🏗️ Migrating to workspace dependencies..."

# List of crates to check
CRATES=(
    "crates/he-api"
    "crates/he-auth"
    "crates/he-core"
    "crates/he-database"
    "crates/he-db"
    "crates/he-game-mechanics"
)

for crate in "${CRATES[@]}"; do
    if [ -f "$crate/Cargo.toml" ]; then
        echo "  Updating $crate..."

        # Replace direct dependencies with workspace references for common deps
        sed -i 's/^tokio = .*/tokio = { workspace = true }/g' "$crate/Cargo.toml"
        sed -i 's/^serde = .*/serde = { workspace = true }/g' "$crate/Cargo.toml"
        sed -i 's/^serde_json = .*/serde_json = { workspace = true }/g' "$crate/Cargo.toml"
        sed -i 's/^anyhow = .*/anyhow = { workspace = true }/g' "$crate/Cargo.toml"
        sed -i 's/^thiserror = .*/thiserror = { workspace = true }/g' "$crate/Cargo.toml"
        sed -i 's/^tracing = .*/tracing = { workspace = true }/g' "$crate/Cargo.toml"
        sed -i 's/^uuid = .*/uuid = { workspace = true }/g' "$crate/Cargo.toml"
        sed -i 's/^chrono = .*/chrono = { workspace = true }/g' "$crate/Cargo.toml"
    fi
done

echo "✅ Dependency conflicts fixed!"
echo ""
echo "Next steps:"
echo "1. Run 'cargo check --workspace' to verify all dependencies resolve correctly"
echo "2. Run 'cargo update' to update Cargo.lock"
echo "3. Run tests to ensure nothing broke: 'cargo test --workspace'"