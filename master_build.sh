#!/bin/bash

# ================================================
# HackerExperience Rust - Master Build Script
# ================================================
# Complete build pipeline with all checks

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
STEPS_COMPLETED=0
TOTAL_STEPS=5
FAILED=false

# Functions
print_banner() {
    echo ""
    echo -e "${CYAN}${BOLD}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}${BOLD}║           HackerExperience Rust Master Build             ║${NC}"
    echo -e "${CYAN}${BOLD}║                  Complete Build Pipeline                 ║${NC}"
    echo -e "${CYAN}${BOLD}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

step_header() {
    STEPS_COMPLETED=$((STEPS_COMPLETED + 1))
    echo ""
    echo -e "${MAGENTA}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${MAGENTA}${BOLD}Step $STEPS_COMPLETED/$TOTAL_STEPS: $1${NC}"
    echo -e "${MAGENTA}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

run_step() {
    local script=$1
    local description=$2

    if [ -x "./$script" ]; then
        echo -e "${BLUE}Running: ${NC}$description"
        if ./"$script"; then
            echo -e "${GREEN}✓ $description completed successfully${NC}"
            return 0
        else
            echo -e "${RED}✗ $description failed${NC}"
            FAILED=true
            return 1
        fi
    else
        echo -e "${YELLOW}⚠ Script $script not found or not executable${NC}"
        return 1
    fi
}

# Main execution
clear
print_banner

# Display system info
echo -e "${BOLD}System Information:${NC}"
echo -e "  ${BLUE}OS:${NC} $(uname -s) $(uname -r)"
echo -e "  ${BLUE}Rust:${NC} $($CARGO_BIN --version 2>/dev/null || echo 'Not found')"
echo -e "  ${BLUE}Date:${NC} $(date)"
echo -e "  ${BLUE}Directory:${NC} $(pwd)"

# Step 1: Database Verification
step_header "Database Setup Verification"
if run_step "verify_database.sh" "Database verification"; then
    echo -e "${GREEN}Database is ready${NC}"
else
    echo -e "${YELLOW}Database setup may need attention${NC}"
    echo "Continuing with build..."
fi

# Step 2: Compilation Checks
step_header "Compilation Checks"
if ! run_step "check_compilation.sh" "Compilation checks"; then
    echo -e "${RED}Compilation checks failed. Fix errors before continuing.${NC}"
    exit 1
fi

# Step 3: Run Tests
step_header "Running Test Suite"
if run_step "run_tests.sh" "Test suite" || [ "$?" = "0" ]; then
    echo -e "${GREEN}Tests completed${NC}"
else
    echo -e "${YELLOW}Some tests may have failed${NC}"
fi

# Step 4: Build Release Version
step_header "Building Release Version"
echo -e "${BLUE}Building optimized release binary...${NC}"
if $CARGO_BIN build --release --bin he-api 2>&1 | grep -E "(Compiling|Finished)" | tail -5; then
    echo -e "${GREEN}✓ Release build completed${NC}"

    # Check binary size
    if [ -f "target/release/he-api" ]; then
        SIZE=$(du -h target/release/he-api | cut -f1)
        echo -e "${BLUE}Binary size:${NC} $SIZE"
    fi
else
    echo -e "${RED}✗ Release build failed${NC}"
    FAILED=true
fi

# Step 5: Final Verification
step_header "Final Verification"
echo -e "${BLUE}Performing final checks...${NC}"

# Check if all important files exist
REQUIRED_FILES=(
    "Cargo.toml"
    "target/release/he-api"
    ".env"
)

MISSING_FILES=0
for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $file exists"
    else
        echo -e "${RED}✗${NC} $file missing"
        MISSING_FILES=$((MISSING_FILES + 1))
    fi
done

# Summary
echo ""
echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}${BOLD}                Build Summary                ${NC}"
echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ "$FAILED" = false ] && [ $MISSING_FILES -eq 0 ]; then
    echo ""
    echo -e "${GREEN}${BOLD}✅ BUILD SUCCESSFUL!${NC}"
    echo ""
    echo -e "${BOLD}The HackerExperience Rust server is ready to run!${NC}"
    echo ""
    echo -e "${BOLD}Quick Start Commands:${NC}"
    echo -e "  ${BLUE}Development:${NC}"
    echo -e "    $CARGO_BIN run --bin he-api"
    echo ""
    echo -e "  ${BLUE}Production:${NC}"
    echo -e "    ./target/release/he-api"
    echo ""
    echo -e "  ${BLUE}With environment:${NC}"
    echo -e "    source .env && ./target/release/he-api"
    echo ""
    echo -e "${BOLD}Additional Commands:${NC}"
    echo -e "  ${BLUE}Run specific tests:${NC} ./run_tests.sh -p <package-name>"
    echo -e "  ${BLUE}Generate coverage:${NC} ./run_tests.sh --coverage"
    echo -e "  ${BLUE}Quick rebuild:${NC}     ./quick_build.sh"
    echo -e "  ${BLUE}Check database:${NC}    ./verify_database.sh"
else
    echo ""
    echo -e "${RED}${BOLD}❌ BUILD FAILED${NC}"
    echo ""
    echo -e "${BOLD}Issues found:${NC}"
    if [ "$FAILED" = true ]; then
        echo -e "  ${RED}•${NC} One or more build steps failed"
    fi
    if [ $MISSING_FILES -gt 0 ]; then
        echo -e "  ${RED}•${NC} Required files are missing"
    fi
    echo ""
    echo -e "${BOLD}Debug Steps:${NC}"
    echo -e "  1. Check the output above for specific errors"
    echo -e "  2. Run individual scripts to isolate issues:"
    echo -e "     ./verify_database.sh"
    echo -e "     ./check_compilation.sh"
    echo -e "     ./run_tests.sh --verbose"
    echo -e "  3. Check logs in target/debug/ for more details"
    exit 1
fi

# Display build time
END_TIME=$(date +%s)
START_TIME=${START_TIME:-$END_TIME}
ELAPSED=$((END_TIME - START_TIME))
echo ""
echo -e "${BLUE}Build completed in ${ELAPSED} seconds${NC}"
echo ""