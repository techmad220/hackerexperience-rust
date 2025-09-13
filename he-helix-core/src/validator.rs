//! Core validation utilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },
    #[error("Invalid format for field {field}: {reason}")]
    InvalidFormat { field: String, reason: String },
    #[error("Value out of range for field {field}: expected {min} to {max}, got {value}")]
    OutOfRange { field: String, min: i64, max: i64, value: i64 },
}

pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validation context for complex validations
#[derive(Debug, Default)]
pub struct ValidationContext {
    pub fields: HashMap<String, serde_json::Value>,
    pub errors: Vec<ValidationError>,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn validate_required(&mut self, field: &str, value: Option<&serde_json::Value>) {
        if value.is_none() {
            self.add_error(ValidationError::RequiredFieldMissing {
                field: field.to_string(),
            });
        }
    }
}