use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdoError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Query error: {0}")]
    Query(String),
    #[error("Parameter binding error: {0}")]
    Binding(String),
    #[error("Statement preparation error: {0}")]
    Preparation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchMode {
    Assoc,
    Numeric,
    Both,
    Object,
    Lazy,
    Column,
}

/// Represents a prepared statement
pub struct PdoStatement {
    sql: String,
    parameters: HashMap<String, String>,
    executed: bool,
    result_set: Vec<HashMap<String, String>>,
}

impl PdoStatement {
    pub fn new(sql: String) -> Self {
        Self {
            sql,
            parameters: HashMap::new(),
            executed: false,
            result_set: vec![],
        }
    }

    /// Bind a parameter to the statement
    pub fn bind_param(&mut self, parameter: &str, value: &str) -> Result<(), PdoError> {
        self.parameters.insert(parameter.to_string(), value.to_string());
        Ok(())
    }

    /// Bind multiple parameters
    pub fn bind_params(&mut self, params: HashMap<String, String>) -> Result<(), PdoError> {
        for (key, value) in params {
            self.parameters.insert(key, value);
        }
        Ok(())
    }

    /// Execute the prepared statement
    pub fn execute(&mut self) -> Result<bool, PdoError> {
        // Simulate statement execution
        self.executed = true;
        
        // In actual implementation, this would execute the real query
        // and populate result_set with actual data
        self.result_set = vec![];
        
        Ok(true)
    }

    /// Execute with parameters
    pub fn execute_with_params(&mut self, params: HashMap<String, String>) -> Result<bool, PdoError> {
        self.bind_params(params)?;
        self.execute()
    }

    /// Fetch a single row
    pub fn fetch(&mut self, mode: FetchMode) -> Result<Option<HashMap<String, String>>, PdoError> {
        if !self.executed {
            return Err(PdoError::Query("Statement not executed".to_string()));
        }

        // Simulate fetching data
        if self.result_set.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.result_set.remove(0)))
        }
    }

    /// Fetch all rows
    pub fn fetch_all(&mut self, mode: FetchMode) -> Result<Vec<HashMap<String, String>>, PdoError> {
        if !self.executed {
            return Err(PdoError::Query("Statement not executed".to_string()));
        }

        Ok(self.result_set.clone())
    }

    /// Get row count
    pub fn row_count(&self) -> usize {
        self.result_set.len()
    }

    /// Get column count
    pub fn column_count(&self) -> usize {
        if let Some(first_row) = self.result_set.first() {
            first_row.len()
        } else {
            0
        }
    }
}

/// Legacy PDO wrapper ported from PHP PDO_DB class
/// Provides database connection and prepared statement functionality
pub struct Pdo {
    dsn: String,
    username: String,
    password: String,
    options: HashMap<String, String>,
    connected: bool,
    in_transaction: bool,
    last_insert_id: u64,
}

impl Pdo {
    /// Create new PDO instance
    pub fn new(dsn: &str, username: &str, password: &str) -> Result<Self, PdoError> {
        let mut pdo = Self {
            dsn: dsn.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            options: HashMap::new(),
            connected: false,
            in_transaction: false,
            last_insert_id: 0,
        };

        pdo.connect()?;
        Ok(pdo)
    }

    /// Factory method to create PDO instance (mirrors PHP factory pattern)
    pub fn factory() -> Result<Self, PdoError> {
        // Default configuration - in production, load from config
        let dsn = "mysql:host=localhost;dbname=hackerexperience;charset=utf8mb4";
        let username = "root";
        let password = "";
        
        Self::new(dsn, username, password)
    }

    /// Connect to the database
    fn connect(&mut self) -> Result<(), PdoError> {
        // Simulate connection logic
        self.connected = true;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Prepare a statement
    pub fn prepare(&self, sql: &str) -> Result<PdoStatement, PdoError> {
        if !self.connected {
            return Err(PdoError::Connection("Not connected to database".to_string()));
        }

        Ok(PdoStatement::new(sql.to_string()))
    }

    /// Execute a query directly
    pub fn query(&self, sql: &str) -> Result<PdoStatement, PdoError> {
        if !self.connected {
            return Err(PdoError::Connection("Not connected to database".to_string()));
        }

        let mut stmt = PdoStatement::new(sql.to_string());
        stmt.execute()?;
        Ok(stmt)
    }

    /// Execute a statement and return affected rows
    pub fn exec(&mut self, sql: &str) -> Result<u64, PdoError> {
        if !self.connected {
            return Err(PdoError::Connection("Not connected to database".to_string()));
        }

        // Simulate execution and return affected rows
        Ok(0)
    }

    /// Get last insert ID
    pub fn last_insert_id(&self) -> u64 {
        self.last_insert_id
    }

    /// Begin transaction
    pub fn begin_transaction(&mut self) -> Result<bool, PdoError> {
        if !self.connected {
            return Err(PdoError::Connection("Not connected to database".to_string()));
        }

        if self.in_transaction {
            return Err(PdoError::Query("Already in transaction".to_string()));
        }

        self.in_transaction = true;
        Ok(true)
    }

    /// Commit transaction
    pub fn commit(&mut self) -> Result<bool, PdoError> {
        if !self.connected {
            return Err(PdoError::Connection("Not connected to database".to_string()));
        }

        if !self.in_transaction {
            return Err(PdoError::Query("No active transaction".to_string()));
        }

        self.in_transaction = false;
        Ok(true)
    }

    /// Rollback transaction
    pub fn rollback(&mut self) -> Result<bool, PdoError> {
        if !self.connected {
            return Err(PdoError::Connection("Not connected to database".to_string()));
        }

        if !self.in_transaction {
            return Err(PdoError::Query("No active transaction".to_string()));
        }

        self.in_transaction = false;
        Ok(true)
    }

    /// Check if in transaction
    pub fn in_transaction(&self) -> bool {
        self.in_transaction
    }

    /// Quote a string for use in SQL queries
    pub fn quote(&self, string: &str) -> String {
        format!("'{}'", string.replace("'", "''"))
    }

    /// Set a PDO attribute
    pub fn set_attribute(&mut self, attribute: &str, value: &str) {
        self.options.insert(attribute.to_string(), value.to_string());
    }

    /// Get a PDO attribute
    pub fn get_attribute(&self, attribute: &str) -> Option<&String> {
        self.options.get(attribute)
    }

    /// Get available drivers (simulated)
    pub fn available_drivers() -> Vec<String> {
        vec![
            "mysql".to_string(),
            "pgsql".to_string(),
            "sqlite".to_string(),
        ]
    }
}

impl Drop for Pdo {
    fn drop(&mut self) {
        if self.in_transaction {
            let _ = self.rollback();
        }
        self.connected = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdo_creation() {
        let pdo = Pdo::factory();
        assert!(pdo.is_ok());
        let pdo = pdo.map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert!(pdo.is_connected());
    }

    #[test]
    fn test_statement_preparation() {
        let pdo = Pdo::factory().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let stmt = pdo.prepare("SELECT * FROM users WHERE id = :id");
        assert!(stmt.is_ok());
    }

    #[test]
    fn test_parameter_binding() {
        let pdo = Pdo::factory().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let mut stmt = pdo.prepare("SELECT * FROM users WHERE id = :id").map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert!(stmt.bind_param(":id", "123").is_ok());
    }

    #[test]
    fn test_transaction_handling() {
        let mut pdo = Pdo::factory().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        
        assert!(!pdo.in_transaction());
        assert!(pdo.begin_transaction().is_ok());
        assert!(pdo.in_transaction());
        assert!(pdo.commit().is_ok());
        assert!(!pdo.in_transaction());
    }

    #[test]
    fn test_quote_string() {
        let pdo = Pdo::factory().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let quoted = pdo.quote("test'string");
        assert_eq!(quoted, "'test''string'");
    }
}