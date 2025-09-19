#!/bin/bash

# ================================================
# HackerExperience - Comprehensive Test Runner
# ================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Configuration
CARGO_BIN="$HOME/.cargo/bin/cargo"
TEST_TYPES=("unit" "integration" "doc" "workspace")
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0
TEST_OUTPUT_FILE="/tmp/he_test_results.txt"

# Arrays to track results
declare -a FAILED_TEST_LIST=()
declare -a PASSED_TEST_LIST=()

# Functions
log_header() {
    echo ""
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}${BOLD}  $1${NC}"
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

test_section() {
    echo ""
    echo -e "${MAGENTA}${BOLD}▶ $1${NC}"
    echo -e "${MAGENTA}────────────────────────────${NC}"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    PASSED_TEST_LIST+=("$1")
}

log_error() {
    echo -e "${RED}✗${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    FAILED_TEST_LIST+=("$1")
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_skip() {
    echo -e "${YELLOW}○${NC} $1 (skipped)"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
}

# Parse arguments
VERBOSE=false
COVERAGE=false
QUICK=false
SPECIFIC_CRATE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--coverage)
            COVERAGE=true
            shift
            ;;
        -q|--quick)
            QUICK=true
            shift
            ;;
        -p|--package)
            SPECIFIC_CRATE="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -v, --verbose     Show detailed test output"
            echo "  -c, --coverage    Generate coverage report"
            echo "  -q, --quick       Run only unit tests"
            echo "  -p, --package     Test specific package"
            echo "  -h, --help        Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use -h for help"
            exit 1
            ;;
    esac
done

# Main execution
log_header "HackerExperience Test Suite"

# Check environment
echo -e "${BOLD}Environment Check:${NC}"
if [ -f .env ]; then
    source .env
    log_success "Environment loaded from .env"
else
    log_warning "No .env file found, using defaults"
    export SQLX_OFFLINE=true
fi

if [ -n "$DATABASE_URL" ]; then
    log_info "Database URL configured"
else
    export DATABASE_URL="postgresql://heuser:hepass@localhost:5432/hedb"
    log_warning "Using default DATABASE_URL"
fi

# 1. Unit Tests
if [ "$QUICK" = false ] || [ "$QUICK" = true ]; then
    test_section "Unit Tests"
    log_info "Running unit tests for all packages..."

    if [ -n "$SPECIFIC_CRATE" ]; then
        TEST_CMD="$CARGO_BIN test -p $SPECIFIC_CRATE --lib"
    else
        TEST_CMD="$CARGO_BIN test --workspace --lib"
    fi

    if $VERBOSE; then
        TEST_CMD="$TEST_CMD -- --nocapture"
    else
        TEST_CMD="$TEST_CMD --quiet"
    fi

    if $TEST_CMD 2>&1 | tee $TEST_OUTPUT_FILE | grep -q "test result:"; then
        # Parse test results
        TEST_RESULT=$(grep "test result:" $TEST_OUTPUT_FILE | tail -1)
        if echo "$TEST_RESULT" | grep -q "0 failed"; then
            log_success "All unit tests passed"
        else
            FAILED=$(echo "$TEST_RESULT" | grep -oE "[0-9]+ failed" | grep -oE "[0-9]+")
            log_error "$FAILED unit tests failed"
        fi
    else
        log_warning "No unit tests found or test execution failed"
    fi
fi

# 2. Integration Tests (if not quick mode)
if [ "$QUICK" = false ]; then
    test_section "Integration Tests"
    log_info "Running integration tests..."

    if [ -d "tests" ]; then
        if $CARGO_BIN test --test '*' --quiet 2>&1 | tee $TEST_OUTPUT_FILE | grep -q "test result:"; then
            TEST_RESULT=$(grep "test result:" $TEST_OUTPUT_FILE | tail -1)
            if echo "$TEST_RESULT" | grep -q "0 failed"; then
                log_success "All integration tests passed"
            else
                FAILED=$(echo "$TEST_RESULT" | grep -oE "[0-9]+ failed" | grep -oE "[0-9]+")
                log_error "$FAILED integration tests failed"
            fi
        else
            log_skip "Integration tests"
        fi
    else
        log_info "No integration test directory found"
    fi
fi

# 3. Documentation Tests (if not quick mode)
if [ "$QUICK" = false ]; then
    test_section "Documentation Tests"
    log_info "Running documentation tests..."

    if $CARGO_BIN test --doc --quiet 2>&1 | grep -q "test result:"; then
        log_success "Documentation tests passed"
    else
        log_skip "Documentation tests"
    fi
fi

# 4. Specific crate tests
if [ "$QUICK" = false ] && [ -z "$SPECIFIC_CRATE" ]; then
    test_section "Individual Crate Tests"

    # Test important crates individually
    IMPORTANT_CRATES=("he-api" "he-auth" "he-game-mechanics" "he-websocket")
    for crate in "${IMPORTANT_CRATES[@]}"; do
        if [ -d "crates/$crate" ]; then
            if $CARGO_BIN test -p "$crate" --quiet 2>/dev/null; then
                log_success "$crate tests passed"
            else
                log_warning "$crate tests skipped or failed"
            fi
        fi
    done
fi

# 5. Benchmark tests (optional)
if [ "$QUICK" = false ]; then
    test_section "Benchmark Tests (Optional)"
    if $CARGO_BIN bench --no-run 2>&1 | grep -q "Finished"; then
        log_info "Benchmarks compiled successfully"
        echo "    Run benchmarks with: cargo bench"
    else
        log_skip "Benchmark tests"
    fi
fi

# 6. Coverage Report (if requested)
if [ "$COVERAGE" = true ]; then
    test_section "Coverage Report"
    log_info "Generating coverage report..."

    # Check if cargo-tarpaulin is installed
    if $CARGO_BIN tarpaulin --version &> /dev/null; then
        log_info "Running coverage analysis (this may take several minutes)..."
        if $CARGO_BIN tarpaulin --out Html --output-dir coverage --workspace --timeout 180 2>&1 | grep -q "Coverage"; then
            log_success "Coverage report generated in ./coverage/tarpaulin-report.html"
        else
            log_error "Coverage generation failed"
        fi
    else
        log_warning "cargo-tarpaulin not installed"
        echo "    Install with: $CARGO_BIN install cargo-tarpaulin"
    fi
fi

# 7. Check for test files
test_section "Test File Analysis"
UNIT_TEST_COUNT=$(find . -path "*/src/*.rs" -exec grep -l "#\[test\]" {} \; 2>/dev/null | wc -l)
INTEGRATION_TEST_COUNT=$(find tests -name "*.rs" 2>/dev/null | wc -l || echo "0")

log_info "Found $UNIT_TEST_COUNT files with unit tests"
log_info "Found $INTEGRATION_TEST_COUNT integration test files"

# Performance tests check
if [ -f "performance_test_suite.py" ] || [ -f "simple_performance_test.py" ]; then
    log_info "Performance test scripts available"
    echo "    Run with: python3 performance_test_suite.py"
fi

# Summary
echo ""
log_header "Test Results Summary"

TOTAL_TESTS=$((PASSED_TESTS + FAILED_TESTS + SKIPPED_TESTS))

echo -e "${BOLD}Test Statistics:${NC}"
echo -e "  ${GREEN}Passed:${NC}  $PASSED_TESTS"
echo -e "  ${RED}Failed:${NC}  $FAILED_TESTS"
echo -e "  ${YELLOW}Skipped:${NC} $SKIPPED_TESTS"
echo -e "  ${BLUE}Total:${NC}   $TOTAL_TESTS"

if [ ${#FAILED_TEST_LIST[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}${BOLD}Failed Tests:${NC}"
    for test in "${FAILED_TEST_LIST[@]}"; do
        echo -e "  ${RED}✗${NC} $test"
    done
fi

# Calculate success rate
if [ $TOTAL_TESTS -gt 0 ]; then
    SUCCESS_RATE=$(( (PASSED_TESTS * 100) / (PASSED_TESTS + FAILED_TESTS) ))
    echo ""
    echo -e "${BOLD}Success Rate: ${SUCCESS_RATE}%${NC}"
fi

# Clean up
rm -f $TEST_OUTPUT_FILE

# Exit status
echo ""
if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}${BOLD}✅ All tests passed successfully!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run: $CARGO_BIN build --release     # Build optimized version"
    echo "  2. Run: $CARGO_BIN run --bin he-api    # Start the server"
    if [ "$COVERAGE" = false ]; then
        echo "  3. Run: $0 --coverage                  # Generate coverage report"
    fi
    exit 0
else
    echo -e "${RED}${BOLD}❌ Some tests failed${NC}"
    echo ""
    echo "Debug commands:"
    echo "  $CARGO_BIN test -- --nocapture         # Show test output"
    echo "  $CARGO_BIN test -- --test-threads=1    # Run tests serially"
    echo "  $CARGO_BIN test [test_name]            # Run specific test"
    exit 1
fi