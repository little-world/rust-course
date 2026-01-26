
# Build System Pipeline Executor

### Problem Statement

Build a simple but functional build system that executes compilation pipelines, handles process orchestration, and captures/displays output. You'll implement command execution with process spawning, pipe multiple commands together like Unix pipelines, handle concurrent stdout/stderr streams without deadlocks, and provide a developer-friendly interface with colored output and progress reporting.

### Use Cases

**When you need this pattern**:
1. **Build automation**: Compile, test, and package software projects
2. **CI/CD pipelines**: Execute sequential build steps with dependency tracking
3. **Task runners**: Execute development tasks (lint, format, test)
4. **Compilation orchestration**: Parallel compilation of multiple modules
5. **Test execution**: Run test suites with output capture and reporting
6. **Deployment scripts**: Execute deployment commands with error handling

### Why It Matters

**Real-World Impact**: Build systems are essential to software development:

**The Manual Build Problem**:
```bash
# Manual build process - error-prone and slow
$ rustc examples/lib.rs
$ rustc examples/main.rs --extern mylib=libmylib.rlib
$ cargo test
# Problems:
# - Must remember order of commands
# - Errors buried in output
# - Can't parallelize
# - No progress indication
# - Output not captured for analysis
```

**Build System Benefits**:
- **Automation**: One command builds entire project
- **Parallelization**: Compile independent modules concurrently
- **Error reporting**: Parse and highlight errors/warnings
- **Caching**: Skip unchanged files (not in this project, but enabled by it)
- **Reproducibility**: Same build process every time
- **Progress**: Show what's being built in real-time

**How Build Systems Work**:
```
Build Pipeline:
  ┌─────────────┐
  │  Task: fmt  │ (Format code)
  └──────┬──────┘
         │
  ┌──────▼──────┐
  │ Task: build │ (Compile)
  └──────┬──────┘
         │
  ┌──────▼──────┐
  │ Task: test  │ (Run tests)
  └──────┬──────┘
         │
  ┌──────▼──────┐
  │Task: deploy │ (Deploy)
  └─────────────┘

Parallel Execution:
  ┌──────────┐   ┌──────────┐   ┌──────────┐
  │ Module A │   │ Module B │   │ Module C │
  └─────┬────┘   └─────┬────┘   └─────┬────┘
        └──────────┬───────────────────┘
               ┌───▼────┐
               │  Link  │
               └────────┘
```

**Critical Problems Build Systems Solve**:

1. **Pipe Deadlock**:
```rust
// WRONG - This deadlocks!
let mut child = Command::new("rustc")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Waiting for process to finish while buffers full
child.wait()?; // DEADLOCK if stdout/stderr fill up (64KB buffer)

// Read output - but process is already finished
let stdout = child.stdout.take().unwrap();
```

2. **Concurrent Stream Reading**:
```rust
// RIGHT - Read stdout/stderr concurrently
let mut child = Command::new("rustc")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Spawn threads to read both streams simultaneously
let stdout_thread = thread::spawn(|| read_stream(child.stdout.take()));
let stderr_thread = thread::spawn(|| read_stream(child.stderr.take()));

child.wait()?; // Won't deadlock
let stdout = stdout_thread.join()?;
let stderr = stderr_thread.join()?;
```

**Performance Characteristics**:
- **Sequential**: 3 tasks × 10s = 30 seconds total
- **Parallel**: max(10s, 10s, 10s) = 10 seconds total
- **3x speedup** from parallelization

### Learning Goals

By completing this project, you will:

1. **Master process spawning**: `Command`, `spawn()`, `wait()`, exit codes
2. **Understand pipe mechanics**: stdin/stdout/stderr piping
3. **Avoid deadlocks**: Concurrent stream reading patterns
4. **Handle timeouts**: Kill hung processes
5. **Parse command output**: Extract errors/warnings from compiler output
6. **Colorize terminal output**: ANSI color codes for better UX
7. **Orchestrate pipelines**: Execute dependent tasks in order

---

### Milestone 1: Basic Command Execution

**Goal**: Execute single commands and capture output.

**Implementation Steps**:

1. **Implement basic command execution**:
   - Use `Command::new()` to create command
   - Set working directory with `.current_dir()`
   - Set environment variables with `.env()`
   - Capture stdout/stderr with `.output()`

2. **Parse exit codes**:
   - Check `status.success()`
   - Get exit code with `status.code()`
   - Distinguish success, failure, and signal termination

3. **Display output**:
   - Print stdout and stderr
   - Preserve command-line output formatting
   - Handle non-UTF8 output gracefully

4. **Error handling**:
   - Command not found
   - Permission denied
   - Working directory doesn't exist

**Starter Code**:

```rust
use std::process::{Command, Output, ExitStatus};
use std::path::Path;
use std::io;
use std::collections::HashMap;

/// Execute a command and capture output
pub fn execute_command(
    program: &str,
    args: &[&str],
    working_dir: Option<&Path>,
    env_vars: &HashMap<String, String>,
) -> io::Result<Output> {
    // TODO: Create command
    let mut cmd = Command::new(program);

    // TODO: Add arguments
    // Hint: cmd.args(args);

    // TODO: Set working directory if provided
    // Hint: if let Some(dir) = working_dir { cmd.current_dir(dir); }

    // TODO: Add environment variables
    // Hint: for (key, val) in env_vars { cmd.env(key, val); }

    // TODO: Execute and capture output
    // Hint: cmd.output()

    todo!()
}

/// Check if command succeeded
pub fn check_success(output: &Output) -> bool {
    // TODO: Check exit status
    // Hint: output.status.success()
    todo!()
}

/// Get exit code from output
pub fn get_exit_code(output: &Output) -> Option<i32> {
    // TODO: Return exit code
    // Hint: output.status.code()
    todo!()
}

/// Print command output to console
pub fn print_output(output: &Output) {
    // TODO: Print stdout
    // Hint: println!("{}", String::from_utf8_lossy(&output.stdout));

    // TODO: Print stderr
    // Hint: eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    todo!()
}
```

**Checkpoint Tests**:
```rust
use std::collections::HashMap;

#[test]
fn test_execute_simple_command() {
    let output = execute_command(
        "echo",
        &["Hello, World!"],
        None,
        &HashMap::new(),
    )
    .unwrap();

    assert!(check_success(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World!"));
}

#[test]
fn test_command_with_args() {
    let output = execute_command(
        "ls",
        &["-la"],
        Some(Path::new(".")),
        &HashMap::new(),
    )
    .unwrap();

    assert!(check_success(&output));
}

#[test]
fn test_command_failure() {
    let output = execute_command(
        "ls",
        &["/nonexistent/path"],
        None,
        &HashMap::new(),
    )
    .unwrap();

    assert!(!check_success(&output));
    assert!(get_exit_code(&output).unwrap() != 0);
}

#[test]
fn test_working_directory() {
    let output = execute_command(
        "pwd",
        &[],
        Some(Path::new("/tmp")),
        &HashMap::new(),
    )
    .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tmp"));
}

#[test]
fn test_environment_variables() {
    let mut env = HashMap::new();
    env.insert("MY_VAR".to_string(), "test_value".to_string());

    let output = execute_command(
        "sh",
        &["-c", "echo $MY_VAR"],
        None,
        &env,
    )
    .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test_value"));
}

#[test]
fn test_command_not_found() {
    let result = execute_command(
        "nonexistent_command_xyz",
        &[],
        None,
        &HashMap::new(),
    );

    assert!(result.is_err());
}
```

**Check Your Understanding**:
- What's the difference between `.output()` and `.spawn()`?
- Why does `.output()` wait for the command to complete?
- What happens if stdout buffer fills up?
- How do we distinguish command failure from command not found?

---

### Milestone 2: Streaming Output with Concurrent Reading

**Goal**: Execute commands and stream output in real-time without deadlocks.

**Implementation Steps**:

1. **Implement streaming execution**:
   - Use `.spawn()` instead of `.output()`
   - Set `.stdout(Stdio::piped())` and `.stderr(Stdio::piped())`
   - Take ownership of stdout/stderr handles

2. **Concurrent stream reading**:
   - Spawn thread for stdout reader
   - Spawn thread for stderr reader
   - Read streams line-by-line using `BufReader::lines()`
   - Join threads after process completes

3. **Display output in real-time**:
   - Print each line as it's received
   - Distinguish stdout and stderr (optional: different colors)
   - Flush output immediately

4. **Avoid deadlocks**:
   - Never wait for process while holding stream handles
   - Always read both stdout and stderr concurrently
   - Handle case where child writes more than pipe buffer (64KB)

**Starter Code Extension**:

```rust
use std::process::{Child, Stdio, ChildStdout, ChildStderr};
use std::io::{BufReader, BufRead};
use std::thread;

/// Execute command with streaming output
pub fn execute_streaming(
    program: &str,
    args: &[&str],
    working_dir: Option<&Path>,
) -> io::Result<ExecutionResult> {
    // TODO: Create command with piped stdout/stderr
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // TODO: Spawn child process
    let mut child = cmd.spawn()?;

    // TODO: Take stdout and stderr handles
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // TODO: Spawn threads to read streams concurrently
    let stdout_thread = thread::spawn(move || read_stream(stdout, "stdout"));
    let stderr_thread = thread::spawn(move || read_stream(stderr, "stderr"));

    // TODO: Wait for process to complete
    let status = child.wait()?;

    // TODO: Join reader threads
    let stdout_lines = stdout_thread.join().unwrap();
    let stderr_lines = stderr_thread.join().unwrap();

    Ok(ExecutionResult {
        status,
        stdout: stdout_lines,
        stderr: stderr_lines,
    })
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub status: ExitStatus,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

/// Read stream line-by-line
fn read_stream<R: io::Read>(stream: R, label: &str) -> Vec<String> {
    // TODO: Create BufReader
    // TODO: Read lines and print them
    // TODO: Collect lines into Vec

    let reader = BufReader::new(stream);
    let mut lines = Vec::new();

    for line in reader.lines() {
        match line {
            Ok(line) => {
                // Print immediately for streaming output
                println!("[{}] {}", label, line);
                lines.push(line);
            }
            Err(e) => {
                eprintln!("Error reading stream: {}", e);
                break;
            }
        }
    }

    lines
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_streaming_output() {
    let result = execute_streaming(
        "echo",
        &["Line 1\nLine 2\nLine 3"],
        None,
    )
    .unwrap();

    assert!(result.status.success());
    assert!(result.stdout.len() >= 1);
}

#[test]
fn test_stderr_capture() {
    // Command that writes to stderr
    let result = execute_streaming(
        "sh",
        &["-c", "echo error >&2"],
        None,
    )
    .unwrap();

    assert!(result.stderr.iter().any(|line| line.contains("error")));
}

#[test]
fn test_large_output_no_deadlock() {
    // Generate output larger than pipe buffer (>64KB)
    let result = execute_streaming(
        "sh",
        &["-c", "for i in {1..10000}; do echo $i; done"],
        None,
    )
    .unwrap();

    assert!(result.status.success());
    assert!(result.stdout.len() > 1000);
}

#[test]
fn test_concurrent_stdout_stderr() {
    // Command that writes to both stdout and stderr
    let result = execute_streaming(
        "sh",
        &["-c", "echo stdout; echo stderr >&2; echo more stdout"],
        None,
    )
    .unwrap();

    assert!(!result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
}

#[test]
fn test_exit_code_capture() {
    let result = execute_streaming(
        "sh",
        &["-c", "exit 42"],
        None,
    )
    .unwrap();

    assert_eq!(result.status.code(), Some(42));
}
```

**Check Your Understanding**:
- Why spawn threads for stdout and stderr?
- What happens if we only read stdout but child writes to stderr?
- Why use `BufReader::lines()` instead of `read_to_string()`?
- What's the pipe buffer size and why does it matter?

---

### Milestone 3: Process Piping (Command Chaining)

**Goal**: Pipe output from one command to input of another (like `cat | grep | wc`).

**Implementation Steps**:

1. **Implement simple pipe**:
   - First command: `.stdout(Stdio::piped())`
   - Second command: `.stdin(first_child.stdout.take())`
   - Chain commands together

2. **Handle multi-stage pipelines**:
   - Support arbitrary number of commands
   - Pass output from each stage to next
   - Collect final output

3. **Error handling in pipelines**:
   - If early stage fails, stop pipeline
   - Collect exit codes from all stages
   - Report which stage failed

4. **Implement pipeline builder API**:
   - Fluent API for building pipelines
   - `Pipeline::new().add("cat", &["file.txt"]).add("grep", &["pattern"]).execute()`

**Starter Code Extension**:

```rust
/// Execute pipeline of commands (cmd1 | cmd2 | cmd3)
pub fn execute_pipeline(commands: &[(&str, Vec<&str>)]) -> io::Result<ExecutionResult> {
    // TODO: Check if commands is empty
    if commands.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Empty pipeline"));
    }

    // TODO: Spawn first command
    let mut children = Vec::new();
    let mut previous_stdout: Option<Stdio> = None;

    for (i, (program, args)) in commands.iter().enumerate() {
        let mut cmd = Command::new(program);
        cmd.args(args);

        // TODO: Set stdin from previous command's stdout
        if let Some(stdout) = previous_stdout {
            cmd.stdin(stdout);
        }

        // TODO: Pipe stdout to next command (except last)
        if i < commands.len() - 1 {
            cmd.stdout(Stdio::piped());
        } else {
            cmd.stdout(Stdio::piped()); // Capture final output
        }

        cmd.stderr(Stdio::piped());

        // TODO: Spawn child
        let mut child = cmd.spawn()?;

        // TODO: Take stdout for next command
        previous_stdout = child.stdout.take().map(Stdio::from);

        children.push(child);
    }

    // TODO: Wait for all children and collect output
    let mut results = Vec::new();

    for mut child in children {
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let status = child.wait()?;

        // Read remaining output
        let stdout_lines = stdout.map(|s| read_stream(s, "stdout")).unwrap_or_default();
        let stderr_lines = stderr.map(|s| read_stream(s, "stderr")).unwrap_or_default();

        results.push((status, stdout_lines, stderr_lines));
    }

    // TODO: Return result from final command
    let (status, stdout, stderr) = results.pop().unwrap();

    Ok(ExecutionResult { status, stdout, stderr })
}

/// Builder for command pipelines
pub struct Pipeline {
    commands: Vec<(String, Vec<String>)>,
}

impl Pipeline {
    pub fn new() -> Self {
        // TODO: Create empty pipeline
        todo!()
    }

    pub fn add(mut self, program: &str, args: &[&str]) -> Self {
        // TODO: Add command to pipeline
        // Hint: self.commands.push((program.to_string(), args.iter()...));
        todo!()
    }

    pub fn execute(self) -> io::Result<ExecutionResult> {
        // TODO: Convert to command slice and execute
        // TODO: Call execute_pipeline()
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_simple_pipe() {
    // echo "hello" | grep "hello"
    let result = execute_pipeline(&[
        ("echo", vec!["hello\nworld"]),
        ("grep", vec!["hello"]),
    ])
    .unwrap();

    assert!(result.status.success());
    assert!(result.stdout.iter().any(|line| line.contains("hello")));
}

#[test]
fn test_three_stage_pipeline() {
    // echo "..." | grep "..." | wc -l
    let result = execute_pipeline(&[
        ("echo", vec!["line1\nline2\nline3"]),
        ("grep", vec!["line"]),
        ("wc", vec!["-l"]),
    ])
    .unwrap();

    assert!(result.status.success());
    let output = result.stdout.join("\n");
    assert!(output.contains("3"));
}

#[test]
fn test_pipeline_early_failure() {
    // false | echo "should not run"
    let result = execute_pipeline(&[
        ("false", vec![]),
        ("echo", vec!["should not run"]),
    ])
    .unwrap();

    // First command failed
    // (Note: behavior depends on shell settings)
}

#[test]
fn test_pipeline_builder() {
    let result = Pipeline::new()
        .add("echo", &["test"])
        .add("grep", &["test"])
        .execute()
        .unwrap();

    assert!(result.status.success());
}

#[test]
fn test_cat_grep_wc_pipeline() {
    // Create test file
    use std::fs;
    fs::write("/tmp/test_pipe.txt", "apple\nbanana\napple\ncherry\n").unwrap();

    // cat test.txt | grep apple | wc -l
    let result = execute_pipeline(&[
        ("cat", vec!["/tmp/test_pipe.txt"]),
        ("grep", vec!["apple"]),
        ("wc", vec!["-l"]),
    ])
    .unwrap();

    let output = result.stdout.join("\n");
    assert!(output.contains("2")); // Two lines with "apple"
}
```

**Check Your Understanding**:
- How do we connect stdout of one process to stdin of another?
- Why use `Stdio::from(child.stdout.take())`?
- What happens if middle command in pipeline fails?
- How would we implement pipeline error propagation?

---

### Milestone 4: Timeout Handling and Process Control

**Goal**: Kill hung processes and enforce time limits.

**Implementation Steps**:

1. **Implement timeout mechanism**:
   - Use `thread::spawn` with timeout check
   - Use `Child::try_wait()` to check if process finished
   - Kill process if timeout exceeded

2. **Implement process killing**:
   - Use `Child::kill()` to terminate process
   - Handle case where process already exited
   - Clean up zombie processes

3. **Graceful vs forceful termination**:
   - Send SIGTERM first (Unix only)
   - Wait grace period
   - Send SIGKILL if still running

4. **Return timeout error**:
   - Distinguish timeout from other errors
   - Include partial output captured before timeout

**Starter Code Extension**:

```rust
use std::time::{Duration, Instant};

/// Execute command with timeout
pub fn execute_with_timeout(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> io::Result<ExecutionResult> {
    // TODO: Spawn command
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    // TODO: Start timeout timer
    let start = Instant::now();

    // TODO: Take streams for concurrent reading
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // TODO: Spawn reader threads
    let stdout_thread = thread::spawn(move || read_stream(stdout, "stdout"));
    let stderr_thread = thread::spawn(move || read_stream(stderr, "stderr"));

    // TODO: Poll for completion with timeout
    loop {
        // Check if process finished
        match child.try_wait()? {
            Some(status) => {
                // Process finished
                let stdout_lines = stdout_thread.join().unwrap();
                let stderr_lines = stderr_thread.join().unwrap();

                return Ok(ExecutionResult {
                    status,
                    stdout: stdout_lines,
                    stderr: stderr_lines,
                });
            }
            None => {
                // Process still running, check timeout
                if start.elapsed() > timeout {
                    // Timeout! Kill process
                    eprintln!("Process timed out after {:?}, killing...", timeout);
                    child.kill()?;

                    // Wait for process to die
                    let status = child.wait()?;

                    // Get partial output
                    let stdout_lines = stdout_thread.join().unwrap();
                    let stderr_lines = stderr_thread.join().unwrap();

                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!("Command timed out after {:?}", timeout),
                    ));
                }

                // Sleep briefly before next check
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Kill process tree (Unix only)
#[cfg(unix)]
pub fn kill_process_tree(pid: u32) -> io::Result<()> {
    // TODO: Send SIGTERM to process group
    // Hint: Use nix crate or libc::kill
    // For simplicity, just kill the main process

    use std::process::Command;
    Command::new("kill")
        .arg(pid.to_string())
        .output()?;

    Ok(())
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_successful_within_timeout() {
    let result = execute_with_timeout(
        "echo",
        &["hello"],
        Duration::from_secs(5),
    )
    .unwrap();

    assert!(result.status.success());
}

#[test]
fn test_timeout_kills_process() {
    // Sleep for 10 seconds, but timeout after 1 second
    let result = execute_with_timeout(
        "sleep",
        &["10"],
        Duration::from_secs(1),
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
}

#[test]
fn test_fast_command_no_timeout() {
    let start = Instant::now();

    let result = execute_with_timeout(
        "echo",
        &["fast"],
        Duration::from_secs(10),
    )
    .unwrap();

    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(result.status.success());
}

#[test]
fn test_partial_output_on_timeout() {
    // Command that outputs then hangs
    // sh -c "echo start; sleep 10; echo end"
    // Should see "start" but timeout before "end"

    let result = execute_with_timeout(
        "sh",
        &["-c", "echo start; sleep 10; echo end"],
        Duration::from_secs(1),
    );

    // Should timeout but might have partial output
    assert!(result.is_err());
}
```

**Check Your Understanding**:
- Why use `try_wait()` in a loop instead of just `wait()`?
- What's the difference between `kill()` and SIGTERM?
- How do we prevent zombie processes?
- Why might `kill()` fail?

---

### Milestone 5: Build System with Task Dependencies

**Goal**: Create complete build system with task dependencies, parallel execution, and colored output.

**Implementation Steps**:

1. **Define task structure**:
   - Task name, command, dependencies
   - Working directory, environment variables
   - Success/failure status

2. **Implement dependency resolution**:
   - Build directed acyclic graph (DAG)
   - Topological sort for execution order
   - Detect circular dependencies

3. **Parallel task execution**:
   - Execute independent tasks concurrently
   - Use thread pool or spawn threads
   - Wait for dependencies before starting task

4. **Parse compiler output**:
   - Regex patterns for errors/warnings
   - Extract file name, line number, message
   - Categorize by severity

5. **Colorize output**:
   - ANSI color codes for errors (red), warnings (yellow)
   - Progress indicators
   - Success/failure summary

**Complete Implementation**:

```rust
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;

/// A build task
#[derive(Clone)]
pub struct Task {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub dependencies: Vec<String>,
    pub working_dir: Option<String>,
}

impl Task {
    pub fn new(name: &str, command: &str, args: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            dependencies: Vec::new(),
            working_dir: None,
        }
    }

    pub fn with_dependencies(mut self, deps: &[&str]) -> Self {
        self.dependencies = deps.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }
}

/// Build system executor
pub struct BuildSystem {
    tasks: HashMap<String, Task>,
}

impl BuildSystem {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.name.clone(), task);
    }

    /// Execute all tasks respecting dependencies
    pub fn execute_all(&self) -> io::Result<()> {
        // TODO: Build execution order (topological sort)
        let order = self.topological_sort()?;

        println!("Execution order: {:?}", order);

        // TODO: Execute tasks in order
        for task_name in &order {
            self.execute_task(task_name)?;
        }

        Ok(())
    }

    /// Execute specific task and its dependencies
    pub fn execute_task(&self, task_name: &str) -> io::Result<()> {
        let task = self.tasks.get(task_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Task not found"))?;

        // TODO: Execute dependencies first
        for dep in &task.dependencies {
            self.execute_task(dep)?;
        }

        // TODO: Execute task
        println!("{} Running task: {}", colorize("→", Color::Blue), task_name);

        let working_dir = task.working_dir.as_ref().map(|s| Path::new(s));
        let args: Vec<&str> = task.args.iter().map(|s| s.as_str()).collect();

        let result = execute_streaming(&task.command, &args, working_dir)?;

        if result.status.success() {
            println!("{} Task {} completed successfully",
                colorize("✓", Color::Green),
                task_name
            );
        } else {
            eprintln!("{} Task {} failed with exit code {:?}",
                colorize("✗", Color::Red),
                task_name,
                result.status.code()
            );

            // Parse and colorize errors
            for line in &result.stderr {
                if is_error(line) {
                    eprintln!("{}", colorize(line, Color::Red));
                } else if is_warning(line) {
                    eprintln!("{}", colorize(line, Color::Yellow));
                } else {
                    eprintln!("{}", line);
                }
            }

            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Task {} failed", task_name)
            ));
        }

        Ok(())
    }

    /// Topological sort for task execution order
    fn topological_sort(&self) -> io::Result<Vec<String>> {
        // TODO: Implement Kahn's algorithm or DFS-based toposort

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Build graph
        for (name, task) in &self.tasks {
            in_degree.entry(name.clone()).or_insert(0);
            graph.entry(name.clone()).or_insert(Vec::new());

            for dep in &task.dependencies {
                *in_degree.entry(name.clone()).or_insert(0) += 1;
                graph.entry(dep.clone()).or_insert(Vec::new()).push(name.clone());
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<String> = in_degree.iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node.clone());

            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.tasks.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Circular dependency detected"
            ));
        }

        Ok(result)
    }
}

/// ANSI color codes
#[derive(Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
}

impl Color {
    fn code(&self) -> &str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
        }
    }

    fn reset() -> &'static str {
        "\x1b[0m"
    }
}

pub fn colorize(text: &str, color: Color) -> String {
    format!("{}{}{}", color.code(), text, Color::reset())
}

/// Check if line contains error
fn is_error(line: &str) -> bool {
    line.contains("error:") || line.contains("ERROR") || line.contains("Error")
}

/// Check if line contains warning
fn is_warning(line: &str) -> bool {
    line.contains("warning:") || line.contains("WARNING") || line.contains("Warning")
}

/// Parse compiler error/warning
pub struct CompilerMessage {
    pub file: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: Severity,
    pub message: String,
}

pub enum Severity {
    Error,
    Warning,
    Info,
}

pub fn parse_compiler_output(line: &str) -> Option<CompilerMessage> {
    // TODO: Parse patterns like:
    // "examples/main.rs:10:5: error: expected `;`"
    // "warning: unused variable: `x`"

    // Simplified regex pattern
    // Real implementation would use regex crate

    todo!()
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_simple_build_system() {
    let mut build = BuildSystem::new();

    build.add_task(Task::new("clean", "rm", &["-rf", "target"]));
    build.add_task(
        Task::new("build", "cargo", &["build"])
            .with_dependencies(&["clean"])
    );
    build.add_task(
        Task::new("test", "cargo", &["test"])
            .with_dependencies(&["build"])
    );

    // Execute specific task
    let result = build.execute_task("test");
    // Should execute: clean -> build -> test
}

#[test]
fn test_parallel_execution() {
    let mut build = BuildSystem::new();

    // Independent tasks can run in parallel
    build.add_task(Task::new("fmt", "cargo", &["fmt"]));
    build.add_task(Task::new("clippy", "cargo", &["clippy"]));
    build.add_task(
        Task::new("test", "cargo", &["test"])
            .with_dependencies(&["fmt", "clippy"])
    );

    build.execute_all().unwrap();
}

#[test]
fn test_circular_dependency_detection() {
    let mut build = BuildSystem::new();

    build.add_task(Task::new("a", "echo", &["a"]).with_dependencies(&["b"]));
    build.add_task(Task::new("b", "echo", &["b"]).with_dependencies(&["a"]));

    let result = build.execute_all();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Circular"));
}

#[test]
fn test_colorized_output() {
    let red_text = colorize("ERROR", Color::Red);
    assert!(red_text.contains("\x1b[31m"));
    assert!(red_text.contains("\x1b[0m"));
}

#[test]
fn test_error_detection() {
    assert!(is_error("error: expected `;`"));
    assert!(is_error("ERROR: File not found"));
    assert!(!is_error("Info: Starting build"));
}

#[test]
fn test_warning_detection() {
    assert!(is_warning("warning: unused variable"));
    assert!(is_warning("WARNING: Deprecated function"));
    assert!(!is_warning("error: syntax error"));
}
```

**Check Your Understanding**:
- How does topological sort determine execution order?
- Why use Kahn's algorithm instead of DFS?
- How do we detect circular dependencies?
- How would we implement parallel task execution?

---

### Complete Project Summary

**What You Built**:
1. Command execution with environment and working directory control
2. Streaming output with concurrent stdout/stderr reading
3. Process piping (Unix pipeline style)
4. Timeout handling and process killing
5. Build system with task dependencies and colorized output
6. Compiler output parsing and error highlighting

**Key Concepts Practiced**:
- Process spawning with `Command`
- Concurrent stream reading to avoid deadlocks
- Process piping and stdin/stdout/stderr handling
- Timeout mechanisms and process control
- DAG algorithms for dependency resolution
- ANSI color codes for terminal output

**Critical Patterns Learned**:
- **Avoid deadlocks**: Always read stdout/stderr concurrently
- **Pipe buffers**: 64KB limit requires concurrent reading
- **Timeout polling**: Use `try_wait()` in loop with sleep
- **Process cleanup**: Kill children, join threads, avoid zombies
- **Error propagation**: Distinguish errors by kind

**Real-World Applications**:
- Build systems (Cargo, Make, Bazel)
- CI/CD pipelines (GitHub Actions, GitLab CI)
- Task runners (npm scripts, just, task)
- Test frameworks (pytest, jest)
- Deployment automation

**Extension Ideas**:
1. **Caching**: Skip tasks if inputs unchanged
2. **Incremental builds**: Only rebuild changed modules
3. **Build server**: Remote execution with caching
4. **Distributed builds**: Execute tasks across machines
5. **Build visualization**: Show dependency graph
6. **Interactive mode**: Allow user to select tasks
7. **Watch mode**: Re-run on file changes
8. **Benchmarking**: Track task execution times
9. **Artifact management**: Store build outputs
10. **Hermetic builds**: Reproducible builds with locked deps

**Performance Optimizations**:
- Parallel execution of independent tasks
- Concurrent stream reading prevents blocking
- Buffered I/O reduces syscall overhead
- Early termination on first error (optional)

This project teaches the internals of build systems, process orchestration, and how tools like Cargo, Make, and CI systems work under the hood!
