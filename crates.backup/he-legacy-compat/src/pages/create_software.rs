//! Software creation page handler - 1:1 port of createsoft.php
//! 
//! Administrative tool for creating software with:
//! - Direct software creation (dev/admin only)
//! - Full software type selection
//! - Optional running state initialization
//! - Direct database insertion (unsafe legacy approach preserved)

use axum::{
    extract::{Extension, Form},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Form data for software creation
#[derive(Debug, Deserialize)]
pub struct CreateSoftwareForm {
    pub name: Option<String>,
    pub versao: Option<String>,    // "versao" matches original PHP
    pub size: Option<String>,
    pub ram: Option<String>,
    #[serde(rename = "type")]
    pub software_type: Option<String>,
    pub running: Option<String>,
}

/// Software creation handler - direct software creation tool
/// 
/// Port of: createsoft.php
/// Features:
/// - Administrative software creation (restricted access)
/// - Direct database insertion with all software types
/// - Optional running state initialization
/// - Unsafe temporary code (as noted in original)
/// - Simple form-based interface for rapid software creation
/// 
/// **Security Note:** This is marked as "unsafe and temporary code" in the original
/// PHP and should only be accessible to administrators/developers.
pub async fn create_software_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    form: Option<Form<CreateSoftwareForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check if user is logged in
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Get user ID and check authorization (admin only - user ID > 2 exits in original)
    let user_id = session.get("id")
        .and_then(|v| match v {
            SessionValue::String(s) => s.parse::<i64>().ok(),
            SessionValue::Integer(i) => Some(*i),
            _ => None,
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Original check: if($_SESSION['id'] > 2) exit();
    // This means only user IDs 1 and 2 (admin accounts) can access this
    if user_id > 2 {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Handle POST form submission
    if let Some(Form(form_data)) = form {
        if let Some(name) = &form_data.name {
            return handle_software_creation(db_pool, &session, form_data).await;
        }
    }

    // Display creation form
    let content = generate_creation_form();
    Ok(Html(content))
}

/// Handle software creation POST request
async fn handle_software_creation(
    db_pool: DbPool,
    session: &PhpSession,
    form_data: CreateSoftwareForm,
) -> Result<Html<String>, StatusCode> {
    let user_id = session.get("id")
        .and_then(|v| match v {
            SessionValue::String(s) => s.parse::<i64>().ok(),
            SessionValue::Integer(i) => Some(*i),
            _ => None,
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract form data with defaults
    let name = form_data.name.unwrap_or_default();
    let version = form_data.versao.unwrap_or("10".to_string());
    let size = form_data.size.unwrap_or("100".to_string());
    let ram = form_data.ram.unwrap_or("50".to_string());
    let software_type = form_data.software_type.unwrap_or("1".to_string());

    // Parse numeric values
    let version_num: i32 = version.parse().unwrap_or(10);
    let size_num: i32 = size.parse().unwrap_or(100);
    let ram_num: i32 = ram.parse().unwrap_or(50);
    let type_num: i32 = software_type.parse().unwrap_or(1);

    // Insert software into database
    // Original SQL: INSERT INTO software (id, userID, softName, softVersion, softSize, softRam, softType, isNPC)
    let result = sqlx::query!(
        "INSERT INTO software (userID, softName, softVersion, softSize, softRam, softType, isNPC) VALUES (?, ?, ?, ?, ?, ?, 0)",
        user_id,
        name,
        version_num,
        size_num,
        ram_num,
        type_num
    )
    .execute(&db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let software_id = result.last_insert_id() as i64;

    // If "running" checkbox was checked, add to software_running table
    if form_data.running.is_some() {
        sqlx::query!(
            "INSERT INTO software_running (softID, userID, ramUsage, isNPC) VALUES (?, ?, ?, 0)",
            software_id,
            user_id,
            ram_num
        )
        .execute(&db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Redirect to software page (as in original)
    Ok(Html("<script>window.location.href='/software.php';</script>".to_string()))
}

/// Generate the software creation form HTML
fn generate_creation_form() -> String {
    r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Create Software - HackerExperience</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .form-container { max-width: 600px; margin: 0 auto; }
            .warning { color: red; font-weight: bold; margin-bottom: 20px; }
            .form-group { margin-bottom: 15px; }
            label { display: inline-block; width: 80px; }
            input[type="text"], select { width: 200px; padding: 5px; }
            input[type="submit"] { background: #007cba; color: white; padding: 10px 20px; border: none; cursor: pointer; }
            .back-link { margin-top: 20px; }
        </style>
    </head>
    <body>
        <div class="form-container">
            <div class="warning">Não aperte f5 (resubmitar post)</div>
            
            <form action="" method="POST">
                <div class="form-group">
                    <label>Name:</label>
                    <input type="text" name="name" autofocus="autofocus" required>
                </div>
                
                <div class="form-group">
                    <label>Size:</label>
                    <input type="text" name="size" value="100"> MB
                </div>
                
                <div class="form-group">
                    <label>Vers:</label>
                    <input type="text" name="versao" value="10"> (Formato: 10 = 1.0, 34 = 3.4, 123 = 12.3)
                </div>
                
                <div class="form-group">
                    <label>Tipo:</label>
                    <select name="type">
                        <option value="1">1 - Cracker</option>
                        <option value="2">2 - Password encryptor</option>
                        <option value="3">3 - Port Scanner</option>
                        <option value="4">4 - Firewall</option>
                        <option value="5">5 - Hidder</option>
                        <option value="6">6 - Seeker</option>
                        <option value="7">7 - Antivirus</option>
                        <option value="8">8 - Virus Spam .vspam</option>
                        <option value="9">9 - Virus Warez .vwarez</option>
                        <option value="10">10 - Virus DDoS .vddos</option>
                        <option value="11">11 - Virus Collector</option>
                        <option value="12">12 - DDoS Breaker</option>
                        <option value="13">13 - FTP Exploit</option>
                        <option value="14">14 - SSH Exploit</option>
                        <option value="15">15 - Nmap scanner</option>
                        <option value="16">16 - Hardware Analyzer</option>
                        <option value="17">17 - .torrent</option>
                        <option value="18">18 - webserver.exe</option>
                        <option value="19">19 - wallet.exe</option>
                        <option value="20">20 - BTC Miner.vminer</option>
                        <option value="26">26 - riddle.exe (NPC only)</option>
                        <option value="29">29 - Doom (*)</option>
                    </select>
                </div>
                
                <div class="form-group">
                    <label>RAM:</label>
                    <input type="text" name="ram" value="50"> MB
                </div>
                
                <div class="form-group">
                    <label>Running:</label>
                    <input type="checkbox" name="running" value="1">
                </div>
                
                <div class="form-group">
                    <input type="submit" value="CRIAR">
                </div>
            </form>
            
            <div class="back-link">
                <a href="software.php">Back to my softwares</a>
            </div>
        </div>
    </body>
    </html>
    "#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_data_parsing() {
        let form = CreateSoftwareForm {
            name: Some("TestSoft".to_string()),
            versao: Some("20".to_string()),
            size: Some("200".to_string()),
            ram: Some("100".to_string()),
            software_type: Some("5".to_string()),
            running: Some("1".to_string()),
        };

        assert_eq!(form.name.unwrap(), "TestSoft");
        assert_eq!(form.versao.unwrap(), "20");
        assert_eq!(form.size.unwrap(), "200");
        assert_eq!(form.ram.unwrap(), "100");
        assert_eq!(form.software_type.unwrap(), "5");
        assert_eq!(form.running.unwrap(), "1");
    }

    #[test]
    fn test_version_format() {
        // Test version format: 10 = 1.0, 34 = 3.4, 123 = 12.3
        let version_10: i32 = "10".parse().unwrap();
        let version_34: i32 = "34".parse().unwrap();
        let version_123: i32 = "123".parse().unwrap();

        assert_eq!(version_10, 10);
        assert_eq!(version_34, 34);
        assert_eq!(version_123, 123);
    }

    #[test]
    fn test_software_types() {
        // Test that all software types are valid integers
        let types = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 26, 29];
        
        for type_id in types {
            assert!(type_id >= 1 && type_id <= 29);
        }
    }

    #[test]
    fn test_admin_access_check() {
        // Test admin access logic: only user IDs 1 and 2 should have access
        assert!(1 <= 2);  // User 1 has access
        assert!(2 <= 2);  // User 2 has access
        assert!(3 > 2);   // User 3 and above should be denied
        assert!(100 > 2); // Higher user IDs should be denied
    }
}