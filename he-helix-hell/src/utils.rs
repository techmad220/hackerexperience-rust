//! Utility functions

use crate::{HellError, HellResult};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Capitalize an atom/string
pub fn capitalize_atom(atom: impl AsRef<str>) -> String {
    let s = atom.as_ref();
    if s.is_empty() {
        return String::new();
    }
    
    let mut chars: Vec<char> = s.chars().collect();
    chars[0] = chars[0].to_ascii_uppercase();
    chars.into_iter().collect()
}

/// Convert string to safe module name
pub fn to_safe_module_name(name: impl AsRef<str>) -> String {
    name.as_ref()
        .replace("_", "")
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_ascii_uppercase()
            } else {
                c
            }
        })
        .collect()
}

/// Parse a string as a specific type
pub fn parse_string<T>(s: impl AsRef<str>) -> HellResult<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    s.as_ref().parse().map_err(|e| HellError::ParseError {
        message: format!("Failed to parse '{}': {}", s.as_ref(), e),
    })
}

/// Validation utilities
pub mod validation {
    use super::*;

    /// Validate that a string is not empty
    pub fn validate_non_empty(field: &str, value: &str) -> HellResult<()> {
        if value.trim().is_empty() {
            Err(HellError::ValidationError {
                field: field.to_string(),
                reason: "cannot be empty".to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Validate string length
    pub fn validate_length(field: &str, value: &str, min: usize, max: usize) -> HellResult<()> {
        let len = value.len();
        if len < min || len > max {
            Err(HellError::ValidationError {
                field: field.to_string(),
                reason: format!("length must be between {} and {}, got {}", min, max, len),
            })
        } else {
            Ok(())
        }
    }
}