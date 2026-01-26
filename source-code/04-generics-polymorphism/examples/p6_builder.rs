//! Pattern 6: Phantom Types and Type-Level State
//! Example: Builder with Required Fields
//!
//! Run with: cargo run --example p6_builder

use std::marker::PhantomData;

// State markers for required fields
struct NoName;
struct HasName;
struct NoEmail;
struct HasEmail;
struct NoAge;
struct HasAge;

// User struct (the final product)
#[derive(Debug)]
struct User {
    name: String,
    email: String,
    age: u32,
    nickname: Option<String>, // Optional field
}

// Builder with phantom types tracking which fields are set
struct UserBuilder<Name, Email, Age> {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    nickname: Option<String>,
    _state: PhantomData<(Name, Email, Age)>,
}

// Initial state: nothing set
impl UserBuilder<NoName, NoEmail, NoAge> {
    fn new() -> Self {
        UserBuilder {
            name: None,
            email: None,
            age: None,
            nickname: None,
            _state: PhantomData,
        }
    }
}

// Set name (required)
impl<E, A> UserBuilder<NoName, E, A> {
    fn name(self, name: &str) -> UserBuilder<HasName, E, A> {
        UserBuilder {
            name: Some(name.to_string()),
            email: self.email,
            age: self.age,
            nickname: self.nickname,
            _state: PhantomData,
        }
    }
}

// Set email (required)
impl<N, A> UserBuilder<N, NoEmail, A> {
    fn email(self, email: &str) -> UserBuilder<N, HasEmail, A> {
        UserBuilder {
            name: self.name,
            email: Some(email.to_string()),
            age: self.age,
            nickname: self.nickname,
            _state: PhantomData,
        }
    }
}

// Set age (required)
impl<N, E> UserBuilder<N, E, NoAge> {
    fn age(self, age: u32) -> UserBuilder<N, E, HasAge> {
        UserBuilder {
            name: self.name,
            email: self.email,
            age: Some(age),
            nickname: self.nickname,
            _state: PhantomData,
        }
    }
}

// Set nickname (optional) - available in any state
impl<N, E, A> UserBuilder<N, E, A> {
    fn nickname(self, nickname: &str) -> UserBuilder<N, E, A> {
        UserBuilder {
            name: self.name,
            email: self.email,
            age: self.age,
            nickname: Some(nickname.to_string()),
            _state: PhantomData,
        }
    }
}

// build() only available when ALL required fields are set
impl UserBuilder<HasName, HasEmail, HasAge> {
    fn build(self) -> User {
        User {
            name: self.name.unwrap(),
            email: self.email.unwrap(),
            age: self.age.unwrap(),
            nickname: self.nickname,
        }
    }
}

fn main() {
    println!("=== Type-Safe Builder Pattern ===\n");

    // Build with all required fields (any order)
    let user1 = UserBuilder::new()
        .name("Alice")
        .email("alice@example.com")
        .age(30)
        .build();
    println!("User 1: {:?}", user1);

    // Different order - still works!
    let user2 = UserBuilder::new()
        .age(25)
        .name("Bob")
        .email("bob@example.com")
        .build();
    println!("User 2: {:?}", user2);

    // With optional field
    let user3 = UserBuilder::new()
        .name("Charlie")
        .nickname("Chuck")
        .email("charlie@example.com")
        .age(35)
        .build();
    println!("User 3: {:?}", user3);

    println!("\n=== Compile-Time Guarantees ===");
    println!("build() is ONLY available when:");
    println!("  - name is set (HasName)");
    println!("  - email is set (HasEmail)");
    println!("  - age is set (HasAge)");
    println!();
    println!("This would NOT compile:");
    println!("  UserBuilder::new().name(\"Alice\").build()");
    println!("    ^ ERROR: email and age not set");
    println!();
    println!("  UserBuilder::new().build()");
    println!("    ^ ERROR: no required fields set");

    // These would NOT compile:
    // let incomplete = UserBuilder::new()
    //     .name("Alice")
    //     .build(); // ERROR: build() not available

    // let incomplete2 = UserBuilder::new()
    //     .name("Alice")
    //     .email("a@b.com")
    //     .build(); // ERROR: age not set

    println!("\n=== Flexibility ===");
    println!("Optional fields can be set at any point:");
    let user4 = UserBuilder::new()
        .nickname("Early nickname")
        .name("Dave")
        .age(40)
        .email("dave@example.com")
        .build();
    println!("User 4: {:?}", user4);
}
