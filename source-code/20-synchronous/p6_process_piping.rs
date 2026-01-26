// Pattern 6: Process Piping
use std::io;
use std::process::{Command, Stdio};

// Pipe output from one command to another (ls | grep)
fn pipe_commands() -> io::Result<()> {
    // First command: echo some lines
    let echo = Command::new("sh")
        .arg("-c")
        .arg("echo -e 'file1.txt\\nfile2.rs\\nfile3.txt\\ndata.json\\ntest.txt'")
        .stdout(Stdio::piped())
        .spawn()?;

    // Second command: grep for txt files
    let grep = Command::new("grep")
        .arg("txt")
        .stdin(Stdio::from(echo.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    let output = grep.wait_with_output()?;
    println!("Files matching 'txt':");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

// Complex pipeline: generate | filter | count
fn complex_pipeline() -> io::Result<()> {
    // Generate some log-like data
    let generate = Command::new("sh")
        .arg("-c")
        .arg("echo -e 'INFO: start\\nERROR: failed\\nINFO: retry\\nERROR: timeout\\nINFO: success\\nERROR: crash'")
        .stdout(Stdio::piped())
        .spawn()?;

    // Filter for ERROR lines
    let grep = Command::new("grep")
        .arg("ERROR")
        .stdin(Stdio::from(generate.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    // Count lines
    let wc = Command::new("wc")
        .arg("-l")
        .stdin(Stdio::from(grep.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    let output = wc.wait_with_output()?;
    println!("Error count: {}", String::from_utf8_lossy(&output.stdout).trim());

    Ok(())
}

// Pipeline with data transformation
fn transform_pipeline() -> io::Result<()> {
    // Generate CSV-like data
    let data = Command::new("sh")
        .arg("-c")
        .arg("echo -e 'name,age,city\\nalice,30,new york\\nbob,25,boston\\ncharlie,35,chicago'")
        .stdout(Stdio::piped())
        .spawn()?;

    // Extract second column (age) using awk
    let awk = Command::new("awk")
        .arg("-F,")
        .arg("{print $2}")
        .stdin(Stdio::from(data.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    // Skip header and sort numerically
    let tail = Command::new("tail")
        .arg("-n")
        .arg("+2")  // Skip first line (header)
        .stdin(Stdio::from(awk.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    let sort = Command::new("sort")
        .arg("-n")
        .stdin(Stdio::from(tail.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    let output = sort.wait_with_output()?;
    println!("Ages sorted:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

// Bidirectional communication with subprocess
fn bidirectional_pipe() -> io::Result<()> {
    use std::io::{BufRead, BufReader, Write};

    // Start a cat process that echoes input
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();

    // Write to child's stdin
    writeln!(stdin, "Hello from parent!")?;
    writeln!(stdin, "Second line")?;
    writeln!(stdin, "Third line")?;
    drop(stdin);  // Close stdin to signal EOF

    // Read from child's stdout
    let reader = BufReader::new(stdout);
    println!("Child echoed:");
    for line in reader.lines() {
        println!("  {}", line?);
    }

    let status = child.wait()?;
    println!("Child exited with: {}", status);

    Ok(())
}

fn main() -> io::Result<()> {
    println!("=== Process Piping Demo ===\n");

    // Simple two-command pipe
    println!("=== pipe_commands (filter txt files) ===");
    pipe_commands()?;

    // Complex multi-stage pipeline
    println!("=== complex_pipeline (count ERROR lines) ===");
    complex_pipeline()?;

    // Data transformation pipeline
    println!("\n=== transform_pipeline (extract and sort ages) ===");
    transform_pipeline()?;

    // Bidirectional pipe
    println!("=== bidirectional_pipe ===");
    bidirectional_pipe()?;

    println!("\nProcess piping examples completed");
    Ok(())
}
