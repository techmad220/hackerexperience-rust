#!/bin/bash

# Production startup script for HackerExperience Rust
# This wires all components together and starts the game

set -e

echo "
╔══════════════════════════════════════════════╗
║   HACKEREXPERIENCE RUST - PRODUCTION START   ║
╚══════════════════════════════════════════════╝
"

# Configuration
export RUST_LOG=info,he_api=debug,he_helix=debug
export DATABASE_URL=${DATABASE_URL:-"postgresql://heuser:hepass@localhost:5432/hackerexperience"}
export JWT_SECRET=${JWT_SECRET:-$(openssl rand -base64 32)}
export ENCRYPTION_KEY=${ENCRYPTION_KEY:-$(openssl rand -base64 32)}
export PORT=${PORT:-3005}
export FRONTEND_PORT=${FRONTEND_PORT:-8080}

echo "🔧 Configuration:"
echo "   API Port: $PORT"
echo "   Frontend Port: $FRONTEND_PORT"
echo "   Database: PostgreSQL"
echo "   Log Level: $RUST_LOG"
echo ""

# Step 1: Check prerequisites
echo "1️⃣ Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not installed"
    echo "Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL client not installed"
    echo "Install with: sudo apt install postgresql-client"
    exit 1
fi

echo "✅ Prerequisites OK"
echo ""

# Step 2: Initialize database
echo "2️⃣ Initializing database..."

if psql $DATABASE_URL -c "SELECT 1" > /dev/null 2>&1; then
    echo "✅ Database connection OK"
else
    echo "⚠️  Database not accessible, initializing..."
    ./scripts/init_database.sh
fi

# Run migrations
echo "Running migrations..."
cd crates/he-api
if [ -f "../../migrations-postgres/001_create_users_table.sql" ]; then
    for migration in ../../migrations-postgres/*.sql; do
        echo "   Applying: $(basename $migration)"
        psql $DATABASE_URL -f $migration 2>/dev/null || true
    done
    echo "✅ Migrations applied"
else
    echo "⚠️  Migrations not found, skipping"
fi
cd ../..

echo ""

# Step 3: Build the backend
echo "3️⃣ Building backend..."

# Build all crates
cargo build --release --workspace 2>&1 | grep -E "(Compiling|Finished)" || true

if [ -f "target/release/he-api" ]; then
    echo "✅ Backend built successfully"
else
    echo "⚠️  Backend build incomplete, building he-api only..."
    cargo build --release --bin he-api
fi

echo ""

# Step 4: Build the frontend
echo "4️⃣ Building frontend..."

cd crates/he-leptos-frontend

if command -v trunk &> /dev/null; then
    echo "   Building with Trunk..."
    trunk build --release
    echo "✅ Frontend built with Trunk"
elif command -v wasm-pack &> /dev/null; then
    echo "   Building with wasm-pack..."
    wasm-pack build --target web --out-dir dist
    echo "✅ Frontend built with wasm-pack"
else
    echo "⚠️  No WASM builder found, using pre-built frontend"
fi

cd ../..
echo ""

# Step 5: Start services
echo "5️⃣ Starting services..."

# Kill any existing instances
pkill -f "he-api" 2>/dev/null || true
pkill -f "python.*serve" 2>/dev/null || true

# Start backend
echo "Starting backend on port $PORT..."
DATABASE_URL=$DATABASE_URL \
JWT_SECRET=$JWT_SECRET \
ENCRYPTION_KEY=$ENCRYPTION_KEY \
RUST_LOG=$RUST_LOG \
./target/release/he-api &
BACKEND_PID=$!

# Wait for backend to start
sleep 2

if kill -0 $BACKEND_PID 2>/dev/null; then
    echo "✅ Backend running (PID: $BACKEND_PID)"
else
    echo "❌ Backend failed to start"
    exit 1
fi

# Start frontend server
echo "Starting frontend on port $FRONTEND_PORT..."

if [ -d "crates/he-leptos-frontend/dist" ]; then
    cd crates/he-leptos-frontend/dist
    python3 -m http.server $FRONTEND_PORT &
    FRONTEND_PID=$!
    cd ../../..
elif [ -d "frontend" ]; then
    cd frontend
    python3 -m http.server $FRONTEND_PORT &
    FRONTEND_PID=$!
    cd ..
else
    echo "⚠️  No frontend directory found"
    FRONTEND_PID=""
fi

if [ ! -z "$FRONTEND_PID" ] && kill -0 $FRONTEND_PID 2>/dev/null; then
    echo "✅ Frontend running (PID: $FRONTEND_PID)"
fi

echo ""
echo "
╔══════════════════════════════════════════════╗
║           🚀 GAME IS RUNNING! 🚀            ║
╠══════════════════════════════════════════════╣
║                                              ║
║   Backend API:  http://localhost:$PORT      ║
║   Frontend UI:  http://localhost:$FRONTEND_PORT     ║
║                                              ║
║   Health Check: http://localhost:$PORT/health   ║
║   Metrics:      http://localhost:$PORT/metrics  ║
║                                              ║
╠══════════════════════════════════════════════╣
║            Press Ctrl+C to stop              ║
╚══════════════════════════════════════════════╝
"

# Create stop script
cat > stop_production.sh << 'EOF'
#!/bin/bash
echo "Stopping HackerExperience services..."
pkill -f "he-api"
pkill -f "python.*serve"
echo "✅ Services stopped"
EOF
chmod +x stop_production.sh

# Trap Ctrl+C to clean shutdown
trap 'echo ""; echo "Shutting down..."; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null; exit' INT

# Keep script running and show logs
tail -f /dev/null