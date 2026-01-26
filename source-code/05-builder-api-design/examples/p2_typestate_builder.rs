//! Pattern 2: Typestate Pattern
//! Example: Typestate Builder for Compile-Time Validation
//!
//! Run with: cargo run --example p2_typestate_builder

use std::marker::PhantomData;

// State markers for the builder
#[derive(Default)]
struct NoName;
#[derive(Default)]
struct HasName;
#[derive(Default)]
struct NoEmail;
#[derive(Default)]
struct HasEmail;

#[derive(Debug)]
struct User {
    name: String,
    email: String,
    age: Option<u32>,
}

// The builder is generic over its name and email states.
struct UserBuilder<NameState, EmailState> {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    _name_state: PhantomData<NameState>,
    _email_state: PhantomData<EmailState>,
}

// Initial state: no name, no email.
impl Default for UserBuilder<NoName, NoEmail> {
    fn default() -> Self {
        UserBuilder {
            name: None,
            email: None,
            age: None,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

// Setting name transitions from NoName to HasName
impl<E> UserBuilder<NoName, E> {
    fn name(self, name: impl Into<String>) -> UserBuilder<HasName, E> {
        UserBuilder {
            name: Some(name.into()),
            email: self.email,
            age: self.age,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

// Setting email transitions from NoEmail to HasEmail
impl<N> UserBuilder<N, NoEmail> {
    fn email(self, email: impl Into<String>) -> UserBuilder<N, HasEmail> {
        UserBuilder {
            name: self.name,
            email: Some(email.into()),
            age: self.age,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

// Optional field: age can be set in any state
impl<N, E> UserBuilder<N, E> {
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
}

// `build` only available in `HasName, HasEmail` state.
impl UserBuilder<HasName, HasEmail> {
    fn build(self) -> User {
        User {
            name: self.name.expect("guaranteed by typestate"),
            email: self.email.expect("guaranteed by typestate"),
            age: self.age,
        }
    }
}

fn main() {
    println!("=== Typestate Builder: Compile-Time Validation ===");
    // Usage: build() only available after both name() and email() called.
    let user = UserBuilder::default()
        .name("Alice")
        .email("alice@example.com")
        .age(30)
        .build();

    println!("Built user: {:#?}", user);

    println!("\n=== Order Doesn't Matter ===");
    // Can set email before name
    let user2 = UserBuilder::default()
        .email("bob@example.com")
        .name("Bob")
        .build();

    println!("Built user (email first): {:#?}", user2);

    println!("\n=== Optional Fields Can Be Omitted ===");
    let user3 = UserBuilder::default()
        .name("Charlie")
        .email("charlie@example.com")
        .build();

    println!("Built user (no age): {:#?}", user3);

    println!("\n=== Compile-Time Safety ===");
    println!("These would NOT compile:");
    println!("  UserBuilder::default().name(\"Bob\").build()");
    println!("    -> ERROR: no `build` on UserBuilder<HasName, NoEmail>");
    println!();
    println!("  UserBuilder::default().build()");
    println!("    -> ERROR: no `build` on UserBuilder<NoName, NoEmail>");
    println!();
    println!("  UserBuilder::default().email(\"x@y.com\").build()");
    println!("    -> ERROR: no `build` on UserBuilder<NoName, HasEmail>");

    // Uncomment to see compile errors:
    // UserBuilder::default().name("Bob").build();
    // UserBuilder::default().build();
    // UserBuilder::default().email("x@y.com").build();

    println!("\n=== Why Typestate Builder? ===");
    println!("- Required fields enforced at compile time, not runtime");
    println!("- No Result<T, E> needed for build()");
    println!("- IDE shows which methods are available in each state");
    println!("- Zero runtime overhead (PhantomData is zero-sized)");
}
