//==============================================================================
// Coverage Analyzer - Complete Implementation
//==============================================================================

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
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
        output.push_str(&format!("{{'=':=<60}}\n"));
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
        walkdir::WalkDir::new(self.project_root.join("src"))
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
    let source = SourceFile::parse_file(Path::new("src/lib.rs")).unwrap();
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
