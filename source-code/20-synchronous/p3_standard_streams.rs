// Pattern 3: Standard Streams
use std::io::{self, Write};

// Read with prompt
fn prompt(message: &str) -> io::Result<String> {
    print!("{}", message);
    io::stdout().flush()?;  // CRITICAL: flush before reading

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// Interactive menu (non-interactive demo version)
fn demonstrate_menu() {
    println!("\n=== Interactive Menu Demo ===");
    println!("In a real interactive session, this would loop:");
    println!();
    println!("=== Menu ===");
    println!("1. Process data");
    println!("2. View stats");
    println!("3. Exit");
    println!();
    println!("Enter choice: _");
    println!();
    println!("For invalid input, errors go to stderr:");
    eprintln!("Invalid choice");  // Error to stderr!
}

fn main() -> io::Result<()> {
    println!("=== Standard Streams Demo ===\n");

    // Demonstrate stdout vs stderr
    println!("This goes to stdout (normal output)");
    eprintln!("This goes to stderr (errors/diagnostics)");

    // Demonstrate the menu structure
    demonstrate_menu();

    // Demonstrate prompt (non-blocking for automated testing)
    println!("\n=== Prompt Demo ===");
    println!("The prompt function would display:");
    print!("Enter your name: ");
    io::stdout().flush()?;
    println!("<user input here>");

    // Show how flush is critical
    println!("\n=== Why flush() matters ===");
    println!("Without flush(), the prompt might not appear before read_line blocks.");
    println!("print!() is line-buffered, so partial lines stay in buffer.");
    println!("flush() forces the buffer to the terminal immediately.");

    // Demonstrate locked stdout for efficient bulk output
    println!("\n=== Locked stdout for bulk output ===");
    {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        for i in 1..=5 {
            writeln!(handle, "Locked write {}", i)?;
        }
    }
    println!("Locking stdout avoids repeated lock/unlock overhead.");

    println!("\nStandard streams examples completed");
    Ok(())
}
