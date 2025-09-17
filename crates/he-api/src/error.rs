//! Comprehensive error handling with detailed debugging information

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::error;

/// API Result type alias
pub type ApiResult<T> = Result<T, ApiError>;

/// Comprehensive API error with debugging context
#[derive(Debug)]
pub struct ApiError {
    /// Error kind/category
    pub kind: ErrorKind,
    /// Human-readable message
    pub message: String,
    /// Technical details for debugging
    pub details: Option<String>,
    /// Error context (file, line, function)
    pub context: ErrorContext,
    /// Original error chain
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// Error categories for better organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    // Authentication & Authorization
    Unauthorized,
    Forbidden,
    InvalidToken,
    SessionExpired,

    // Validation
    ValidationError,
    InvalidInput,
    MissingField,

    // Database
    DatabaseError,
    QueryFailed,
    ConnectionLost,
    TransactionFailed,

    // Game Logic
    InsufficientResources,
    InvalidGameState,
    ProcessAlreadyRunning,
    TargetNotFound,
    ActionNotAllowed,

    // Rate Limiting
    RateLimitExceeded,
    TooManyRequests,

    // Server
    InternalError,
    ServiceUnavailable,
    Timeout,

    // Client
    BadRequest,
    NotFound,
    Conflict,
}

/// Error context for debugging
#[derive(Debug, Clone, Serialize)]
pub struct ErrorContext {
    pub file: &'static str,
    pub line: u32,
    pub function: &'static str,
    pub request_id: Option<String>,
    pub user_id: Option<i64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ApiError {
    /// Create a new API error with context
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            details: None,
            context: ErrorContext::capture(),
            source: None,
        }
    }

    /// Add technical details for debugging
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add source error
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Add request context
    pub fn with_request(mut self, request_id: String, user_id: Option<i64>) -> Self {
        self.context.request_id = Some(request_id);
        self.context.user_id = user_id;
        self
    }

    /// Get HTTP status code based on error kind
    fn status_code(&self) -> StatusCode {
        match self.kind {
            ErrorKind::Unauthorized | ErrorKind::InvalidToken => StatusCode::UNAUTHORIZED,
            ErrorKind::Forbidden | ErrorKind::SessionExpired => StatusCode::FORBIDDEN,
            ErrorKind::ValidationError | ErrorKind::InvalidInput | ErrorKind::MissingField => {
                StatusCode::BAD_REQUEST
            }
            ErrorKind::NotFound | ErrorKind::TargetNotFound => StatusCode::NOT_FOUND,
            ErrorKind::Conflict | ErrorKind::ProcessAlreadyRunning => StatusCode::CONFLICT,
            ErrorKind::RateLimitExceeded | ErrorKind::TooManyRequests => {
                StatusCode::TOO_MANY_REQUESTS
            }
            ErrorKind::ServiceUnavailable | ErrorKind::ConnectionLost => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            ErrorKind::Timeout => StatusCode::REQUEST_TIMEOUT,
            ErrorKind::InsufficientResources | ErrorKind::InvalidGameState |
            ErrorKind::ActionNotAllowed => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Create detailed error response
    fn error_response(&self) -> ErrorResponse {
        // Log the full error with context
        error!(
            kind = ?self.kind,
            message = %self.message,
            details = ?self.details,
            context = ?self.context,
            source = ?self.source,
            "API Error occurred"
        );

        // Prepare response based on environment
        let is_debug = cfg!(debug_assertions);

        ErrorResponse {
            error: true,
            code: self.error_code(),
            message: self.message.clone(),
            details: if is_debug { self.details.clone() } else { None },
            kind: Some(format!("{:?}", self.kind)),
            context: if is_debug {
                Some(DebugContext {
                    file: self.context.file,
                    line: self.context.line,
                    function: self.context.function,
                    request_id: self.context.request_id.clone(),
                    timestamp: self.context.timestamp,
                })
            } else {
                None
            },
            trace: if is_debug {
                self.source.as_ref().map(|s| s.to_string())
            } else {
                None
            },
        }
    }

    /// Generate error code for client
    fn error_code(&self) -> String {
        match self.kind {
            ErrorKind::Unauthorized => "AUTH_001",
            ErrorKind::InvalidToken => "AUTH_002",
            ErrorKind::SessionExpired => "AUTH_003",
            ErrorKind::Forbidden => "AUTH_004",
            ErrorKind::ValidationError => "VAL_001",
            ErrorKind::InvalidInput => "VAL_002",
            ErrorKind::MissingField => "VAL_003",
            ErrorKind::DatabaseError => "DB_001",
            ErrorKind::QueryFailed => "DB_002",
            ErrorKind::ConnectionLost => "DB_003",
            ErrorKind::TransactionFailed => "DB_004",
            ErrorKind::InsufficientResources => "GAME_001",
            ErrorKind::InvalidGameState => "GAME_002",
            ErrorKind::ProcessAlreadyRunning => "GAME_003",
            ErrorKind::TargetNotFound => "GAME_004",
            ErrorKind::ActionNotAllowed => "GAME_005",
            ErrorKind::RateLimitExceeded => "RATE_001",
            ErrorKind::TooManyRequests => "RATE_002",
            ErrorKind::InternalError => "SRV_001",
            ErrorKind::ServiceUnavailable => "SRV_002",
            ErrorKind::Timeout => "SRV_003",
            ErrorKind::BadRequest => "CLI_001",
            ErrorKind::NotFound => "CLI_002",
            ErrorKind::Conflict => "CLI_003",
        }
        .to_string()
    }
}

impl ErrorContext {
    /// Capture current context
    fn capture() -> Self {
        Self {
            file: file!(),
            line: line!(),
            function: "unknown", // Would need macro to capture
            request_id: None,
            user_id: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.message)?;
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.error_response())
    }

    fn status_code(&self) -> StatusCode {
        self.status_code()
    }
}

/// Error response structure
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: bool,
    code: String,
    message: String,
    details: Option<String>,
    kind: Option<String>,
    context: Option<DebugContext>,
    trace: Option<String>,
}

/// Debug context in response
#[derive(Debug, Serialize)]
struct DebugContext {
    file: &'static str,
    line: u32,
    function: &'static str,
    request_id: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Conversion helpers for common errors
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        let (kind, message) = match &err {
            sqlx::Error::RowNotFound => (ErrorKind::NotFound, "Resource not found"),
            sqlx::Error::Database(_) => (ErrorKind::DatabaseError, "Database operation failed"),
            sqlx::Error::PoolTimedOut => (ErrorKind::Timeout, "Database connection timeout"),
            sqlx::Error::PoolClosed => (ErrorKind::ConnectionLost, "Database connection lost"),
            _ => (ErrorKind::InternalError, "Database error occurred"),
        };

        ApiError::new(kind, message)
            .with_details(format!("SQLx error: {}", err))
            .with_source(err)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::new(ErrorKind::InternalError, "Internal error occurred")
            .with_details(format!("{:?}", err))
    }
}

/// Macro for creating errors with context
#[macro_export]
macro_rules! api_error {
    ($kind:expr, $msg:expr) => {
        $crate::error::ApiError::new($kind, $msg)
    };
    ($kind:expr, $msg:expr, $details:expr) => {
        $crate::error::ApiError::new($kind, $msg).with_details($details)
    };
}

/// Macro for validation errors
#[macro_export]
macro_rules! validation_error {
    ($field:expr, $issue:expr) => {
        $crate::error::ApiError::new(
            $crate::error::ErrorKind::ValidationError,
            format!("Validation failed for field '{}'", $field)
        )
        .with_details($issue)
    };
}

/// Macro for game logic errors
#[macro_export]
macro_rules! game_error {
    ($msg:expr) => {
        $crate::error::ApiError::new(
            $crate::error::ErrorKind::InvalidGameState,
            $msg
        )
    };
    ($msg:expr, $details:expr) => {
        $crate::error::ApiError::new(
            $crate::error::ErrorKind::InvalidGameState,
            $msg
        )
        .with_details($details)
    };
}