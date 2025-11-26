## Project 2: Type-Safe SQL Query Builder

### Problem Statement

Build a fluent, type-safe SQL query builder that:
- Prevents invalid SQL at compile time (can't ORDER BY before FROM)
- Tracks query state (select → from → where → order by → limit)
- Supports multiple database backends (PostgreSQL, MySQL, SQLite)
- Provides compile-time table/column name validation (where possible)
- Generates parameterized queries (prevents SQL injection)
- Handles complex queries (joins, subqueries, aggregations)
- Returns correctly-typed results based on query structure
- Optimizes query generation at compile time

The builder must ensure that only valid SQL can be constructed, with the type system preventing malformed queries.

### Why It Matters

SQL injection is in the OWASP Top 10. Hand-written SQL is error-prone:
- **Security**: Parameterized queries prevent SQL injection
- **Correctness**: Invalid SQL caught at compile time, not runtime
- **Refactoring**: Rename columns, compiler finds all uses
- **Type Safety**: Query results typed based on selected columns
- **Maintainability**: Self-documenting query construction

Type-safe query builders appear in:
- **ORMs**: Diesel, SeaORM, SQLx use type-level SQL
- **GraphQL**: Query structure validated before execution
- **Database Migrations**: Schema changes verified at compile time

### Use Cases

1. **Web Applications**: CRUD operations with compile-time safety
2. **Admin Dashboards**: Complex filtering and reporting queries
3. **API Backends**: Safe query construction from user input
4. **Data Analytics**: Complex aggregations and joins
5. **Microservices**: Database queries with type guarantees
6. **CLI Tools**: Database exploration and querying
7. **ETL Pipelines**: Data extraction with validated queries

### Solution Outline

**Core Type-State Structure:**
```rust
// Query states
pub struct Empty;
pub struct HasSelect;
pub struct HasFrom;
pub struct HasWhere;
pub struct Finalized;

pub struct Query<State, Backend> {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by_fields: Vec<String>,
    limit_value: Option<usize>,
    params: Vec<QueryParam>,
    _state: PhantomData<State>,
    _backend: PhantomData<Backend>,
}

// Only certain methods available in certain states
impl<B> Query<Empty, B> {
    pub fn select(fields: &[&str]) -> Query<HasSelect, B> { /* ... */ }
}

impl<B> Query<HasSelect, B> {
    pub fn from(table: &str) -> Query<HasFrom, B> { /* ... */ }
}

impl<B> Query<HasFrom, B> {
    pub fn where_clause(condition: &str) -> Query<HasWhere, B> { /* ... */ }
    pub fn finalize(self) -> Query<Finalized, B> { /* ... */ }
}

impl<B: Backend> Query<Finalized, B> {
    pub fn to_sql(&self) -> String { /* ... */ }
}
```

**Key Features:**
- **Type-State**: Track query construction stages
- **Backend Abstraction**: Different SQL dialects
- **Parameterization**: Safe value binding
- **Compile-Time Validation**: Wrong order = compile error
- **Fluent API**: Readable query construction

**SQL Generation:**
```rust
// PostgreSQL: SELECT * FROM users WHERE id = $1
// MySQL: SELECT * FROM users WHERE id = ?
// SQLite: SELECT * FROM users WHERE id = ?
```

### Testing Hints

**Compile-Time Tests:**
```rust
// Should NOT compile
fn invalid_query() {
    let q = Query::select(&["id", "name"])
        .where_clause("age > 18")  // ERROR: can't WHERE without FROM
        .to_sql();
}

// Should compile
fn valid_query() {
    let q = Query::select(&["id", "name"])
        .from("users")
        .where_clause("age > ?")
        .order_by("name")
        .limit(10)
        .to_sql();
}
```

**Runtime Tests:**
```rust
#[test]
fn test_simple_select() {
    let sql = Query::select(&["*"])
        .from("users")
        .to_sql();

    assert_eq!(sql, "SELECT * FROM users");
}

#[test]
fn test_parameterized_query() {
    let sql = Query::select(&["id", "name"])
        .from("users")
        .where_clause("age > ?")
        .bind(18)
        .to_sql();

    assert_eq!(sql, "SELECT id, name FROM users WHERE age > $1");
}

#[test]
fn test_backend_differences() {
    let pg_query = Query::<_, PostgreSQL>::select(&["id"]).from("users").to_sql();
    let mysql_query = Query::<_, MySQL>::select(&["id"]).from("users").to_sql();

    assert!(pg_query.contains("$1"));
    assert!(mysql_query.contains("?"));
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Query Builder with String Concatenation

**Goal:** Create a working query builder using simple string concatenation.

**What to implement:**
```rust
pub struct Query {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
}

impl Query {
    pub fn new() -> Self {
        Query {
            select_fields: Vec::new(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
        }
    }

    pub fn select(mut self, fields: &[&str]) -> Self {
        self.select_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn from(mut self, table: &str) -> Self {
        self.from_table = Some(table.to_string());
        self
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by.push(field.to_string());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = String::from("SELECT ");

        if self.select_fields.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.select_fields.join(", "));
        }

        if let Some(ref table) = self.from_table {
            sql.push_str(&format!(" FROM {}", table));
        }

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }

        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }

        sql
    }
}
```

**Check/Test:**
- Test simple SELECT * FROM table
- Test with WHERE clauses
- Test ORDER BY and LIMIT
- Test method chaining
- Verify generated SQL is correct

**Why this isn't enough:**
This naive implementation allows invalid SQL. You can call `where_clause()` before `from()`, or call `order_by()` without a `from()`. The type system doesn't prevent SQL injection—users can pass raw strings with malicious content. No parameterization means values are directly concatenated into queries. We need type-state to enforce correct ordering and parameterization for safety.

---

### Step 2: Add Type-State for Query Stages

**Goal:** Use phantom types to enforce correct query construction order.

**What to improve:**
```rust
use std::marker::PhantomData;

// State markers
pub struct Empty;
pub struct HasSelect;
pub struct HasFrom;

pub struct Query<State> {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    _state: PhantomData<State>,
}

// Start with SELECT
impl Query<Empty> {
    pub fn new() -> Self {
        Query {
            select_fields: Vec::new(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            _state: PhantomData,
        }
    }

    pub fn select(self, fields: &[&str]) -> Query<HasSelect> {
        Query {
            select_fields: fields.iter().map(|s| s.to_string()).collect(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            _state: PhantomData,
        }
    }
}

// After SELECT, must have FROM
impl Query<HasSelect> {
    pub fn from(self, table: &str) -> Query<HasFrom> {
        Query {
            select_fields: self.select_fields,
            from_table: Some(table.to_string()),
            where_clauses: self.where_clauses,
            order_by: self.order_by,
            limit: self.limit,
            _state: PhantomData,
        }
    }
}

// After FROM, can add WHERE, ORDER BY, LIMIT
impl Query<HasFrom> {
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by.push(field.to_string());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn to_sql(&self) -> String {
        // Same as before
        let mut sql = String::from("SELECT ");
        if self.select_fields.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.select_fields.join(", "));
        }
        if let Some(ref table) = self.from_table {
            sql.push_str(&format!(" FROM {}", table));
        }
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }
        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }
        sql
    }
}
```

**Check/Test:**
- Verify cannot call `where_clause()` before `from()` (compile error)
- Verify cannot call `order_by()` on `Query<HasSelect>` (compile error)
- Test that valid query order compiles
- Verify `to_sql()` only available after `from()`

**Why this isn't enough:**
Type-state prevents ordering errors, but we still have no parameterization—SQL injection is still possible. We're also limited to a single query type (SELECT). No support for INSERT, UPDATE, DELETE. No backend abstraction for different SQL dialects (PostgreSQL uses $1, MySQL uses ?). We need parameterized queries and backend support.

---

### Step 3: Add Parameterized Queries and SQL Injection Prevention

**Goal:** Support safe parameter binding to prevent SQL injection.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub enum QueryParam {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

pub struct Query<State> {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    params: Vec<QueryParam>,
    _state: PhantomData<State>,
}

impl Query<HasFrom> {
    // Instead of raw string, use placeholders
    pub fn where_eq(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        self.where_clauses.push(format!("{} = ${}", field, param_index));
        self.params.push(value.into());
        self
    }

    pub fn where_gt(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        self.where_clauses.push(format!("{} > ${}", field, param_index));
        self.params.push(value.into());
        self
    }

    pub fn where_lt(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        self.where_clauses.push(format!("{} < ${}", field, param_index));
        self.params.push(value.into());
        self
    }

    pub fn where_in(mut self, field: &str, values: &[impl Clone + Into<QueryParam>]) -> Self {
        let start_index = self.params.len() + 1;
        let placeholders: Vec<String> = (start_index..start_index + values.len())
            .map(|i| format!("${}", i))
            .collect();

        self.where_clauses.push(format!("{} IN ({})", field, placeholders.join(", ")));

        for value in values {
            self.params.push(value.clone().into());
        }

        self
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<QueryParam>) {
        (self.to_sql(), self.params.clone())
    }
}

// Implement Into<QueryParam> for common types
impl From<String> for QueryParam {
    fn from(s: String) -> Self {
        QueryParam::String(s)
    }
}

impl From<&str> for QueryParam {
    fn from(s: &str) -> Self {
        QueryParam::String(s.to_string())
    }
}

impl From<i64> for QueryParam {
    fn from(i: i64) -> Self {
        QueryParam::Int(i)
    }
}

impl From<i32> for QueryParam {
    fn from(i: i32) -> Self {
        QueryParam::Int(i as i64)
    }
}

impl From<f64> for QueryParam {
    fn from(f: f64) -> Self {
        QueryParam::Float(f)
    }
}

impl From<bool> for QueryParam {
    fn from(b: bool) -> Self {
        QueryParam::Bool(b)
    }
}
```

**Usage:**
```rust
let (sql, params) = Query::new()
    .select(&["id", "name", "email"])
    .from("users")
    .where_eq("active", true)
    .where_gt("age", 18)
    .where_in("status", &["active", "pending"])
    .to_sql_with_params();

// sql: "SELECT id, name, email FROM users WHERE active = $1 AND age > $2 AND status IN ($3, $4)"
// params: [Bool(true), Int(18), String("active"), String("pending")]
```

**Check/Test:**
- Test parameterized queries generate correct placeholders
- Test parameter values are collected correctly
- Test various data types (string, int, float, bool)
- Test IN clause with multiple values
- Verify user input cannot inject SQL

**Why this isn't enough:**
We have parameterization for PostgreSQL-style ($1, $2), but different databases use different placeholder syntax. MySQL and SQLite use `?`, Oracle uses `:1, :2`. We also only support SELECT queries—no INSERT, UPDATE, DELETE. The query builder is growing but lacks proper backend abstraction and query type generality.

---

### Step 4: Add Backend Abstraction for Multiple Databases

**Goal:** Support multiple SQL dialects (PostgreSQL, MySQL, SQLite).

**What to improve:**
```rust
// Backend trait
pub trait Backend {
    fn placeholder(index: usize) -> String;
    fn quote_identifier(name: &str) -> String;
}

pub struct PostgreSQL;
pub struct MySQL;
pub struct SQLite;

impl Backend for PostgreSQL {
    fn placeholder(index: usize) -> String {
        format!("${}", index)
    }

    fn quote_identifier(name: &str) -> String {
        format!("\"{}\"", name)
    }
}

impl Backend for MySQL {
    fn placeholder(_index: usize) -> String {
        "?".to_string()
    }

    fn quote_identifier(name: &str) -> String {
        format!("`{}`", name)
    }
}

impl Backend for SQLite {
    fn placeholder(_index: usize) -> String {
        "?".to_string()
    }

    fn quote_identifier(name: &str) -> String {
        format!("\"{}\"", name)
    }
}

// Add backend to query state
pub struct Query<State, Backend> {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    params: Vec<QueryParam>,
    _state: PhantomData<State>,
    _backend: PhantomData<Backend>,
}

impl<B: Backend> Query<Empty, B> {
    pub fn new() -> Self {
        Query {
            select_fields: Vec::new(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            params: Vec::new(),
            _state: PhantomData,
            _backend: PhantomData,
        }
    }

    pub fn select(self, fields: &[&str]) -> Query<HasSelect, B> {
        Query {
            select_fields: fields.iter().map(|s| s.to_string()).collect(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            params: Vec::new(),
            _state: PhantomData,
            _backend: PhantomData,
        }
    }
}

impl<B: Backend> Query<HasFrom, B> {
    pub fn where_eq(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        let placeholder = B::placeholder(param_index);
        let quoted_field = B::quote_identifier(field);
        self.where_clauses.push(format!("{} = {}", quoted_field, placeholder));
        self.params.push(value.into());
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = String::from("SELECT ");

        if self.select_fields.is_empty() {
            sql.push('*');
        } else {
            let quoted_fields: Vec<String> = self.select_fields
                .iter()
                .map(|f| B::quote_identifier(f))
                .collect();
            sql.push_str(&quoted_fields.join(", "));
        }

        if let Some(ref table) = self.from_table {
            sql.push_str(&format!(" FROM {}", B::quote_identifier(table)));
        }

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let quoted_order: Vec<String> = self.order_by
                .iter()
                .map(|f| B::quote_identifier(f))
                .collect();
            sql.push_str(&quoted_order.join(", "));
        }

        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }

        sql
    }
}
```

**Type-safe usage:**
```rust
// PostgreSQL query
let pg_query = Query::<_, PostgreSQL>::new()
    .select(&["id", "name"])
    .from("users")
    .where_eq("active", true)
    .to_sql();
// SELECT "id", "name" FROM "users" WHERE "active" = $1

// MySQL query
let mysql_query = Query::<_, MySQL>::new()
    .select(&["id", "name"])
    .from("users")
    .where_eq("active", true)
    .to_sql();
// SELECT `id`, `name` FROM `users` WHERE `active` = ?
```

**Check/Test:**
- Test PostgreSQL generates $1, $2, etc.
- Test MySQL generates ? placeholders
- Test SQLite generates ? placeholders
- Test identifier quoting differs by backend
- Test same query code works for all backends

**Why this isn't enough:**
We support multiple backends, but only for SELECT queries. Real applications need INSERT, UPDATE, DELETE. Also, no support for JOINs, subqueries, or aggregations (COUNT, SUM, AVG). The builder is growing complex but lacks full SQL feature support. We need more query types and advanced features.

---

### Step 5: Add INSERT, UPDATE, DELETE and JOIN Support

**Goal:** Support full CRUD operations and table joins.

**What to improve:**

**1. INSERT queries:**
```rust
pub struct InsertQuery<Backend> {
    table: String,
    columns: Vec<String>,
    values: Vec<QueryParam>,
    _backend: PhantomData<Backend>,
}

impl<B: Backend> InsertQuery<B> {
    pub fn into(table: &str) -> Self {
        InsertQuery {
            table: table.to_string(),
            columns: Vec::new(),
            values: Vec::new(),
            _backend: PhantomData,
        }
    }

    pub fn column(mut self, name: &str, value: impl Into<QueryParam>) -> Self {
        self.columns.push(name.to_string());
        self.values.push(value.into());
        self
    }

    pub fn columns(mut self, cols: &[(&str, impl Clone + Into<QueryParam>)]) -> Self {
        for (name, value) in cols {
            self.columns.push(name.to_string());
            self.values.push(value.clone().into());
        }
        self
    }

    pub fn to_sql(&self) -> String {
        let quoted_table = B::quote_identifier(&self.table);
        let quoted_cols: Vec<String> = self.columns
            .iter()
            .map(|c| B::quote_identifier(c))
            .collect();

        let placeholders: Vec<String> = (1..=self.values.len())
            .map(|i| B::placeholder(i))
            .collect();

        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            quoted_table,
            quoted_cols.join(", "),
            placeholders.join(", ")
        )
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<QueryParam>) {
        (self.to_sql(), self.values.clone())
    }
}
```

**2. UPDATE queries:**
```rust
pub struct UpdateQuery<B: Backend> {
    table: String,
    set_clauses: Vec<String>,
    where_clauses: Vec<String>,
    params: Vec<QueryParam>,
    _backend: PhantomData<B>,
}

impl<B: Backend> UpdateQuery<B> {
    pub fn table(table: &str) -> Self {
        UpdateQuery {
            table: table.to_string(),
            set_clauses: Vec::new(),
            where_clauses: Vec::new(),
            params: Vec::new(),
            _backend: PhantomData,
        }
    }

    pub fn set(mut self, column: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        let placeholder = B::placeholder(param_index);
        let quoted_col = B::quote_identifier(column);
        self.set_clauses.push(format!("{} = {}", quoted_col, placeholder));
        self.params.push(value.into());
        self
    }

    pub fn where_eq(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        let placeholder = B::placeholder(param_index);
        let quoted_field = B::quote_identifier(field);
        self.where_clauses.push(format!("{} = {}", quoted_field, placeholder));
        self.params.push(value.into());
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = format!("UPDATE {}", B::quote_identifier(&self.table));

        if !self.set_clauses.is_empty() {
            sql.push_str(" SET ");
            sql.push_str(&self.set_clauses.join(", "));
        }

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        sql
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<QueryParam>) {
        (self.to_sql(), self.params.clone())
    }
}
```

**3. DELETE queries:**
```rust
pub struct DeleteQuery<B: Backend> {
    table: String,
    where_clauses: Vec<String>,
    params: Vec<QueryParam>,
    _backend: PhantomData<B>,
}

impl<B: Backend> DeleteQuery<B> {
    pub fn from(table: &str) -> Self {
        DeleteQuery {
            table: table.to_string(),
            where_clauses: Vec::new(),
            params: Vec::new(),
            _backend: PhantomData,
        }
    }

    pub fn where_eq(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        let placeholder = B::placeholder(param_index);
        let quoted_field = B::quote_identifier(field);
        self.where_clauses.push(format!("{} = {}", quoted_field, placeholder));
        self.params.push(value.into());
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = format!("DELETE FROM {}", B::quote_identifier(&self.table));

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        sql
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<QueryParam>) {
        (self.to_sql(), self.params.clone())
    }
}
```

**4. JOIN support for SELECT:**
```rust
impl<B: Backend> Query<HasFrom, B> {
    pub fn inner_join(mut self, table: &str, on_condition: &str) -> Self {
        let join_clause = format!(
            "INNER JOIN {} ON {}",
            B::quote_identifier(table),
            on_condition
        );
        // Store in a joins vector (add to struct)
        self
    }

    pub fn left_join(mut self, table: &str, on_condition: &str) -> Self {
        let join_clause = format!(
            "LEFT JOIN {} ON {}",
            B::quote_identifier(table),
            on_condition
        );
        self
    }
}
```

**Usage examples:**
```rust
// INSERT
let (sql, params) = InsertQuery::<PostgreSQL>::into("users")
    .column("name", "Alice")
    .column("email", "alice@example.com")
    .column("age", 30)
    .to_sql_with_params();

// UPDATE
let (sql, params) = UpdateQuery::<PostgreSQL>::table("users")
    .set("name", "Bob")
    .set("email", "bob@example.com")
    .where_eq("id", 1)
    .to_sql_with_params();

// DELETE
let (sql, params) = DeleteQuery::<PostgreSQL>::from("users")
    .where_eq("id", 1)
    .to_sql_with_params();

// JOIN
let sql = Query::<_, PostgreSQL>::new()
    .select(&["users.name", "orders.total"])
    .from("users")
    .inner_join("orders", "users.id = orders.user_id")
    .where_eq("users.active", true)
    .to_sql();
```

**Check/Test:**
- Test INSERT generates correct SQL
- Test UPDATE with multiple SET clauses
- Test DELETE with WHERE clause
- Test INNER JOIN and LEFT JOIN
- Test parameterization for all query types
- Verify backend-specific syntax

**Why this isn't enough:**
We now have full CRUD and joins, but no support for aggregations (COUNT, SUM, GROUP BY, HAVING). No transactions support. No query execution—just SQL generation. The builder is getting quite large and complex. We need better organization and actual database integration.

---

### Step 6: Add Aggregations, Transactions, and Query Execution

**Goal:** Complete the query builder with aggregations, transactions, and actual database execution.

**What to improve:**

**1. Aggregation support:**
```rust
pub enum Aggregation {
    Count { field: String, alias: Option<String> },
    Sum { field: String, alias: Option<String> },
    Avg { field: String, alias: Option<String> },
    Max { field: String, alias: Option<String> },
    Min { field: String, alias: Option<String> },
}

impl<B: Backend> Query<HasFrom, B> {
    pub fn count(mut self, field: &str, alias: Option<&str>) -> Self {
        let count_expr = if field == "*" {
            "COUNT(*)".to_string()
        } else {
            format!("COUNT({})", B::quote_identifier(field))
        };

        let full_expr = if let Some(alias) = alias {
            format!("{} AS {}", count_expr, B::quote_identifier(alias))
        } else {
            count_expr
        };

        self.select_fields.push(full_expr);
        self
    }

    pub fn sum(mut self, field: &str, alias: Option<&str>) -> Self {
        let sum_expr = format!("SUM({})", B::quote_identifier(field));
        let full_expr = if let Some(alias) = alias {
            format!("{} AS {}", sum_expr, B::quote_identifier(alias))
        } else {
            sum_expr
        };
        self.select_fields.push(full_expr);
        self
    }

    pub fn group_by(mut self, field: &str) -> Self {
        // Add group_by field to struct
        self
    }

    pub fn having(mut self, condition: &str) -> Self {
        // Add having clause to struct
        self
    }
}
```

**2. Transaction support:**
```rust
pub struct Transaction<B: Backend> {
    connection: Connection, // Actual DB connection
    _backend: PhantomData<B>,
}

impl<B: Backend> Transaction<B> {
    pub fn begin(conn: Connection) -> Result<Self, QueryError> {
        // Execute BEGIN
        conn.execute("BEGIN")?;
        Ok(Transaction {
            connection: conn,
            _backend: PhantomData,
        })
    }

    pub fn execute_query<S>(&mut self, query: &Query<HasFrom, B>) -> Result<Vec<Row>, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)
    }

    pub fn execute_insert(&mut self, query: &InsertQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)?;
        Ok(self.connection.last_insert_id())
    }

    pub fn execute_update(&mut self, query: &UpdateQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        let rows_affected = self.connection.execute_with_params(&sql, &params)?;
        Ok(rows_affected)
    }

    pub fn commit(self) -> Result<(), QueryError> {
        self.connection.execute("COMMIT")?;
        Ok(())
    }

    pub fn rollback(self) -> Result<(), QueryError> {
        self.connection.execute("ROLLBACK")?;
        Ok(())
    }
}

#[must_use = "Transaction must be committed or rolled back"]
impl<B: Backend> Drop for Transaction<B> {
    fn drop(&mut self) {
        // Auto-rollback if not explicitly committed
        let _ = self.connection.execute("ROLLBACK");
    }
}
```

**3. Query execution:**
```rust
pub struct QueryExecutor<B: Backend> {
    connection: Connection,
    _backend: PhantomData<B>,
}

impl<B: Backend> QueryExecutor<B> {
    pub fn new(connection: Connection) -> Self {
        QueryExecutor {
            connection,
            _backend: PhantomData,
        }
    }

    pub fn execute<S>(&self, query: &Query<HasFrom, B>) -> Result<Vec<Row>, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)
    }

    pub fn execute_one<S>(&self, query: &Query<HasFrom, B>) -> Result<Option<Row>, QueryError> {
        let mut results = self.execute(query)?;
        Ok(results.pop())
    }

    pub fn execute_insert(&self, query: &InsertQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)?;
        Ok(self.connection.last_insert_id())
    }

    pub fn execute_update(&self, query: &UpdateQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)
    }

    pub fn execute_delete(&self, query: &DeleteQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)
    }

    pub fn begin_transaction(&self) -> Result<Transaction<B>, QueryError> {
        Transaction::begin(self.connection.clone())
    }
}
```

**4. Macro for type-safe table/column definitions:**
```rust
macro_rules! define_table {
    ($table_name:ident { $($column:ident: $type:ty),* $(,)? }) => {
        pub struct $table_name;

        impl $table_name {
            pub fn table_name() -> &'static str {
                stringify!($table_name)
            }

            $(
                pub fn $column() -> Column<$type> {
                    Column {
                        name: stringify!($column),
                        _type: PhantomData,
                    }
                }
            )*
        }
    };
}

pub struct Column<T> {
    name: &'static str,
    _type: PhantomData<T>,
}

// Usage
define_table!(users {
    id: i64,
    name: String,
    email: String,
    age: i32,
    active: bool,
});

// Type-safe query
let query = Query::<_, PostgreSQL>::new()
    .select(&[users::id(), users::name(), users::email()])
    .from(users::table_name())
    .where_eq(users::active(), true)
    .where_gt(users::age(), 18);
```

**Complete usage example:**
```rust
let executor = QueryExecutor::<PostgreSQL>::new(connection);

// Select with aggregation
let query = Query::new()
    .select(&["status"])
    .count("*", Some("total"))
    .from("orders")
    .group_by("status")
    .having("COUNT(*) > 10");

let results = executor.execute(&query)?;

// Transaction
let mut tx = executor.begin_transaction()?;

let insert = InsertQuery::into("users")
    .column("name", "Charlie")
    .column("email", "charlie@example.com");

let user_id = tx.execute_insert(&insert)?;

let update = UpdateQuery::table("profiles")
    .set("user_id", user_id)
    .where_eq("email", "charlie@example.com");

tx.execute_update(&update)?;

tx.commit()?; // Must commit or auto-rollback on drop
```

**Check/Test:**
- Test aggregation functions generate correct SQL
- Test GROUP BY and HAVING clauses
- Test transaction commit and rollback
- Test auto-rollback on Drop
- Test query execution with real database connection
- Test type-safe column references with macro
- Benchmark query execution performance

**What this achieves:**
A production-ready, type-safe SQL query builder:
- **Type-Safe**: Invalid query order prevented at compile time
- **SQL Injection Prevention**: All values parameterized
- **Multi-Backend**: PostgreSQL, MySQL, SQLite support
- **Full CRUD**: SELECT, INSERT, UPDATE, DELETE
- **Advanced Features**: Joins, aggregations, transactions
- **Safe Transactions**: Must commit or auto-rollback
- **Ergonomic**: Fluent API, macro-generated type-safe tables
- **Performant**: Zero-cost abstractions, prepared statements

**Extensions to explore:**
- Async query execution with tokio/async-std
- Connection pooling integration
- Query result deserialization to structs
- Compile-time schema validation
- Query plan optimization hints
- Streaming result iterators for large datasets

---
