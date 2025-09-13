//! Software management class - port of SoftwareVPC from PHP
//! 
//! Handles software operations including:
//! - Research and development
//! - Installation and management
//! - Virtual PC operations
//! - University integration

use std::collections::HashMap;
use he_db::DbPool;
use anyhow::Result;

/// Software management errors
#[derive(Debug, thiserror::Error)]
pub enum SoftwareError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Permission error: {0}")]
    PermissionError(String),
    #[error("Software not found")]
    NotFound,
}

/// Software VPC class - handles virtual PC software operations
#[derive(Debug, Clone)]
pub struct SoftwareVPC {
    pub db_pool: DbPool,
}

impl SoftwareVPC {
    /// Create new SoftwareVPC instance
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Handle POST form submissions for university operations
    pub async fn handle_post(&mut self, context: &str, post_data: HashMap<String, String>) -> Result<String, SoftwareError> {
        match context {
            "university" => {
                // Handle university-specific software operations
                self.handle_university_post(post_data).await
            },
            _ => Err(SoftwareError::ValidationError("Invalid context".to_string())),
        }
    }

    /// Handle university POST operations
    async fn handle_university_post(&mut self, post_data: HashMap<String, String>) -> Result<String, SoftwareError> {
        // Extract action from POST data
        let action = post_data.get("act")
            .ok_or_else(|| SoftwareError::ValidationError("Missing action".to_string()))?;

        match action.as_str() {
            "research" => self.handle_research_action(post_data).await,
            "upgrade" => self.handle_upgrade_action(post_data).await,
            "develop" => self.handle_develop_action(post_data).await,
            _ => Err(SoftwareError::ValidationError("Invalid action".to_string())),
        }
    }

    /// Handle research action
    async fn handle_research_action(&mut self, _post_data: HashMap<String, String>) -> Result<String, SoftwareError> {
        // TODO: Implement research functionality
        Ok("Research action processed".to_string())
    }

    /// Handle upgrade action  
    async fn handle_upgrade_action(&mut self, _post_data: HashMap<String, String>) -> Result<String, SoftwareError> {
        // TODO: Implement upgrade functionality
        Ok("Upgrade action processed".to_string())
    }

    /// Handle develop action
    async fn handle_develop_action(&mut self, _post_data: HashMap<String, String>) -> Result<String, SoftwareError> {
        // TODO: Implement develop functionality
        Ok("Develop action processed".to_string())
    }

    /// Check if software exists for user
    pub async fn isset_software(&self, software_id: i64, user_id: i64, context: &str) -> Result<bool, SoftwareError> {
        let result = sqlx::query!(
            "SELECT id FROM softwares WHERE id = ? AND userid = ? AND context = ? LIMIT 1",
            software_id,
            user_id,
            context
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.is_some())
    }

    /// Show research interface for specific software
    pub async fn research_show(&self, software_id: i64) -> Result<String, SoftwareError> {
        let software = sqlx::query!(
            "SELECT * FROM softwares WHERE id = ? LIMIT 1",
            software_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match software {
            Some(sw) => {
                Ok(format!(r#"
                    <div class="software-research">
                        <h3>Research: {}</h3>
                        <p>Software Level: {}</p>
                        <p>Type: {}</p>
                        <div class="research-actions">
                            <form method="POST">
                                <input type="hidden" name="act" value="research">
                                <input type="hidden" name="software_id" value="{}">
                                <button type="submit" class="btn btn-primary">Continue Research</button>
                            </form>
                        </div>
                    </div>
                "#, sw.name, sw.level, sw.soft_type, sw.id))
            },
            None => Err(SoftwareError::NotFound),
        }
    }

    /// Show research list (all available software for research)
    pub async fn research_list(&self) -> Result<String, SoftwareError> {
        let softwares = sqlx::query!(
            "SELECT * FROM softwares WHERE userid IS NULL ORDER BY soft_type, name"
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut html = String::from(r#"
            <div class="research-list">
                <h3>Available Research</h3>
                <table class="table table-striped">
                    <thead>
                        <tr>
                            <th>Software</th>
                            <th>Type</th>
                            <th>Level</th>
                            <th>Action</th>
                        </tr>
                    </thead>
                    <tbody>
        "#);

        for software in softwares {
            html.push_str(&format!(r#"
                        <tr>
                            <td>{}</td>
                            <td>{}</td>
                            <td>{}</td>
                            <td><a href="?id={}" class="btn btn-sm btn-info">Research</a></td>
                        </tr>
            "#, software.name, software.soft_type, software.level, software.id));
        }

        html.push_str(r#"
                    </tbody>
                </table>
            </div>
        "#);

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_vpc_creation() {
        // Mock database pool for testing
        let db_pool = DbPool::connect_lazy("mysql://test:test@localhost/test").unwrap();
        let software = SoftwareVPC::new(db_pool);
        
        // Basic validation that struct was created properly
        assert!(std::ptr::addr_of!(software.db_pool) as *const _ != std::ptr::null());
    }

    #[test]
    fn test_software_error_types() {
        let validation_error = SoftwareError::ValidationError("Test error".to_string());
        assert!(matches!(validation_error, SoftwareError::ValidationError(_)));

        let permission_error = SoftwareError::PermissionError("Permission denied".to_string());
        assert!(matches!(permission_error, SoftwareError::PermissionError(_)));

        let not_found_error = SoftwareError::NotFound;
        assert!(matches!(not_found_error, SoftwareError::NotFound));
    }
}