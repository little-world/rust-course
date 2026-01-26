// Pattern 6: Basic Process Spawning
use std::io;
use std::process::Command;

// Run command and capture output
fn run_command() -> io::Result<()> {
    let output = Command::new("echo")
        .arg("Hello from subprocess!")
        .output()?;  // Waits for completion, captures all output

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

// Check if command succeeded
fn run_command_check() -> io::Result<()> {
    let status = Command::new("true")  // Unix command that always succeeds
        .status()?;  // Inherits stdin/stdout/stderr, just waits for completion

    if status.success() {
        println!("Command succeeded!");
    } else {
        println!("Command failed with: {}", status);
    }

    // Also try a failing command
    let status = Command::new("false")  // Unix command that always fails
        .status()?;

    if status.success() {
        println!("Command succeeded!");
    } else {
        println!("Command failed (expected) with exit code: {:?}", status.code());
    }

    Ok(())
}

// Run with environment variables
fn run_with_env() -> io::Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo MY_VAR=$MY_VAR, ANOTHER=$ANOTHER_VAR")
        .env("MY_VAR", "my_value")
        .env("ANOTHER_VAR", "another_value")
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

// Run in specific directory
fn run_in_directory() -> io::Result<()> {
    let output = Command::new("pwd")
        .current_dir("/tmp")
        .output()?;

    println!("Working directory: {}", String::from_utf8_lossy(&output.stdout).trim());
    Ok(())
}

// Run ls command (cross-platform alternative)
fn list_current_directory() -> io::Result<()> {
    #[cfg(unix)]
    let output = Command::new("ls")
        .arg("-la")
        .output()?;

    #[cfg(windows)]
    let output = Command::new("cmd")
        .args(["/C", "dir"])
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn main() -> io::Result<()> {
    println!("=== Basic Process Spawning Demo ===\n");

    // Run command and capture output
    println!("=== run_command (capture output) ===");
    run_command()?;

    // Check if command succeeded
    println!("\n=== run_command_check (exit status) ===");
    run_command_check()?;

    // Run with environment variables
    println!("\n=== run_with_env ===");
    run_with_env()?;

    // Run in specific directory
    println!("=== run_in_directory ===");
    run_in_directory()?;

    // List current directory
    println!("\n=== list_current_directory (first 10 lines) ===");
    let output = Command::new("ls")
        .arg("-la")
        .output()?;
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout_str
        .lines()
        .take(10)
        .collect();
    for line in lines {
        println!("{}", line);
    }
    println!("...");

    println!("\nBasic process spawning examples completed");
    Ok(())
}
