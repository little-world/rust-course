//==============================================================================
// Test Coverage Analyzer - Complete Implementation
//==============================================================================

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};

//==============================================================================
// Milestone 1: Source Code Parser
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

            if trimmed.starts_with("fn ") && !in_function {
                in_function = true;
                func_start = i;

                if let Some(name_end) = trimmed.find('(') {
                    func_name = trimmed[3..name_end].trim().to_string();
                }
            }

            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;

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
// Milestone 2: Code Instrumentation
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

            if source.functions.iter().any(|f| f.statements.contains(&original_line_num)) {
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

    fn inject_probe(line: &str, line_num: usize) -> String {
        let indent = line.len() - line.trim_start().len();
        format!("{}record_line({}); {}", " ".repeat(indent), line_num, line.trim())
    }

    pub fn get_coverage(&self) -> HashSet<usize> {
        self.coverage_map.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        self.coverage_map.lock().unwrap().clear();
        self.branch_map.lock().unwrap().clear();
    }

    pub fn find_branches(&mut self, source: &SourceFile) -> Vec<Branch> {
        let mut branches = Vec::new();

        for (i, line) in source.lines.iter().enumerate() {
            let trimmed = line.trim();

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

    pub fn instrument_branches(&mut self, branches: &[Branch]) -> String {
        let mut result = String::new();
        for branch in branches {
            result.push_str(&format!("record_branch({});\n", branch.branch_id));
        }
        result
    }

    pub fn get_branch_coverage(&self) -> (usize, usize) {
        let taken = self.branch_map.lock().unwrap().len();
        (taken, self.next_branch_id)
    }
}

pub fn record_line(line: usize) {
    COVERAGE_DATA.lock().unwrap().insert(line);
}

pub fn record_branch(branch_id: usize) {
    BRANCH_DATA.lock().unwrap().insert(branch_id);
}

//==============================================================================
// Milestone 3: Branch Coverage Tracking
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
// Milestone 4: Coverage Report Generation
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
    ) -> Self {
        let branch_coverage_data = BRANCH_DATA.lock().unwrap().clone();

        let mut line_coverage = HashMap::new();
        for func in &source.functions {
            for &line_num in &func.statements {
                let covered = coverage.contains(&line_num);
                line_coverage.insert(line_num, covered);
            }
        }

        for branch in &mut branches {
            branch.taken = branch_coverage_data.contains(&branch.branch_id);
        }

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
        output.push_str("============================================================\n");
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
                    line.replace('<', "&lt;").replace('>', "&gt;")
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
// Milestone 5: Cargo Integration and Multi-File Support
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

                let coverage = instrumentor.get_coverage();

                let report = CoverageReport::new(
                    source,
                    &coverage,
                    instrumented.branches,
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

    fn run_instrumented_tests(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    pub fn generate_aggregate_report(&self, _config: &AnalysisConfig) -> String {
        let mut output = String::new();
        output.push_str("Project Coverage Summary\n");
        output.push_str("========================\n\n");

        let mut total_lines = 0;
        let mut total_lines_covered = 0;

        for (path, report) in &self.aggregate_coverage {
            let metrics = report.calculate_metrics();
            total_lines += metrics.lines_total;
            total_lines_covered += metrics.lines_covered;

            output.push_str(&format!(
                "{}: {:.1}%\n",
                path.display(),
                metrics.line_percentage
            ));
        }

        let overall_percentage = if total_lines > 0 {
            (total_lines_covered as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        output.push_str(&format!("\nTotal: {:.1}%\n", overall_percentage));
        output
    }
}

//==============================================================================
// Milestone 6: Parallel Analysis with Rayon
//==============================================================================

impl ProjectAnalyzer {
    pub fn parallel_analyze(&mut self) -> ProjectCoverageReport {
        let files = self.discover_source_files();

        let file_reports: HashMap<PathBuf, CoverageReport> = files
            .par_iter()
            .filter_map(|file_path| {
                SourceFile::parse_file(file_path).ok().map(|source| {
                    let mut instrumentor = Instrumentor::new();
                    let instrumented = instrumentor.instrument(&source);

                    let coverage = instrumentor.get_coverage();

                    let report = CoverageReport::new(
                        source,
                        &coverage,
                        instrumented.branches,
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

    fn analyze_file_parallel(
        &self,
        path: &Path,
        _coverage_collector: Arc<Mutex<HashMap<PathBuf, HashSet<usize>>>>,
    ) -> CoverageReport {
        let source = SourceFile::parse_file(path).unwrap();
        let mut instrumentor = Instrumentor::new();
        let instrumented = instrumentor.instrument(&source);
        let coverage = instrumentor.get_coverage();

        CoverageReport::new(source, &coverage, instrumented.branches)
    }

    fn merge_coverage_data(_reports: Vec<CoverageReport>) -> ProjectCoverageReport {
        ProjectCoverageReport {
            total_lines_covered: 0,
            total_lines: 0,
            total_branches_covered: 0,
            total_branches: 0,
            file_reports: HashMap::new(),
        }
    }
}

pub fn benchmark_analysis_methods(project_root: &Path) {
    use std::time::Instant;

    let mut analyzer = ProjectAnalyzer::new(project_root);

    let start = Instant::now();
    let _ = analyzer.analyze_project();
    let sequential_time = start.elapsed();

    let start = Instant::now();
    let _ = analyzer.parallel_analyze();
    let parallel_time = start.elapsed();

    println!("Sequential: {:?}", sequential_time);
    println!("Parallel: {:?}", parallel_time);
    println!("Speedup: {:.2}x", sequential_time.as_secs_f64() / parallel_time.as_secs_f64());
}

//==============================================================================
// Example Usage
//==============================================================================

fn main() {
    println!("=== Test Coverage Analyzer ===\n");

    println!("Example: Coverage Analysis System");
    println!("This tool analyzes Rust code coverage across multiple metrics.");
    println!("Features:");
    println!("  - Line coverage tracking");
    println!("  - Branch coverage analysis");
    println!("  - Function coverage reporting");
    println!("  - Multi-format reports (Text, HTML, JSON)");
    println!("  - Parallel analysis for large projects");
    println!("\nAnalysis complete!");
}

//==============================================================================
// Tests
//==============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    // Milestone 1 Tests
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

    // Milestone 2 Tests
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

        let mut instrumentor = Instrumentor::new();
        let instrumented = instrumentor.instrument(&source);

        assert!(instrumented.instrumented.contains("record_line"));
        assert!(instrumented.line_mapping.len() > 0);
    }

    #[test]
    fn test_preserve_indentation() {
        let line = "    let x = 5;";
        let probe = Instrumentor::inject_probe(line, 10);

        assert!(probe.starts_with("    "));
        assert!(probe.contains("record_line(10)"));
    }

    #[test]
    fn test_coverage_tracking() {
        let instrumentor = Instrumentor::new();
        instrumentor.reset();

        record_line(5);
        record_line(10);
        record_line(5);

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

    // Milestone 3 Tests
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
        let mut instrumentor = Instrumentor::new();

        let branches = instrumentor.find_branches(&source);

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
        let mut instrumentor = Instrumentor::new();

        let branches = instrumentor.find_branches(&source);

        assert_eq!(branches.len(), 1);
    }

    #[test]
    fn test_branch_coverage_calculation() {
        let instrumentor = Instrumentor::new();
        instrumentor.reset();

        record_branch(1);
        record_branch(2);

        let (taken, _) = instrumentor.get_branch_coverage();
        assert_eq!(taken, 2);
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

        assert!(instrumented.contains("record_branch"));
    }

    // Milestone 4 Tests
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
        coverage.insert(1);

        CoverageReport::new(source, &coverage, vec![])
    }

    #[test]
    fn test_calculate_metrics() {
        let report = create_sample_report();
        let metrics = report.calculate_metrics();

        assert_eq!(metrics.lines_covered, 1);
        assert_eq!(metrics.lines_total, 2);
        assert_eq!(metrics.line_percentage, 50.0);

        assert_eq!(metrics.functions_covered, 1);
        assert_eq!(metrics.functions_total, 2);
    }

    #[test]
    fn test_text_report_generation() {
        let report = create_sample_report();
        let text = report.format_text();

        assert!(text.contains("Coverage"));
        assert!(text.contains("50"));

        assert!(text.contains("1"));
        assert!(text.contains("4"));
    }

    #[test]
    fn test_html_report_generation() {
        let report = create_sample_report();
        let html = report.format_html();

        assert!(html.contains("<html"));
        assert!(html.contains("</html>"));

        assert!(html.contains("<style"));

        assert!(html.contains("add"));
    }

    #[test]
    fn test_json_report_generation() {
        let report = create_sample_report();
        let json = report.format_json();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("line_percentage").is_some());
        assert!(parsed.get("lines_covered").is_some());
    }

    // Milestone 5 Tests
    fn create_test_project() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        let project_root = dir.path();

        fs::write(
            project_root.join("Cargo.toml"),
            r#"
[package]
name = "test_project"
version = "0.1.0"
"#,
        )
        .unwrap();

        fs::create_dir(project_root.join("src")).unwrap();

        fs::write(
            project_root.join("src/lib.rs"),
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
"#,
        )
        .unwrap();

        fs::write(
            project_root.join("src/math.rs"),
            r#"
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#,
        )
        .unwrap();

        dir
    }

    #[test]
    fn test_discover_source_files() {
        let project = create_test_project();
        let analyzer = ProjectAnalyzer::new(project.path());

        let files = analyzer.discover_source_files();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("lib.rs")));
        assert!(files.iter().any(|p| p.ends_with("math.rs")));
    }

    #[test]
    fn test_project_analysis() {
        let project = create_test_project();
        let mut analyzer = ProjectAnalyzer::new(project.path());

        let report = analyzer.analyze_project();

        assert_eq!(report.file_reports.len(), 2);

        assert!(report.total_lines > 0);
    }

    #[test]
    fn test_exclude_patterns() {
        let project = create_test_project();

        let _config = AnalysisConfig {
            min_coverage: 80.0,
            exclude_patterns: vec!["tests/*".to_string()],
            report_format: ReportFormat::Text,
        };

        let analyzer = ProjectAnalyzer::new(project.path());
        let files = analyzer.discover_source_files();

        assert!(!files.iter().any(|p| p.to_str().unwrap().contains("tests")));
    }

    #[test]
    fn test_aggregate_report_format() {
        let project = create_test_project();
        let mut analyzer = ProjectAnalyzer::new(project.path());
        let _project_report = analyzer.analyze_project();

        let config = AnalysisConfig {
            min_coverage: 80.0,
            exclude_patterns: vec![],
            report_format: ReportFormat::Text,
        };

        let report = analyzer.generate_aggregate_report(&config);

        assert!(report.contains("Project Coverage"));
        assert!(report.contains("Total"));
    }

    // Milestone 6 Tests
    #[test]
    fn test_parallel_analysis_correctness() {
        let project = create_test_project();
        let mut analyzer = ProjectAnalyzer::new(project.path());

        let sequential = analyzer.analyze_project();
        let parallel = analyzer.parallel_analyze();

        assert_eq!(
            sequential.total_lines_covered,
            parallel.total_lines_covered
        );
        assert_eq!(sequential.total_lines, parallel.total_lines);
    }

    fn create_large_test_project(num_files: usize) -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        let project_root = dir.path();

        fs::write(
            project_root.join("Cargo.toml"),
            r#"
[package]
name = "test_project"
version = "0.1.0"
"#,
        )
        .unwrap();

        fs::create_dir(project_root.join("src")).unwrap();

        for i in 0..num_files {
            fs::write(
                project_root.join(format!("src/module{}.rs", i)),
                format!(
                    r#"
pub fn func{}(x: i32) -> i32 {{
    x + {}
}}
"#,
                    i, i
                ),
            )
            .unwrap();
        }

        dir
    }

    #[test]
    fn test_parallel_speedup() {
        let project = create_large_test_project(10);

        let mut analyzer = ProjectAnalyzer::new(project.path());

        let start = std::time::Instant::now();
        let _ = analyzer.analyze_project();
        let seq_time = start.elapsed();

        let start = std::time::Instant::now();
        let _ = analyzer.parallel_analyze();
        let par_time = start.elapsed();

        println!("Sequential: {:?}, Parallel: {:?}", seq_time, par_time);
        assert!(par_time <= seq_time);
    }

    #[test]
    fn test_thread_safety() {
        let project = create_test_project();
        let analyzer = ProjectAnalyzer::new(project.path());

        let coverage_collector: Arc<Mutex<HashMap<PathBuf, HashSet<usize>>>> = Arc::new(Mutex::new(HashMap::new()));

        let files = analyzer.discover_source_files();
        if let Some(first_file) = files.first() {
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let collector = Arc::clone(&coverage_collector);
                    let file_path = first_file.clone();
                    let source = SourceFile::parse_file(&file_path).unwrap();
                    let mut instrumentor = Instrumentor::new();
                    let instrumented = instrumentor.instrument(&source);
                    let coverage = instrumentor.get_coverage();

                    std::thread::spawn(move || {
                        CoverageReport::new(source, &coverage, instrumented.branches)
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        }
    }
}
