//==============================================================================
// Performance Profiling and Optimization Toolkit - Complete Implementation
//==============================================================================

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Mutex;

//==============================================================================
// Milestone 1: CPU Time Profiler
//==============================================================================

thread_local! {
    static PROFILER: std::cell::RefCell<Profiler> = std::cell::RefCell::new(Profiler::new());
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub entry_time: Instant,
    pub parent_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FunctionStats {
    pub function_name: String,
    pub total_time: Duration,
    pub self_time: Duration,
    pub call_count: usize,
    pub avg_time: Duration,
}

pub struct Profiler {
    call_stack: Vec<CallFrame>,
    function_stats: HashMap<String, FunctionStats>,
    start_time: Instant,
    enabled: bool,
}

impl Profiler {
    pub fn new() -> Self {
        Profiler {
            call_stack: Vec::new(),
            function_stats: HashMap::new(),
            start_time: Instant::now(),
            enabled: true,
        }
    }

    pub fn enter_function(&mut self, name: &str) {
        if !self.enabled {
            return;
        }

        let parent_index = if self.call_stack.is_empty() {
            None
        } else {
            Some(self.call_stack.len() - 1)
        };

        self.call_stack.push(CallFrame {
            function_name: name.to_string(),
            entry_time: Instant::now(),
            parent_index,
        });
    }

    pub fn exit_function(&mut self) {
        if !self.enabled {
            return;
        }

        if let Some(frame) = self.call_stack.pop() {
            let duration = frame.entry_time.elapsed();
            let parent_name = frame.parent_index
                .and_then(|idx| self.call_stack.get(idx))
                .map(|f| f.function_name.clone());

            // Update current function stats
            {
                let stats = self.function_stats.entry(frame.function_name.clone())
                    .or_insert(FunctionStats {
                        function_name: frame.function_name.clone(),
                        total_time: Duration::ZERO,
                        self_time: Duration::ZERO,
                        call_count: 0,
                        avg_time: Duration::ZERO,
                    });

                stats.total_time += duration;
                stats.self_time += duration;
                stats.call_count += 1;

                // Update average time
                if stats.call_count > 0 {
                    stats.avg_time = stats.total_time / stats.call_count as u32;
                }
            }

            // Subtract child time from parent's self_time
            if let Some(parent_name) = parent_name {
                if let Some(parent_stats) = self.function_stats.get_mut(&parent_name) {
                    parent_stats.self_time = parent_stats.self_time.saturating_sub(duration);
                }
            }
        }
    }

    pub fn get_stats(&self) -> Vec<FunctionStats> {
        let mut stats: Vec<FunctionStats> = self.function_stats.values().cloned().collect();
        stats.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        stats
    }

    pub fn reset(&mut self) {
        self.call_stack.clear();
        self.function_stats.clear();
        self.start_time = Instant::now();
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

pub struct ProfileGuard {
    _name: String,
}

impl ProfileGuard {
    pub fn new(name: &str) -> Self {
        PROFILER.with(|p| p.borrow_mut().enter_function(name));
        ProfileGuard {
            _name: name.to_string(),
        }
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        PROFILER.with(|p| p.borrow_mut().exit_function());
    }
}

// Public API
pub fn profile_enter(name: &str) {
    PROFILER.with(|p| p.borrow_mut().enter_function(name));
}

pub fn profile_exit() {
    PROFILER.with(|p| p.borrow_mut().exit_function());
}

pub fn get_profile_stats() -> Vec<FunctionStats> {
    PROFILER.with(|p| p.borrow().get_stats())
}

pub fn reset_profiler() {
    PROFILER.with(|p| p.borrow_mut().reset());
}

#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _guard = $crate::ProfileGuard::new($name);
    };
}

//==============================================================================
// Milestone 2: Memory Allocation Tracker
//==============================================================================

lazy_static::lazy_static! {
    static ref ALLOCATION_TRACKER: Mutex<AllocationTracker> =
        Mutex::new(AllocationTracker::new());
}

#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub size: usize,
    pub location: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct AllocStats {
    pub location: String,
    pub count: usize,
    pub total_bytes: usize,
    pub peak_bytes: usize,
    pub avg_size: usize,
    pub current_bytes: usize,
}

pub struct AllocationTracker {
    allocations: HashMap<usize, AllocationInfo>,
    allocation_stats: HashMap<String, AllocStats>,
    total_allocated: usize,
    total_freed: usize,
}

impl AllocationTracker {
    pub fn new() -> Self {
        AllocationTracker {
            allocations: HashMap::new(),
            allocation_stats: HashMap::new(),
            total_allocated: 0,
            total_freed: 0,
        }
    }

    pub fn track_allocation(&mut self, ptr: usize, size: usize, location: &str) {
        self.allocations.insert(ptr, AllocationInfo {
            size,
            location: location.to_string(),
            timestamp: Instant::now(),
        });

        let stats = self.allocation_stats.entry(location.to_string())
            .or_insert(AllocStats {
                location: location.to_string(),
                count: 0,
                total_bytes: 0,
                peak_bytes: 0,
                avg_size: 0,
                current_bytes: 0,
            });

        stats.count += 1;
        stats.total_bytes += size;
        stats.current_bytes += size;

        if stats.current_bytes > stats.peak_bytes {
            stats.peak_bytes = stats.current_bytes;
        }

        stats.avg_size = stats.total_bytes / stats.count;

        self.total_allocated += size;
    }

    pub fn track_deallocation(&mut self, ptr: usize) {
        if let Some(info) = self.allocations.remove(&ptr) {
            self.total_freed += info.size;

            if let Some(stats) = self.allocation_stats.get_mut(&info.location) {
                stats.current_bytes = stats.current_bytes.saturating_sub(info.size);
            }
        }
    }

    pub fn get_hotspots(&self) -> Vec<AllocStats> {
        let mut stats: Vec<AllocStats> = self.allocation_stats.values().cloned().collect();
        stats.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        stats
    }

    pub fn get_live_allocations(&self) -> Vec<AllocationInfo> {
        self.allocations.values().cloned().collect()
    }

    pub fn get_total_allocated(&self) -> usize {
        self.total_allocated
    }

    pub fn get_total_freed(&self) -> usize {
        self.total_freed
    }

    pub fn get_current_usage(&self) -> usize {
        self.total_allocated - self.total_freed
    }
}

// Public API
pub fn track_alloc(ptr: usize, size: usize, location: &str) {
    ALLOCATION_TRACKER.lock().unwrap().track_allocation(ptr, size, location);
}

pub fn track_dealloc(ptr: usize) {
    ALLOCATION_TRACKER.lock().unwrap().track_deallocation(ptr);
}

pub fn get_allocation_hotspots() -> Vec<AllocStats> {
    ALLOCATION_TRACKER.lock().unwrap().get_hotspots()
}

pub fn get_memory_usage() -> usize {
    ALLOCATION_TRACKER.lock().unwrap().get_current_usage()
}

#[macro_export]
macro_rules! track_allocations {
    ($name:expr, $block:block) => {{
        let before = $crate::get_memory_usage();
        let result = $block;
        let after = $crate::get_memory_usage();
        println!("{}: allocated {} bytes", $name, after.saturating_sub(before));
        result
    }};
}

//==============================================================================
// Milestone 3: Flamegraph Generation
//==============================================================================

#[derive(Debug, Clone)]
pub struct CallTree {
    pub name: String,
    pub total_time: Duration,
    pub self_time: Duration,
    pub children: Vec<CallTree>,
}

pub struct Flamegraph {
    call_tree: CallTree,
    max_depth: usize,
}

impl Flamegraph {
    pub fn new(call_tree: CallTree) -> Self {
        let max_depth = Self::calculate_max_depth(&call_tree);
        Flamegraph {
            call_tree,
            max_depth,
        }
    }

    pub fn generate_svg(&self) -> String {
        let width = 1200.0;
        let height = (self.max_depth * 20) as f64 + 40.0;

        let mut svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
<style>
    text {{ font-family: monospace; font-size: 12px; }}
    .frame {{ stroke: white; }}
</style>
"#,
            width, height
        );

        svg.push_str(&self.render_node(&self.call_tree, 0.0, 0.0, width, 20.0));
        svg.push_str("</svg>");

        svg
    }

    fn render_node(&self, node: &CallTree, x: f64, y: f64, width: f64, height: f64) -> String {
        let mut result = String::new();

        // Calculate color
        let color = Self::hash_color(&node.name);

        // Render this node
        result.push_str(&format!(
            r#"<rect class="frame" x="{}" y="{}" width="{}" height="{}" fill="{}" />
<title>{}: {:.2}ms</title>
<text x="{}" y="{}" fill="black">{}</text>
"#,
            x,
            y,
            width,
            height,
            color,
            node.name,
            node.total_time.as_secs_f64() * 1000.0,
            x + 5.0,
            y + 15.0,
            node.name
        ));

        // Render children
        if !node.children.is_empty() {
            let total_child_time: Duration = node.children.iter().map(|c| c.total_time).sum();
            let mut current_x = x;

            for child in &node.children {
                let child_width = if total_child_time.as_nanos() > 0 {
                    width * (child.total_time.as_nanos() as f64 / total_child_time.as_nanos() as f64)
                } else {
                    width / node.children.len() as f64
                };

                result.push_str(&self.render_node(child, current_x, y + height, child_width, height));
                current_x += child_width;
            }
        }

        result
    }

    fn calculate_max_depth(node: &CallTree) -> usize {
        if node.children.is_empty() {
            1
        } else {
            1 + node.children.iter()
                .map(|c| Self::calculate_max_depth(c))
                .max()
                .unwrap_or(0)
        }
    }

    fn hash_color(name: &str) -> String {
        let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let hue = (hash % 360) as f64;
        format!("hsl({}, 70%, 60%)", hue)
    }
}

pub fn build_call_tree_from_stats(stats: &[FunctionStats]) -> CallTree {
    if stats.is_empty() {
        return CallTree {
            name: "root".to_string(),
            total_time: Duration::ZERO,
            self_time: Duration::ZERO,
            children: vec![],
        };
    }

    // Simple implementation: create flat tree
    let total_time: Duration = stats.iter().map(|s| s.total_time).max().unwrap_or(Duration::ZERO);

    CallTree {
        name: "root".to_string(),
        total_time,
        self_time: Duration::ZERO,
        children: stats.iter().map(|s| CallTree {
            name: s.function_name.clone(),
            total_time: s.total_time,
            self_time: s.self_time,
            children: vec![],
        }).collect(),
    }
}

//==============================================================================
// Milestone 5: Automated Performance Analysis
//==============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Category {
    ExcessiveAllocation,
    HotLoop,
    LargeAllocation,
    FrequentAllocation,
    DeepCallStack,
    SlowFunction,
}

#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub severity: Severity,
    pub category: Category,
    pub description: String,
    pub recommendation: String,
    pub location: String,
}

pub struct PerformanceAnalyzer {
    cpu_stats: Vec<FunctionStats>,
    alloc_stats: Vec<AllocStats>,
}

impl PerformanceAnalyzer {
    pub fn new(cpu_stats: Vec<FunctionStats>, alloc_stats: Vec<AllocStats>) -> Self {
        PerformanceAnalyzer {
            cpu_stats,
            alloc_stats,
        }
    }

    pub fn analyze(&self) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        issues.extend(self.detect_allocation_issues());
        issues.extend(self.detect_cpu_issues());
        issues.extend(self.detect_hotloops());

        issues.sort_by_key(|issue| issue.severity);

        issues
    }

    fn detect_allocation_issues(&self) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        for stats in &self.alloc_stats {
            // Detect frequent allocations
            if stats.count > 1000 {
                issues.push(PerformanceIssue {
                    severity: if stats.count > 10000 { Severity::Critical } else { Severity::High },
                    category: Category::FrequentAllocation,
                    description: format!("{} allocations at {}", stats.count, stats.location),
                    recommendation: format!(
                        "Consider using Vec::with_capacity() or SmallVec to reduce allocations"
                    ),
                    location: stats.location.clone(),
                });
            }

            // Detect large allocations
            if stats.avg_size > 1024 * 1024 {
                issues.push(PerformanceIssue {
                    severity: Severity::Medium,
                    category: Category::LargeAllocation,
                    description: format!("Large allocation: {} bytes", stats.avg_size),
                    recommendation: "Consider streaming or chunking data".to_string(),
                    location: stats.location.clone(),
                });
            }

            // Detect excessive total allocation
            if stats.total_bytes > 100 * 1024 * 1024 {
                issues.push(PerformanceIssue {
                    severity: Severity::High,
                    category: Category::ExcessiveAllocation,
                    description: format!("Total {} MB allocated", stats.total_bytes / (1024 * 1024)),
                    recommendation: "Review memory usage patterns".to_string(),
                    location: stats.location.clone(),
                });
            }
        }

        issues
    }

    fn detect_cpu_issues(&self) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        if self.cpu_stats.is_empty() {
            return issues;
        }

        let total_time: Duration = self.cpu_stats.iter()
            .map(|s| s.total_time)
            .max()
            .unwrap_or(Duration::ZERO);

        for stats in &self.cpu_stats {
            let percent = if total_time.as_nanos() > 0 {
                (stats.total_time.as_nanos() as f64 / total_time.as_nanos() as f64) * 100.0
            } else {
                0.0
            };

            // Detect functions taking significant time
            if percent > 50.0 {
                issues.push(PerformanceIssue {
                    severity: Severity::Critical,
                    category: Category::SlowFunction,
                    description: format!("{} takes {:.1}% of total time", stats.function_name, percent),
                    recommendation: "Profile and optimize this function".to_string(),
                    location: stats.function_name.clone(),
                });
            }

            // Detect slow individual calls
            if stats.total_time > Duration::from_secs(10) {
                issues.push(PerformanceIssue {
                    severity: Severity::High,
                    category: Category::SlowFunction,
                    description: format!("{} takes {:.2}s total", stats.function_name, stats.total_time.as_secs_f64()),
                    recommendation: "Review algorithm complexity".to_string(),
                    location: stats.function_name.clone(),
                });
            }
        }

        issues
    }

    fn detect_hotloops(&self) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        for stats in &self.cpu_stats {
            // Detect functions called very frequently
            if stats.call_count > 10000 {
                issues.push(PerformanceIssue {
                    severity: if stats.call_count > 100000 { Severity::High } else { Severity::Medium },
                    category: Category::HotLoop,
                    description: format!("{} called {} times", stats.function_name, stats.call_count),
                    recommendation: "Consider caching, memoization, or loop hoisting".to_string(),
                    location: stats.function_name.clone(),
                });
            }
        }

        issues
    }

    pub fn generate_report(&self) -> String {
        let issues = self.analyze();

        let mut report = String::new();
        report.push_str("=== Performance Analysis Report ===\n\n");

        if issues.is_empty() {
            report.push_str("No issues found. Performance looks good!\n");
            return report;
        }

        report.push_str(&format!("Found {} issues:\n\n", issues.len()));

        let mut critical: Vec<&PerformanceIssue> = issues.iter().filter(|i| i.severity == Severity::Critical).collect();
        let mut high: Vec<&PerformanceIssue> = issues.iter().filter(|i| i.severity == Severity::High).collect();
        let mut medium: Vec<&PerformanceIssue> = issues.iter().filter(|i| i.severity == Severity::Medium).collect();
        let mut low: Vec<&PerformanceIssue> = issues.iter().filter(|i| i.severity == Severity::Low).collect();

        if !critical.is_empty() {
            report.push_str("CRITICAL Issues:\n");
            for issue in critical {
                report.push_str(&format!("  - {}\n", issue.description));
                report.push_str(&format!("    → {}\n", issue.recommendation));
            }
            report.push_str("\n");
        }

        if !high.is_empty() {
            report.push_str("HIGH Priority Issues:\n");
            for issue in high {
                report.push_str(&format!("  - {}\n", issue.description));
                report.push_str(&format!("    → {}\n", issue.recommendation));
            }
            report.push_str("\n");
        }

        if !medium.is_empty() {
            report.push_str("MEDIUM Priority Issues:\n");
            for issue in medium {
                report.push_str(&format!("  - {}\n", issue.description));
            }
            report.push_str("\n");
        }

        report
    }
}

//==============================================================================
// Milestone 6: Optimization Validation and Comparison
//==============================================================================

#[derive(Debug, Clone)]
pub enum Metric {
    TotalTime,
    AllocationCount,
    AllocationBytes,
    CallCount,
}

#[derive(Debug, Clone)]
pub struct Improvement {
    pub function: String,
    pub metric: Metric,
    pub before_value: f64,
    pub after_value: f64,
    pub percent_change: f64,
}

#[derive(Debug, Clone)]
pub struct ProfileSnapshot {
    pub name: String,
    pub cpu_stats: Vec<FunctionStats>,
    pub alloc_stats: Vec<AllocStats>,
    pub total_time: Duration,
}

pub struct ProfileComparison {
    before: ProfileSnapshot,
    after: ProfileSnapshot,
}

impl ProfileComparison {
    pub fn new(before: ProfileSnapshot, after: ProfileSnapshot) -> Self {
        ProfileComparison { before, after }
    }

    pub fn find_improvements(&self) -> Vec<Improvement> {
        let mut improvements = Vec::new();

        // Compare CPU stats
        for before_stat in &self.before.cpu_stats {
            if let Some(after_stat) = self.after.cpu_stats.iter()
                .find(|s| s.function_name == before_stat.function_name) {

                let before_ms = before_stat.total_time.as_secs_f64() * 1000.0;
                let after_ms = after_stat.total_time.as_secs_f64() * 1000.0;
                let percent_change = ((after_ms - before_ms) / before_ms) * 100.0;

                if percent_change < -5.0 {  // At least 5% improvement
                    improvements.push(Improvement {
                        function: before_stat.function_name.clone(),
                        metric: Metric::TotalTime,
                        before_value: before_ms,
                        after_value: after_ms,
                        percent_change,
                    });
                }
            }
        }

        // Compare allocation stats
        for before_alloc in &self.before.alloc_stats {
            if let Some(after_alloc) = self.after.alloc_stats.iter()
                .find(|s| s.location == before_alloc.location) {

                let percent_change = ((after_alloc.count as f64 - before_alloc.count as f64) / before_alloc.count as f64) * 100.0;

                if percent_change < -5.0 {
                    improvements.push(Improvement {
                        function: before_alloc.location.clone(),
                        metric: Metric::AllocationCount,
                        before_value: before_alloc.count as f64,
                        after_value: after_alloc.count as f64,
                        percent_change,
                    });
                }
            }
        }

        improvements
    }

    pub fn find_regressions(&self) -> Vec<Improvement> {
        let mut regressions = Vec::new();

        // Compare CPU stats
        for before_stat in &self.before.cpu_stats {
            if let Some(after_stat) = self.after.cpu_stats.iter()
                .find(|s| s.function_name == before_stat.function_name) {

                let before_ms = before_stat.total_time.as_secs_f64() * 1000.0;
                let after_ms = after_stat.total_time.as_secs_f64() * 1000.0;
                let percent_change = ((after_ms - before_ms) / before_ms) * 100.0;

                if percent_change > 5.0 {  // At least 5% worse
                    regressions.push(Improvement {
                        function: before_stat.function_name.clone(),
                        metric: Metric::TotalTime,
                        before_value: before_ms,
                        after_value: after_ms,
                        percent_change,
                    });
                }
            }
        }

        regressions
    }

    pub fn overall_speedup(&self) -> f64 {
        let before_ms = self.before.total_time.as_secs_f64() * 1000.0;
        let after_ms = self.after.total_time.as_secs_f64() * 1000.0;

        if after_ms > 0.0 {
            before_ms / after_ms
        } else {
            1.0
        }
    }

    pub fn generate_comparison_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Optimization Comparison Report ===\n\n");

        let speedup = self.overall_speedup();
        report.push_str(&format!("Overall Speedup: {:.2}x\n", speedup));
        report.push_str(&format!("Before: {:.2}ms\n", self.before.total_time.as_secs_f64() * 1000.0));
        report.push_str(&format!("After: {:.2}ms\n\n", self.after.total_time.as_secs_f64() * 1000.0));

        let improvements = self.find_improvements();
        if !improvements.is_empty() {
            report.push_str("Improvements:\n");
            for imp in &improvements {
                report.push_str(&format!(
                    "  - {}: {:.2} → {:.2} ({:.1}% faster)\n",
                    imp.function,
                    imp.before_value,
                    imp.after_value,
                    imp.percent_change.abs()
                ));
            }
            report.push_str("\n");
        }

        let regressions = self.find_regressions();
        if !regressions.is_empty() {
            report.push_str("⚠ Regressions:\n");
            for reg in &regressions {
                report.push_str(&format!(
                    "  - {}: {:.2} → {:.2} ({:.1}% slower)\n",
                    reg.function,
                    reg.before_value,
                    reg.after_value,
                    reg.percent_change
                ));
            }
        }

        report
    }
}

pub fn capture_snapshot(name: &str) -> ProfileSnapshot {
    let cpu_stats = get_profile_stats();
    let total_time = cpu_stats.iter()
        .map(|s| s.total_time)
        .max()
        .unwrap_or(Duration::ZERO);

    ProfileSnapshot {
        name: name.to_string(),
        cpu_stats,
        alloc_stats: get_allocation_hotspots(),
        total_time,
    }
}

//==============================================================================
// Main Example
//==============================================================================

fn main() {
    println!("=== Performance Profiling Toolkit ===\n");

    println!("This toolkit provides comprehensive performance profiling:");
    println!("  1. CPU time profiling with call stack tracking");
    println!("  2. Memory allocation profiling and tracking");
    println!("  3. Flamegraph generation for visualization");
    println!("  4. Automated bottleneck detection");
    println!("  5. Before/after comparison for validation");
    println!("\nProfile your code to find the critical 20% that takes 80% of the time!");
}

//==============================================================================
// Tests
//==============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // Milestone 1 Tests
    fn slow_function() {
        let _guard = ProfileGuard::new("slow_function");
        thread::sleep(Duration::from_millis(10));
    }

    fn fast_function() {
        let _guard = ProfileGuard::new("fast_function");
        thread::sleep(Duration::from_millis(1));
    }

    fn outer_function() {
        let _guard = ProfileGuard::new("outer_function");
        fast_function();
        slow_function();
    }

    #[test]
    fn test_basic_profiling() {
        reset_profiler();

        slow_function();

        let stats = get_profile_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].function_name, "slow_function");
        assert_eq!(stats[0].call_count, 1);
        assert!(stats[0].total_time >= Duration::from_millis(10));
    }

    #[test]
    fn test_multiple_calls() {
        reset_profiler();

        fast_function();
        fast_function();
        fast_function();

        let stats = get_profile_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].call_count, 3);
    }

    #[test]
    fn test_nested_calls() {
        reset_profiler();

        outer_function();

        let stats = get_profile_stats();

        assert_eq!(stats.len(), 3);

        let outer = stats.iter().find(|s| s.function_name == "outer_function").unwrap();
        let slow = stats.iter().find(|s| s.function_name == "slow_function").unwrap();

        assert!(outer.total_time > slow.total_time);
        assert!(outer.self_time < Duration::from_millis(5));
    }

    #[test]
    fn test_average_time() {
        reset_profiler();

        for _ in 0..10 {
            fast_function();
        }

        let stats = get_profile_stats();
        let fast = &stats[0];

        assert_eq!(fast.call_count, 10);
        assert!(fast.avg_time >= Duration::from_millis(1));
        assert!(fast.avg_time <= fast.total_time);
    }

    // Milestone 2 Tests
    #[test]
    fn test_allocation_tracking() {
        let mut tracker = AllocationTracker::new();

        tracker.track_allocation(0x1000, 100, "test_function");

        assert_eq!(tracker.get_total_allocated(), 100);
        assert_eq!(tracker.get_current_usage(), 100);

        tracker.track_deallocation(0x1000);

        assert_eq!(tracker.get_total_freed(), 100);
        assert_eq!(tracker.get_current_usage(), 0);
    }

    #[test]
    fn test_hotspot_detection() {
        let mut tracker = AllocationTracker::new();

        for i in 0..10 {
            tracker.track_allocation(i, 100, "hot_function");
        }

        for i in 10..12 {
            tracker.track_allocation(i, 50, "cold_function");
        }

        let hotspots = tracker.get_hotspots();

        assert_eq!(hotspots[0].location, "hot_function");
        assert_eq!(hotspots[0].count, 10);
        assert_eq!(hotspots[0].total_bytes, 1000);
    }

    #[test]
    fn test_memory_leak_detection() {
        let mut tracker = AllocationTracker::new();

        tracker.track_allocation(0x1000, 100, "leak_function");
        tracker.track_allocation(0x2000, 200, "leak_function");
        tracker.track_deallocation(0x1000);

        let leaks = tracker.get_live_allocations();

        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].size, 200);
    }

    #[test]
    fn test_allocation_stats() {
        let mut tracker = AllocationTracker::new();

        tracker.track_allocation(0x1000, 100, "func");
        tracker.track_allocation(0x2000, 200, "func");
        tracker.track_allocation(0x3000, 300, "func");

        let hotspots = tracker.get_hotspots();
        let stats = &hotspots[0];

        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_bytes, 600);
        assert_eq!(stats.avg_size, 200);
    }

    // Milestone 3 Tests
    #[test]
    fn test_flamegraph_generation() {
        let tree = CallTree {
            name: "main".to_string(),
            total_time: Duration::from_millis(100),
            self_time: Duration::from_millis(10),
            children: vec![
                CallTree {
                    name: "slow_func".to_string(),
                    total_time: Duration::from_millis(80),
                    self_time: Duration::from_millis(80),
                    children: vec![],
                },
                CallTree {
                    name: "fast_func".to_string(),
                    total_time: Duration::from_millis(10),
                    self_time: Duration::from_millis(10),
                    children: vec![],
                },
            ],
        };

        let flamegraph = Flamegraph::new(tree);
        let svg = flamegraph.generate_svg();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));

        assert!(svg.contains("main"));
        assert!(svg.contains("slow_func"));
        assert!(svg.contains("fast_func"));
    }

    #[test]
    fn test_max_depth_calculation() {
        let tree = CallTree {
            name: "a".to_string(),
            total_time: Duration::from_millis(100),
            self_time: Duration::from_millis(0),
            children: vec![
                CallTree {
                    name: "b".to_string(),
                    total_time: Duration::from_millis(100),
                    self_time: Duration::from_millis(0),
                    children: vec![
                        CallTree {
                            name: "c".to_string(),
                            total_time: Duration::from_millis(100),
                            self_time: Duration::from_millis(100),
                            children: vec![],
                        },
                    ],
                },
            ],
        };

        let depth = Flamegraph::calculate_max_depth(&tree);
        assert_eq!(depth, 3);
    }

    #[test]
    fn test_color_consistency() {
        let color1 = Flamegraph::hash_color("test_function");
        let color2 = Flamegraph::hash_color("test_function");

        assert_eq!(color1, color2);
    }

    // Milestone 5 Tests
    #[test]
    fn test_detect_excessive_allocation() {
        let alloc_stats = vec![
            AllocStats {
                location: "hot_loop".to_string(),
                count: 10000,
                total_bytes: 1000000,
                peak_bytes: 100000,
                avg_size: 100,
                current_bytes: 0,
            },
        ];

        let analyzer = PerformanceAnalyzer::new(vec![], alloc_stats);
        let issues = analyzer.detect_allocation_issues();

        assert!(issues.iter().any(|i| {
            i.category == Category::FrequentAllocation
        }));
    }

    #[test]
    fn test_detect_slow_function() {
        let cpu_stats = vec![
            FunctionStats {
                function_name: "slow".to_string(),
                total_time: Duration::from_secs(10),
                self_time: Duration::from_secs(10),
                call_count: 1,
                avg_time: Duration::from_secs(10),
            },
            FunctionStats {
                function_name: "fast".to_string(),
                total_time: Duration::from_millis(100),
                self_time: Duration::from_millis(100),
                call_count: 100,
                avg_time: Duration::from_millis(1),
            },
        ];

        let analyzer = PerformanceAnalyzer::new(cpu_stats, vec![]);
        let issues = analyzer.detect_cpu_issues();

        assert!(issues.iter().any(|i| {
            i.location == "slow" && i.category == Category::SlowFunction
        }));
    }

    #[test]
    fn test_severity_prioritization() {
        let cpu_stats = vec![
            FunctionStats {
                function_name: "critical".to_string(),
                total_time: Duration::from_secs(100),
                self_time: Duration::from_secs(100),
                call_count: 1,
                avg_time: Duration::from_secs(100),
            },
        ];

        let analyzer = PerformanceAnalyzer::new(cpu_stats, vec![]);
        let issues = analyzer.analyze();

        assert!(issues[0].severity == Severity::Critical);
    }

    #[test]
    fn test_generate_report() {
        let analyzer = PerformanceAnalyzer::new(vec![], vec![]);
        let report = analyzer.generate_report();

        assert!(report.contains("Performance Analysis"));
        assert!(report.contains("No issues"));
    }

    // Milestone 6 Tests
    fn create_test_snapshot(name: &str, total_ms: u64) -> ProfileSnapshot {
        ProfileSnapshot {
            name: name.to_string(),
            cpu_stats: vec![],
            alloc_stats: vec![],
            total_time: Duration::from_millis(total_ms),
        }
    }

    #[test]
    fn test_speedup_calculation() {
        let before = create_test_snapshot("before", 1000);
        let after = create_test_snapshot("after", 500);

        let comparison = ProfileComparison::new(before, after);
        let speedup = comparison.overall_speedup();

        assert_eq!(speedup, 2.0);
    }

    #[test]
    fn test_detect_improvement() {
        let before = ProfileSnapshot {
            name: "before".to_string(),
            cpu_stats: vec![
                FunctionStats {
                    function_name: "optimized".to_string(),
                    total_time: Duration::from_millis(1000),
                    self_time: Duration::from_millis(1000),
                    call_count: 100,
                    avg_time: Duration::from_millis(10),
                },
            ],
            alloc_stats: vec![],
            total_time: Duration::from_secs(1),
        };

        let after = ProfileSnapshot {
            name: "after".to_string(),
            cpu_stats: vec![
                FunctionStats {
                    function_name: "optimized".to_string(),
                    total_time: Duration::from_millis(500),
                    self_time: Duration::from_millis(500),
                    call_count: 100,
                    avg_time: Duration::from_millis(5),
                },
            ],
            alloc_stats: vec![],
            total_time: Duration::from_millis(500),
        };

        let comparison = ProfileComparison::new(before, after);
        let improvements = comparison.find_improvements();

        assert!(improvements.len() > 0);
        assert_eq!(improvements[0].function, "optimized");
        assert!(improvements[0].percent_change < 0.0);
    }

    #[test]
    fn test_detect_regression() {
        let before = create_test_snapshot("before", 100);
        let after = create_test_snapshot("after", 200);

        let comparison = ProfileComparison::new(before, after);
        let speedup = comparison.overall_speedup();

        assert!(speedup < 1.0);
    }
}
