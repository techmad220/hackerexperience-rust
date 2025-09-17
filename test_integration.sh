#!/bin/bash

# Integration test script for HackerExperience Rust
# Tests all major components are working together

set -e

echo "
╔══════════════════════════════════════════════╗
║     HACKEREXPERIENCE INTEGRATION TEST        ║
╚══════════════════════════════════════════════╝
"

# Configuration
API_URL=${API_URL:-"http://localhost:3005"}
FRONTEND_URL=${FRONTEND_URL:-"http://localhost:8080"}

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
run_test() {
    local test_name=$1
    local test_command=$2

    echo -n "Testing $test_name... "

    if eval $test_command > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗${NC}"
        ((TESTS_FAILED++))
    fi
}

echo "1. Backend Tests"
echo "----------------"

# Health check
run_test "Backend health" "curl -f $API_URL/health"

# Metrics endpoint
run_test "Metrics endpoint" "curl -f $API_URL/metrics"

# Test registration
run_test "User registration" "curl -f -X POST $API_URL/api/register \
    -H 'Content-Type: application/json' \
    -d '{\"username\":\"testuser\",\"password\":\"test123\",\"email\":\"test@example.com\"}'"

# Test login
LOGIN_RESPONSE=$(curl -s -X POST $API_URL/api/login \
    -H 'Content-Type: application/json' \
    -d '{"username":"testuser","password":"test123"}' 2>/dev/null || echo "{}")

if echo "$LOGIN_RESPONSE" | grep -q "token"; then
    echo -e "Login endpoint... ${GREEN}✓${NC}"
    ((TESTS_PASSED++))
    TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)
else
    echo -e "Login endpoint... ${RED}✗${NC}"
    ((TESTS_FAILED++))
    TOKEN=""
fi

# Test authenticated endpoints if we have a token
if [ ! -z "$TOKEN" ]; then
    run_test "Get game state" "curl -f $API_URL/api/state \
        -H 'Authorization: Bearer $TOKEN'"

    run_test "Get processes" "curl -f $API_URL/api/processes \
        -H 'Authorization: Bearer $TOKEN'"

    run_test "Get hardware" "curl -f $API_URL/api/hardware \
        -H 'Authorization: Bearer $TOKEN'"
fi

echo ""
echo "2. Database Tests"
echo "-----------------"

# Test database connection
if psql ${DATABASE_URL:-"postgresql://heuser:hepass@localhost:5432/hackerexperience"} -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "Database connection... ${GREEN}✓${NC}"
    ((TESTS_PASSED++))

    # Check tables exist
    TABLES=$(psql ${DATABASE_URL:-"postgresql://heuser:hepass@localhost:5432/hackerexperience"} -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public'" 2>/dev/null || echo "0")

    if [ "$TABLES" -gt "0" ]; then
        echo -e "Database migrations... ${GREEN}✓${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "Database migrations... ${RED}✗${NC}"
        ((TESTS_FAILED++))
    fi
else
    echo -e "Database connection... ${RED}✗${NC}"
    ((TESTS_FAILED++))
fi

echo ""
echo "3. Frontend Tests"
echo "-----------------"

# Test frontend is accessible
run_test "Frontend accessible" "curl -f $FRONTEND_URL"

# Test static assets
run_test "Frontend assets" "curl -f $FRONTEND_URL/index.html"

echo ""
echo "4. WebSocket Tests"
echo "------------------"

# Test WebSocket endpoint exists
run_test "WebSocket endpoint" "curl -f -i $API_URL/ws | grep -E '(Upgrade|400)'"

echo ""
echo "5. Security Tests"
echo "-----------------"

# Test rate limiting
echo -n "Testing rate limiting... "
RATE_LIMITED=false
for i in {1..150}; do
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $API_URL/api/login \
        -X POST -H 'Content-Type: application/json' \
        -d '{"username":"test","password":"test"}' 2>/dev/null)

    if [ "$RESPONSE" = "429" ]; then
        RATE_LIMITED=true
        break
    fi
done

if [ "$RATE_LIMITED" = true ]; then
    echo -e "${GREEN}✓${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⚠${NC} (not triggered)"
fi

# Test auth protection
run_test "Auth protection" "[ $(curl -s -o /dev/null -w '%{http_code}' $API_URL/api/processes) = '401' ]"

# Test security headers
HEADERS=$(curl -s -I $API_URL/health 2>/dev/null || echo "")
if echo "$HEADERS" | grep -q "X-Frame-Options"; then
    echo -e "Security headers... ${GREEN}✓${NC}"
    ((TESTS_PASSED++))
else
    echo -e "Security headers... ${YELLOW}⚠${NC}"
fi

echo ""
echo "6. Performance Tests"
echo "--------------------"

# Test response time
START_TIME=$(date +%s%N)
curl -s $API_URL/health > /dev/null 2>&1
END_TIME=$(date +%s%N)
RESPONSE_TIME=$(( ($END_TIME - $START_TIME) / 1000000 ))

if [ "$RESPONSE_TIME" -lt "100" ]; then
    echo -e "Response time (<100ms)... ${GREEN}✓${NC} (${RESPONSE_TIME}ms)"
    ((TESTS_PASSED++))
else
    echo -e "Response time (<100ms)... ${YELLOW}⚠${NC} (${RESPONSE_TIME}ms)"
fi

# Memory check (if available)
if command -v docker &> /dev/null; then
    MEMORY=$(docker stats --no-stream --format "{{.MemUsage}}" he-api 2>/dev/null | cut -d'/' -f1 || echo "N/A")
    echo "Memory usage: $MEMORY"
fi

echo ""
echo "════════════════════════════════════════"
echo "         TEST RESULTS SUMMARY"
echo "════════════════════════════════════════"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"

if [ "$TESTS_FAILED" -eq "0" ]; then
    echo -e "\n${GREEN}✅ ALL TESTS PASSED!${NC}"
    echo "The game is ready for production!"
    exit 0
else
    PERCENTAGE=$((TESTS_PASSED * 100 / (TESTS_PASSED + TESTS_FAILED)))
    echo -e "\n${YELLOW}⚠️  $PERCENTAGE% tests passing${NC}"

    if [ "$PERCENTAGE" -ge "80" ]; then
        echo "The game is mostly functional but needs some fixes."
    else
        echo "Critical issues detected. Please review the failures."
    fi
    exit 1
fi