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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let auth_error = HeError::AuthenticationFailed;
        assert_eq!(auth_error.to_string(), "Authentication failed");

        let user_error = HeError::UserNotFound;
        assert_eq!(user_error.to_string(), "User not found");

        let process_error = HeError::InvalidProcess("Invalid process ID".to_string());
        assert_eq!(process_error.to_string(), "Invalid process: Invalid process ID");

        let resource_error = HeError::InsufficientResources("Not enough CPU".to_string());
        assert_eq!(resource_error.to_string(), "Insufficient resources: Not enough CPU");

        let permission_error = HeError::PermissionDenied;
        assert_eq!(permission_error.to_string(), "Permission denied");

        let input_error = HeError::InvalidInput("Invalid IP address".to_string());
        assert_eq!(input_error.to_string(), "Invalid input: Invalid IP address");

        let game_error = HeError::GameLogic("Cannot hack yourself".to_string());
        assert_eq!(game_error.to_string(), "Game logic error: Cannot hack yourself");
    }

    #[test]
    fn test_error_result_type() {
        fn returns_result() -> HeResult<String> {
            Ok("Success".to_string())
        }

        let result = returns_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");

        fn returns_error() -> HeResult<String> {
            Err(HeError::UserNotFound)
        }

        let error_result = returns_error();
        assert!(error_result.is_err());
    }

    #[test]
    fn test_error_conversion() {
        use anyhow::anyhow;

        // Test that anyhow errors can be converted to HeError
        let anyhow_error = anyhow!("Database connection failed");
        let he_error = HeError::Database(anyhow_error);
        assert!(he_error.to_string().contains("Database error"));
    }
}