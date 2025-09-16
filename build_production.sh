#!/bin/bash

# HackerExperience - 100% Pure Rust Production Build

set -e

echo "🦀 HackerExperience Pure Rust Production Build"
echo "============================================="

# Check for required tools
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust."
    exit 1
fi

if ! command -v trunk &> /dev/null; then
    echo "📦 Installing trunk for WASM builds..."
    cargo install trunk
fi

if ! command -v wasm-bindgen &> /dev/null; then
    echo "📦 Installing wasm-bindgen..."
    cargo install wasm-bindgen-cli
fi

# Build the backend
echo ""
echo "🔨 Building backend server..."
cargo build --release --package he-api

# Build the Leptos frontend
echo ""
echo "🎨 Building Leptos WASM frontend..."
cd crates/he-leptos-frontend

# Create index.html for trunk if it doesn't exist
if [ ! -f "index.html" ]; then
    echo "Creating index.html for trunk..."
    cat > index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HackerExperience - 100% Pure Rust</title>
    <link data-trunk rel="rust" data-wasm-opt="z" />
    <style>
        body {
            margin: 0;
            padding: 0;
            background: #0a0a0a;
            color: #00ff00;
            font-family: 'Courier New', monospace;
        }
        #loading {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            font-size: 24px;
            animation: pulse 1s infinite;
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
    </style>
</head>
<body>
    <div id="loading">Initializing HackerExperience...</div>
</body>
</html>
EOF
fi

# Build with trunk
trunk build --release

cd ../..

# Create production directory
echo ""
echo "📦 Preparing production bundle..."
mkdir -p production
cp target/release/he-api production/server
cp -r crates/he-leptos-frontend/dist production/static

# Create systemd service file
cat > production/hackerexperience.service << 'EOF'
[Unit]
Description=HackerExperience Game Server
After=network.target postgresql.service

[Service]
Type=simple
User=hackerexperience
WorkingDirectory=/opt/hackerexperience
Environment="DATABASE_URL=postgresql://localhost/hackerexperience"
Environment="PORT=3000"
ExecStart=/opt/hackerexperience/server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Create Docker production image
cat > production/Dockerfile << 'EOF'
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY server /app/server
COPY static /app/static

ENV PORT=3000
ENV DATABASE_URL=postgresql://postgres:password@db/hackerexperience

EXPOSE 3000

CMD ["./server"]
EOF

# Create docker-compose for production
cat > production/docker-compose.yml << 'EOF'
version: '3.8'

services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: hackerexperience
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: ${DB_PASSWORD:-changeme}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ../migrations:/docker-entrypoint-initdb.d
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  game:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://postgres:${DB_PASSWORD:-changeme}@db/hackerexperience
      PORT: 3000
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped

volumes:
  postgres_data:
EOF

# Create startup script
cat > production/start.sh << 'EOF'
#!/bin/bash
echo "🚀 Starting HackerExperience Production Server"
echo "🦀 100% Pure Rust Implementation"
echo ""

# Check if running in Docker
if [ -f /.dockerenv ]; then
    exec ./server
else
    # Local production run
    export DATABASE_URL="${DATABASE_URL:-postgresql://localhost/hackerexperience}"
    export PORT="${PORT:-3000}"

    echo "📍 Server: http://0.0.0.0:$PORT"
    echo "🗄️  Database: $DATABASE_URL"
    echo ""

    exec ./server
fi
EOF

chmod +x production/start.sh

echo ""
echo "✅ Production build complete!"
echo ""
echo "📦 Production files in: ./production/"
echo ""
echo "🚀 To run in production:"
echo "   Local:  cd production && ./start.sh"
echo "   Docker: cd production && docker-compose up -d"
echo "   System: sudo cp production/hackerexperience.service /etc/systemd/system/"
echo "           sudo systemctl start hackerexperience"
echo ""
echo "🦀 100% Pure Rust - No JavaScript Required!"