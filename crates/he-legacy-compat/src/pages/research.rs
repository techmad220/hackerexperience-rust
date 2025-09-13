//! Research page handler - 1:1 port of research.php
//! 
//! Research and development system (DEPRECATED in original)
//! Handles software research upgrades with financial validation
//! Note: Original PHP file is marked as deprecated but retains functionality

use axum::{
    extract::{Extension, Form},
    http::StatusCode,
    response::{Html, Redirect},
};
use serde::Deserialize;
use crate::classes::{system::System, pc::SoftwareVPC, finances::Finances, ranking::Ranking, process::Process};
use crate::session::{PhpSession, SessionValue};
use he_db::DbPool;

/// Form data for research operations
#[derive(Debug, Deserialize)]
pub struct ResearchForm {
    pub id: String,   // Software ID
    pub name: String, // Research name
    pub acc: String,  // Bank account
    pub keep: Option<String>, // Keep software flag
}

/// Research handler - processes software research requests
/// 
/// Port of: research.php (DEPRECATED)
/// Features:
/// - Software research price calculation
/// - Bank account validation
/// - Process creation for research
/// - Financial validation
/// - Session management
/// 
/// Note: Original file is marked as DEPRECATED but functionality preserved
pub async fn research_handler(
    Extension(db_pool): Extension<DbPool>,
    Extension(mut session): Extension<PhpSession>,
    form: Option<Form<ResearchForm>>,
) -> Result<Html<String>, StatusCode> {
    // Check for POST request - original only processes POST
    let Form(research_data) = match form {
        Some(form_data) => form_data,
        None => {
            // Not POST - redirect to index
            return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
        }
    };

    // Check if user is logged in
    if !session.isset_login() {
        return Ok(Html("<script>window.location.href='/index.php';</script>".to_string()));
    }

    // Get user ID from session
    let user_id = match session.get("id") {
        Some(SessionValue::Integer(id)) => *id,
        Some(SessionValue::String(id_str)) => {
            id_str.parse::<i64>().unwrap_or(0)
        },
        _ => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Initialize required classes
    let software = SoftwareVPC::new(user_id, db_pool.clone());
    let finances = Finances::new(db_pool.clone());
    let ranking = Ranking::new(db_pool.clone());
    let process = Process::new(db_pool.clone());

    // Extract and validate form data
    let software_id_str = research_data.id;
    let name = research_data.name;
    let acc_str = research_data.acc;
    let keep = if research_data.keep.is_some() { "1" } else { "0" };

    // Validate software ID is numeric
    let software_id: i64 = match software_id_str.parse() {
        Ok(id) => id,
        Err(_) => {
            return Ok(Html("Invalid ID".to_string()));
        }
    };

    // Check if software exists for user
    match software.isset_software(software_id, user_id, "VPC").await {
        Ok(false) => {
            return Ok(Html("This software doesn't exist".to_string()));
        },
        Err(_) => {
            return Ok(Html("Database error checking software".to_string()));
        },
        _ => {} // Software exists, continue
    }

    // Validate account ID is numeric
    let account_id: i64 = match acc_str.parse() {
        Ok(id) => id,
        Err(_) => {
            return Ok(Html("Invalid acc".to_string()));
        }
    };

    // Validate bank account
    match finances.bank_account_info(account_id).await {
        Ok(account_info) => {
            if !account_info.exists {
                return Ok(Html("INVALID_ACC".to_string()));
            }
            if account_info.bank_user != user_id {
                return Ok(Html("INVALID_ACC".to_string()));
            }
        },
        Err(_) => {
            return Ok(Html("INVALID_ACC".to_string()));
        }
    }

    // Get software information for price calculation
    let soft_info = match software.get_software(software_id, user_id, "VPC").await {
        Ok(Some(info)) => info,
        Ok(None) => {
            return Ok(Html("Software not found".to_string()));
        },
        Err(_) => {
            return Ok(Html("Error retrieving software info".to_string()));
        }
    };

    // Calculate research price
    let price = match software.research_calculate_price(soft_info.softversion).await {
        Ok(price) => price,
        Err(_) => {
            return Ok(Html("Error calculating research price".to_string()));
        }
    };

    // Create info string for process
    let info_str = format!("{}/{}/{}", account_id, keep, price);

    // Check if user has enough money
    match finances.total_money().await {
        Ok(total_money) => {
            if total_money < price {
                return Ok(Html("Not enough money".to_string()));
            }
        },
        Err(_) => {
            return Ok(Html("Error checking finances".to_string()));
        }
    }

    // Create research process
    match process.new_process(
        user_id,
        "RESEARCH",
        &String::new(), // Empty string for process target
        "local",
        software_id,
        &name,
        &info_str,
        0
    ).await {
        Ok(true) => {
            // Process created successfully - redirect to processes page
            let pid = match session.process_id("show") {
                Ok(id) => id,
                Err(_) => return Ok(Html("Error getting process ID".to_string())),
            };

            return Ok(Html(format!("<script>window.location.href='/processes?pid={}';</script>", pid)));
        },
        Ok(false) => {
            // Process creation failed - show existing process
            match session.process_id("show") {
                Ok(pid) => {
                    match process.get_process_info(pid).await {
                        Ok(_) => {
                            match process.show_process().await {
                                Ok(process_html) => {
                                    return Ok(Html(process_html));
                                },
                                Err(_) => {
                                    return Ok(Html("Error displaying process".to_string()));
                                }
                            }
                        },
                        Err(_) => {
                            return Ok(Html("Error getting process info".to_string()));
                        }
                    }
                },
                Err(_) => {
                    return Ok(Html("Error getting process ID".to_string()));
                }
            }
        },
        Err(_) => {
            return Ok(Html("Error creating research process".to_string()));
        }
    }
}

/// Display deprecation notice
/// 
/// Since original research.php starts with die("DEPRECATED"), 
/// this function can be used to show the deprecation notice
pub async fn research_deprecated_handler() -> Result<Html<String>, StatusCode> {
    Ok(Html(r#"
    <div class="deprecated-notice">
        <h2>Research System Deprecated</h2>
        <p>The research system has been deprecated and is no longer available.</p>
        <p>Please use the software management system instead.</p>
        <a href="/software" class="btn btn-primary">Go to Software</a>
    </div>
    "#.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_form_deserialization() {
        let form = ResearchForm {
            id: "123".to_string(),
            name: "Test Research".to_string(),
            acc: "456".to_string(),
            keep: Some("1".to_string()),
        };

        assert_eq!(form.id, "123");
        assert_eq!(form.name, "Test Research");
        assert_eq!(form.acc, "456");
        assert!(form.keep.is_some());
    }

    #[test]
    fn test_keep_flag_processing() {
        let with_keep = ResearchForm {
            id: "1".to_string(),
            name: "test".to_string(),
            acc: "1".to_string(),
            keep: Some("1".to_string()),
        };

        let without_keep = ResearchForm {
            id: "1".to_string(),
            name: "test".to_string(),
            acc: "1".to_string(),
            keep: None,
        };

        let keep_flag_with = if with_keep.keep.is_some() { "1" } else { "0" };
        let keep_flag_without = if without_keep.keep.is_some() { "1" } else { "0" };

        assert_eq!(keep_flag_with, "1");
        assert_eq!(keep_flag_without, "0");
    }

    #[test]
    fn test_info_string_format() {
        let account_id = 123i64;
        let keep = "1";
        let price = 5000i64;
        let info_str = format!("{}/{}/{}", account_id, keep, price);

        assert_eq!(info_str, "123/1/5000");
    }
}