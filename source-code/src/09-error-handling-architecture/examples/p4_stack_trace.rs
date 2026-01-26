//! Pattern 4: Custom Error Types with Context
//! Example: Error with Stack Trace
//!
//! Run with: RUST_BACKTRACE=1 cargo run --example p4_stack_trace

use std::backtrace::Backtrace;
use std::fmt;

/// Error that captures backtrace at creation time.
pub struct DetailedError {
    message: String,
    context: Vec<String>,
    backtrace: Backtrace,
}

impl DetailedError {
    pub fn new(message: impl Into<String>) -> Self {
        DetailedError {
            message: message.into(),
            context: Vec::new(),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for DetailedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        for ctx in &self.context {
            write!(f, "\n  Context: {}", ctx)?;
        }
        Ok(())
    }
}

impl fmt::Debug for DetailedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DetailedError")
            .field("message", &self.message)
            .field("context", &self.context)
            .field("backtrace", &self.backtrace)
            .finish()
    }
}

impl std::error::Error for DetailedError {}

fn inner_function() -> Result<(), DetailedError> {
    Err(DetailedError::new("something went wrong in inner function"))
}

fn middle_function() -> Result<(), DetailedError> {
    inner_function().map_err(|e| e.with_context("called from middle_function"))
}

fn outer_function() -> Result<(), DetailedError> {
    middle_function().map_err(|e| e.with_context("called from outer_function"))
}

fn main() {
    println!("=== Error with Stack Trace ===\n");

    match outer_function() {
        Ok(_) => println!("Success!"),
        Err(e) => {
            println!("Error occurred:");
            println!("  {}\n", e);

            println!("Backtrace status: {:?}", e.backtrace().status());
            println!("\nNote: Set RUST_BACKTRACE=1 to see full backtrace\n");

            // Print backtrace if available
            let bt = e.backtrace().to_string();
            if !bt.is_empty() && bt != "disabled backtrace" {
                println!("Backtrace:");
                for line in bt.lines().take(20) {
                    println!("  {}", line);
                }
                if bt.lines().count() > 20 {
                    println!("  ... (truncated)");
                }
            }
        }
    }

    println!("\n=== Context Accumulation ===");
    println!("Error flows: inner -> middle -> outer");
    println!("Each level adds context explaining the call chain");

    println!("\n=== Backtrace Capture ===");
    println!("Backtrace::capture() records call stack at error creation");
    println!("Set RUST_BACKTRACE=1 to enable (disabled by default)");
    println!("Set RUST_BACKTRACE=full for more detail");

    println!("\n=== Key Points ===");
    println!("1. Backtrace captured at error creation, not propagation");
    println!("2. Context added as error bubbles up");
    println!("3. Display for users, Debug for developers");
    println!("4. Backtrace useful for debugging but has overhead");
}
