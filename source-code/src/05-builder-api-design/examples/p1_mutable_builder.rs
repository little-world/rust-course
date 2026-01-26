//! Pattern 1: Builder Pattern Variations
//! Example: Non-Consuming (Mutable) Builder
//!
//! Run with: cargo run --example p1_mutable_builder

#[derive(Debug)]
pub struct Email {
    to: Vec<String>,
    subject: String,
    body: String,
}

pub struct EmailBuilder {
    to: Vec<String>,
    subject: String,
    body: String,
}

impl EmailBuilder {
    pub fn new() -> Self {
        EmailBuilder {
            to: Vec::new(),
            subject: String::new(),
            body: String::new(),
        }
    }

    // Methods take `&mut self` and return `&mut Self` for chaining.
    pub fn to(&mut self, email: impl Into<String>) -> &mut Self {
        self.to.push(email.into());
        self
    }

    pub fn subject(&mut self, subject: impl Into<String>) -> &mut Self {
        self.subject = subject.into();
        self
    }

    pub fn body(&mut self, body: impl Into<String>) -> &mut Self {
        self.body = body.into();
        self
    }

    // The build method borrows the builder (doesn't consume it).
    pub fn build(&self) -> Email {
        Email {
            to: self.to.clone(),
            subject: self.subject.clone(),
            body: self.body.clone(),
        }
    }

    pub fn clear(&mut self) {
        self.to.clear();
        self.subject.clear();
        self.body.clear();
    }
}

impl Default for EmailBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    println!("=== Non-Consuming (Mutable) Builder ===");
    // Usage: Mutable builder can be reused; clear() resets for next build.
    let mut builder = EmailBuilder::new();
    builder
        .to("alice@example.com")
        .subject("Hello!")
        .body("This is the first email.");
    let email1 = builder.build();
    println!("Email 1: {:#?}", email1);

    println!("\n=== Reusing the Builder ===");
    builder.clear();
    builder
        .to("bob@example.com")
        .to("charlie@example.com")
        .subject("Team Update")
        .body("This is the second email to multiple recipients.");
    let email2 = builder.build();
    println!("Email 2: {:#?}", email2);

    println!("\n=== Building Multiple Without Clear ===");
    // Without clear, we can incrementally add recipients
    let mut builder2 = EmailBuilder::new();
    builder2.subject("Newsletter").body("Weekly updates");

    builder2.to("subscriber1@example.com");
    let newsletter1 = builder2.build();

    builder2.to("subscriber2@example.com");
    let newsletter2 = builder2.build();

    println!("Newsletter 1 recipients: {:?}", newsletter1.to);
    println!("Newsletter 2 recipients: {:?}", newsletter2.to);

    println!("\n=== Key Differences from Consuming Builder ===");
    println!("- Methods take &mut self instead of self");
    println!("- build() borrows instead of consuming");
    println!("- Builder can be reused multiple times");
    println!("- Requires cloning data in build()");
}
