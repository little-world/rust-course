//! Pattern 3: Function-like Macro (SQL DSL)
//!
//! Demonstrates custom syntax parsing for domain-specific languages.
//! This simplified example shows the pattern used by sqlx and diesel.

use my_macros::sql;

fn main() {
    println!("=== SQL Function-like Macro Demo ===\n");

    let query1 = sql!("SELECT * FROM users WHERE age > 18");
    println!("Query 1: {}\n", query1);

    let query2 = sql!("INSERT INTO products (name, price) VALUES ('Widget', 9.99)");
    println!("Query 2: {}\n", query2);

    let query3 = sql!("UPDATE orders SET status = 'shipped' WHERE id = 123");
    println!("Query 3: {}", query3);
}
