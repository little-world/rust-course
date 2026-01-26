// Pattern 6: Process Streaming Output
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};

// Stream stdout in real-time
fn stream_output() -> io::Result<()> {
    // Use a command that produces multiple lines of output
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("for i in 1 2 3 4 5; do echo \"Line $i\"; sleep 0.1; done")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        println!("Output: {}", line?);
    }

    let status = child.wait()?;
    println!("Exit status: {}", status);

    Ok(())
}

// Capture both stdout and stderr separately (requires threading)
fn capture_both_streams() -> io::Result<()> {
    // Command that writes to both stdout and stderr
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("echo 'stdout line 1'; echo 'stderr line 1' >&2; echo 'stdout line 2'; echo 'stderr line 2' >&2")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Read stdout in one thread
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("[OUT] {}", line);
            }
        }
    });

    // Read stderr in another thread
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                eprintln!("[ERR] {}", line);
            }
        }
    });

    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    let status = child.wait()?;
    println!("Process exited with: {}", status);

    Ok(())
}

// Stream with timeout simulation
fn stream_with_progress() -> io::Result<()> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("echo 'Starting...'; sleep 0.2; echo 'Step 1 complete'; sleep 0.2; echo 'Step 2 complete'; sleep 0.2; echo 'Done!'")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    println!("Monitoring progress:");
    for (i, line) in reader.lines().enumerate() {
        println!("  [{:02}] {}", i + 1, line?);
    }

    let status = child.wait()?;
    println!("Completed with status: {}", status);

    Ok(())
}

fn main() -> io::Result<()> {
    println!("=== Process Streaming Demo ===\n");

    // Stream stdout in real-time
    println!("=== stream_output ===");
    stream_output()?;

    // Capture both streams with threads
    println!("\n=== capture_both_streams ===");
    capture_both_streams()?;

    // Stream with progress indication
    println!("\n=== stream_with_progress ===");
    stream_with_progress()?;

    println!("\nProcess streaming examples completed");
    Ok(())
}
