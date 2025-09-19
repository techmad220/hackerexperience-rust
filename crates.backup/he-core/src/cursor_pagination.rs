//! Cursor-based Pagination for Extreme Scale
//!
//! Implements efficient cursor-based pagination that scales to millions of records
//! without performance degradation, unlike offset-based pagination.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Cursor encoding strategy
#[derive(Debug, Clone, Copy)]
pub enum CursorStrategy {
    /// Use primary key as cursor
    Id,
    /// Use timestamp as cursor
    Timestamp,
    /// Use composite cursor (id + timestamp)
    Composite,
    /// Custom cursor field
    Custom,
}

/// Cursor for pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    /// Primary identifier
    pub id: Option<i64>,
    /// Timestamp for time-based sorting
    pub timestamp: Option<DateTime<Utc>>,
    /// Additional sorting field
    pub sort_field: Option<String>,
    /// Sort direction
    pub direction: SortDirection,
}

impl Cursor {
    /// Create a new ID-based cursor
    pub fn from_id(id: i64, direction: SortDirection) -> Self {
        Self {
            id: Some(id),
            timestamp: None,
            sort_field: None,
            direction,
        }
    }

    /// Create a new timestamp-based cursor
    pub fn from_timestamp(timestamp: DateTime<Utc>, direction: SortDirection) -> Self {
        Self {
            id: None,
            timestamp: Some(timestamp),
            sort_field: None,
            direction,
        }
    }

    /// Create a composite cursor
    pub fn composite(id: i64, timestamp: DateTime<Utc>, direction: SortDirection) -> Self {
        Self {
            id: Some(id),
            timestamp: Some(timestamp),
            sort_field: None,
            direction,
        }
    }

    /// Encode cursor to base64 string
    pub fn encode(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        BASE64.encode(json.as_bytes())
    }

    /// Decode cursor from base64 string
    pub fn decode(encoded: &str) -> Result<Self, CursorError> {
        let bytes = BASE64.decode(encoded)
            .map_err(|_| CursorError::InvalidCursor)?;

        let json = String::from_utf8(bytes)
            .map_err(|_| CursorError::InvalidCursor)?;

        serde_json::from_str(&json)
            .map_err(|_| CursorError::InvalidCursor)
    }

    /// Build WHERE clause for SQL query
    pub fn to_sql_where(&self, table_alias: Option<&str>) -> String {
        let prefix = table_alias.map(|a| format!("{}.", a)).unwrap_or_default();

        match (self.id, self.timestamp.as_ref(), self.direction) {
            (Some(id), None, SortDirection::Asc) => {
                format!("{}id > {}", prefix, id)
            }
            (Some(id), None, SortDirection::Desc) => {
                format!("{}id < {}", prefix, id)
            }
            (None, Some(ts), SortDirection::Asc) => {
                format!("{}created_at > '{}'", prefix, ts.to_rfc3339())
            }
            (None, Some(ts), SortDirection::Desc) => {
                format!("{}created_at < '{}'", prefix, ts.to_rfc3339())
            }
            (Some(id), Some(ts), SortDirection::Asc) => {
                format!(
                    "({}created_at > '{}' OR ({}created_at = '{}' AND {}id > {}))",
                    prefix, ts.to_rfc3339(),
                    prefix, ts.to_rfc3339(),
                    prefix, id
                )
            }
            (Some(id), Some(ts), SortDirection::Desc) => {
                format!(
                    "({}created_at < '{}' OR ({}created_at = '{}' AND {}id < {}))",
                    prefix, ts.to_rfc3339(),
                    prefix, ts.to_rfc3339(),
                    prefix, id
                )
            }
            _ => "1=1".to_string(),
        }
    }
}

/// Sort direction for cursor pagination
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[serde(alias = "ASC")]
    Asc,
    #[serde(alias = "DESC")]
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
}

impl Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "ASC"),
            SortDirection::Desc => write!(f, "DESC"),
        }
    }
}

/// Cursor pagination parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorParams {
    /// Cursor for next page
    pub cursor: Option<String>,
    /// Number of items to fetch
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Sort direction
    #[serde(default)]
    pub direction: SortDirection,
}

fn default_limit() -> u32 {
    20
}

impl Default for CursorParams {
    fn default() -> Self {
        Self {
            cursor: None,
            limit: default_limit(),
            direction: SortDirection::default(),
        }
    }
}

impl CursorParams {
    /// Create new cursor parameters
    pub fn new(cursor: Option<String>, limit: Option<u32>, direction: Option<SortDirection>) -> Self {
        Self {
            cursor,
            limit: limit.unwrap_or(default_limit()).min(100),
            direction: direction.unwrap_or_default(),
        }
    }

    /// Get decoded cursor
    pub fn get_cursor(&self) -> Result<Option<Cursor>, CursorError> {
        match &self.cursor {
            Some(encoded) => Ok(Some(Cursor::decode(encoded)?)),
            None => Ok(None),
        }
    }

    /// SQL LIMIT value
    pub fn sql_limit(&self) -> i64 {
        (self.limit + 1) as i64 // Fetch one extra to determine if there's a next page
    }
}

/// Cursor pagination response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorResponse<T> {
    /// The actual data items
    pub data: Vec<T>,
    /// Cursor for next page (if available)
    pub next_cursor: Option<String>,
    /// Cursor for previous page (if available)
    pub prev_cursor: Option<String>,
    /// Whether there are more items
    pub has_next: bool,
    /// Whether there are previous items
    pub has_prev: bool,
}

impl<T> CursorResponse<T> {
    /// Create a new cursor response
    pub fn new(
        mut data: Vec<T>,
        params: &CursorParams,
        get_cursor: impl Fn(&T) -> Cursor,
    ) -> Self {
        let has_next = data.len() > params.limit as usize;

        // Remove the extra item if present
        if has_next {
            data.pop();
        }

        let next_cursor = if has_next && !data.is_empty() {
            Some(get_cursor(data.last().unwrap()).encode())
        } else {
            None
        };

        let prev_cursor = if !data.is_empty() && params.cursor.is_some() {
            Some(get_cursor(data.first().unwrap()).encode())
        } else {
            None
        };

        Self {
            data,
            next_cursor,
            prev_cursor,
            has_next,
            has_prev: params.cursor.is_some(),
        }
    }

    /// Create an empty response
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            next_cursor: None,
            prev_cursor: None,
            has_next: false,
            has_prev: false,
        }
    }
}

/// Cursor-based query builder
pub struct CursorQueryBuilder {
    base_query: String,
    where_clauses: Vec<String>,
    order_by: String,
    limit: i64,
    cursor: Option<Cursor>,
}

impl CursorQueryBuilder {
    /// Create a new cursor query builder
    pub fn new(base_query: &str) -> Self {
        Self {
            base_query: base_query.to_string(),
            where_clauses: Vec::new(),
            order_by: String::new(),
            limit: 20,
            cursor: None,
        }
    }

    /// Set cursor parameters
    pub fn with_cursor_params(mut self, params: &CursorParams) -> Result<Self, CursorError> {
        self.limit = params.sql_limit();
        self.cursor = params.get_cursor()?;

        // Set default ordering based on cursor type
        if let Some(ref cursor) = self.cursor {
            self.order_by = match (cursor.id, cursor.timestamp.as_ref()) {
                (Some(_), None) => format!("id {}", cursor.direction),
                (None, Some(_)) => format!("created_at {}", cursor.direction),
                (Some(_), Some(_)) => format!("created_at {}, id {}", cursor.direction, cursor.direction),
                _ => "id ASC".to_string(),
            };
        } else {
            self.order_by = format!("id {}", params.direction);
        }

        Ok(self)
    }

    /// Add WHERE clause
    pub fn add_where(mut self, clause: &str) -> Self {
        self.where_clauses.push(clause.to_string());
        self
    }

    /// Add cursor WHERE clause
    pub fn add_cursor_where(mut self, table_alias: Option<&str>) -> Self {
        if let Some(ref cursor) = self.cursor {
            self.where_clauses.push(cursor.to_sql_where(table_alias));
        }
        self
    }

    /// Set custom ORDER BY
    pub fn set_order_by(mut self, order_by: &str) -> Self {
        self.order_by = order_by.to_string();
        self
    }

    /// Build the final SQL query
    pub fn build(self) -> String {
        let mut query = self.base_query;

        // Add WHERE clauses
        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        // Add ORDER BY
        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            query.push_str(&self.order_by);
        }

        // Add LIMIT
        query.push_str(&format!(" LIMIT {}", self.limit));

        query
    }
}

/// Error types for cursor pagination
#[derive(Debug, thiserror::Error)]
pub enum CursorError {
    #[error("Invalid cursor format")]
    InvalidCursor,

    #[error("Cursor decode error")]
    DecodeError,

    #[error("Invalid sort field")]
    InvalidSortField,
}

/// Extension trait for types that support cursor pagination
pub trait CursorPaginate {
    /// Get cursor for this item
    fn to_cursor(&self) -> Cursor;
}

// Example implementation for a User type
#[derive(Debug, Clone)]
pub struct ExampleUser {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub name: String,
}

impl CursorPaginate for ExampleUser {
    fn to_cursor(&self) -> Cursor {
        Cursor::composite(self.id, self.created_at, SortDirection::Asc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_encoding() {
        let cursor = Cursor::from_id(123, SortDirection::Asc);
        let encoded = cursor.encode();
        let decoded = Cursor::decode(&encoded).unwrap();

        assert_eq!(decoded.id, Some(123));
        assert_eq!(decoded.direction, SortDirection::Asc);
    }

    #[test]
    fn test_cursor_sql_where() {
        let cursor = Cursor::from_id(100, SortDirection::Asc);
        assert_eq!(cursor.to_sql_where(None), "id > 100");

        let cursor = Cursor::from_id(100, SortDirection::Desc);
        assert_eq!(cursor.to_sql_where(Some("u")), "u.id < 100");

        let cursor = Cursor::from_timestamp(
            DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
            SortDirection::Asc
        );
        assert!(cursor.to_sql_where(None).contains("created_at >"));
    }

    #[test]
    fn test_cursor_params() {
        let params = CursorParams::new(None, Some(50), None);
        assert_eq!(params.limit, 50);
        assert_eq!(params.sql_limit(), 51); // One extra for next page detection
    }

    #[test]
    fn test_cursor_response() {
        let data = vec![
            ExampleUser { id: 1, created_at: Utc::now(), name: "User1".to_string() },
            ExampleUser { id: 2, created_at: Utc::now(), name: "User2".to_string() },
            ExampleUser { id: 3, created_at: Utc::now(), name: "User3".to_string() },
        ];

        let params = CursorParams::new(None, Some(2), None);
        let response = CursorResponse::new(data, &params, |u| u.to_cursor());

        assert_eq!(response.data.len(), 2);
        assert!(response.has_next);
        assert!(response.next_cursor.is_some());
    }

    #[test]
    fn test_query_builder() {
        let query = CursorQueryBuilder::new("SELECT * FROM users")
            .add_where("active = true")
            .add_cursor_where(Some("u"))
            .set_order_by("created_at DESC")
            .build();

        assert!(query.contains("WHERE active = true"));
        assert!(query.contains("ORDER BY created_at DESC"));
        assert!(query.contains("LIMIT"));
    }
}