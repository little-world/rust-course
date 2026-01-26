//! Pattern 1: Struct Design Patterns
//! Example: Named Field Structs
//!
//! Run with: cargo run --example p1_named_struct

#[derive(Debug, Clone)]
struct User {
    id: u64,
    username: String,
    email: String,
    active: bool,
}

impl User {
    fn new(id: u64, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
            active: true,
        }
    }

    fn deactivate(&mut self) {
        self.active = false;
    }
}

fn main() {
    // Usage: Create user with constructor, mutate with methods.
    let mut user = User::new(1, "alice".to_string(), "alice@example.com".to_string());

    println!("Created user: {:?}", user);
    println!("User is active: {}", user.active);

    user.deactivate();
    println!("After deactivate: {}", user.active);

    // Named fields provide self-documenting code
    println!("\nAccessing fields by name:");
    println!("  user.id = {}", user.id);
    println!("  user.username = {}", user.username);
    println!("  user.email = {}", user.email);
}
