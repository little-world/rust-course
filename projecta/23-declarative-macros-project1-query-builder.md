
# Query Builder with Macro DSL

### Problem Statement

Build a SQL-like query DSL using Rust's declarative macros. Transform queries like `SELECT name FROM users WHERE |u| u.age > 30` into efficient iterator chains at compile time.

### Why It Matters

**Compile-Time Safety**: Runtime query builders parse "SELECT name FROM users WHERE age > 30" at runtime—typos become runtime errors after deployment. Macro DSLs parse at compile time: `query!(SELECT name FROM users WHERE age > 30)` fails to compile if `users` table or `age` column doesn't exist. A typo caught in CI is 100x cheaper than one found in production.

**Zero Runtime Overhead**: String-based query builders parse SQL every execution. A query executed 1 million times parses the same string 1 million times. Macro-based DSLs expand to direct iterator chains at compile time—zero parsing overhead. `query!(SELECT x FROM data WHERE x > 10)` compiles to `data.iter().filter(|row| row.x > 10).map(|row| row.x)`, exactly what you'd write by hand.

**Ergonomics vs Performance**: Hand-written iterator chains are fast but unreadable for complex queries. `.iter().filter().flat_map().group_by()` chains hard to understand. SQL syntax familiar to all developers. Macros give SQL ergonomics with iterator performance—best of both worlds.

Example performance comparison:
```
Runtime query builder: parse SQL (5μs) + execute → 1M queries = 5 seconds overhead
Macro DSL: parse at compile time (0μs runtime) + execute → 1M queries = 0 overhead
Hand-written iterators: same as macro (but 5x more code, harder to read)
```

## Understanding Declarative Macros

Before diving into milestones, let's understand the key concepts.

### Basic Macro Syntax

```rust
macro_rules! my_macro {
    // Pattern => Expansion
    (some tokens $variable:type) => {
        // Code to generate
    };
}
```

### Fragment Specifiers

| Specifier | Matches | Example                         |
|-----------|---------|---------------------------------|
| `$x:ident` | Identifier | `name`, `users`, `age`          |
| `$x:expr` | Expression | `\|u\| u.age > 30`, `5 + 3`     |
| `$x:ty` | Type | `u32`, `String`, `Vec<i32>`     |
| `$x:tt` | Token tree | Any single token or `(...)` group |

For example, a simple macro that greets a user:

```rust
macro_rules! greet {
    // Pattern: matches an expression and captures it as $name
    ($name:expr) => {
        // Expansion: the code that replaces the macro call
        println!("Hello, {}!", $name);
    };
}

fn main() {
    // Usage
    greet!("Rustacean"); 
    // At compile time, this expands to:
    // println!("Hello, {}!", "Rustacean");
}
```

Macros can also match multiple patterns, similar to a `match` statement:

```rust
macro_rules! calculate {
    // Pattern for addition
    (add $a:expr and $b:expr) => {
        $a + $b
    };
    // Pattern for subtraction
    (sub $a:expr from $b:expr) => {
        $b - $a
    };
}

fn main() {
    let sum = calculate!(add 5 and 10);      // Expands to: 5 + 10
    let diff = calculate!(sub 5 from 10);    // Expands to: 10 - 5
}
```

---
## Build the Project

### Milestone 1: Basic SELECT Queries

#### Goal

Create two macros:
1. `table!` - Define a struct representing a table row
2. `query!` - Transform SQL-like syntax into iterator chains

### Starter Code

**The `table!` Macro**

```rust
macro_rules! table {
    (
        name: $name:ident {
            $( $field:ident: $type:ty ),* $(,)?
        }
    ) => {
        // TODO: write the expanded code
    };
}

// Usage:
table! {
    name: User {
        id: u32,
        name: String,
        age: u32,
    }
}

// Expands to:
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub age: u32,
}
```

**The `query!` Macro**

```rust
macro_rules! query {
    // Pattern 1: SELECT * FROM table
    (SELECT * FROM $table:ident) => {{
        // TODO: clone the table
    }};

    // Pattern 2: SELECT field FROM table
    (SELECT $field:ident FROM $table:ident) => {{
        // TODO: map(|row| row.$field.clone()) 
    }};

    // Pattern 3: SELECT field FROM table WHERE condition
    (SELECT $field:ident FROM $table:ident WHERE $condition:expr) => {{
                // TODO: filter($condition)
                // TODO: map(|row| row.$field.clone()) 
 
    }};
}
```

#### How It Works

```rust
// This query:
let names = query! {
    SELECT name FROM users WHERE |u| u.age > 28
};

// Expands to:
let names = {
    users
        .iter()
        .filter(|u| u.age > 28)
        .map(|row| row.name.clone())
        .collect::<Vec<_>>()
};
```

### Checkpoint Tests

```rust
#[test]
fn test_table_macro() {
    table! {
        name: User {
            id: u32,
            name: String,
            age: u32,
        }
    }

    let user = User { id: 1, name: "Alice".to_string(), age: 30 };
    assert_eq!(user.id, 1);
}

#[test]
fn test_select_all() {
    table! { name: User { id: u32, name: String } }

    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    let results = query! { SELECT * FROM users };
    assert_eq!(results.len(), 2);
}

#[test]
fn test_select_field() {
    table! { name: User { id: u32, name: String } }

    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    let names = query! { SELECT name FROM users };
    assert_eq!(names, vec!["Alice", "Bob"]);
}

#[test]
fn test_select_with_where() {
    table! { name: User { id: u32, name: String, age: u32 } }

    let users = vec![
        User { id: 1, name: "Alice".to_string(), age: 30 },
        User { id: 2, name: "Bob".to_string(), age: 25 },
        User { id: 3, name: "Carol".to_string(), age: 35 },
    ];

    let names = query! {
        SELECT name FROM users WHERE |u| u.age > 28
    };
    assert_eq!(names, vec!["Alice", "Carol"]);
}
```

---

## Milestone 2: Multiple Fields (Tuples)

Select multiple fields: `SELECT name, age FROM users` → `Vec<(String, u32)>`

### Starter Code

**New Pattern with Repetition**

```rust
macro_rules! query {
    // ... previous patterns ...

    // Multiple fields: SELECT field1, field2, ... FROM table
    (SELECT $first:ident, $($rest:ident),+ FROM $table:ident) => {{
         // TODO: map(|row| (row.$first.clone(), $(row.$rest.clone()),+))
    }};

    // Multiple fields with WHERE
    (SELECT $first:ident, $($rest:ident),+ FROM $table:ident WHERE $condition:expr) => {{
             // TODO: filter($condition)
             // TODO: map(|row| (row.$first.clone(), $(row.$rest.clone()),+))
  
    }};
}
```

**Understanding Repetition**

```rust
// Pattern: $($rest:ident),+
// Matches: one or more comma-separated identifiers

// In expansion: $(row.$rest.clone()),+
// For input "name, age, score", generates:
// row.name.clone(), row.age.clone(), row.score.clone()
```

**Why Separate First and Rest**

```rust
// This pattern: $($field:ident),+
// Would also match single field, causing ambiguity!

// Solution: $first:ident, $($rest:ident),+
// Matches: name, age       → first=name, rest=[age]
// Matches: name, age, score → first=name, rest=[age, score]
// Does NOT match: name     → Use single-field pattern instead
```

### Checkpoint Tests

```rust
#[test]
fn test_select_two_fields() {
    table! { name: User { id: u32, name: String, age: u32 } }

    let users = vec![
        User { id: 1, name: "Alice".to_string(), age: 30 },
        User { id: 2, name: "Bob".to_string(), age: 25 },
    ];

    let results: Vec<(String, u32)> = query! {
        SELECT name, age FROM users
    };

    assert_eq!(results, vec![
        ("Alice".to_string(), 30),
        ("Bob".to_string(), 25),
    ]);
}

#[test]
fn test_three_fields_with_where() {
    table! { name: Product { id: u32, name: String, price: f64, stock: u32 } }

    let products = vec![
        Product { id: 1, name: "Widget".to_string(), price: 9.99, stock: 100 },
        Product { id: 2, name: "Gadget".to_string(), price: 19.99, stock: 0 },
    ];

    let results: Vec<(String, f64, u32)> = query! {
        SELECT name, price, stock FROM products WHERE |p| p.stock > 0
    };

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "Widget");
}
```

---

### Milestone 3: ORDER BY and LIMIT

Add sorting and pagination: `SELECT name FROM users ORDER BY age ASC LIMIT 2`

**The Float Problem**

`f64` doesn't implement `Ord` (because of `NaN`), so we can't use `.cmp()`:

```rust
// ❌ Doesn't compile for f64
.sorted_by(|a, b| a.price.cmp(&b.price))

// ✅ Works for all types
.sorted_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal))
```

### Starter Code

```rust
use itertools::Itertools;  // Add to Cargo.toml: itertools = "0.12"

macro_rules! query {
    // ... previous patterns ...

    // ORDER BY ASC
    (SELECT $field:ident FROM $table:ident ORDER BY $sort:ident ASC) => {{
          // TODO:  sorted_by(...)
          // TODO:  map(...)
    }};

    // ORDER BY DESC
    (SELECT $field:ident FROM $table:ident ORDER BY $sort:ident DESC) => {{
          // TODO:  sorted_by(...)
          // TODO:  map(...)
      :
    }};

    // LIMIT
    (SELECT $field:ident FROM $table:ident LIMIT $n:expr) => {{
         // TODO:  take($n)
         // TODO:  map(...)
    }};

    // WHERE + ORDER BY + LIMIT (note the comma after condition!)
    (SELECT $field:ident FROM $table:ident WHERE $cond:expr, ORDER BY $sort:ident ASC LIMIT $n:expr) => {{
      
            // TODO: filter
            // TODO: sorted_by
            // TODO: take
            // TODO: map
            // TODO: collect
    }};
}
```

### The Comma Trick

Remember: `expr` can only be followed by `,`, `;`, or `=>`.

```rust
// Query syntax with comma:
query! {
    SELECT name FROM users WHERE |u| u.age > 27, ORDER BY age ASC LIMIT 2
}
//                                             ^ comma required!
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use itertools::Itertools;  // Import needed for sorted_by!

    #[test]
    fn test_order_by_ascending() {
        table! { name: User { id: u32, name: String, age: u32 } }

        let users = vec![
            User { id: 1, name: "Alice".to_string(), age: 30 },
            User { id: 2, name: "Bob".to_string(), age: 25 },
            User { id: 3, name: "Carol".to_string(), age: 35 },
        ];

        let results = query! {
            SELECT name FROM users ORDER BY age ASC
        };

        assert_eq!(results, vec!["Bob", "Alice", "Carol"]);
    }

    #[test]
    fn test_where_order_limit() {
        table! { name: User { id: u32, name: String, age: u32 } }

        let users = vec![
            User { id: 1, name: "Alice".to_string(), age: 30 },
            User { id: 2, name: "Bob".to_string(), age: 25 },
            User { id: 3, name: "Carol".to_string(), age: 35 },
            User { id: 4, name: "Dave".to_string(), age: 28 },
        ];

        // Note the comma after the WHERE condition!
        let results = query! {
            SELECT name FROM users WHERE |u: &&User| u.age > 27, ORDER BY age ASC LIMIT 2
        };

        assert_eq!(results, vec!["Dave", "Alice"]);
    }
}
```

---

## Milestone 4: Simple JOINs

Join two tables: `users JOIN orders`

### Simplified Approach

Instead of complex field selection syntax like `users.name, orders.total`, we use `SELECT *` which returns tuples of full rows. This is simpler and avoids macro complexity.

### Starter Code
```rust
macro_rules! query {
    // ... previous patterns ...

    // Simple JOIN - returns Vec<(LeftRow, RightRow)>
    (
        SELECT * FROM $left:ident
        JOIN $right:ident ON $condition:expr
    ) => {{
        let mut results = Vec::new();
        // TODO: for $left iter 
        // TODO:     for $right iter 
        // TODO:         if $condition
        // TODO:             results.push (left_row.clone(), right_row.clone())
            
        results
    }};

    // JOIN with WHERE (note semicolon after ON condition!)
    (
        SELECT * FROM $left:ident
        JOIN $right:ident ON $join_cond:expr;
        WHERE $where_cond:expr
    ) => {{
        let mut results = Vec::new();
        // TODO: for $left iter 
        // TODO:     for $right iter 
        // TODO:         if $condition
        // TODO:            pair 
        // TODO:               if $where_cond 
        // TODO:                    results.push pair
  
        results
    }};
}
```

### Why Semicolons?

The ON condition is an `expr`, so it must be followed by `,` or `;`:

```rust
// Syntax with semicolons:
query! {
    SELECT * FROM users
    JOIN orders ON |u: &User, o: &Order| u.id == o.user_id;
    WHERE |(u, o): (&User, &Order)| u.age > 26
}
```

### Checkpoint Tests

```rust
#[test]
fn test_simple_join() {
    table! { name: User { id: u32, name: String } }
    table! { name: Order { id: u32, user_id: u32, total: f64 } }

    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    let orders = vec![
        Order { id: 1, user_id: 1, total: 100.0 },
        Order { id: 2, user_id: 1, total: 50.0 },
        Order { id: 3, user_id: 2, total: 75.0 },
    ];

    let results = query! {
        SELECT * FROM users
        JOIN orders ON |u: &User, o: &Order| u.id == o.user_id
    };

    assert_eq!(results.len(), 3);
    // Access fields from the tuple:
    assert_eq!(results[0].0.name, "Alice");
    assert_eq!(results[0].1.total, 100.0);
}

#[test]
fn test_join_with_where() {
    table! { name: User { id: u32, name: String, age: u32 } }
    table! { name: Order { id: u32, user_id: u32, total: f64 } }

    let users = vec![
        User { id: 1, name: "Alice".to_string(), age: 30 },
        User { id: 2, name: "Bob".to_string(), age: 25 },
    ];

    let orders = vec![
        Order { id: 1, user_id: 1, total: 100.0 },
        Order { id: 2, user_id: 2, total: 50.0 },
    ];

    let results = query! {
        SELECT * FROM users
        JOIN orders ON |u: &User, o: &Order| u.id == o.user_id;
        WHERE |(u, _o): (&User, &Order)| u.age > 26
    };

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "Alice");
}
```

---

## Milestone 5: Aggregations

Implement COUNT, SUM, AVG, MIN, MAX.

### The Type Annotation Problem

SUM needs to know the return type:

```rust
// ❌ Type cannot be inferred
let sum = query! { SELECT SUM(total) FROM orders };

// ✅ Explicit type annotation
let sum: f64 = query! { SELECT SUM(total) FROM orders };
```

### Starter Code

```rust
macro_rules! query {
    // COUNT(*)
    (SELECT COUNT(*) FROM $table:ident) => {{
        // TODO: len()
    }};

    // COUNT(*) with WHERE
    (SELECT COUNT(*) FROM $table:ident WHERE $condition:expr) => {{
        // TODO: filter($condition).len()
    }};

    // SUM(field)
    (SELECT SUM($field:ident) FROM $table:ident) => {{
        // TODO: map(|row|...).sum()
    }};

    // AVG(field)
    (SELECT AVG($field:ident) FROM $table:ident) => {{
        let count = $table.len();
        // TODO: let sum = map(|row|...).sum()
        // TODO: result = sum / count 
        // TODO: check 0 
    
    
    
    }};

    // MIN(field) - uses partial_cmp for f64 support
    (SELECT MIN($field:ident) FROM $table:ident) => {{
        $table
            // TODO: map(|row|...).sum()
            // TODO: min_by(|a, b| a.partial_cmp(b)...)
    
    
    }};

    // MAX(field)
    (SELECT MAX($field:ident) FROM $table:ident) => {{
        $table
            // TODO: map(|row|...).sum()
            // TODO: max_by(|a, b| a.partial_cmp(b)...)
    
}
```

### Checkpoint Tests

```rust
#[test]
fn test_count() {
    table! { name: User { id: u32, name: String, age: u32 } }

    let users = vec![
        User { id: 1, name: "Alice".to_string(), age: 30 },
        User { id: 2, name: "Bob".to_string(), age: 25 },
        User { id: 3, name: "Carol".to_string(), age: 35 },
    ];

    let count = query! { SELECT COUNT(*) FROM users };
    assert_eq!(count, 3);

    let count_filtered = query! {
        SELECT COUNT(*) FROM users WHERE |u| u.age > 28
    };
    assert_eq!(count_filtered, 2);
}

#[test]
fn test_sum() {
    table! { name: Order { id: u32, total: f64 } }

    let orders = vec![
        Order { id: 1, total: 100.0 },
        Order { id: 2, total: 50.0 },
        Order { id: 3, total: 75.0 },
    ];

    // Type annotation required!
    let sum: f64 = query! { SELECT SUM(total) FROM orders };
    assert_eq!(sum, 225.0);
}

#[test]
fn test_avg() {
    table! { name: User { id: u32, age: u32 } }

    let users = vec![
        User { id: 1, age: 30 },
        User { id: 2, age: 25 },
        User { id: 3, age: 35 },
    ];

    let avg = query! { SELECT AVG(age) FROM users };
    assert_eq!(avg, 30.0);
}

#[test]
fn test_min_max() {
    table! { name: Product { id: u32, price: f64 } }

    let products = vec![
        Product { id: 1, price: 9.99 },
        Product { id: 2, price: 19.99 },
        Product { id: 3, price: 14.99 },
    ];

    let min_price = query! { SELECT MIN(price) FROM products };
    let max_price = query! { SELECT MAX(price) FROM products };

    assert_eq!(min_price, 9.99);
    assert_eq!(max_price, 19.99);
}
```

---


## Complete Working Example

```rust
#![allow(clippy::needless_clone)]

// =============================================================================
// table! macro - Generates struct definitions
// =============================================================================

macro_rules! table {
    (
        name: $name:ident {
            $( $field:ident: $type:ty ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $(pub $field: $type,)*
        }
    };
}

// =============================================================================
// query! macro - SQL-like DSL
// =============================================================================

macro_rules! query {
    // ===== Aggregations =====

    (SELECT COUNT(*) FROM $table:ident) => {{
        $table.len()
    }};

    (SELECT COUNT(*) FROM $table:ident WHERE $condition:expr) => {{
        $table.iter().filter($condition).count()
    }};

    (SELECT SUM($field:ident) FROM $table:ident) => {{
        $table.iter().map(|row| row.$field.clone()).sum::<_>()
    }};

    (SELECT AVG($field:ident) FROM $table:ident) => {{
        let count = $table.len();
        if count == 0 { 0.0 } else {
            let sum: f64 = $table.iter().map(|row| row.$field as f64).sum();
            sum / count as f64
        }
    }};

    (SELECT MIN($field:ident) FROM $table:ident) => {{
        $table.iter().map(|row| row.$field.clone())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .expect("MIN on empty table")
    }};

    (SELECT MAX($field:ident) FROM $table:ident) => {{
        $table.iter().map(|row| row.$field.clone())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .expect("MAX on empty table")
    }};

    // ===== SELECT * =====

    (SELECT * FROM $table:ident) => {{
        $table.clone()
    }};

    // ===== Single field =====

    (SELECT $field:ident FROM $table:ident) => {{
        $table.iter().map(|row| row.$field.clone()).collect::<Vec<_>>()
    }};

    (SELECT $field:ident FROM $table:ident WHERE $condition:expr) => {{
        $table.iter().filter($condition)
            .map(|row| row.$field.clone()).collect::<Vec<_>>()
    }};

    (SELECT $field:ident FROM $table:ident ORDER BY $sort:ident ASC) => {{
        $table.iter()
            .sorted_by(|a, b| a.$sort.partial_cmp(&b.$sort).unwrap_or(std::cmp::Ordering::Equal))
            .map(|row| row.$field.clone()).collect::<Vec<_>>()
    }};

    (SELECT $field:ident FROM $table:ident ORDER BY $sort:ident DESC) => {{
        $table.iter()
            .sorted_by(|a, b| b.$sort.partial_cmp(&a.$sort).unwrap_or(std::cmp::Ordering::Equal))
            .map(|row| row.$field.clone()).collect::<Vec<_>>()
    }};

    (SELECT $field:ident FROM $table:ident LIMIT $n:expr) => {{
        $table.iter().take($n).map(|row| row.$field.clone()).collect::<Vec<_>>()
    }};

    // WHERE + ORDER BY + LIMIT (comma after WHERE condition)
    (SELECT $field:ident FROM $table:ident WHERE $cond:expr, ORDER BY $sort:ident ASC LIMIT $n:expr) => {{
        $table.iter().filter($cond)
            .sorted_by(|a, b| a.$sort.partial_cmp(&b.$sort).unwrap_or(std::cmp::Ordering::Equal))
            .take($n).map(|row| row.$field.clone()).collect::<Vec<_>>()
    }};

    // ===== Multiple fields =====

    (SELECT $first:ident, $($rest:ident),+ FROM $table:ident) => {{
        $table.iter()
            .map(|row| (row.$first.clone(), $(row.$rest.clone()),+))
            .collect::<Vec<_>>()
    }};

    (SELECT $first:ident, $($rest:ident),+ FROM $table:ident WHERE $condition:expr) => {{
        $table.iter().filter($condition)
            .map(|row| (row.$first.clone(), $(row.$rest.clone()),+))
            .collect::<Vec<_>>()
    }};

    (SELECT $first:ident, $($rest:ident),+ FROM $table:ident ORDER BY $sort:ident DESC LIMIT $n:expr) => {{
        $table.iter()
            .sorted_by(|a, b| b.$sort.partial_cmp(&a.$sort).unwrap_or(std::cmp::Ordering::Equal))
            .take($n)
            .map(|row| (row.$first.clone(), $(row.$rest.clone()),+))
            .collect::<Vec<_>>()
    }};

    // ===== JOINs =====

    (SELECT * FROM $left:ident JOIN $right:ident ON $join_cond:expr) => {{
        let mut results = Vec::new();
        for left_row in $left.iter() {
            for right_row in $right.iter() {
                if ($join_cond)(left_row, right_row) {
                    results.push((left_row.clone(), right_row.clone()));
                }
            }
        }
        results
    }};

    (SELECT * FROM $left:ident JOIN $right:ident ON $join_cond:expr; WHERE $where_cond:expr) => {{
        let mut results = Vec::new();
        for left_row in $left.iter() {
            for right_row in $right.iter() {
                if ($join_cond)(left_row, right_row) {
                    if ($where_cond)((&left_row, &right_row)) {
                        results.push((left_row.clone(), right_row.clone()));
                    }
                }
            }
        }
        results
    }};
}

// =============================================================================
// Example Usage
// =============================================================================

fn main() {
    use itertools::Itertools;

    table! {
        name: User {
            id: u32,
            name: String,
            age: u32,
        }
    }

    let users = vec![
        User { id: 1, name: "Alice".to_string(), age: 30 },
        User { id: 2, name: "Bob".to_string(), age: 25 },
        User { id: 3, name: "Carol".to_string(), age: 35 },
    ];

    // Basic SELECT
    let names = query! { SELECT name FROM users };
    println!("Names: {:?}", names);

    // SELECT with WHERE
    let adults = query! { SELECT name FROM users WHERE |u| u.age >= 30 };
    println!("Adults: {:?}", adults);

    // Multiple fields
    let name_ages: Vec<(String, u32)> = query! { SELECT name, age FROM users };
    println!("Name/Age: {:?}", name_ages);

    // ORDER BY
    let by_age = query! { SELECT name FROM users ORDER BY age ASC };
    println!("By age: {:?}", by_age);

    // Aggregations
    let count = query! { SELECT COUNT(*) FROM users };
    let avg_age = query! { SELECT AVG(age) FROM users };
    println!("Count: {}, Avg age: {}", count, avg_age);
}
```

---

## Key Lessons Learned

### 1. Follow-Set Rules Are Strict

`expr` fragments can ONLY be followed by `=>`, `,`, or `;`. Plan your syntax around this.

### 2. Closure Types Matter

When using `iter().filter()`, closures receive `&&T`. Use type inference or explicit `&&T`.

### 3. Floats Need Special Handling

`f64` doesn't implement `Ord`. Use `partial_cmp` with `unwrap_or(Ordering::Equal)` for sorting.

### 4. Type Annotations Sometimes Required

Aggregations like SUM may need explicit type annotations: `let sum: f64 = ...`

### 5. Keep JOINs Simple

Returning full row tuples is much simpler than trying to select specific fields from multiple tables.

### 6. Pattern Order Matters

More specific patterns should come before general ones, or use distinguishing syntax.

---

## Cargo.toml

```toml
[package]
name = "query-builder"
version = "0.1.0"
edition = "2021"

[dependencies]
itertools = "0.12"
```
