//! HELL - Hacker Experience Low Level utilities
//! 
//! This crate provides low-level utilities, error handling, and common functionality
//! used throughout the Helix system. Named "HELL" in the original Elixir codebase.

pub mod hell;
pub mod mix;
pub mod client_utils;
pub mod macros;
pub mod types;
pub mod utils;

pub use hell::*;
pub use types::*;
pub use utils::*;

use thiserror::Error;

/// HELL error types
#[derive(Debug, Error)]
pub enum HellError {
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    #[error("Validation error: {field} - {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Conversion error: {from} to {to} - {reason}")]
    ConversionError { from: String, to: String, reason: String },
    
    #[error("Network error: {error}")]
    NetworkError { error: String },
    
    #[error("Internal error: {error}")]
    InternalError { error: String },
}

pub type HellResult<T> = Result<T, HellError>;