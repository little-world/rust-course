//! Pattern 5: Recoverable vs Unrecoverable Errors
//! Example: Catch Unwind for FFI Boundaries
//!
//! Run with: cargo run --example p5_catch_unwind

use std::panic::{catch_unwind, AssertUnwindSafe};

/// Function that might panic.
fn risky_computation(input: i32) -> i32 {
    if input < 0 {
        panic!("negative input: {}", input);
    }
    if input > 100 {
        panic!("input too large: {}", input);
    }
    input * 2
}

/// FFI-safe wrapper that catches panics.
#[no_mangle]
pub extern "C" fn safe_compute(input: i32) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| risky_computation(input)));

    match result {
        Ok(value) => value,
        Err(_) => {
            eprintln!("  [PANIC CAUGHT] Computation failed for input {}", input);
            -1 // Error sentinel value
        }
    }
}

/// Process items, isolating panics per item.
fn process_items_resilient(items: Vec<i32>) -> Vec<Result<i32, String>> {
    items
        .into_iter()
        .map(|item| {
            catch_unwind(AssertUnwindSafe(|| risky_computation(item)))
                .map_err(|_| format!("panic processing {}", item))
        })
        .collect()
}

/// Worker that catches panics to avoid crashing the whole program.
fn worker_with_catch(tasks: Vec<i32>) {
    for (i, task) in tasks.iter().enumerate() {
        let result = catch_unwind(AssertUnwindSafe(|| {
            println!("  Processing task {}: input = {}", i, task);
            let result = risky_computation(*task);
            println!("  Task {} completed: result = {}", i, result);
            result
        }));

        if result.is_err() {
            eprintln!("  Task {} panicked, continuing with next task", i);
        }
    }
}

fn main() {
    println!("=== Catch Unwind for FFI Boundaries ===\n");

    // Basic catch_unwind
    println!("=== Basic catch_unwind ===");
    for input in [10, -5, 50, 200] {
        let result = safe_compute(input);
        println!("  safe_compute({}) = {}", input, result);
    }

    // Process multiple items
    println!("\n=== Resilient Batch Processing ===");
    let items = vec![5, -3, 20, 150, 10];
    let results = process_items_resilient(items);
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(v) => println!("  Item {}: Ok({})", i, v),
            Err(e) => println!("  Item {}: Err({})", i, e),
        }
    }

    // Worker pattern
    println!("\n=== Worker with Panic Recovery ===");
    worker_with_catch(vec![10, 20, -5, 30, 200, 40]);

    println!("\n=== When to Use catch_unwind ===");
    println!("MUST use at:");
    println!("  - FFI boundaries (panic across C is undefined behavior)");
    println!("  - Thread pool workers (one task shouldn't crash pool)");
    println!("  - Plugin systems (plugin panic shouldn't crash host)");
    println!();
    println!("AVOID for:");
    println!("  - Normal error handling (use Result)");
    println!("  - Hiding bugs (panic often indicates real problem)");
    println!("  - Performance-critical code (has overhead)");

    println!("\n=== AssertUnwindSafe ===");
    println!("Needed because:");
    println!("  - catch_unwind requires UnwindSafe bound");
    println!("  - Prevents catching panic with references to mutable state");
    println!("  - AssertUnwindSafe says 'I know what I'm doing'");
    println!("  - Use carefully - data may be inconsistent after panic");

    println!("\n=== Pattern: FFI Safe Function ===");
    println!("#[no_mangle]");
    println!("pub extern \"C\" fn safe_api(input: i32) -> i32 {{");
    println!("    catch_unwind(AssertUnwindSafe(|| {{");
    println!("        risky_rust_code(input)");
    println!("    }}))");
    println!("    .unwrap_or(-1)  // Return error code on panic");
    println!("}}");

    println!("\n=== Key Points ===");
    println!("1. catch_unwind catches panics, not aborts");
    println!("2. Returns Result<T, Box<dyn Any>>");
    println!("3. AssertUnwindSafe wraps closures with mutable captures");
    println!("4. Essential for FFI - panic across C is UB");
    println!("5. Has runtime overhead - don't use for normal control flow");
}
