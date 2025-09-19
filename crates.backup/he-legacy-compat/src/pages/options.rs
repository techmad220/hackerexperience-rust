//! Options page handler - 1:1 port of options.php
//! 
//! Returns JSON data with beverage options for the game.
//! Simple endpoint that provides menu options data.

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use he_db::DbPool;
use crate::session::PhpSession;

/// Option item structure
#[derive(Debug, Serialize)]
pub struct OptionItem {
    pub data: u32,
    pub label: String,
}

/// Query parameters for options page (unused but kept for consistency)
#[derive(Debug, Deserialize)]
pub struct OptionsQuery {
    // No query parameters in original
}

/// Options handler - returns JSON data with beverage options
/// 
/// Port of: options.php
/// Behavior:
/// - Returns hardcoded beverage options as JSON
/// - No authentication or session checking required
/// - Simple data endpoint for game UI
pub async fn options_handler(
    Extension(_db_pool): Extension<DbPool>,
    Extension(_session): Extension<PhpSession>,
    Query(_params): Query<OptionsQuery>,
) -> Result<Json<Vec<OptionItem>>, StatusCode> {
    // 2019: I have no idea (preserving original comment)
    let mut options = Vec::new();
    
    options.push(OptionItem {
        data: 0,
        label: "Water".to_string(),
    });
    
    options.push(OptionItem {
        data: 0, 
        label: "Wine".to_string(),
    });
    
    options.push(OptionItem {
        data: 0,
        label: "Beer".to_string(), 
    });
    
    options.push(OptionItem {
        data: 0,
        label: "Coke".to_string(),
    });
    
    Ok(Json(options))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_option_item_serialization() {
        let item = OptionItem {
            data: 0,
            label: "Test".to_string(),
        };
        
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"data\":0"));
        assert!(json.contains("\"label\":\"Test\""));
    }
}