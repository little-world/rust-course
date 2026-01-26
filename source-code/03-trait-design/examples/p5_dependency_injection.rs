//! Pattern 5: Sealed Traits
//! Example: Dependency Injection with Traits
//!
//! Run with: cargo run --example p5_dependency_injection

use std::cell::RefCell;

// Define traits for external services
trait Database {
    fn get_user(&self, id: i32) -> Option<User>;
    fn save_user(&self, user: &User) -> Result<(), Error>;
}

trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Error>;
}

// Supporting types
#[derive(Clone, Debug, PartialEq)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(Debug)]
struct Error;

fn generate_id() -> i32 {
    1
}

// Service that depends on traits, not concrete types
struct UserService<D, E> {
    database: D,
    email: E,
}

impl<D: Database, E: EmailService> UserService<D, E> {
    fn new(database: D, email: E) -> Self {
        UserService { database, email }
    }

    fn register_user(&self, name: &str, email: &str) -> Result<User, Error> {
        let user = User {
            id: generate_id(),
            name: name.to_string(),
            email: email.to_string(),
        };

        self.database.save_user(&user)?;
        self.email.send_email(email, "Welcome!", "Thanks for registering!")?;

        Ok(user)
    }

    fn get_user(&self, id: i32) -> Option<User> {
        self.database.get_user(id)
    }
}

// Mock implementations for testing
struct MockDb {
    users: RefCell<Vec<User>>,
}

struct MockEmail {
    sent: RefCell<Vec<String>>,
}

impl Database for MockDb {
    fn get_user(&self, id: i32) -> Option<User> {
        self.users.borrow().iter().find(|u| u.id == id).cloned()
    }

    fn save_user(&self, user: &User) -> Result<(), Error> {
        self.users.borrow_mut().push(user.clone());
        Ok(())
    }
}

impl EmailService for MockEmail {
    fn send_email(&self, to: &str, _subject: &str, _body: &str) -> Result<(), Error> {
        self.sent.borrow_mut().push(to.to_string());
        Ok(())
    }
}

fn main() {
    // Usage: Trait bounds enable swapping real services for mocks in tests.
    println!("=== Dependency Injection Demo ===\n");

    let db = MockDb {
        users: RefCell::new(vec![]),
    };
    let email = MockEmail {
        sent: RefCell::new(vec![]),
    };

    let service = UserService::new(db, email);

    // Register a user
    println!("Registering user 'Alice'...");
    match service.register_user("Alice", "alice@example.com") {
        Ok(user) => println!("Created user: {:?}", user),
        Err(_) => println!("Failed to create user"),
    }

    // Verify user was saved
    println!("\nLooking up user with id 1...");
    if let Some(user) = service.get_user(1) {
        println!("Found user: {:?}", user);
    }

    // Check emails sent
    println!("\nEmails sent: {:?}", service.email.sent.borrow());

    // Demonstrate the testing advantage
    println!("\n=== Testing Advantage ===");
    println!("With trait-based DI, we can:");
    println!("- Swap MockDb for PostgresDb in production");
    println!("- Swap MockEmail for SmtpEmail in production");
    println!("- Test business logic without real services");
}
