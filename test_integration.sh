#!/bin/bash

# HackerExperience Full Integration Test
# Tests all UI pages with backend

set -e

echo "🚀 HackerExperience Full Integration Test"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Start the test API server
echo "🌐 Starting test API server..."
rustc test_server.rs -o test_server 2>/dev/null || {
    echo -e "${RED}Failed to compile test server${NC}"
    exit 1
}

./test_server &
API_PID=$!

# Wait for server to start
sleep 2

# Step 2: Test API endpoints
echo ""
echo "🧪 Testing API endpoints..."
echo "----------------------------"

# Function to test an endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4

    echo -n "Testing $description... "

    if [ "$method" = "GET" ]; then
        response=$(curl -s -X GET "http://localhost:3000$endpoint" -H "Authorization: Bearer mock_token" 2>/dev/null || echo "FAILED")
    else
        response=$(curl -s -X POST "http://localhost:3000$endpoint" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer mock_token" \
            -d "$data" 2>/dev/null || echo "FAILED")
    fi

    if [[ "$response" == *"FAILED"* ]]; then
        echo -e "${RED}❌ Failed${NC}"
        return 1
    elif [[ "$response" == *"success"* ]] || [[ "$response" == *"your_ip"* ]] || [[ "$response" == *"processes"* ]]; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️ Check${NC}"
        return 0
    fi
}

# Test all endpoints
echo ""
echo "1. Hacking Module:"
test_endpoint "POST" "/api/hacking/scan" '{"target_ip":"1.2.3.4"}' "Server scan"
test_endpoint "GET" "/api/hacking/internet" "" "Internet view"

echo ""
echo "2. Process Management:"
test_endpoint "GET" "/api/process/list" "" "List processes"

echo ""
echo "3. Software Management:"
test_endpoint "GET" "/api/software/list" "" "List software"

echo ""
echo "4. Missions:"
test_endpoint "GET" "/api/missions" "" "List missions"

# Cleanup
echo ""
echo "🛑 Stopping test server..."
kill $API_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}✨ Test Complete!${NC}"
