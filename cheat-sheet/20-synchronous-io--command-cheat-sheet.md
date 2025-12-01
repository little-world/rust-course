
### Command Cheat Sheet

```rust
use std::process::{Command, Stdio, Child, ExitStatus, Output};
use std::io::{Write, BufRead, BufReader};

// Basic command execution
Command::new("ls")                                   // Create new command
    .spawn()?                                        // Spawn child process, returns Child
Command::new("ls").output()?                        // Run and capture output, returns Output
Command::new("ls").status()?                        // Run and get exit status

// Command arguments
Command::new("echo")
    .arg("hello")                                    // Add single argument
    .args(&["hello", "world"])                      // Add multiple arguments
    .args(vec!["a", "b", "c"])                      // From vec
    .spawn()?

// Environment variables
Command::new("program")
    .env("KEY", "value")                            // Set single env var
    .envs(vec![("K1", "V1"), ("K2", "V2")])        // Set multiple env vars
    .env_remove("PATH")                             // Remove env var
    .env_clear()                                    // Clear all env vars
    .spawn()?

// Working directory
Command::new("ls")
    .current_dir("/tmp")                            // Set working directory
    .spawn()?

// Standard I/O configuration
Command::new("program")
    .stdin(Stdio::null())                           // No stdin
    .stdin(Stdio::inherit())                        // Inherit from parent
    .stdin(Stdio::piped())                          // Create pipe
    .stdout(Stdio::null())                          // No stdout
    .stdout(Stdio::inherit())                       // Inherit from parent
    .stdout(Stdio::piped())                         // Create pipe
    .stderr(Stdio::null())                          // No stderr
    .stderr(Stdio::inherit())                       // Inherit from parent
    .stderr(Stdio::piped())                         // Create pipe
    .spawn()?

// Child process methods
let mut child = Command::new("sleep").arg("5").spawn()?;
child.id()                                          // Get process ID
child.kill()?                                       // Kill process
child.wait()?                                       // Wait for completion, returns ExitStatus
child.try_wait()?                                   // Non-blocking wait, returns Option<ExitStatus>
child.wait_with_output()?                          // Wait and capture output

// Accessing child stdio
let mut child = Command::new("cat")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

let stdin = child.stdin.as_mut().unwrap();          // Get stdin handle
stdin.write_all(b"data")?;                          // Write to child's stdin
drop(stdin)                                          // Close stdin (important!)

let stdout = child.stdout.take().unwrap();          // Take ownership of stdout
let reader = BufReader::new(stdout);
for line in reader.lines() {
    println!("{}", line?);
}

child.wait()?;

// Output struct methods
let output = Command::new("echo").arg("hello").output()?;
output.status                                        // ExitStatus
output.stdout                                        // Vec<u8>
output.stderr                                        // Vec<u8>
String::from_utf8_lossy(&output.stdout)             // Convert to string

// ExitStatus methods
let status = Command::new("ls").status()?;
status.success()                                     // Check if exit code 0
status.code()                                        // Get exit code as Option<i32>
status.exit_ok()?                                    // Return Ok(()) if success, Err otherwise

// Unix-specific (ExitStatusExt trait)
#[cfg(unix)]
{
    use std::os::unix::process::ExitStatusExt;
    status.signal()                                  // Get signal if terminated by signal
    status.core_dumped()                            // Check if core dumped
    status.stopped_signal()                         // Get stop signal
    status.continued()                              // Check if continued
}

// Running shell commands
#[cfg(unix)]
Command::new("sh")
    .arg("-c")
    .arg("ls | grep txt")
    .output()?

#[cfg(windows)]
Command::new("cmd")
    .args(&["/C", "dir"])
    .output()?

// Piping between processes
let process1 = Command::new("ls")
    .stdout(Stdio::piped())
    .spawn()?;

let process2 = Command::new("grep")
    .arg("txt")
    .stdin(process1.stdout.unwrap())
    .stdout(Stdio::piped())
    .spawn()?;

let output = process2.wait_with_output()?;

// Common patterns
// Capture output as string
let output = Command::new("echo")
    .arg("hello")
    .output()?;
let stdout = String::from_utf8_lossy(&output.stdout);
let stderr = String::from_utf8_lossy(&output.stderr);

// Check if command succeeded
let status = Command::new("ls").status()?;
if status.success() {
    println!("Command succeeded");
} else {
    eprintln!("Command failed with: {}", status);
}

// Write to stdin and read from stdout
let mut child = Command::new("cat")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

{
    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(b"Hello from Rust\n")?;
}

let output = child.wait_with_output()?;
println!("{}", String::from_utf8_lossy(&output.stdout));

// Stream output line by line
let mut child = Command::new("ping")
    .arg("localhost")
    .stdout(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().unwrap();
let reader = BufReader::new(stdout);

for line in reader.lines() {
    println!("Output: {}", line?);
}

// Run with timeout (requires external crate or manual implementation)
use std::time::Duration;
use std::thread;

let mut child = Command::new("sleep").arg("10").spawn()?;
let timeout = Duration::from_secs(2);

thread::sleep(timeout);
match child.try_wait()? {
    Some(status) => println!("Exited with: {}", status),
    None => {
        child.kill()?;
        println!("Killed after timeout");
    }
}

// Execute and get exit code
let code = Command::new("false")
    .status()?
    .code()
    .unwrap_or(-1);
println!("Exit code: {}", code);

// Redirect stderr to stdout
let output = Command::new("ls")
    .arg("/nonexistent")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()?;

// Builder pattern for complex commands
let output = Command::new("gcc")
    .args(&["-o", "program", "main.c"])
    .current_dir("/tmp")
    .env("CC", "clang")
    .output()?;

// Spawning multiple processes
let mut children = vec![];
for i in 0..5 {
    let child = Command::new("echo")
        .arg(format!("Process {}", i))
        .spawn()?;
    children.push(child);
}

for mut child in children {
    child.wait()?;
}

// Conditional execution based on OS
#[cfg(target_os = "linux")]
let output = Command::new("ps").arg("aux").output()?;

#[cfg(target_os = "windows")]
let output = Command::new("tasklist").output()?;

#[cfg(target_os = "macos")]
let output = Command::new("ps").arg("aux").output()?;

// Execute script file
#[cfg(unix)]
Command::new("bash")
    .arg("script.sh")
    .spawn()?;

#[cfg(windows)]
Command::new("cmd")
    .args(&["/C", "script.bat"])
    .spawn()?;

// Detached process (Unix)
#[cfg(unix)]
{
    use std::os::unix::process::CommandExt;
    Command::new("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
}

// Set process group (Unix)
#[cfg(unix)]
{
    use std::os::unix::process::CommandExt;
    Command::new("program")
        .process_group(0)                            // Create new process group
        .spawn()?;
}

// Set user/group (Unix, requires privileges)
#[cfg(unix)]
{
    use std::os::unix::process::CommandExt;
    Command::new("program")
        .uid(1000)                                   // Set user ID
        .gid(1000)                                   // Set group ID
        .spawn()?;
}

// Execute and replace current process (Unix only)
#[cfg(unix)]
{
    use std::os::unix::process::CommandExt;
    let error = Command::new("ls").exec();          // Never returns on success
    eprintln!("Failed to exec: {}", error);
}

// Real-time output streaming with both stdout and stderr
let mut child = Command::new("program")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().unwrap();
let stderr = child.stderr.take().unwrap();

let stdout_thread = thread::spawn(move || {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        println!("STDOUT: {}", line.unwrap());
    }
});

let stderr_thread = thread::spawn(move || {
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        eprintln!("STDERR: {}", line.unwrap());
    }
});

stdout_thread.join().unwrap();
stderr_thread.join().unwrap();
child.wait()?;

// Check if command exists
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// Run command and ignore errors
let _ = Command::new("optional_tool").status();

// Chain commands with AND logic
let status1 = Command::new("cmd1").status()?;
if status1.success() {
    let status2 = Command::new("cmd2").status()?;
}
```