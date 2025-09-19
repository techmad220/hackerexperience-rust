#!/bin/bash

# ================================================
# HackerExperience - Compilation Checks
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
RUSTC_BIN="$HOME/.cargo/bin/rustc"
MIN_RUST_VERSION="1.70.0"
TOTAL_CHECKS=10
CURRENT_CHECK=0
FAILED_CHECKS=0
WARNINGS=0

# Arrays to track results
declare -a PASSED_CHECKS=()
declare -a FAILED_CHECK_LIST=()
declare -a WARNING_LIST=()

# Functions
log_header() {
    echo ""
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}${BOLD}  $1${NC}"
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

check_step() {
    CURRENT_CHECK=$((CURRENT_CHECK + 1))
    echo ""
    echo -e "${BLUE}[Check $CURRENT_CHECK/$TOTAL_CHECKS]${NC} ${BOLD}$1${NC}"
    echo -e "${BLUE}────────────────────────────${NC}"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
    PASSED_CHECKS+=("$1")
}

log_error() {
    echo -e "${RED}✗${NC} $1"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
    FAILED_CHECK_LIST+=("$1")
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
    WARNINGS=$((WARNINGS + 1))
    WARNING_LIST+=("$1")
}

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Main checks
log_header "Compilation Checks for HackerExperience Rust"

# 1. Check Rust installation
check_step "Rust Installation"
if [ -x "$RUSTC_BIN" ]; then
    RUST_VERSION=$($RUSTC_BIN --version | cut -d' ' -f2)
    log_success "Rust compiler found: $RUST_VERSION"

    # Check minimum version
    if [ "$(printf '%s\n' "$MIN_RUST_VERSION" "$RUST_VERSION" | sort -V | head -n1)" = "$MIN_RUST_VERSION" ]; then
        log_success "Rust version meets minimum requirement ($MIN_RUST_VERSION)"
    else
        log_warning "Rust version may be too old (minimum: $MIN_RUST_VERSION)"
    fi
else
    log_error "Rust compiler not found at $RUSTC_BIN"
    echo "    Install from: https://rustup.rs/"
    exit 1
fi

# 2. Check Cargo
check_step "Cargo Build Tool"
if [ -x "$CARGO_BIN" ]; then
    CARGO_VERSION=$($CARGO_BIN --version | cut -d' ' -f2)
    log_success "Cargo found: $CARGO_VERSION"
else
    log_error "Cargo not found at $CARGO_BIN"
    exit 1
fi

# 3. Check workspace structure
check_step "Workspace Structure"
if [ -f "Cargo.toml" ]; then
    log_success "Root Cargo.toml found"

    if grep -q "\[workspace\]" Cargo.toml; then
        log_success "Workspace configuration detected"

        # Count workspace members
        MEMBER_COUNT=$(grep -c "^he-" Cargo.toml || echo "0")
        log_info "Found $MEMBER_COUNT workspace members"
    else
        log_warning "No workspace configuration found"
    fi
else
    log_error "No Cargo.toml found in root directory"
    exit 1
fi

# 4. Check dependencies
check_step "Dependency Resolution"
log_info "Fetching dependencies..."
if $CARGO_BIN fetch 2>&1 | grep -q "Finished"; then
    log_success "All dependencies fetched successfully"
else
    log_warning "Some dependencies may have issues"
fi

# 5. Check workspace compilation
check_step "Workspace Compilation Check"
log_info "Running cargo check (this may take a moment)..."
CHECK_OUTPUT=$($CARGO_BIN check --workspace --message-format=short 2>&1 || true)

ERROR_COUNT=$(echo "$CHECK_OUTPUT" | grep -c "error\[" || echo "0")
WARN_COUNT=$(echo "$CHECK_OUTPUT" | grep -c "warning\[" || echo "0")

if [ "$ERROR_COUNT" -eq 0 ]; then
    log_success "No compilation errors found"
else
    log_error "Found $ERROR_COUNT compilation errors"
    echo "$CHECK_OUTPUT" | grep "error\[" | head -5
fi

if [ "$WARN_COUNT" -gt 0 ]; then
    log_warning "Found $WARN_COUNT warnings"
fi

# 6. Check individual crates
check_step "Individual Crate Checks"
CRATES=$(ls -d crates/* 2>/dev/null | head -5)
if [ -n "$CRATES" ]; then
    for crate_dir in $CRATES; do
        crate_name=$(basename "$crate_dir")
        if $CARGO_BIN check -p "$crate_name" --message-format=short 2>&1 | grep -q "Finished"; then
            log_success "$crate_name compiles"
        else
            log_warning "$crate_name has issues"
        fi
    done
else
    log_info "No crates directory found"
fi

# 7. Check binary targets
check_step "Binary Targets"
BINARIES=$($CARGO_BIN metadata --format-version 1 2>/dev/null | grep -o '"name":"[^"]*".*"kind":\["bin"\]' | grep -o '"name":"[^"]*"' | cut -d'"' -f4 | head -5)

if [ -n "$BINARIES" ]; then
    for binary in $BINARIES; do
        if $CARGO_BIN check --bin "$binary" 2>&1 | grep -q "Finished"; then
            log_success "Binary '$binary' compiles"
        else
            log_warning "Binary '$binary' has issues"
        fi
    done
else
    log_warning "No binary targets found"
fi

# 8. Check for common issues
check_step "Common Issue Detection"

# Check for unwrap() usage
UNWRAP_COUNT=$(grep -r "\.unwrap()" --include="*.rs" 2>/dev/null | wc -l || echo "0")
if [ "$UNWRAP_COUNT" -gt 50 ]; then
    log_warning "High number of unwrap() calls found: $UNWRAP_COUNT"
    echo "    Consider using proper error handling with Result/Option"
else
    log_info "Unwrap usage: $UNWRAP_COUNT occurrences"
fi

# Check for TODO comments
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX" --include="*.rs" 2>/dev/null | wc -l || echo "0")
if [ "$TODO_COUNT" -gt 0 ]; then
    log_info "Found $TODO_COUNT TODO/FIXME comments"
fi

# 9. Check Clippy (if available)
check_step "Clippy Lints (Optional)"
if $CARGO_BIN clippy --version &> /dev/null; then
    log_info "Running clippy..."
    if $CARGO_BIN clippy --all-targets --quiet 2>&1 | grep -q "warning"; then
        log_warning "Clippy found some suggestions"
    else
        log_success "Clippy checks passed"
    fi
else
    log_info "Clippy not installed (install with: rustup component add clippy)"
fi

# 10. Check formatting
check_step "Code Formatting (Optional)"
if $CARGO_BIN fmt --version &> /dev/null; then
    if $CARGO_BIN fmt --check 2>&1 | grep -q "Diff"; then
        log_warning "Code needs formatting (run: cargo fmt)"
    else
        log_success "Code is properly formatted"
    fi
else
    log_info "rustfmt not installed (install with: rustup component add rustfmt)"
fi

# Summary
echo ""
log_header "Compilation Check Summary"

echo -e "${BOLD}Results:${NC}"
echo -e "  ${GREEN}Passed:${NC} ${#PASSED_CHECKS[@]} checks"
echo -e "  ${RED}Failed:${NC} $FAILED_CHECKS checks"
echo -e "  ${YELLOW}Warnings:${NC} $WARNINGS"

if [ $FAILED_CHECKS -gt 0 ]; then
    echo ""
    echo -e "${RED}${BOLD}Failed Checks:${NC}"
    for check in "${FAILED_CHECK_LIST[@]}"; do
        echo -e "  ${RED}✗${NC} $check"
    done
fi

if [ ${#WARNING_LIST[@]} -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}${BOLD}Warnings:${NC}"
    for warning in "${WARNING_LIST[@]}"; do
        echo -e "  ${YELLOW}⚠${NC} $warning"
    done
fi

echo ""
if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}${BOLD}✅ Compilation checks passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run: ./verify_database.sh    # Check database setup"
    echo "  2. Run: ./run_tests.sh          # Run test suite"
    echo "  3. Run: $CARGO_BIN build --release  # Build release version"
    exit 0
else
    echo -e "${RED}${BOLD}❌ Compilation checks failed${NC}"
    echo ""
    echo "Fix the issues above and run this script again."
    echo "For detailed errors, run: $CARGO_BIN check --workspace"
    exit 1
fi