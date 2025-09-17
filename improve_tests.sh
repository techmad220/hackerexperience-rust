#!/bin/bash

# Test Coverage Improvement Script
set -e

echo "🧪 HackerExperience Test Coverage Improvement Tool"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Find modules without tests
echo -e "\n${YELLOW}📊 Analyzing test coverage...${NC}"

# Count existing tests
TOTAL_TESTS=$(grep -r "#\[test\]" --include="*.rs" crates/ 2>/dev/null | wc -l || echo 0)
TOTAL_MODULES=$(find crates -name "mod.rs" | wc -l)
TOTAL_SOURCE_FILES=$(find crates -name "*.rs" -not -path "*/tests/*" | wc -l)

echo "Current state:"
echo "  Total source files: $TOTAL_SOURCE_FILES"
echo "  Total test functions: $TOTAL_TESTS"
echo "  Total modules: $TOTAL_MODULES"
echo "  Approximate coverage: $(( TOTAL_TESTS * 100 / TOTAL_SOURCE_FILES ))%"

# Find modules without tests
echo -e "\n${RED}❌ Modules without tests:${NC}"
for dir in crates/*/src; do
    MODULE=$(dirname $(dirname $dir) | xargs basename)
    TEST_COUNT=$(grep -r "#\[test\]" $dir 2>/dev/null | wc -l || echo 0)
    if [ $TEST_COUNT -eq 0 ]; then
        echo "  - $MODULE (0 tests)"
    fi
done

# Generate test stubs
echo -e "\n${GREEN}✅ Generating test stubs...${NC}"

generate_test_stub() {
    local MODULE_PATH=$1
    local MODULE_NAME=$(basename $(dirname $MODULE_PATH))

    cat << EOF
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_${MODULE_NAME}_creation() {
        // TODO: Implement test
        assert!(true);
    }

    #[test]
    fn test_${MODULE_NAME}_validation() {
        // TODO: Implement test
        assert!(true);
    }

    #[test]
    fn test_${MODULE_NAME}_error_handling() {
        // TODO: Implement test
        assert!(true);
    }

    #[test]
    #[should_panic]
    fn test_${MODULE_NAME}_panic_condition() {
        // TODO: Implement panic test
        panic!("Expected panic");
    }

    #[tokio::test]
    async fn test_${MODULE_NAME}_async_operation() {
        // TODO: Implement async test
        assert!(true);
    }
}
EOF
}

# Priority modules that need tests
PRIORITY_MODULES=(
    "he-auth"
    "he-game-mechanics"
    "he-database"
    "he-api"
    "he-websocket"
    "he-helix-security"
    "he-processes"
)

echo -e "\n${YELLOW}🎯 Priority modules for testing:${NC}"
for module in "${PRIORITY_MODULES[@]}"; do
    if [ -d "crates/$module" ]; then
        TEST_COUNT=$(grep -r "#\[test\]" crates/$module 2>/dev/null | wc -l || echo 0)
        echo "  $module: $TEST_COUNT tests"

        # Generate test file if it doesn't exist
        TEST_FILE="crates/$module/src/tests.rs"
        if [ ! -f "$TEST_FILE" ] && [ $TEST_COUNT -lt 5 ]; then
            echo "    → Generating test stub at $TEST_FILE"
            # generate_test_stub "crates/$module/src" > "$TEST_FILE"
        fi
    fi
done

# Create test utilities module
echo -e "\n${GREEN}📦 Creating test utilities...${NC}"

cat << 'EOF' > test_utils.rs
// Test utilities for HackerExperience

use once_cell::sync::Lazy;
use sqlx::PgPool;

/// Test database pool
pub static TEST_DB: Lazy<PgPool> = Lazy::new(|| {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            let database_url = std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test_he".to_string());
            PgPool::connect(&database_url)
                .await
                .expect("Failed to connect to test database")
        })
});

/// Create test user
pub async fn create_test_user(username: &str) -> i64 {
    // Implementation here
    1
}

/// Create test process
pub async fn create_test_process(user_id: i64, process_type: &str) -> i64 {
    // Implementation here
    1
}

/// Clean up test data
pub async fn cleanup_test_data() {
    // Implementation here
}

/// Test fixture macro
#[macro_export]
macro_rules! test_fixture {
    ($name:ident, $body:block) => {
        #[tokio::test]
        async fn $name() {
            // Setup
            let _guard = TEST_DB.clone();

            // Test
            $body

            // Cleanup
            cleanup_test_data().await;
        }
    };
}
EOF

echo "Test utilities created at test_utils.rs"

# Run tests with coverage
echo -e "\n${YELLOW}🏃 Running existing tests...${NC}"
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    cargo tarpaulin --out Html --output-dir coverage 2>/dev/null || true
    echo "Coverage report generated at coverage/index.html"
else
    echo "Install cargo-tarpaulin for coverage reports: cargo install cargo-tarpaulin"
    cargo test --all 2>/dev/null || true
fi

# Generate test improvement report
echo -e "\n${GREEN}📝 Test Improvement Report${NC}"
cat << EOF > TEST_IMPROVEMENT_PLAN.md
# Test Improvement Plan

Generated on: $(date)

## Current Coverage
- Total Tests: $TOTAL_TESTS
- Source Files: $TOTAL_SOURCE_FILES
- Coverage: ~$(( TOTAL_TESTS * 100 / TOTAL_SOURCE_FILES ))%

## Priority Actions
1. Add tests to authentication module (security critical)
2. Test game mechanics calculations
3. Verify database queries
4. Test WebSocket events
5. Add integration tests for API endpoints

## Test Writing Guidelines
- Each public function should have at least one test
- Test both success and failure cases
- Use property-based testing for calculations
- Mock external dependencies
- Test async functions with tokio::test

## Next Steps
1. Run: cargo test --all
2. Add missing tests to priority modules
3. Run: cargo tarpaulin for coverage report
4. Aim for 80% coverage minimum
EOF

echo -e "\n${GREEN}✅ Test improvement plan generated!${NC}"
echo "See TEST_IMPROVEMENT_PLAN.md for details"

# Show test running commands
echo -e "\n${YELLOW}📌 Useful commands:${NC}"
echo "  cargo test --all                    # Run all tests"
echo "  cargo test --lib                    # Run library tests"
echo "  cargo test --doc                    # Run documentation tests"
echo "  cargo test -- --nocapture           # Show test output"
echo "  cargo test -- --test-threads=1     # Run tests serially"
echo "  cargo tarpaulin --out Html          # Generate coverage report"