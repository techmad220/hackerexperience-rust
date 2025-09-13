//! Query builder for type-safe SQL construction

use sqlx::{MySql, QueryBuilder, Arguments, mysql::MySqlArguments};
use std::marker::PhantomData;

/// Type-safe query builder for MySQL
pub struct HeQueryBuilder<'a> {
    builder: QueryBuilder<'a, MySql>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> HeQueryBuilder<'a> {
    /// Create a new query builder with initial SQL
    pub fn new(sql: &'a str) -> Self {
        Self {
            builder: QueryBuilder::new(sql),
            _phantom: PhantomData,
        }
    }

    /// Add a parameter to the query
    pub fn bind<T>(&mut self, value: T) -> &mut Self
    where
        T: 'a + Send + sqlx::Encode<'a, MySql> + sqlx::Type<MySql>,
    {
        self.builder.push_bind(value);
        self
    }

    /// Add literal SQL to the query
    pub fn push(&mut self, sql: &str) -> &mut Self {
        self.builder.push(sql);
        self
    }

    /// Add a separator (like comma or AND)
    pub fn push_separator(&mut self, separator: &str) -> &mut Self {
        self.builder.push(separator);
        self
    }

    /// Build the final query
    pub fn build(self) -> sqlx::query::Query<'a, MySql, MySqlArguments> {
        self.builder.build()
    }

    /// Build a query that returns results
    pub fn build_query_as<T>(self) -> sqlx::query::QueryAs<'a, MySql, T, MySqlArguments>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>,
    {
        self.builder.build_query_as()
    }
}

/// Select query builder
pub struct SelectBuilder<'a> {
    query: HeQueryBuilder<'a>,
    has_where: bool,
    has_order: bool,
}

impl<'a> SelectBuilder<'a> {
    /// Create a new SELECT query
    pub fn new() -> Self {
        Self {
            query: HeQueryBuilder::new("SELECT "),
            has_where: false,
            has_order: false,
        }
    }

    /// Add columns to select
    pub fn columns(mut self, columns: &[&str]) -> Self {
        for (i, column) in columns.iter().enumerate() {
            if i > 0 {
                self.query.push(", ");
            }
            self.query.push(column);
        }
        self
    }

    /// Add FROM clause
    pub fn from(mut self, table: &str) -> Self {
        self.query.push(" FROM ");
        self.query.push(table);
        self
    }

    /// Add WHERE clause
    pub fn where_clause(mut self, condition: &str) -> Self {
        if !self.has_where {
            self.query.push(" WHERE ");
            self.has_where = true;
        } else {
            self.query.push(" AND ");
        }
        self.query.push(condition);
        self
    }

    /// Add WHERE clause with parameter binding
    pub fn where_bind<T>(mut self, condition: &str, value: T) -> Self
    where
        T: 'a + Send + sqlx::Encode<'a, MySql> + sqlx::Type<MySql>,
    {
        if !self.has_where {
            self.query.push(" WHERE ");
            self.has_where = true;
        } else {
            self.query.push(" AND ");
        }
        self.query.push(condition);
        self.query.bind(value);
        self
    }

    /// Add OR condition
    pub fn or_where(mut self, condition: &str) -> Self {
        if self.has_where {
            self.query.push(" OR ");
        } else {
            self.query.push(" WHERE ");
            self.has_where = true;
        }
        self.query.push(condition);
        self
    }

    /// Add JOIN clause
    pub fn join(mut self, join_type: &str, table: &str, condition: &str) -> Self {
        self.query.push(" ");
        self.query.push(join_type);
        self.query.push(" JOIN ");
        self.query.push(table);
        self.query.push(" ON ");
        self.query.push(condition);
        self
    }

    /// Add INNER JOIN
    pub fn inner_join(self, table: &str, condition: &str) -> Self {
        self.join("INNER", table, condition)
    }

    /// Add LEFT JOIN
    pub fn left_join(self, table: &str, condition: &str) -> Self {
        self.join("LEFT", table, condition)
    }

    /// Add ORDER BY clause
    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        if !self.has_order {
            self.query.push(" ORDER BY ");
            self.has_order = true;
        } else {
            self.query.push(", ");
        }
        self.query.push(column);
        self.query.push(" ");
        self.query.push(direction);
        self
    }

    /// Add LIMIT clause
    pub fn limit(mut self, count: u64) -> Self {
        self.query.push(" LIMIT ");
        self.query.bind(count);
        self
    }

    /// Add OFFSET clause
    pub fn offset(mut self, count: u64) -> Self {
        self.query.push(" OFFSET ");
        self.query.bind(count);
        self
    }

    /// Build the final query
    pub fn build(self) -> sqlx::query::Query<'a, MySql, MySqlArguments> {
        self.query.build()
    }

    /// Build query that returns specific type
    pub fn build_as<T>(self) -> sqlx::query::QueryAs<'a, MySql, T, MySqlArguments>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow>,
    {
        self.query.build_query_as()
    }
}

/// Insert query builder
pub struct InsertBuilder<'a> {
    query: HeQueryBuilder<'a>,
    table: String,
    columns: Vec<String>,
    value_count: usize,
}

impl<'a> InsertBuilder<'a> {
    /// Create a new INSERT query
    pub fn new(table: &str) -> Self {
        Self {
            query: HeQueryBuilder::new("INSERT INTO "),
            table: table.to_string(),
            columns: Vec::new(),
            value_count: 0,
        }
    }

    /// Add a column and value
    pub fn column_value<T>(mut self, column: &str, value: T) -> Self
    where
        T: 'a + Send + sqlx::Encode<'a, MySql> + sqlx::Type<MySql>,
    {
        self.columns.push(column.to_string());
        
        if self.value_count == 0 {
            self.query.push(&self.table);
            self.query.push(" (");
            
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    self.query.push(", ");
                }
                self.query.push(col);
            }
            
            self.query.push(") VALUES (");
        } else {
            self.query.push(", ");
        }
        
        self.query.push("?");
        self.query.bind(value);
        self.value_count += 1;
        
        self
    }

    /// Build the final query
    pub fn build(mut self) -> sqlx::query::Query<'a, MySql, MySqlArguments> {
        if self.value_count > 0 {
            self.query.push(")");
        }
        self.query.build()
    }
}

/// Update query builder
pub struct UpdateBuilder<'a> {
    query: HeQueryBuilder<'a>,
    has_set: bool,
    has_where: bool,
}

impl<'a> UpdateBuilder<'a> {
    /// Create a new UPDATE query
    pub fn new(table: &str) -> Self {
        let mut query = HeQueryBuilder::new("UPDATE ");
        query.push(table);
        
        Self {
            query,
            has_set: false,
            has_where: false,
        }
    }

    /// Add SET clause
    pub fn set<T>(mut self, column: &str, value: T) -> Self
    where
        T: 'a + Send + sqlx::Encode<'a, MySql> + sqlx::Type<MySql>,
    {
        if !self.has_set {
            self.query.push(" SET ");
            self.has_set = true;
        } else {
            self.query.push(", ");
        }
        
        self.query.push(column);
        self.query.push(" = ?");
        self.query.bind(value);
        
        self
    }

    /// Add WHERE clause
    pub fn where_bind<T>(mut self, condition: &str, value: T) -> Self
    where
        T: 'a + Send + sqlx::Encode<'a, MySql> + sqlx::Type<MySql>,
    {
        if !self.has_where {
            self.query.push(" WHERE ");
            self.has_where = true;
        } else {
            self.query.push(" AND ");
        }
        
        self.query.push(condition);
        self.query.bind(value);
        
        self
    }

    /// Build the final query
    pub fn build(self) -> sqlx::query::Query<'a, MySql, MySqlArguments> {
        self.query.build()
    }
}

/// Delete query builder
pub struct DeleteBuilder<'a> {
    query: HeQueryBuilder<'a>,
    has_where: bool,
}

impl<'a> DeleteBuilder<'a> {
    /// Create a new DELETE query
    pub fn new(table: &str) -> Self {
        let mut query = HeQueryBuilder::new("DELETE FROM ");
        query.push(table);
        
        Self {
            query,
            has_where: false,
        }
    }

    /// Add WHERE clause
    pub fn where_bind<T>(mut self, condition: &str, value: T) -> Self
    where
        T: 'a + Send + sqlx::Encode<'a, MySql> + sqlx::Type<MySql>,
    {
        if !self.has_where {
            self.query.push(" WHERE ");
            self.has_where = true;
        } else {
            self.query.push(" AND ");
        }
        
        self.query.push(condition);
        self.query.bind(value);
        
        self
    }

    /// Build the final query
    pub fn build(self) -> sqlx::query::Query<'a, MySql, MySqlArguments> {
        self.query.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_builder() {
        let query = SelectBuilder::new()
            .columns(&["id", "name", "email"])
            .from("users")
            .where_clause("active = 1")
            .order_by("name", "ASC")
            .limit(10);
            
        // Test that the builder works without panicking
        let _built = query.build();
    }

    #[test]
    fn test_insert_builder() {
        let query = InsertBuilder::new("users")
            .column_value("name", "John Doe")
            .column_value("email", "john@example.com");
            
        let _built = query.build();
    }

    #[test]
    fn test_update_builder() {
        let query = UpdateBuilder::new("users")
            .set("name", "Jane Doe")
            .set("email", "jane@example.com")
            .where_bind("id = ?", 123);
            
        let _built = query.build();
    }

    #[test]
    fn test_delete_builder() {
        let query = DeleteBuilder::new("users")
            .where_bind("id = ?", 123);
            
        let _built = query.build();
    }
}