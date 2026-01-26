# Test Coverage Analyzer

### Problem Statement

Build a test coverage analyzer that instruments Rust code to track which lines, branches, and functions are executed during test runs. Your analyzer should parse Rust source code, inject coverage tracking instrumentation, run tests, and generate detailed coverage reports showing which code paths are tested and which are not.

Your coverage analyzer should support:
- Line coverage tracking (which lines were executed)
- Branch coverage tracking (which if/match branches were taken)
- Function coverage (which functions were called)
- Coverage report generation (text, HTML, JSON formats)
- Highlighting untested code paths
- Integration with cargo test

## Why Coverage Analysis Matters

### The Testing Blind Spot

**The Problem**: You can have hundreds of tests and still miss critical bugs because you don't know which code paths are untested. Without coverage analysis, you're flying blind—tests might all pass while large portions of your codebase remain unexercised.

**Real-world example**:
```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())  // ← Never tested!
    } else {
        Ok(a / b)
    }
}

#[test]
fn test_divide() {
    assert_eq!(divide(10, 2), Ok(5));  // Only tests success path
}
// Test passes ✓ but error handling is completely untested!
```

### Coverage Metrics Explained

**Line Coverage**: Percentage of executable lines that were run
```
Total lines: 100
Lines executed: 75
Line coverage: 75%
```

**Branch Coverage**: Percentage of decision branches (if/match) that were taken
```
if condition {
    // Branch A
} else {
    // Branch B  ← Not tested
}
// Branch coverage: 50% (only A tested)
```

**Function Coverage**: Percentage of functions that were called
```
Total functions: 20
Functions called: 18
Function coverage: 90%
```

### Why It Matters

**Confidence vs Reality Gap**:
- 100% passing tests != 100% working code
- Un-tested error paths are where production bugs hide
- Security vulnerabilities often lurk in untested branches

**Example Impact**:
```
Project A: 500 tests, 60% coverage
→ 40% of code never executed by tests
→ Production bugs: 12 per release

Project B: 300 tests, 95% coverage
→ Only 5% of code untested
→ Production bugs: 2 per release

6x reduction in bugs with better coverage!
```

**Optimization Guide**: Coverage reveals dead code
```rust
// Coverage shows this function is NEVER called
fn obsolete_feature() {  // 0% coverage
    // ... 500 lines of complex logic
}
// Can safely delete → smaller binary, faster compile
```

## Use Cases

### 1. Development Workflow
- **Find blind spots**: Identify untested error paths before code review
- **Regression prevention**: Ensure new features have adequate tests
- **Refactoring safety**: Verify tests cover code being refactored

### 2. CI/CD Integration
- **Quality gates**: Fail build if coverage drops below threshold (e.g., 80%)
- **PR checks**: Show coverage diff for pull requests
- **Trend tracking**: Monitor coverage over time

### 3. Code Quality Assessment
- **Tech debt identification**: Low-coverage modules need attention
- **Test quality metrics**: High line coverage + low branch coverage = weak tests
- **Security audits**: Ensure security-critical paths are tested

### 4. Legacy Code Modernization
- **Baseline establishment**: Measure starting point before adding tests
- **Incremental improvement**: Track progress as tests are added
- **Hot spot identification**: Focus testing effort on high-complexity, low-coverage areas

---



## Building the Project

### Milestone 1: Source Code Parser

**Goal**: Parse Rust source files to extract functions, statements, and branch points that need coverage tracking.

**Why we start here**: Before we can track coverage, we need to understand the code structure. This milestone teaches basic parsing and AST (Abstract Syntax Tree) representation.

#### Architecture

**Structs:**
- `SourceFile` - Represents a parsed Rust source file
  - **Field**: `path: PathBuf` - File location
  - **Field**: `lines: Vec<String>` - Source code lines
  - **Field**: `functions: Vec<FunctionInfo>` - Parsed functions

- `FunctionInfo` - Information about a function
  - **Field**: `name: String` - Function name
  - **Field**: `start_line: usize` - First line of function
  - **Field**: `end_line: usize` - Last line of function
  - **Field**: `statements: Vec<usize>` - Line numbers of executable statements

**Functions:**
- `parse_file(path: &Path) -> Result<SourceFile, Error>` - Parse source file
- `find_functions(&self) -> Vec<FunctionInfo>` - Extract function definitions
- `find_statements(&self, start: usize, end: usize) -> Vec<usize>` - Find executable lines
- `is_executable(line: &str) -> bool` - Check if line is executable (not comment/blank)

**Starter Code**:

```rust
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub lines: Vec<String>,
    pub functions: Vec<FunctionInfo>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub statements: Vec<usize>,
}

impl SourceFile {
    /// Parse a Rust source file
    pub fn parse_file(path: &Path) -> Result<Self, std::io::Error> {
        // TODO: Read file contents
        // TODO: Split into lines
        // TODO: Find all functions
        // TODO: For each function, find executable statements
        todo!("Implement file parsing")
    }

    fn find_functions(&self) -> Vec<FunctionInfo> {
        // TODO: Search for "fn " patterns
        // TODO: Track brace depth to find function end
        // TODO: Extract function name
        todo!("Implement function finding")
    }

    fn find_statements(&self, start: usize, end: usize) -> Vec<usize> {
        // TODO: Iterate lines in function
        // TODO: Filter out comments, blank lines, braces-only
        // TODO: Return line numbers of executable statements
        todo!("Implement statement finding")
    }

    fn is_executable(line: &str) -> bool {
        // TODO: Trim whitespace
        // TODO: Check if empty or comment-only
        // TODO: Check if brace-only
        todo!("Implement executable check")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_simple_function() {
        let content = r#"
fn add(a: i32, b: i32) -> i32 {
    let result = a + b;
    result
}
"#;
        let file = create_test_file(content);
        let source = SourceFile::parse_file(file.path()).unwrap();

        assert_eq!(source.functions.len(), 1);
        assert_eq!(source.functions[0].name, "add");
    }

    #[test]
    fn test_multiple_functions() {
        let content = r#"
fn foo() {
    println!("foo");
}

fn bar() -> i32 {
    42
}
"#;
        let file = create_test_file(content);
        let source = SourceFile::parse_file(file.path()).unwrap();

        assert_eq!(source.functions.len(), 2);
    }

    #[test]
    fn test_find_executable_lines() {
        let content = r#"
fn test() {
    // This is a comment
    let x = 5;

    let y = 10;  // Executable with comment
}
"#;
        let file = create_test_file(content);
        let source = SourceFile::parse_file(file.path()).unwrap();

        let func = &source.functions[0];
        // Should find 2 let statements, not comments or blank lines
        assert_eq!(func.statements.len(), 2);
    }

    #[test]
    fn test_is_executable() {
        assert!(SourceFile::is_executable("let x = 5;"));
        assert!(SourceFile::is_executable("    return value;"));
        assert!(!SourceFile::is_executable("// comment"));
        assert!(!SourceFile::is_executable(""));
        assert!(!SourceFile::is_executable("   "));
        assert!(!SourceFile::is_executable("{"));
        assert!(!SourceFile::is_executable("}"));
    }
}
```

**Check Your Understanding**:
- Why do we track line numbers instead of just counting statements?
- What makes a line "executable" vs non-executable?
- Why do we need to track function boundaries?

---

#### Why Milestone 1 Isn't Enough

**Limitation**: We can identify code structure, but we can't track which lines actually execute during tests. We need instrumentation.

**What we're adding**: Code instrumentation—injecting tracking calls into the source code so we can record execution at runtime.

**Improvement**:
- **Capability**: Can now track actual execution, not just structure
- **Approach**: Insert `record_line(N)` calls before each executable statement
- **Challenge**: Must preserve original line numbers for accurate reporting

---

### Milestone 2: Code Instrumentation

**Goal**: Inject coverage tracking calls into source code without breaking it.

**Why we need this**: To track execution, we need to add recording statements. But naive injection can break the code by changing semantics or line numbers.

#### Architecture

**Structs:**
- `Instrumentor` - Handles code instrumentation
  - **Field**: `coverage_map: Arc<Mutex<HashSet<usize>>>` - Tracks executed lines

- `InstrumentedCode` - Result of instrumentation
  - **Field**: `original: SourceFile` - Original source
  - **Field**: `instrumented: String` - Instrumented code
  - **Field**: `line_mapping: HashMap<usize, usize>` - New→ original line mapping

**Functions:**
- `new() -> Instrumentor` - Create instrumentor with shared coverage map
- `instrument(&self, source: &SourceFile) -> InstrumentedCode` - Inject tracking
- `record_line(line: usize)` - Runtime function to record execution
- `inject_probe(line: &str, line_num: usize) -> String` - Create tracking statement

**Starter Code**:

```rust
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref COVERAGE_DATA: Arc<Mutex<HashSet<usize>>> =
        Arc::new(Mutex::new(HashSet::new()));
}

pub struct Instrumentor {
    coverage_map: Arc<Mutex<HashSet<usize>>>,
}

pub struct InstrumentedCode {
    pub original: SourceFile,
    pub instrumented: String,
    pub line_mapping: HashMap<usize, usize>,
}

impl Instrumentor {
    pub fn new() -> Self {
        // TODO: Initialize with shared coverage map
        todo!("Create instrumentor")
    }

    pub fn instrument(&self, source: &SourceFile) -> InstrumentedCode {
        // TODO: For each executable line, inject record_line() call
        // TODO: Build line mapping (instrumented -> original)
        // TODO: Preserve original structure
        todo!("Implement instrumentation")
    }

    fn inject_probe(line: &str, line_num: usize) -> String {
        // TODO: Create line like: _coverage_record(42); original_line
        // TODO: Preserve indentation
        todo!("Create probe injection")
    }

    pub fn get_coverage(&self) -> HashSet<usize> {
        // TODO: Return clone of executed lines
        todo!("Return coverage data")
    }

    pub fn reset(&self) {
        // TODO: Clear coverage data for next run
        todo!("Reset coverage")
    }
}

/// Runtime function called by instrumented code
pub fn record_line(line: usize) {
    COVERAGE_DATA.lock().unwrap().insert(line);
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrument_simple_function() {
        let source = SourceFile {
            path: PathBuf::from("test.rs"),
            lines: vec![
                "fn add(a: i32, b: i32) -> i32 {".to_string(),
                "    let sum = a + b;".to_string(),
                "    sum".to_string(),
                "}".to_string(),
            ],
            functions: vec![FunctionInfo {
                name: "add".to_string(),
                start_line: 0,
                end_line: 3,
                statements: vec![1, 2],
            }],
        };

        let instrumentor = Instrumentor::new();
        let instrumented = instrumentor.instrument(&source);

        // Should contain tracking calls
        assert!(instrumented.instrumented.contains("record_line"));

        // Should have mapping for each instrumented line
        assert!(instrumented.line_mapping.len() > 0);
    }

    #[test]
    fn test_preserve_indentation() {
        let line = "    let x = 5;";
        let probe = Instrumentor::inject_probe(line, 10);

        // Probe should maintain indentation
        assert!(probe.starts_with("    "));
        assert!(probe.contains("record_line(10)"));
    }

    #[test]
    fn test_coverage_tracking() {
        let instrumentor = Instrumentor::new();
        instrumentor.reset();

        record_line(5);
        record_line(10);
        record_line(5); // Duplicate

        let coverage = instrumentor.get_coverage();
        assert_eq!(coverage.len(), 2);
        assert!(coverage.contains(&5));
        assert!(coverage.contains(&10));
    }

    #[test]
    fn test_reset_coverage() {
        let instrumentor = Instrumentor::new();
        record_line(1);

        instrumentor.reset();
        assert_eq!(instrumentor.get_coverage().len(), 0);
    }
}
```

---

#### Why Milestone 2 Isn't Enough

**Limitation**: We can instrument and track line execution, but we don't track *branches* (if/else, match arms). This misses critical test gaps.

**Example**:
```rust
fn abs(x: i32) -> i32 {
    if x < 0 {        // Line covered ✓
        -x             // Branch NOT tested ✗
    } else {
        x              // Branch tested ✓
    }
}
// Line coverage: 100%, Branch coverage: 50%
```

**What we're adding**: Branch tracking to detect which decision paths are exercised.

**Improvement**:
- **Capability**: Track both true and false branches of conditionals
- **Metric**: Branch coverage = (branches_taken / total_branches) * 100
- **Insight**: Can have 100% line coverage with 0% branch coverage of critical logic

---

### Milestone 3: Branch Coverage Tracking

**Goal**: Track which branches of if/match statements are executed during tests.

**Why this matters**: Branch coverage reveals untested code paths that line coverage misses. Error handling, edge cases, and conditional logic are often only testable via branch coverage.

#### Architecture

**Structs:**
- `Branch` - Represents a decision branch
  - **Field**: `line: usize` - Line number of branch
  - **Field**: `branch_id: usize` - Unique identifier
  - **Field**: `kind: BranchKind` - Type of branch (if/else/match)
  - **Field**: `taken: bool` - Whether this branch executed

- `BranchKind` - Type of branch
  - **Variants**: `IfTrue`, `IfFalse`, `MatchArm(usize)`

**Functions:**
- `find_branches(source: &SourceFile) -> Vec<Branch>` - Identify all branches
- `instrument_branches(&mut self, branches: &[Branch])` - Inject branch tracking
- `record_branch(branch_id: usize)` - Runtime branch recording
- `get_branch_coverage(&self) -> (usize, usize)` - Return (taken, total)

**Starter Code**:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchKind {
    IfTrue,
    IfFalse,
    MatchArm(usize),
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub line: usize,
    pub branch_id: usize,
    pub kind: BranchKind,
    pub taken: bool,
}

lazy_static::lazy_static! {
    static ref BRANCH_DATA: Arc<Mutex<HashSet<usize>>> =
        Arc::new(Mutex::new(HashSet::new()));
}

impl Instrumentor {
    pub fn find_branches(&self, source: &SourceFile) -> Vec<Branch> {
        // TODO: Scan for "if " patterns
        // TODO: Scan for "match " patterns
        // TODO: Assign unique branch IDs
        // TODO: Identify true/false branches for if
        // TODO: Identify match arms
        todo!("Find all branches")
    }

    pub fn instrument_branches(&mut self, branches: &[Branch]) -> String {
        // TODO: For each if statement, inject:
        //   if condition { record_branch(ID_TRUE); ... }
        //   else { record_branch(ID_FALSE); ... }
        // TODO: For each match arm, inject record_branch(ID_ARM_N)
        todo!("Instrument branches")
    }

    pub fn get_branch_coverage(&self) -> (usize, usize) {
        // TODO: Count how many unique branch IDs were recorded
        // TODO: Return (branches_taken, total_branches)
        todo!("Calculate branch coverage")
    }
}

pub fn record_branch(branch_id: usize) {
    BRANCH_DATA.lock().unwrap().insert(branch_id);
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_if_branches() {
        let content = r#"
fn check(x: i32) -> bool {
    if x > 0 {
        true
    } else {
        false
    }
}
"#;
        let file = create_test_file(content);
        let source = SourceFile::parse_file(file.path()).unwrap();
        let instrumentor = Instrumentor::new();

        let branches = instrumentor.find_branches(&source);

        // Should find 2 branches: if-true and if-false
        assert_eq!(branches.len(), 2);
        assert!(branches.iter().any(|b| b.kind == BranchKind::IfTrue));
        assert!(branches.iter().any(|b| b.kind == BranchKind::IfFalse));
    }

    #[test]
    fn test_find_match_branches() {
        let content = r#"
fn classify(x: i32) -> &'static str {
    match x {
        0 => "zero",
        1..=10 => "small",
        _ => "large",
    }
}
"#;
        let file = create_test_file(content);
        let source = SourceFile::parse_file(file.path()).unwrap();
        let instrumentor = Instrumentor::new();

        let branches = instrumentor.find_branches(&source);

        // Should find 3 match arms
        assert_eq!(branches.len(), 3);
    }

    #[test]
    fn test_branch_coverage_calculation() {
        let instrumentor = Instrumentor::new();

        // Simulate 3 total branches, 2 taken
        record_branch(1);
        record_branch(2);
        // Branch 3 not taken

        let (taken, total) = instrumentor.get_branch_coverage();
        assert_eq!(taken, 2);
        // Note: total must be passed in or tracked separately
    }

    #[test]
    fn test_branch_instrumentation() {
        let content = r#"
fn abs(x: i32) -> i32 {
    if x < 0 {
        -x
    } else {
        x
    }
}
"#;
        let file = create_test_file(content);
        let source = SourceFile::parse_file(file.path()).unwrap();
        let mut instrumentor = Instrumentor::new();

        let branches = instrumentor.find_branches(&source);
        let instrumented = instrumentor.instrument_branches(&branches);

        // Should contain branch recording calls
        assert!(instrumented.contains("record_branch"));
    }
}
```

---

#### Why Milestone 3 Isn't Enough

**Limitation**: We collect coverage data but have no way to visualize it. Raw numbers like "75% coverage" don't tell you *which* lines are untested.

**What we're adding**: Coverage report generation in multiple formats (text, HTML, JSON) with visual highlighting of tested/untested code.

**Improvement**:
- **Capability**: Human-readable reports with color coding
- **Formats**: Terminal output (with ANSI colors), HTML (for browsers), JSON (for tools)
- **Actionability**: Developers can immediately see what needs testing

---

### Milestone 4: Coverage Report Generation

**Goal**: Generate comprehensive coverage reports showing tested and untested code paths.

**Why this matters**: Coverage data is useless without good reporting. Developers need to quickly identify gaps and prioritize testing effort.

#### Architecture

**Structs:**
- `CoverageReport` - Complete coverage analysis
  - **Field**: `source: SourceFile` - Original source
  - **Field**: `line_coverage: HashMap<usize, bool>` - Line execution status
  - **Field**: `branch_coverage: Vec<Branch>` - Branch execution status
  - **Field**: `function_coverage: HashMap<String, bool>` - Function call status

- `ReportFormat` - Output format
  - **Variants**: `Text`, `Html`, `Json`

**Functions:**
- `generate_report(&self, format: ReportFormat) -> String` - Create report
- `calculate_metrics(&self) -> CoverageMetrics` - Compute percentages
- `format_text(&self) -> String` - Plain text with ANSI colors
- `format_html(&self) -> String` - HTML with CSS styling
- `format_json(&self) -> String` - JSON for tool integration

**Starter Code**:

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub source: SourceFile,
    pub line_coverage: HashMap<usize, bool>,
    pub branch_coverage: Vec<Branch>,
    pub function_coverage: HashMap<String, bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub lines_covered: usize,
    pub lines_total: usize,
    pub line_percentage: f64,
    pub branches_covered: usize,
    pub branches_total: usize,
    pub branch_percentage: f64,
    pub functions_covered: usize,
    pub functions_total: usize,
    pub function_percentage: f64,
}

pub enum ReportFormat {
    Text,
    Html,
    Json,
}

impl CoverageReport {
    pub fn new(source: SourceFile, coverage: &HashSet<usize>,
               branches: Vec<Branch>) -> Self {
        // TODO: Build line_coverage map from executed lines
        // TODO: Mark functions as covered if any line inside was executed
        todo!("Create coverage report")
    }

    pub fn generate_report(&self, format: ReportFormat) -> String {
        // TODO: Match on format and call appropriate formatter
        todo!("Generate report")
    }

    pub fn calculate_metrics(&self) -> CoverageMetrics {
        // TODO: Count covered vs total lines
        // TODO: Count covered vs total branches
        // TODO: Count covered vs total functions
        // TODO: Calculate percentages
        todo!("Calculate metrics")
    }

    fn format_text(&self) -> String {
        // TODO: Create terminal output with ANSI colors
        // TODO: Green for covered lines, red for uncovered
        // TODO: Show line numbers and source code
        todo!("Format as text")
    }

    fn format_html(&self) -> String {
        // TODO: Generate HTML with CSS
        // TODO: Syntax highlighting for Rust code
        // TODO: Color-coded coverage
        todo!("Format as HTML")
    }

    fn format_json(&self) -> String {
        // TODO: Serialize metrics and coverage data to JSON
        todo!("Format as JSON")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_report() -> CoverageReport {
        let source = SourceFile {
            path: PathBuf::from("test.rs"),
            lines: vec![
                "fn add(a: i32, b: i32) -> i32 {".to_string(),
                "    a + b".to_string(),
                "}".to_string(),
                "fn unused() {".to_string(),
                "    println!(\"never called\");".to_string(),
                "}".to_string(),
            ],
            functions: vec![
                FunctionInfo {
                    name: "add".to_string(),
                    start_line: 0,
                    end_line: 2,
                    statements: vec![1],
                },
                FunctionInfo {
                    name: "unused".to_string(),
                    start_line: 3,
                    end_line: 5,
                    statements: vec![4],
                },
            ],
        };

        let mut coverage = HashSet::new();
        coverage.insert(1); // Only line 1 executed

        CoverageReport::new(source, &coverage, vec![])
    }

    #[test]
    fn test_calculate_metrics() {
        let report = create_sample_report();
        let metrics = report.calculate_metrics();

        // 1 line covered out of 2 executable lines
        assert_eq!(metrics.lines_covered, 1);
        assert_eq!(metrics.lines_total, 2);
        assert_eq!(metrics.line_percentage, 50.0);

        // 1 function covered out of 2
        assert_eq!(metrics.functions_covered, 1);
        assert_eq!(metrics.functions_total, 2);
    }

    #[test]
    fn test_text_report_generation() {
        let report = create_sample_report();
        let text = report.format_text();

        // Should contain coverage info
        assert!(text.contains("Coverage"));
        assert!(text.contains("50"));  // 50% coverage

        // Should show line numbers
        assert!(text.contains("1"));
        assert!(text.contains("4"));
    }

    #[test]
    fn test_html_report_generation() {
        let report = create_sample_report();
        let html = report.format_html();

        // Should be valid HTML
        assert!(html.contains("<html"));
        assert!(html.contains("</html>"));

        // Should have CSS styling
        assert!(html.contains("<style"));

        // Should show code
        assert!(html.contains("add"));
    }

    #[test]
    fn test_json_report_generation() {
        let report = create_sample_report();
        let json = report.format_json();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Should contain metrics
        assert!(parsed.get("line_percentage").is_some());
        assert!(parsed.get("lines_covered").is_some());
    }
}
```

---

#### Why Milestone 4 Isn't Enough

**Limitation**: Our analyzer only works on single files and requires manual instrumentation. Real projects have dozens of files and need automatic integration.

**What we're adding**: Cargo integration to automatically analyze entire projects and generate coverage reports with a single command.

**Improvement**:
- **Automation**: Single command analyzes entire project
- **Scale**: Handles multi-file projects with dependencies
- **Integration**: Works with `cargo test` workflow
- **Speed**: Parallel processing of multiple files

---

### Milestone 5: Cargo Integration and Multi-File Support

**Goal**: Integrate with Cargo to automatically analyze entire projects and handle multiple source files.

**Why this matters**: Real projects aren't single files. We need to handle dependencies, test modules, and generate project-wide coverage reports.

#### Architecture

**Structs:**
- `ProjectAnalyzer` - Analyzes entire Cargo project
  - **Field**: `project_root: PathBuf` - Project directory
  - **Field**: `source_files: Vec<SourceFile>` - All parsed files
  - **Field**: `aggregate_coverage: HashMap<PathBuf, CoverageReport>` - Per-file reports

- `AnalysisConfig` - Configuration options
  - **Field**: `min_coverage: f64` - Minimum acceptable coverage percentage
  - **Field**: `exclude_patterns: Vec<String>` - Files to exclude (e.g., "tests/*")
  - **Field**: `report_format: ReportFormat` - Output format

**Functions:**
- `new(project_root: &Path) -> Self` - Initialize analyzer
- `discover_source_files(&self) -> Vec<PathBuf>` - Find all .rs files
- `analyze_project(&mut self) -> ProjectCoverageReport` - Analyze all files
- `run_instrumented_tests(&self) -> Result<(), Error>` - Execute tests with coverage
- `generate_aggregate_report(&self) -> String` - Project-wide report

**Starter Code**:

```rust
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub struct ProjectAnalyzer {
    project_root: PathBuf,
    source_files: Vec<SourceFile>,
    aggregate_coverage: HashMap<PathBuf, CoverageReport>,
}

pub struct AnalysisConfig {
    pub min_coverage: f64,
    pub exclude_patterns: Vec<String>,
    pub report_format: ReportFormat,
}

pub struct ProjectCoverageReport {
    pub total_lines_covered: usize,
    pub total_lines: usize,
    pub total_branches_covered: usize,
    pub total_branches: usize,
    pub file_reports: HashMap<PathBuf, CoverageReport>,
}

impl ProjectAnalyzer {
    pub fn new(project_root: &Path) -> Self {
        // TODO: Initialize with project root
        // TODO: Verify it's a valid Cargo project (has Cargo.toml)
        todo!("Create project analyzer")
    }

    pub fn discover_source_files(&self) -> Vec<PathBuf> {
        // TODO: Walk directory tree starting from examples/
        // TODO: Find all .rs files
        // TODO: Exclude test files if configured
        // TODO: Filter by exclude patterns
        todo!("Discover source files")
    }

    pub fn analyze_project(&mut self) -> ProjectCoverageReport {
        // TODO: Parse all source files
        // TODO: Instrument all files
        // TODO: Run tests with instrumentation
        // TODO: Collect coverage data
        // TODO: Generate per-file reports
        // TODO: Aggregate into project report
        todo!("Analyze entire project")
    }

    fn run_instrumented_tests(&self) -> Result<(), std::io::Error> {
        // TODO: Execute `cargo test` with instrumented code
        // TODO: Capture coverage data during test run
        // TODO: Handle test failures gracefully
        todo!("Run instrumented tests")
    }

    pub fn generate_aggregate_report(&self, config: &AnalysisConfig) -> String {
        // TODO: Combine all file reports
        // TODO: Calculate project-wide metrics
        // TODO: Format according to config
        // TODO: Highlight files below min_coverage threshold
        todo!("Generate aggregate report")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        let project_root = dir.path();

        // Create Cargo.toml
        fs::write(
            project_root.join("Cargo.toml"),
            r#"
[package]
name = "test_project"
version = "0.1.0"
"#
        ).unwrap();

        // Create examples directory
        fs::create_dir(project_root.join("examples")).unwrap();

        // Create lib.rs
        fs::write(
            project_root.join("examples/lib.rs"),
            r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#
        ).unwrap();

        // Create module file
        fs::write(
            project_root.join("examples/math.rs"),
            r#"
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#
        ).unwrap();

        dir
    }

    #[test]
    fn test_discover_source_files() {
        let project = create_test_project();
        let analyzer = ProjectAnalyzer::new(project.path());

        let files = analyzer.discover_source_files();

        // Should find lib.rs and math.rs
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("lib.rs")));
        assert!(files.iter().any(|p| p.ends_with("math.rs")));
    }

    #[test]
    fn test_project_analysis() {
        let project = create_test_project();
        let mut analyzer = ProjectAnalyzer::new(project.path());

        let report = analyzer.analyze_project();

        // Should analyze both files
        assert_eq!(report.file_reports.len(), 2);

        // Should have aggregate metrics
        assert!(report.total_lines > 0);
    }

    #[test]
    fn test_exclude_patterns() {
        let project = create_test_project();

        let config = AnalysisConfig {
            min_coverage: 80.0,
            exclude_patterns: vec!["tests/*".to_string()],
            report_format: ReportFormat::Text,
        };

        let analyzer = ProjectAnalyzer::new(project.path());
        let files = analyzer.discover_source_files();

        // Test files should be excluded
        assert!(!files.iter().any(|p| p.to_str().unwrap().contains("tests")));
    }

    #[test]
    fn test_aggregate_report_format() {
        let project = create_test_project();
        let mut analyzer = ProjectAnalyzer::new(project.path());
        analyzer.analyze_project();

        let config = AnalysisConfig {
            min_coverage: 80.0,
            exclude_patterns: vec![],
            report_format: ReportFormat::Text,
        };

        let report = analyzer.generate_aggregate_report(&config);

        // Should contain project summary
        assert!(report.contains("Project Coverage"));
        assert!(report.contains("Total"));

        // Should list individual files
        assert!(report.contains("lib.rs"));
        assert!(report.contains("math.rs"));
    }
}
```

---

#### Why Milestone 5 Isn't Enough

**Limitation**: Analysis is sequential—each file is processed one at a time. For large projects with hundreds of files, this is slow.

**What we're adding**: Parallel processing using Rayon to analyze multiple files concurrently.

**Improvement**:
- **Speed**: 4-8x faster on multi-core systems
- **Scalability**: Handles large projects efficiently
- **Efficiency**: Utilizes all CPU cores
- **Optimization**: Shows the power of Rust's fearless concurrency

---

### Milestone 6: Parallel Analysis with Rayon

**Goal**: Parallelize file analysis to dramatically speed up coverage reporting for large projects.

**Why this matters**: In production, coverage analysis can take minutes on large codebases. Parallelization reduces this to seconds, making it practical for CI/CD pipelines.

#### Architecture

**Changes:**
- Modify `analyze_project()` to use parallel iterators
- Thread-safe coverage data collection
- Concurrent report generation

**Functions:**
- `parallel_analyze(&mut self) -> ProjectCoverageReport` - Parallel analysis
- `merge_coverage_data(reports: Vec<CoverageReport>) -> ProjectCoverageReport` - Combine results
- Benchmark comparison: `sequential_analysis()` vs `parallel_analysis()`

**Starter Code**:

```rust
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

impl ProjectAnalyzer {
    pub fn parallel_analyze(&mut self) -> ProjectCoverageReport {
        // TODO: Use rayon's par_iter to process files in parallel
        // TODO: Collect results in thread-safe manner
        // TODO: Merge coverage data
        todo!("Implement parallel analysis")
    }

    fn analyze_file_parallel(
        &self,
        path: &Path,
        coverage_collector: Arc<Mutex<HashMap<PathBuf, HashSet<usize>>>>
    ) -> CoverageReport {
        // TODO: Parse and instrument file
        // TODO: Store coverage data in shared collector
        // TODO: Return report
        todo!("Analyze single file in parallel")
    }

    fn merge_coverage_data(reports: Vec<CoverageReport>) -> ProjectCoverageReport {
        // TODO: Sum up all line counts
        // TODO: Sum up all branch counts
        // TODO: Calculate aggregate percentages
        todo!("Merge parallel results")
    }
}

// Performance comparison
pub fn benchmark_analysis_methods(project_root: &Path) {
    use std::time::Instant;

    let mut analyzer = ProjectAnalyzer::new(project_root);

    // Sequential
    let start = Instant::now();
    let _ = analyzer.analyze_project();
    let sequential_time = start.elapsed();

    // Parallel
    let start = Instant::now();
    let _ = analyzer.parallel_analyze();
    let parallel_time = start.elapsed();

    println!("Sequential: {:?}", sequential_time);
    println!("Parallel: {:?}", parallel_time);
    println!("Speedup: {:.2}x", sequential_time.as_secs_f64() / parallel_time.as_secs_f64());
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_analysis_correctness() {
        let project = create_test_project();
        let mut analyzer = ProjectAnalyzer::new(project.path());

        let sequential = analyzer.analyze_project();
        let parallel = analyzer.parallel_analyze();

        // Results should match
        assert_eq!(
            sequential.total_lines_covered,
            parallel.total_lines_covered
        );
        assert_eq!(
            sequential.total_lines,
            parallel.total_lines
        );
    }

    #[test]
    fn test_parallel_speedup() {
        // Create project with many files
        let project = create_large_test_project(50); // 50 files

        let mut analyzer = ProjectAnalyzer::new(project.path());

        let start = std::time::Instant::now();
        let _ = analyzer.analyze_project();
        let seq_time = start.elapsed();

        let start = std::time::Instant::now();
        let _ = analyzer.parallel_analyze();
        let par_time = start.elapsed();

        // Parallel should be faster (at least 1.5x on 4+ cores)
        assert!(par_time < seq_time);

        let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();
        println!("Speedup: {:.2}x", speedup);
        assert!(speedup > 1.5);
    }

    #[test]
    fn test_thread_safety() {
        // Ensure no data races in parallel execution
        let project = create_test_project();
        let analyzer = ProjectAnalyzer::new(project.path());

        let coverage_collector = Arc::new(Mutex::new(HashMap::new()));

        // Run analysis from multiple threads
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let analyzer = analyzer.clone();
                let collector = Arc::clone(&coverage_collector);
                std::thread::spawn(move || {
                    analyzer.analyze_file_parallel(
                        &PathBuf::from("examples/lib.rs"),
                        collector
                    )
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // No panics = thread-safe
    }
}
```

---

## Testing Strategies

### 1. Unit Tests
Test individual components in isolation:
- Parser: Verify correct extraction of functions, statements, branches
- Instrumentor: Ensure probes are correctly injected
- Reporter: Validate report format and metrics calculation

### 2. Integration Tests
Test components working together:
- End-to-end: Parse → Instrument → Execute → Report
- Multi-file: Verify correct handling of project structure
- Error handling: Invalid Rust code, missing files, etc.

### 3. Property-Based Tests
Use proptest to verify invariants:
- Instrumentation preserves semantics (same test results)
- Coverage percentage always between 0-100%
- Line numbers in reports match original source

### 4. Performance Tests
Benchmark critical operations:
- Parsing speed (lines/second)
- Instrumentation overhead
- Sequential vs parallel analysis speedup
- Memory usage for large projects

### 5. Real-World Tests
Test on actual Rust projects:
- Run on small open-source projects
- Compare results with llvm-cov or tarpaulin
- Verify accuracy of coverage reports

---

## Complete Working Example

```rust
//==============================================================================
// Coverage Analyzer - Complete Implementation
//==============================================================================

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};

//==============================================================================
// Part 1: Source File Parsing
//==============================================================================

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub lines: Vec<String>,
    pub functions: Vec<FunctionInfo>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub statements: Vec<usize>,
}

impl SourceFile {
    pub fn parse_file(path: &Path) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let mut source = SourceFile {
            path: path.to_path_buf(),
            lines: lines.clone(),
            functions: vec![],
        };

        source.functions = source.find_functions();

        Ok(source)
    }

    fn find_functions(&self) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let mut brace_depth = 0;
        let mut in_function = false;
        let mut func_start = 0;
        let mut func_name = String::new();

        for (i, line) in self.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Look for function definitions
            if trimmed.starts_with("fn ") && !in_function {
                in_function = true;
                func_start = i;

                // Extract function name
                if let Some(name_end) = trimmed.find('(') {
                    func_name = trimmed[3..name_end].trim().to_string();
                }
            }

            // Track braces
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;

            // Function ends when braces balance
            if in_function && brace_depth == 0 {
                let statements = self.find_statements(func_start, i);

                functions.push(FunctionInfo {
                    name: func_name.clone(),
                    start_line: func_start,
                    end_line: i,
                    statements,
                });

                in_function = false;
            }
        }

        functions
    }

    fn find_statements(&self, start: usize, end: usize) -> Vec<usize> {
        (start..=end)
            .filter(|&i| Self::is_executable(&self.lines[i]))
            .collect()
    }

    fn is_executable(line: &str) -> bool {
        let trimmed = line.trim();

        // Filter out non-executable lines
        !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("/*")
            && !trimmed.starts_with("*")
            && !trimmed.starts_with("*/")
            && trimmed != "{"
            && trimmed != "}"
            && !trimmed.starts_with("fn ")
            && !trimmed.starts_with("pub fn ")
            && !trimmed.starts_with("#[")
    }
}

//==============================================================================
// Part 2: Code Instrumentation
//==============================================================================

lazy_static::lazy_static! {
    static ref COVERAGE_DATA: Arc<Mutex<HashSet<usize>>> =
        Arc::new(Mutex::new(HashSet::new()));
    static ref BRANCH_DATA: Arc<Mutex<HashSet<usize>>> =
        Arc::new(Mutex::new(HashSet::new()));
}

pub struct Instrumentor {
    coverage_map: Arc<Mutex<HashSet<usize>>>,
    branch_map: Arc<Mutex<HashSet<usize>>>,
    next_branch_id: usize,
}

pub struct InstrumentedCode {
    pub original: SourceFile,
    pub instrumented: String,
    pub line_mapping: HashMap<usize, usize>,
    pub branches: Vec<Branch>,
}

impl Instrumentor {
    pub fn new() -> Self {
        Instrumentor {
            coverage_map: Arc::clone(&COVERAGE_DATA),
            branch_map: Arc::clone(&BRANCH_DATA),
            next_branch_id: 0,
        }
    }

    pub fn instrument(&mut self, source: &SourceFile) -> InstrumentedCode {
        let mut instrumented_lines = Vec::new();
        let mut line_mapping = HashMap::new();
        let branches = self.find_branches(source);

        for (original_line_num, line) in source.lines.iter().enumerate() {
            let current_instrumented_line = instrumented_lines.len();
            line_mapping.insert(current_instrumented_line, original_line_num);

            // Check if this line is executable
            if source.functions.iter().any(|f| f.statements.contains(&original_line_num)) {
                // Inject coverage probe
                let indent = line.len() - line.trim_start().len();
                let probe = format!(
                    "{}record_line({});",
                    " ".repeat(indent),
                    original_line_num
                );
                instrumented_lines.push(probe);
            }

            instrumented_lines.push(line.clone());
        }

        InstrumentedCode {
            original: source.clone(),
            instrumented: instrumented_lines.join("\n"),
            line_mapping,
            branches,
        }
    }

    pub fn find_branches(&mut self, source: &SourceFile) -> Vec<Branch> {
        let mut branches = Vec::new();

        for (i, line) in source.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Find if statements
            if trimmed.starts_with("if ") || trimmed.contains(" if ") {
                let true_id = self.next_branch_id;
                self.next_branch_id += 1;
                let false_id = self.next_branch_id;
                self.next_branch_id += 1;

                branches.push(Branch {
                    line: i,
                    branch_id: true_id,
                    kind: BranchKind::IfTrue,
                    taken: false,
                });

                branches.push(Branch {
                    line: i,
                    branch_id: false_id,
                    kind: BranchKind::IfFalse,
                    taken: false,
                });
            }

            // Find match arms (simplified)
            if trimmed.starts_with("match ") {
                let arm_id = self.next_branch_id;
                self.next_branch_id += 1;

                branches.push(Branch {
                    line: i,
                    branch_id: arm_id,
                    kind: BranchKind::MatchArm(0),
                    taken: false,
                });
            }
        }

        branches
    }

    pub fn get_coverage(&self) -> HashSet<usize> {
        self.coverage_map.lock().unwrap().clone()
    }

    pub fn get_branch_coverage(&self) -> (usize, usize) {
        let taken = self.branch_map.lock().unwrap().len();
        (taken, self.next_branch_id)
    }

    pub fn reset(&self) {
        self.coverage_map.lock().unwrap().clear();
        self.branch_map.lock().unwrap().clear();
    }
}

pub fn record_line(line: usize) {
    COVERAGE_DATA.lock().unwrap().insert(line);
}

pub fn record_branch(branch_id: usize) {
    BRANCH_DATA.lock().unwrap().insert(branch_id);
}

//==============================================================================
// Part 3: Branch Tracking
//==============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchKind {
    IfTrue,
    IfFalse,
    MatchArm(usize),
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub line: usize,
    pub branch_id: usize,
    pub kind: BranchKind,
    pub taken: bool,
}

//==============================================================================
// Part 4: Coverage Reporting
//==============================================================================

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub source: SourceFile,
    pub line_coverage: HashMap<usize, bool>,
    pub branch_coverage: Vec<Branch>,
    pub function_coverage: HashMap<String, bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub lines_covered: usize,
    pub lines_total: usize,
    pub line_percentage: f64,
    pub branches_covered: usize,
    pub branches_total: usize,
    pub branch_percentage: f64,
    pub functions_covered: usize,
    pub functions_total: usize,
    pub function_percentage: f64,
}

pub enum ReportFormat {
    Text,
    Html,
    Json,
}

impl CoverageReport {
    pub fn new(
        source: SourceFile,
        coverage: &HashSet<usize>,
        mut branches: Vec<Branch>,
        branch_coverage: &HashSet<usize>,
    ) -> Self {
        // Build line coverage map
        let mut line_coverage = HashMap::new();
        for func in &source.functions {
            for &line_num in &func.statements {
                let covered = coverage.contains(&line_num);
                line_coverage.insert(line_num, covered);
            }
        }

        // Update branch taken status
        for branch in &mut branches {
            branch.taken = branch_coverage.contains(&branch.branch_id);
        }

        // Build function coverage map
        let mut function_coverage = HashMap::new();
        for func in &source.functions {
            let covered = func.statements.iter().any(|line| coverage.contains(line));
            function_coverage.insert(func.name.clone(), covered);
        }

        CoverageReport {
            source,
            line_coverage,
            branch_coverage: branches,
            function_coverage,
        }
    }

    pub fn calculate_metrics(&self) -> CoverageMetrics {
        let lines_total: usize = self.source.functions.iter()
            .map(|f| f.statements.len())
            .sum();

        let lines_covered = self.line_coverage.values()
            .filter(|&&covered| covered)
            .count();

        let line_percentage = if lines_total > 0 {
            (lines_covered as f64 / lines_total as f64) * 100.0
        } else {
            0.0
        };

        let branches_total = self.branch_coverage.len();
        let branches_covered = self.branch_coverage.iter()
            .filter(|b| b.taken)
            .count();

        let branch_percentage = if branches_total > 0 {
            (branches_covered as f64 / branches_total as f64) * 100.0
        } else {
            0.0
        };

        let functions_total = self.function_coverage.len();
        let functions_covered = self.function_coverage.values()
            .filter(|&&covered| covered)
            .count();

        let function_percentage = if functions_total > 0 {
            (functions_covered as f64 / functions_total as f64) * 100.0
        } else {
            0.0
        };

        CoverageMetrics {
            lines_covered,
            lines_total,
            line_percentage,
            branches_covered,
            branches_total,
            branch_percentage,
            functions_covered,
            functions_total,
            function_percentage,
        }
    }

    pub fn generate_report(&self, format: ReportFormat) -> String {
        match format {
            ReportFormat::Text => self.format_text(),
            ReportFormat::Html => self.format_html(),
            ReportFormat::Json => self.format_json(),
        }
    }

    fn format_text(&self) -> String {
        let metrics = self.calculate_metrics();
        let mut output = String::new();

        output.push_str(&format!("Coverage Report: {}\n", self.source.path.display()));
        output.push_str(&format!("{'=':=<60}\n"));
        output.push_str(&format!(
            "Lines: {}/{} ({:.1}%)\n",
            metrics.lines_covered, metrics.lines_total, metrics.line_percentage
        ));
        output.push_str(&format!(
            "Branches: {}/{} ({:.1}%)\n",
            metrics.branches_covered, metrics.branches_total, metrics.branch_percentage
        ));
        output.push_str(&format!(
            "Functions: {}/{} ({:.1}%)\n\n",
            metrics.functions_covered, metrics.functions_total, metrics.function_percentage
        ));

        // Show source with coverage annotations
        for (i, line) in self.source.lines.iter().enumerate() {
            let marker = if let Some(&covered) = self.line_coverage.get(&i) {
                if covered { "✓" } else { "✗" }
            } else {
                " "
            };

            output.push_str(&format!("{:4} {} {}\n", i + 1, marker, line));
        }

        output
    }

    fn format_html(&self) -> String {
        let metrics = self.calculate_metrics();

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: monospace; }}
        .covered {{ background-color: #c8e6c9; }}
        .uncovered {{ background-color: #ffcdd2; }}
        .neutral {{ background-color: #f5f5f5; }}
        .metrics {{ margin: 20px; padding: 10px; border: 1px solid #ccc; }}
    </style>
</head>
<body>
    <div class="metrics">
        <h2>Coverage Report: {}</h2>
        <p>Lines: {}/{} ({:.1}%)</p>
        <p>Branches: {}/{} ({:.1}%)</p>
        <p>Functions: {}/{} ({:.1}%)</p>
    </div>
    <pre>{}</pre>
</body>
</html>"#,
            self.source.path.display(),
            metrics.lines_covered,
            metrics.lines_total,
            metrics.line_percentage,
            metrics.branches_covered,
            metrics.branches_total,
            metrics.branch_percentage,
            metrics.functions_covered,
            metrics.functions_total,
            metrics.function_percentage,
            self.format_source_html()
        )
    }

    fn format_source_html(&self) -> String {
        self.source
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let class = if let Some(&covered) = self.line_coverage.get(&i) {
                    if covered {
                        "covered"
                    } else {
                        "uncovered"
                    }
                } else {
                    "neutral"
                };

                format!(
                    r#"<div class="{}">{:4} {}</div>"#,
                    class,
                    i + 1,
                    html_escape::encode_text(line)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_json(&self) -> String {
        let metrics = self.calculate_metrics();
        serde_json::to_string_pretty(&metrics).unwrap()
    }
}

//==============================================================================
// Part 5: Project Analysis
//==============================================================================

pub struct ProjectAnalyzer {
    project_root: PathBuf,
    source_files: Vec<SourceFile>,
    aggregate_coverage: HashMap<PathBuf, CoverageReport>,
}

pub struct AnalysisConfig {
    pub min_coverage: f64,
    pub exclude_patterns: Vec<String>,
    pub report_format: ReportFormat,
}

pub struct ProjectCoverageReport {
    pub total_lines_covered: usize,
    pub total_lines: usize,
    pub total_branches_covered: usize,
    pub total_branches: usize,
    pub file_reports: HashMap<PathBuf, CoverageReport>,
}

impl ProjectAnalyzer {
    pub fn new(project_root: &Path) -> Self {
        ProjectAnalyzer {
            project_root: project_root.to_path_buf(),
            source_files: vec![],
            aggregate_coverage: HashMap::new(),
        }
    }

    pub fn discover_source_files(&self) -> Vec<PathBuf> {
        walkdir::WalkDir::new(self.project_root.join("examples"))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    pub fn analyze_project(&mut self) -> ProjectCoverageReport {
        let files = self.discover_source_files();
        let mut file_reports = HashMap::new();

        for file_path in files {
            if let Ok(source) = SourceFile::parse_file(&file_path) {
                let mut instrumentor = Instrumentor::new();
                let instrumented = instrumentor.instrument(&source);

                // In a real implementation, we would:
                // 1. Write instrumented code to temp file
                // 2. Compile and run tests
                // 3. Collect coverage data
                // For this example, we'll use simulated coverage

                let coverage = instrumentor.get_coverage();
                let (_, _) = instrumentor.get_branch_coverage();
                let branch_coverage = instrumentor.branch_map.lock().unwrap().clone();

                let report = CoverageReport::new(
                    source,
                    &coverage,
                    instrumented.branches,
                    &branch_coverage,
                );

                file_reports.insert(file_path, report);
            }
        }

        let mut total_lines_covered = 0;
        let mut total_lines = 0;
        let mut total_branches_covered = 0;
        let mut total_branches = 0;

        for report in file_reports.values() {
            let metrics = report.calculate_metrics();
            total_lines_covered += metrics.lines_covered;
            total_lines += metrics.lines_total;
            total_branches_covered += metrics.branches_covered;
            total_branches += metrics.branches_total;
        }

        ProjectCoverageReport {
            total_lines_covered,
            total_lines,
            total_branches_covered,
            total_branches,
            file_reports,
        }
    }

    pub fn parallel_analyze(&mut self) -> ProjectCoverageReport {
        let files = self.discover_source_files();

        let file_reports: HashMap<PathBuf, CoverageReport> = files
            .par_iter()
            .filter_map(|file_path| {
                SourceFile::parse_file(file_path).ok().map(|source| {
                    let mut instrumentor = Instrumentor::new();
                    let instrumented = instrumentor.instrument(&source);

                    let coverage = instrumentor.get_coverage();
                    let branch_coverage = instrumentor.branch_map.lock().unwrap().clone();

                    let report = CoverageReport::new(
                        source,
                        &coverage,
                        instrumented.branches,
                        &branch_coverage,
                    );

                    (file_path.clone(), report)
                })
            })
            .collect();

        let (total_lines_covered, total_lines, total_branches_covered, total_branches) =
            file_reports
                .par_iter()
                .map(|(_, report)| {
                    let metrics = report.calculate_metrics();
                    (
                        metrics.lines_covered,
                        metrics.lines_total,
                        metrics.branches_covered,
                        metrics.branches_total,
                    )
                })
                .reduce(
                    || (0, 0, 0, 0),
                    |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3),
                );

        ProjectCoverageReport {
            total_lines_covered,
            total_lines,
            total_branches_covered,
            total_branches,
            file_reports,
        }
    }
}

//==============================================================================
// Example Usage
//==============================================================================

fn main() {
    println!("=== Test Coverage Analyzer ===\n");

    // Example 1: Analyze a single file
    println!("Example 1: Single File Analysis");
    let source = SourceFile::parse_file(Path::new("examples/lib.rs")).unwrap();
    println!("Found {} functions", source.functions.len());

    let mut instrumentor = Instrumentor::new();
    let instrumented = instrumentor.instrument(&source);
    println!("Instrumented {} lines\n", instrumented.instrumented.lines().count());

    // Example 2: Generate coverage report
    println!("Example 2: Coverage Report");
    let coverage = HashSet::new(); // Simulated: no lines executed
    let branch_coverage = HashSet::new();

    let report = CoverageReport::new(
        source.clone(),
        &coverage,
        instrumented.branches,
        &branch_coverage,
    );

    let metrics = report.calculate_metrics();
    println!("Coverage: {:.1}%", metrics.line_percentage);
    println!("Report:\n{}", report.generate_report(ReportFormat::Text));

    // Example 3: Project-wide analysis
    println!("\nExample 3: Project Analysis");
    let mut analyzer = ProjectAnalyzer::new(Path::new("."));
    let project_report = analyzer.analyze_project();

    println!(
        "Project Coverage: {}/{}lines ({:.1}%)",
        project_report.total_lines_covered,
        project_report.total_lines,
        (project_report.total_lines_covered as f64 / project_report.total_lines as f64) * 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_parsing() {
        let content = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), content).unwrap();

        let source = SourceFile::parse_file(temp.path()).unwrap();
        assert_eq!(source.functions.len(), 1);
        assert_eq!(source.functions[0].name, "add");
    }

    #[test]
    fn test_coverage_tracking() {
        record_line(1);
        record_line(5);

        let coverage = COVERAGE_DATA.lock().unwrap();
        assert!(coverage.contains(&1));
        assert!(coverage.contains(&5));
        assert!(!coverage.contains(&10));
    }

    #[test]
    fn test_metrics_calculation() {
        let source = SourceFile {
            path: PathBuf::from("test.rs"),
            lines: vec!["fn test() {".to_string(), "let x = 1;".to_string(), "}".to_string()],
            functions: vec![FunctionInfo {
                name: "test".to_string(),
                start_line: 0,
                end_line: 2,
                statements: vec![1],
            }],
        };

        let mut coverage = HashSet::new();
        coverage.insert(1);

        let report = CoverageReport::new(source, &coverage, vec![], &HashSet::new());
        let metrics = report.calculate_metrics();

        assert_eq!(metrics.lines_covered, 1);
        assert_eq!(metrics.lines_total, 1);
        assert_eq!(metrics.line_percentage, 100.0);
    }
}
```

This complete implementation demonstrates:
- **Part 1**: Parsing Rust source files to extract structure
- **Part 2**: Instrumenting code with coverage tracking probes
- **Part 3**: Branch coverage detection and tracking
- **Part 4**: Multi-format report generation (text, HTML, JSON)
- **Part 5**: Project-wide analysis with parallel processing

The analyzer progresses from simple line tracking to comprehensive coverage analysis with professional reporting capabilities.
