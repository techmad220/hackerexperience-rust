use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeError {
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Invalid process: {0}")]
    InvalidProcess(String),
    
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Game logic error: {0}")]
    GameLogic(String),
}

pub type HeResult<T> = Result<T, HeError>;