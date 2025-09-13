use axum::{
    extract::Query,
    http::StatusCode,
    response::Html,
    Extension,
};
use serde::Deserialize;
use he_core::session::PhpSession;
use sqlx::PgPool;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Database connection configuration - 1:1 port of connect.php variables
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub dsn: String,
    pub db_user: String,
    pub db_pass: String,
    pub host: String,
    pub port: u16,
    pub database: String,
}

/// Database connection options - port of $dbOptions array
#[derive(Debug, Clone)]
pub struct DatabaseOptions {
    pub persistent: bool,
    pub case_lower: bool,
}

/// Global PDO connection counter - equivalent to $_SESSION['PDO']
static PDO_CONNECTION_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Query parameters for connect page
#[derive(Debug, Deserialize)]
pub struct ConnectQuery {
    pub test: Option<String>,
    pub reset: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            dsn: "mysql:host=localhost;port=3306;dbname=game".to_string(),
            db_user: "he".to_string(),
            db_pass: "REDACTED".to_string(), // Original had actual password
            host: "localhost".to_string(),
            port: 3306,
            database: "game".to_string(),
        }
    }
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self {
            persistent: true,    // PDO::ATTR_PERSISTENT => true
            case_lower: true,    // PDO::ATTR_CASE => PDO::CASE_LOWER
        }
    }
}

/// Connect page handler - 1:1 port of connect.php
/// 
/// Original: Database connection configuration and session tracking
/// Features:
/// - Defines database connection parameters (DSN, user, password, options)
/// - Tracks PDO connection attempts in session ($_SESSION['PDO'])
/// - Contains commented-out PDO connection code
/// - Provides error handling with Portuguese error message
/// - Connection counter incrementation before each attempt
/// 
/// Note: Original had actual database connection logic commented out.
/// This port maintains the same structure and session tracking behavior.
pub async fn connect_handler(
    Extension(db): Extension<PgPool>,
    Extension(session): Extension<PhpSession>,
    Query(params): Query<ConnectQuery>,
) -> Result<Html<String>, StatusCode> {
    
    // Get or initialize PDO connection counter - equivalent to $_SESSION['PDO']
    let mut pdo_count = PDO_CONNECTION_COUNTER.load(Ordering::SeqCst);
    
    // Increment PDO counter (equivalent to $_SESSION['PDO']++)
    pdo_count += 1;
    PDO_CONNECTION_COUNTER.store(pdo_count, Ordering::SeqCst);
    
    // Database configuration - equivalent to original PHP variables
    let db_config = DatabaseConfig::default();
    let db_options = DatabaseOptions::default();
    
    let mut content = String::new();
    let mut error_occurred = false;
    
    // Handle test connection parameter
    if params.test.is_some() {
        content.push_str("<h3>Testing Database Connection</h3>\n");
        
        // Simulate connection attempt (original had this commented out)
        // try {
        //     $pdo = new PDO($dsn, $dbUser, $dbPass, $dbOptions);
        // } catch (PDOException $e) {
        //     die('Erro ao conectar ao banco de dados');
        // }
        
        // In the original, this was commented out, so we simulate the same
        content.push_str("<p><em>Connection attempt simulation (original code was commented out)</em></p>\n");
        content.push_str("<pre>\n");
        content.push_str("// Original commented PHP code:\n");
        content.push_str("// try {\n");
        content.push_str("//     $pdo = new PDO($dsn, $dbUser, $dbPass, $dbOptions);\n");
        content.push_str("// } catch (PDOException $e) {\n");
        content.push_str("//     die('Erro ao conectar ao banco de dados');\n");
        content.push_str("// }\n");
        content.push_str("</pre>\n");
        
        // Show what would happen if connection failed
        if params.reset.is_some() {
            error_occurred = true;
        }
    }
    
    // Display connection configuration and status
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Database Connection - Hacker Experience</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }}
        .config-box {{
            background: white;
            border: 1px solid #ddd;
            padding: 20px;
            margin: 10px 0;
            border-radius: 5px;
        }}
        .error {{
            background-color: #ffe6e6;
            border-color: #ff9999;
            color: #cc0000;
        }}
        pre {{
            background: #f8f8f8;
            padding: 10px;
            border-left: 4px solid #007acc;
            overflow-x: auto;
        }}
        .counter {{
            background: #e6f3ff;
            border-color: #99ccff;
        }}
    </style>
</head>
<body>
    <h2>Database Connection Configuration</h2>
    
    <div class="config-box counter">
        <h3>Connection Counter</h3>
        <p><strong>PDO Connection Attempts:</strong> {}</p>
        <p><em>Equivalent to $_SESSION['PDO'] in original PHP</em></p>
    </div>
    
    <div class="config-box">
        <h3>Database Configuration</h3>
        <table border="1" cellpadding="5" cellspacing="0">
            <tr><th>Parameter</th><th>Value</th></tr>
            <tr><td>DSN</td><td>{}</td></tr>
            <tr><td>Database User</td><td>{}</td></tr>
            <tr><td>Database Password</td><td>{}</td></tr>
            <tr><td>Host</td><td>{}</td></tr>
            <tr><td>Port</td><td>{}</td></tr>
            <tr><td>Database</td><td>{}</td></tr>
        </table>
    </div>
    
    <div class="config-box">
        <h3>Database Options</h3>
        <table border="1" cellpadding="5" cellspacing="0">
            <tr><th>Option</th><th>Value</th><th>PHP Constant</th></tr>
            <tr><td>Persistent Connection</td><td>{}</td><td>PDO::ATTR_PERSISTENT</td></tr>
            <tr><td>Case Conversion</td><td>{}</td><td>PDO::ATTR_CASE => PDO::CASE_LOWER</td></tr>
        </table>
    </div>
    
    {}
    
    {error_content}
    
    <div class="config-box">
        <h3>Original PHP Code Structure</h3>
        <pre>
&lt;?php

$dsn = 'mysql:host=localhost;port=3306;dbname=game';
$dbUser = 'he';
$dbPass = 'REDACTED';
$dbOptions = array(
    PDO::ATTR_PERSISTENT =&gt; true,
    PDO::ATTR_CASE =&gt; PDO::CASE_LOWER
);

        if(!isset($_SESSION['PDO'])){{
            $_SESSION['PDO'] = 0;
        }}
        
        $_SESSION['PDO']++;

try {{
    //$pdo = new PDO($dsn, $dbUser, $dbPass, $dbOptions);
    
}} catch (PDOException $e) {{
    die('Erro ao conectar ao banco de dados');
}}

?&gt;
        </pre>
    </div>
    
    <div class="config-box">
        <p><a href="?test=1">Test Connection</a> | <a href="?test=1&reset=1">Simulate Connection Error</a></p>
    </div>
    
</body>
</html>"#,
        pdo_count,
        db_config.dsn,
        db_config.db_user,
        db_config.db_pass,
        db_config.host,
        db_config.port,
        db_config.database,
        db_options.persistent,
        if db_options.case_lower { "CASE_LOWER" } else { "CASE_NATURAL" },
        content,
        error_content = if error_occurred {
            r#"<div class="config-box error">
                <h3>Connection Error Simulation</h3>
                <p><strong>Error:</strong> Erro ao conectar ao banco de dados</p>
                <p><em>This is the exact error message from the original PHP code</em></p>
            </div>"#
        } else {
            ""
        }
    );
    
    Ok(Html(html))
}

/// Get current PDO connection count
pub fn get_pdo_connection_count() -> usize {
    PDO_CONNECTION_COUNTER.load(Ordering::SeqCst)
}

/// Reset PDO connection count (for testing)
pub fn reset_pdo_connection_count() {
    PDO_CONNECTION_COUNTER.store(0, Ordering::SeqCst);
}

/// Increment PDO connection count - equivalent to $_SESSION['PDO']++
pub fn increment_pdo_connection_count() -> usize {
    PDO_CONNECTION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}

/// Get database configuration for other modules
pub fn get_database_config() -> DatabaseConfig {
    DatabaseConfig::default()
}

/// Get database options for other modules  
pub fn get_database_options() -> DatabaseOptions {
    DatabaseOptions::default()
}

/// Create DSN string from configuration
pub fn create_dsn(config: &DatabaseConfig) -> String {
    format!("mysql:host={};port={};dbname={}", config.host, config.port, config.database)
}

/// Simulate the original PHP connection error
pub fn simulate_connection_error() -> &'static str {
    "Erro ao conectar ao banco de dados" // Exact Portuguese error message from original
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.database, "game");
        assert_eq!(config.db_user, "he");
        assert_eq!(config.db_pass, "REDACTED");
        assert_eq!(config.dsn, "mysql:host=localhost;port=3306;dbname=game");
    }
    
    #[test]
    fn test_database_options_defaults() {
        let options = DatabaseOptions::default();
        assert!(options.persistent);  // PDO::ATTR_PERSISTENT => true
        assert!(options.case_lower);  // PDO::ATTR_CASE => PDO::CASE_LOWER
    }
    
    #[test]
    fn test_pdo_connection_counter() {
        // Reset counter for test
        reset_pdo_connection_count();
        assert_eq!(get_pdo_connection_count(), 0);
        
        // Test increment
        let count1 = increment_pdo_connection_count();
        assert_eq!(count1, 1);
        assert_eq!(get_pdo_connection_count(), 1);
        
        let count2 = increment_pdo_connection_count();
        assert_eq!(count2, 2);
        assert_eq!(get_pdo_connection_count(), 2);
        
        // Reset for other tests
        reset_pdo_connection_count();
    }
    
    #[test]
    fn test_dsn_creation() {
        let config = DatabaseConfig {
            host: "testhost".to_string(),
            port: 3307,
            database: "testdb".to_string(),
            ..DatabaseConfig::default()
        };
        
        let dsn = create_dsn(&config);
        assert_eq!(dsn, "mysql:host=testhost;port=3307;dbname=testdb");
    }
    
    #[test]
    fn test_connection_error_message() {
        let error = simulate_connection_error();
        assert_eq!(error, "Erro ao conectar ao banco de dados");
    }
    
    #[test]
    fn test_query_parameter_parsing() {
        // Test query parameters
        let query = ConnectQuery {
            test: Some("1".to_string()),
            reset: None,
        };
        assert_eq!(query.test.as_deref(), Some("1"));
        assert!(query.reset.is_none());
        
        let query_with_reset = ConnectQuery {
            test: Some("1".to_string()),
            reset: Some("1".to_string()),
        };
        assert!(query_with_reset.test.is_some());
        assert!(query_with_reset.reset.is_some());
    }
    
    #[test]
    fn test_original_config_values() {
        // Verify configuration matches original PHP exactly
        let config = get_database_config();
        let options = get_database_options();
        
        // Original PHP values
        assert_eq!(config.dsn, "mysql:host=localhost;port=3306;dbname=game");
        assert_eq!(config.db_user, "he");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.database, "game");
        
        // Original PHP options
        assert!(options.persistent);   // PDO::ATTR_PERSISTENT => true
        assert!(options.case_lower);   // PDO::ATTR_CASE => PDO::CASE_LOWER
    }
}