//! Pattern 2: Newtype and Wrapper Patterns
//! Example: Newtype for Type Safety
//!
//! Run with: cargo run --example p2_newtype

// Newtype for semantic clarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UserId(u64);

#[derive(Debug, Clone, Copy)]
struct OrderId(u64);

// User struct that requires UserId
#[derive(Debug)]
struct User {
    id: UserId,
    name: String,
}

// Prevent accidentally mixing IDs
fn get_user(id: UserId) -> User {
    println!("Fetching user with id: {:?}", id);
    User {
        id,
        name: "Alice".to_string(),
    }
}

fn get_order(id: OrderId) {
    println!("Fetching order with id: {:?}", id);
}

fn main() {
    // Usage: Distinct types prevent mixing IDs even with same inner value.
    let user_id = UserId(42);
    let order_id = OrderId(42);

    println!("user_id: {:?}", user_id);
    println!("order_id: {:?}", order_id);

    // These work correctly
    let user = get_user(user_id);
    println!("Got user: {:?}", user);

    get_order(order_id);

    // This won't compile - type safety prevents mixing:
    // get_user(order_id); // Error: expected UserId, got OrderId

    // Newtypes also enable implementing traits
    let ids = vec![UserId(3), UserId(1), UserId(2)];
    let mut sorted = ids.clone();
    sorted.sort(); // Works because we derived Ord
    println!("\nSorted user IDs: {:?}", sorted);

    // Access inner value when needed
    println!("\nInner value of user_id: {}", user_id.0);
}
