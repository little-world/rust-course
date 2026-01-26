//! Pattern 2: Exhaustiveness and Match Ergonomics
//! Example: Exhaustive Matching for Safety
//!
//! Run with: cargo run --example p2_exhaustiveness

#[derive(Debug)]
enum RequestState {
    Pending,
    InProgress { started_at: u64 },
    Completed { result: String },
    Failed { error: String },
    // Try uncommenting this to see the compiler error:
    // Cancelled { reason: String },
}

fn state_duration(state: &RequestState, now: u64) -> Option<u64> {
    // This match is EXHAUSTIVE - every variant is handled.
    // If we add a new variant, this won't compile until we handle it!
    match state {
        RequestState::Pending => None,
        RequestState::InProgress { started_at } => Some(now - started_at),
        RequestState::Completed { .. } => None,
        RequestState::Failed { .. } => None,
    }
}

fn state_description(state: &RequestState) -> &'static str {
    match state {
        RequestState::Pending => "Waiting to start",
        RequestState::InProgress { .. } => "Currently running",
        RequestState::Completed { .. } => "Finished successfully",
        RequestState::Failed { .. } => "Encountered an error",
    }
}

// This function demonstrates how NOT to use wildcards
fn bad_state_handler(state: &RequestState) -> &'static str {
    match state {
        RequestState::Pending => "pending",
        RequestState::InProgress { .. } => "in progress",
        // BAD: This wildcard hides new variants!
        // If we add Cancelled, it silently gets handled here.
        _ => "done",
    }
}

#[derive(Debug)]
enum Color {
    Red,
    Green,
    Blue,
    Custom(u8, u8, u8),
}

fn color_to_hex(color: &Color) -> String {
    match color {
        Color::Red => "#FF0000".to_string(),
        Color::Green => "#00FF00".to_string(),
        Color::Blue => "#0000FF".to_string(),
        Color::Custom(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
    }
}

fn main() {
    println!("=== Exhaustive Matching ===");
    // Usage: calculate how long a request has been in progress
    let states = [
        RequestState::Pending,
        RequestState::InProgress { started_at: 100 },
        RequestState::Completed {
            result: "success".to_string(),
        },
        RequestState::Failed {
            error: "timeout".to_string(),
        },
    ];

    let now = 150;
    for state in &states {
        println!(
            "  {:?}\n    -> {} (duration: {:?})",
            state,
            state_description(state),
            state_duration(state, now)
        );
    }

    assert_eq!(
        state_duration(&RequestState::InProgress { started_at: 100 }, 150),
        Some(50)
    );

    println!("\n=== Color Conversion ===");
    let colors = [
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Custom(128, 64, 255),
    ];

    for color in &colors {
        println!("  {:?} => {}", color, color_to_hex(color));
    }

    println!("\n=== Why Exhaustiveness Matters ===");
    println!("If you add a new variant to an enum:");
    println!("  1. WITHOUT wildcard: Compiler error at every match");
    println!("  2. WITH wildcard: Silent bug - new variant handled incorrectly");

    println!("\n=== Bad Practice: Wildcards Hide Bugs ===");
    for state in &states {
        let desc = bad_state_handler(state);
        println!("  {:?} => {}", state, desc);
    }
    println!("  ^ 'Completed' and 'Failed' both map to 'done' - is that intentional?");

    println!("\n=== Best Practices ===");
    println!("1. Avoid wildcards in application code");
    println!("2. Handle each variant explicitly");
    println!("3. Let compiler catch missing cases when you add variants");
    println!("4. Use wildcards only for truly \"catch-all\" cases (external enums)");
}
