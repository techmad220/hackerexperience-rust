//! Pagination and filtering support for API endpoints
//!
//! Provides reusable pagination and filtering structures for consistent API behavior

use serde::{Deserialize, Serialize};
use validator::Validate;
use std::collections::HashMap;

/// Default page size if not specified
pub const DEFAULT_PAGE_SIZE: u32 = 20;

/// Maximum allowed page size to prevent resource exhaustion
pub const MAX_PAGE_SIZE: u32 = 100;

/// Minimum page size
pub const MIN_PAGE_SIZE: u32 = 1;

/// Pagination request parameters
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct PaginationParams {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    #[validate(range(min = 1))]
    pub page: u32,

    /// Number of items per page
    #[serde(default = "default_page_size", alias = "per_page", alias = "limit")]
    #[validate(range(min = 1, max = 100))]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }
}

impl PaginationParams {
    /// Create pagination params with defaults
    pub fn new(page: Option<u32>, page_size: Option<u32>) -> Self {
        Self {
            page: page.unwrap_or(1),
            page_size: page_size
                .map(|s| s.min(MAX_PAGE_SIZE).max(MIN_PAGE_SIZE))
                .unwrap_or(DEFAULT_PAGE_SIZE),
        }
    }

    /// Calculate the SQL LIMIT value
    pub fn limit(&self) -> i64 {
        self.page_size as i64
    }

    /// Calculate the SQL OFFSET value
    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }

    /// Validate and sanitize the parameters
    pub fn sanitize(mut self) -> Self {
        if self.page < 1 {
            self.page = 1;
        }
        if self.page_size < MIN_PAGE_SIZE {
            self.page_size = MIN_PAGE_SIZE;
        }
        if self.page_size > MAX_PAGE_SIZE {
            self.page_size = MAX_PAGE_SIZE;
        }
        self
    }
}

/// Sorting parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SortParams {
    /// Field to sort by
    #[serde(default)]
    pub sort_by: Option<String>,

    /// Sort order (asc or desc)
    #[serde(default)]
    pub order: SortOrder,
}

/// Sort order enumeration
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[serde(alias = "ASC")]
    Asc,
    #[serde(alias = "DESC")]
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Asc
    }
}

impl SortOrder {
    pub fn to_sql(&self) -> &'static str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

/// Generic filter parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterParams {
    /// Search query for text fields
    #[serde(default)]
    pub search: Option<String>,

    /// Generic filters as key-value pairs
    #[serde(flatten)]
    pub filters: HashMap<String, String>,
}

impl FilterParams {
    /// Check if any filters are applied
    pub fn has_filters(&self) -> bool {
        self.search.is_some() || !self.filters.is_empty()
    }

    /// Get a specific filter value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.filters.get(key)
    }

    /// Sanitize search query to prevent injection
    pub fn sanitized_search(&self) -> Option<String> {
        self.search.as_ref().map(|s| {
            s.chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
                .take(100)
                .collect()
        })
    }
}

/// Combined pagination, sorting, and filtering parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,

    #[serde(flatten)]
    pub sort: SortParams,

    #[serde(flatten)]
    pub filter: FilterParams,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            pagination: PaginationParams::default(),
            sort: SortParams {
                sort_by: None,
                order: SortOrder::default(),
            },
            filter: FilterParams {
                search: None,
                filters: HashMap::new(),
            },
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The actual data items
    pub data: Vec<T>,

    /// Pagination metadata
    pub meta: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number
    pub page: u32,

    /// Items per page
    pub page_size: u32,

    /// Total number of items
    pub total: u64,

    /// Total number of pages
    pub total_pages: u32,

    /// Whether there's a next page
    pub has_next: bool,

    /// Whether there's a previous page
    pub has_prev: bool,
}

impl PaginationMeta {
    /// Create pagination metadata from parameters and total count
    pub fn new(params: &PaginationParams, total: u64) -> Self {
        let total_pages = ((total as f64) / (params.page_size as f64)).ceil() as u32;

        Self {
            page: params.page,
            page_size: params.page_size,
            total,
            total_pages,
            has_next: params.page < total_pages,
            has_prev: params.page > 1,
        }
    }

    /// Create pagination metadata for empty results
    pub fn empty() -> Self {
        Self {
            page: 1,
            page_size: DEFAULT_PAGE_SIZE,
            total: 0,
            total_pages: 0,
            has_next: false,
            has_prev: false,
        }
    }
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, params: &PaginationParams, total: u64) -> Self {
        Self {
            data,
            meta: PaginationMeta::new(params, total),
        }
    }

    /// Create an empty paginated response
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            meta: PaginationMeta::empty(),
        }
    }
}

/// SQL query builder helper for pagination
pub struct PaginationQueryBuilder {
    base_query: String,
    count_query: String,
    where_clauses: Vec<String>,
    order_by: Option<String>,
    limit: i64,
    offset: i64,
}

impl PaginationQueryBuilder {
    /// Create a new query builder
    pub fn new(base_query: &str, count_query: &str) -> Self {
        Self {
            base_query: base_query.to_string(),
            count_query: count_query.to_string(),
            where_clauses: Vec::new(),
            order_by: None,
            limit: DEFAULT_PAGE_SIZE as i64,
            offset: 0,
        }
    }

    /// Add a WHERE clause
    pub fn add_where(&mut self, clause: String) -> &mut Self {
        self.where_clauses.push(clause);
        self
    }

    /// Add search filter
    pub fn add_search(&mut self, fields: &[&str], search: &str) -> &mut Self {
        if !search.is_empty() && !fields.is_empty() {
            let search_clause = fields
                .iter()
                .map(|field| format!("{} ILIKE '%{}%'", field, search))
                .collect::<Vec<_>>()
                .join(" OR ");

            self.where_clauses.push(format!("({})", search_clause));
        }
        self
    }

    /// Set ORDER BY clause
    pub fn set_order_by(&mut self, field: &str, order: SortOrder) -> &mut Self {
        self.order_by = Some(format!("{} {}", field, order.to_sql()));
        self
    }

    /// Set pagination parameters
    pub fn set_pagination(&mut self, params: &PaginationParams) -> &mut Self {
        self.limit = params.limit();
        self.offset = params.offset();
        self
    }

    /// Build the final query
    pub fn build_query(&self) -> String {
        let mut query = self.base_query.clone();

        // Add WHERE clauses
        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        // Add ORDER BY
        if let Some(ref order_by) = self.order_by {
            query.push_str(" ORDER BY ");
            query.push_str(order_by);
        }

        // Add LIMIT and OFFSET
        query.push_str(&format!(" LIMIT {} OFFSET {}", self.limit, self.offset));

        query
    }

    /// Build the count query
    pub fn build_count_query(&self) -> String {
        let mut query = self.count_query.clone();

        // Add WHERE clauses
        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        query
    }
}

/// Trait for types that support pagination
pub trait Pageable {
    /// Apply pagination to a query
    fn paginate(self, params: &PaginationParams) -> Self;
}

/// Trait for types that support filtering
pub trait Filterable {
    /// Apply filters to a query
    fn filter(self, params: &FilterParams) -> Self;
}

/// Trait for types that support sorting
pub trait Sortable {
    /// Apply sorting to a query
    fn sort(self, params: &SortParams) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_defaults() {
        let params = PaginationParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn test_pagination_offset_calculation() {
        let params = PaginationParams::new(Some(3), Some(10));
        assert_eq!(params.offset(), 20); // (3-1) * 10
        assert_eq!(params.limit(), 10);
    }

    #[test]
    fn test_pagination_sanitization() {
        let params = PaginationParams::new(Some(0), Some(1000)).sanitize();
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, MAX_PAGE_SIZE);
    }

    #[test]
    fn test_pagination_meta() {
        let params = PaginationParams::new(Some(2), Some(10));
        let meta = PaginationMeta::new(&params, 35);

        assert_eq!(meta.page, 2);
        assert_eq!(meta.page_size, 10);
        assert_eq!(meta.total, 35);
        assert_eq!(meta.total_pages, 4);
        assert!(meta.has_next);
        assert!(meta.has_prev);
    }

    #[test]
    fn test_sort_order() {
        assert_eq!(SortOrder::Asc.to_sql(), "ASC");
        assert_eq!(SortOrder::Desc.to_sql(), "DESC");
    }

    #[test]
    fn test_filter_sanitization() {
        let mut filter = FilterParams {
            search: Some("test<script>alert()</script>".to_string()),
            filters: HashMap::new(),
        };

        assert_eq!(
            filter.sanitized_search(),
            Some("testscriptalertscript".to_string())
        );
    }

    #[test]
    fn test_query_builder() {
        let mut builder = PaginationQueryBuilder::new(
            "SELECT * FROM users",
            "SELECT COUNT(*) FROM users"
        );

        let params = PaginationParams::new(Some(2), Some(10));

        builder
            .add_search(&["username", "email"], "john")
            .set_order_by("created_at", SortOrder::Desc)
            .set_pagination(&params);

        let query = builder.build_query();
        assert!(query.contains("WHERE"));
        assert!(query.contains("ILIKE"));
        assert!(query.contains("ORDER BY created_at DESC"));
        assert!(query.contains("LIMIT 10 OFFSET 10"));
    }
}