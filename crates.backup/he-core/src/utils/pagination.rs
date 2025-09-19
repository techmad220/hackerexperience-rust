use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaginationError {
    #[error("Invalid page number: {0}")]
    InvalidPage(u32),
    #[error("Invalid page size: {0}")]
    InvalidPageSize(u32),
    #[error("Invalid total items: {0}")]
    InvalidTotalItems(u64),
    #[error("Page size too large: {0} (max: {1})")]
    PageSizeTooLarge(u32, u32),
}

/// Pagination configuration and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    pub default_page_size: u32,
    pub max_page_size: u32,
    pub show_page_numbers: bool,
    pub show_first_last: bool,
    pub adjacent_pages: u32,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            default_page_size: 20,
            max_page_size: 100,
            show_page_numbers: true,
            show_first_last: true,
            adjacent_pages: 2,
        }
    }
}

/// Pagination information and navigation data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pagination {
    pub current_page: u32,
    pub page_size: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub offset: u64,
    pub has_previous: bool,
    pub has_next: bool,
    pub previous_page: Option<u32>,
    pub next_page: Option<u32>,
    pub first_page: u32,
    pub last_page: u32,
    pub start_item: u64,
    pub end_item: u64,
}

impl Pagination {
    /// Create new pagination instance
    pub fn new(
        current_page: u32,
        page_size: u32,
        total_items: u64,
    ) -> Result<Self, PaginationError> {
        Self::with_config(current_page, page_size, total_items, &PaginationConfig::default())
    }

    /// Create pagination with custom configuration
    pub fn with_config(
        current_page: u32,
        page_size: u32,
        total_items: u64,
        config: &PaginationConfig,
    ) -> Result<Self, PaginationError> {
        // Validate inputs
        if current_page == 0 {
            return Err(PaginationError::InvalidPage(current_page));
        }

        if page_size == 0 {
            return Err(PaginationError::InvalidPageSize(page_size));
        }

        if page_size > config.max_page_size {
            return Err(PaginationError::PageSizeTooLarge(page_size, config.max_page_size));
        }

        // Calculate pagination values
        let total_pages = if total_items == 0 {
            1
        } else {
            ((total_items as f64) / (page_size as f64)).ceil() as u32
        };

        // Ensure current page is within valid range
        let current_page = if current_page > total_pages {
            total_pages
        } else {
            current_page
        };

        let offset = ((current_page - 1) * page_size) as u64;
        
        let has_previous = current_page > 1;
        let has_next = current_page < total_pages;
        
        let previous_page = if has_previous {
            Some(current_page - 1)
        } else {
            None
        };
        
        let next_page = if has_next {
            Some(current_page + 1)
        } else {
            None
        };

        let start_item = if total_items == 0 {
            0
        } else {
            offset + 1
        };
        
        let end_item = if total_items == 0 {
            0
        } else {
            std::cmp::min(offset + page_size as u64, total_items)
        };

        Ok(Self {
            current_page,
            page_size,
            total_items,
            total_pages,
            offset,
            has_previous,
            has_next,
            previous_page,
            next_page,
            first_page: 1,
            last_page: total_pages,
            start_item,
            end_item,
        })
    }

    /// Get page numbers to display in pagination UI
    pub fn get_page_numbers(&self, adjacent_pages: u32) -> Vec<u32> {
        let mut pages = Vec::new();
        
        if self.total_pages <= 1 {
            return vec![1];
        }

        let start = if self.current_page <= adjacent_pages {
            1
        } else {
            self.current_page - adjacent_pages
        };

        let end = if self.current_page + adjacent_pages >= self.total_pages {
            self.total_pages
        } else {
            self.current_page + adjacent_pages
        };

        for page in start..=end {
            pages.push(page);
        }

        pages
    }

    /// Get pagination links with gaps
    pub fn get_pagination_links(&self, config: &PaginationConfig) -> PaginationLinks {
        let mut links = PaginationLinks::new();

        if self.total_pages <= 1 {
            links.pages.push(PageLink::new(1, true, false));
            return links;
        }

        let adjacent = config.adjacent_pages;
        let show_first_last = config.show_first_last;

        // Calculate visible page range
        let start = if self.current_page <= adjacent {
            1
        } else {
            self.current_page - adjacent
        };

        let end = if self.current_page + adjacent >= self.total_pages {
            self.total_pages
        } else {
            self.current_page + adjacent
        };

        // Add first page and gap if needed
        if show_first_last && start > 1 {
            links.pages.push(PageLink::new(1, false, false));
            
            if start > 2 {
                links.pages.push(PageLink::gap());
            }
        }

        // Add page range
        for page in start..=end {
            let is_current = page == self.current_page;
            links.pages.push(PageLink::new(page, is_current, false));
        }

        // Add gap and last page if needed
        if show_first_last && end < self.total_pages {
            if end < self.total_pages - 1 {
                links.pages.push(PageLink::gap());
            }
            
            links.pages.push(PageLink::new(self.total_pages, false, false));
        }

        links
    }

    /// Get summary text (e.g., "Showing 1-20 of 150 items")
    pub fn get_summary(&self) -> String {
        if self.total_items == 0 {
            "No items found".to_string()
        } else if self.total_items == 1 {
            "Showing 1 item".to_string()
        } else {
            format!(
                "Showing {}-{} of {} items",
                self.start_item,
                self.end_item,
                self.total_items
            )
        }
    }

    /// Check if current page is valid
    pub fn is_valid_page(&self) -> bool {
        self.current_page >= 1 && self.current_page <= self.total_pages
    }

    /// Get SQL LIMIT and OFFSET values
    pub fn get_sql_limit_offset(&self) -> (u32, u64) {
        (self.page_size, self.offset)
    }

    /// Navigate to specific page
    pub fn goto_page(&self, page: u32) -> Result<Self, PaginationError> {
        Self::new(page, self.page_size, self.total_items)
    }

    /// Navigate to first page
    pub fn first_page(&self) -> Result<Self, PaginationError> {
        self.goto_page(1)
    }

    /// Navigate to last page
    pub fn last_page(&self) -> Result<Self, PaginationError> {
        self.goto_page(self.total_pages)
    }

    /// Navigate to previous page
    pub fn previous_page(&self) -> Result<Option<Self>, PaginationError> {
        if let Some(prev_page) = self.previous_page {
            Ok(Some(self.goto_page(prev_page)?))
        } else {
            Ok(None)
        }
    }

    /// Navigate to next page
    pub fn next_page(&self) -> Result<Option<Self>, PaginationError> {
        if let Some(next_page) = self.next_page {
            Ok(Some(self.goto_page(next_page)?))
        } else {
            Ok(None)
        }
    }

    /// Change page size and recalculate pagination
    pub fn with_page_size(&self, new_page_size: u32) -> Result<Self, PaginationError> {
        // Calculate which item would be first on the current page
        let current_first_item = self.offset;
        
        // Calculate which page that item would be on with the new page size
        let new_page = if new_page_size == 0 {
            return Err(PaginationError::InvalidPageSize(new_page_size));
        } else {
            (current_first_item / new_page_size as u64) + 1
        };

        Self::new(new_page as u32, new_page_size, self.total_items)
    }
}

impl fmt::Display for Pagination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_summary())
    }
}

/// Represents a link in pagination navigation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageLink {
    pub page: Option<u32>,
    pub is_current: bool,
    pub is_gap: bool,
    pub is_disabled: bool,
}

impl PageLink {
    pub fn new(page: u32, is_current: bool, is_disabled: bool) -> Self {
        Self {
            page: Some(page),
            is_current,
            is_gap: false,
            is_disabled,
        }
    }

    pub fn gap() -> Self {
        Self {
            page: None,
            is_current: false,
            is_gap: true,
            is_disabled: false,
        }
    }
}

/// Collection of pagination links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationLinks {
    pub pages: Vec<PageLink>,
}

impl PaginationLinks {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
        }
    }
}

impl Default for PaginationLinks {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility struct for building pagination URLs
#[derive(Debug, Clone)]
pub struct PaginationUrlBuilder {
    base_url: String,
    params: std::collections::HashMap<String, String>,
    page_param: String,
    size_param: String,
}

impl PaginationUrlBuilder {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            params: std::collections::HashMap::new(),
            page_param: "page".to_string(),
            size_param: "size".to_string(),
        }
    }

    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_page_param(mut self, param: &str) -> Self {
        self.page_param = param.to_string();
        self
    }

    pub fn with_size_param(mut self, param: &str) -> Self {
        self.size_param = param.to_string();
        self
    }

    pub fn build_url(&self, page: u32, size: Option<u32>) -> String {
        let mut params = self.params.clone();
        params.insert(self.page_param.clone(), page.to_string());
        
        if let Some(page_size) = size {
            params.insert(self.size_param.clone(), page_size.to_string());
        }

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        if query_string.is_empty() {
            self.base_url.clone()
        } else {
            format!("{}?{}", self.base_url, query_string)
        }
    }
}

/// Helper functions for common pagination patterns
pub mod helpers {
    use super::*;

    /// Create pagination for database query results
    pub fn paginate_query_results<T>(
        items: Vec<T>,
        page: u32,
        page_size: u32,
        total_count: u64,
    ) -> Result<(Vec<T>, Pagination), PaginationError> {
        let pagination = Pagination::new(page, page_size, total_count)?;
        Ok((items, pagination))
    }

    /// Calculate offset and limit for database queries
    pub fn get_query_params(page: u32, page_size: u32) -> Result<(u64, u32), PaginationError> {
        if page == 0 {
            return Err(PaginationError::InvalidPage(page));
        }
        if page_size == 0 {
            return Err(PaginationError::InvalidPageSize(page_size));
        }

        let offset = ((page - 1) * page_size) as u64;
        Ok((offset, page_size))
    }

    /// Validate pagination parameters
    pub fn validate_params(
        page: Option<u32>,
        page_size: Option<u32>,
        config: &PaginationConfig,
    ) -> Result<(u32, u32), PaginationError> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(config.default_page_size);

        if page == 0 {
            return Err(PaginationError::InvalidPage(page));
        }

        if page_size == 0 {
            return Err(PaginationError::InvalidPageSize(page_size));
        }

        if page_size > config.max_page_size {
            return Err(PaginationError::PageSizeTooLarge(page_size, config.max_page_size));
        }

        Ok((page, page_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_creation() {
        let pagination = Pagination::new(1, 10, 100).unwrap();
        
        assert_eq!(pagination.current_page, 1);
        assert_eq!(pagination.page_size, 10);
        assert_eq!(pagination.total_items, 100);
        assert_eq!(pagination.total_pages, 10);
        assert_eq!(pagination.offset, 0);
        assert!(!pagination.has_previous);
        assert!(pagination.has_next);
    }

    #[test]
    fn test_pagination_middle_page() {
        let pagination = Pagination::new(5, 10, 100).unwrap();
        
        assert_eq!(pagination.current_page, 5);
        assert_eq!(pagination.offset, 40);
        assert!(pagination.has_previous);
        assert!(pagination.has_next);
        assert_eq!(pagination.previous_page, Some(4));
        assert_eq!(pagination.next_page, Some(6));
        assert_eq!(pagination.start_item, 41);
        assert_eq!(pagination.end_item, 50);
    }

    #[test]
    fn test_pagination_last_page() {
        let pagination = Pagination::new(10, 10, 100).unwrap();
        
        assert_eq!(pagination.current_page, 10);
        assert_eq!(pagination.offset, 90);
        assert!(pagination.has_previous);
        assert!(!pagination.has_next);
        assert_eq!(pagination.previous_page, Some(9));
        assert_eq!(pagination.next_page, None);
        assert_eq!(pagination.start_item, 91);
        assert_eq!(pagination.end_item, 100);
    }

    #[test]
    fn test_pagination_empty_results() {
        let pagination = Pagination::new(1, 10, 0).unwrap();
        
        assert_eq!(pagination.current_page, 1);
        assert_eq!(pagination.total_pages, 1);
        assert_eq!(pagination.start_item, 0);
        assert_eq!(pagination.end_item, 0);
        assert!(!pagination.has_previous);
        assert!(!pagination.has_next);
    }

    #[test]
    fn test_pagination_invalid_page() {
        let result = Pagination::new(0, 10, 100);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaginationError::InvalidPage(0)));
    }

    #[test]
    fn test_pagination_invalid_page_size() {
        let result = Pagination::new(1, 0, 100);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaginationError::InvalidPageSize(0)));
    }

    #[test]
    fn test_pagination_page_too_high() {
        let pagination = Pagination::new(15, 10, 100).unwrap();
        // Should automatically adjust to last valid page
        assert_eq!(pagination.current_page, 10);
    }

    #[test]
    fn test_get_page_numbers() {
        let pagination = Pagination::new(5, 10, 200).unwrap();
        let pages = pagination.get_page_numbers(2);
        
        assert_eq!(pages, vec![3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_get_page_numbers_start() {
        let pagination = Pagination::new(1, 10, 200).unwrap();
        let pages = pagination.get_page_numbers(2);
        
        assert_eq!(pages, vec![1, 2, 3]);
    }

    #[test]
    fn test_get_page_numbers_end() {
        let pagination = Pagination::new(20, 10, 200).unwrap();
        let pages = pagination.get_page_numbers(2);
        
        assert_eq!(pages, vec![18, 19, 20]);
    }

    #[test]
    fn test_pagination_summary() {
        let pagination = Pagination::new(5, 10, 157).unwrap();
        assert_eq!(pagination.get_summary(), "Showing 41-50 of 157 items");

        let empty = Pagination::new(1, 10, 0).unwrap();
        assert_eq!(empty.get_summary(), "No items found");

        let single = Pagination::new(1, 10, 1).unwrap();
        assert_eq!(single.get_summary(), "Showing 1 item");
    }

    #[test]
    fn test_pagination_navigation() {
        let pagination = Pagination::new(5, 10, 100).unwrap();
        
        let first = pagination.first_page().unwrap();
        assert_eq!(first.current_page, 1);

        let last = pagination.last_page().unwrap();
        assert_eq!(last.current_page, 10);

        let prev = pagination.previous_page().unwrap().unwrap();
        assert_eq!(prev.current_page, 4);

        let next = pagination.next_page().unwrap().unwrap();
        assert_eq!(next.current_page, 6);
    }

    #[test]
    fn test_pagination_with_page_size() {
        let pagination = Pagination::new(3, 10, 100).unwrap();
        // Current page shows items 21-30
        
        let new_pagination = pagination.with_page_size(20).unwrap();
        // Should show page 2 (items 21-40 with page size 20)
        assert_eq!(new_pagination.current_page, 2);
        assert_eq!(new_pagination.page_size, 20);
    }

    #[test]
    fn test_url_builder() {
        let builder = PaginationUrlBuilder::new("/search")
            .with_param("q", "test")
            .with_param("category", "news");
        
        let url = builder.build_url(3, Some(20));
        assert!(url.contains("page=3"));
        assert!(url.contains("size=20"));
        assert!(url.contains("q=test"));
        assert!(url.contains("category=news"));
    }

    #[test]
    fn test_validate_params() {
        let config = PaginationConfig::default();
        
        let (page, size) = helpers::validate_params(Some(5), Some(25), &config).unwrap();
        assert_eq!(page, 5);
        assert_eq!(size, 25);

        let (page, size) = helpers::validate_params(None, None, &config).unwrap();
        assert_eq!(page, 1);
        assert_eq!(size, config.default_page_size);

        let result = helpers::validate_params(Some(0), Some(25), &config);
        assert!(result.is_err());

        let result = helpers::validate_params(Some(5), Some(200), &config);
        assert!(result.is_err());
    }
}