//! Utility macros

/// Macro for creating compile-time checked strings
#[macro_export]
macro_rules! static_string {
    ($s:expr) => {
        $s
    };
}