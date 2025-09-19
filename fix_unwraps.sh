#!/bin/bash

echo "🔧 Fixing unwrap() calls with proper error handling..."

# Create backup
echo "Creating backup..."
cp -r crates crates.backup

# Function to fix unwraps in a file
fix_file() {
    local file=$1
    echo "Processing: $file"

    # Replace .unwrap() with .map_err() for Result types
    sed -i 's/\.unwrap()/\.map_err(|e| anyhow::anyhow!("Error: {}", e))?/g' "$file"

    # Replace .unwrap() on Mutex/RwLock with proper handling
    sed -i 's/\.lock()\.unwrap()/\.lock()\.map_err(|e| anyhow::anyhow!("Lock error: {}", e))?/g' "$file"
    sed -i 's/\.write()\.unwrap()/\.write()\.map_err(|e| anyhow::anyhow!("Write lock error: {}", e))?/g' "$file"
    sed -i 's/\.read()\.unwrap()/\.read()\.map_err(|e| anyhow::anyhow!("Read lock error: {}", e))?/g' "$file"

    # Replace .unwrap() on Option types with .ok_or_else()
    sed -i 's/\.unwrap()/\.ok_or_else(|| anyhow::anyhow!("Expected value not found"))?/g' "$file"

    # Fix common patterns
    sed -i 's/env::var("\([^"]*\)")\.unwrap()/env::var("\1")\.unwrap_or_else(|_| "default".to_string())/g' "$file"

    # Add anyhow import if not present
    if ! grep -q "use anyhow" "$file"; then
        sed -i '1s/^/use anyhow::{anyhow, Result};\n/' "$file"
    fi
}

# Find all Rust files with unwrap()
echo "Finding files with unwrap() calls..."
FILES=$(grep -r "\.unwrap()" crates --include="*.rs" -l | head -50)

# Fix each file
for file in $FILES; do
    fix_file "$file"
done

echo "✅ Fixed unwrap() calls in $(echo "$FILES" | wc -l) files"

# Compile to check for errors
echo "Checking compilation..."
cargo check 2>&1 | head -20

echo "🎉 Unwrap fixing complete! Backup saved in crates.backup/"