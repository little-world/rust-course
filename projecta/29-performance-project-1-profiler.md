# Performance Profiling and Optimization Toolkit

### Problem Statement

Build a comprehensive performance profiling toolkit that tracks CPU time, memory allocations, cache behavior, and function call statistics. Your profiler should instrument code to measure performance metrics, generate flamegraphs showing hotspots, track allocation patterns, and provide actionable optimization recommendations based on collected data.

Your profiling toolkit should support:
- CPU time profiling with call stack tracking
- Memory allocation profiling and tracking
- Cache miss detection and analysis
- Function-level statistics (call count, average/total time)
- Flamegraph generation for visualization
- Automated bottleneck detection and recommendations
- Before/after comparison for optimization validation

## Why Performance Profiling Matters

Performance is not just about making code "fast"—it's about efficiency, scalability, user experience, and cost. Profiling is the only scientific way to achieve these goals.

### 1. The Intuition Gap (Developer Efficiency)

**The Problem**: Developer intuition about performance is notoriously unreliable. Humans are bad at estimating the cost of complex instruction sequences, cache misses, and lock contention. Without profiling, you are optimizing in the dark.

**Real-world example**:
```rust
fn process_data(items: Vec<String>) -> Vec<String> {
    items.iter()
        .filter(|s| validate(s))      // Developer thinks: "This is slow!"
        .map(|s| transform(s))         // Developer thinks: "Lots of allocation!"
        .collect()                      // Developer thinks: "collect() is cheap"
}
```

### 2. User Experience and Business Impact

Latency directly correlates with user satisfaction and conversion rates.
*   **Web**: Amazon found every 100ms of latency cost them 1% in sales. Google found an extra 0.5 seconds in search generation dropped traffic by 20%.
*   **Interactive Apps**: UI freezes (jank) of even 50ms feel "sluggish" to users. 16ms (60fps) is the gold standard.
*   **API Response**: Slow APIs cause timeouts, retries, and cascading failures in microservices.

### 3. Resource Efficiency and Cost

Inefficient code burns money and energy.
*   **Cloud Bills**: If your service requires 100 servers to handle traffic that 10 optimized servers could manage, you are wasting massive amounts of money.
*   **Battery Life**: On mobile/embedded devices, CPU cycles drain battery. An unoptimized background loop can kill a device's battery in hours.
*   **Sustainability**: Data centers consume vast amounts of electricity. Efficient code is green code.

### 4. Scalability and System Stability

Performance bottlenecks are often invisible at low load but catastrophic at scale.
*   **The "Death Spiral"**: A slow endpoint might work fine for 10 users but cause a thread pool exhaustion and total system crash with 1000 users.
*   **Memory Pressure**: Unchecked allocations lead to OOM (Out Of Memory) kills, causing service instability.

### 5. The 80/20 Rule (Pareto Principle)

In almost every program, **80% of the execution time is spent in 20% of the code**. Profiling identifies that critical 20%.

**Profiling reveals the truth**:
```
Function          Time    Calls    Avg Time    % Total
validate()        900ms   100,000  9μs         90%
transform()       80ms    10,000   8μs         8%
collect()         20ms    1        20ms        2%

Total: 1000ms
```

**Impact of profiling-driven optimization**:
- **Blind Optimization**: Spending 2 days on `transform()` (8% impact) yields a maximum 1.08x speedup.
- **Targeted Optimization**: Spending 2 hours on `validate()` (90% impact) could yield a 10x speedup.

### Common Performance Myths vs Reality

| Myth | Reality (from profiling) |
|------|-------------------------|
| "Allocations are slow" | Often true, but 90% of time might be in string processing logic, not the allocation itself. |
| "This loop is the bottleneck" | Actually, it might be the hash lookups *inside* the loop. |
| "Micro-optimizations matter" | 99% of time is usually in one poorly-chosen algorithm (O(n²) vs O(n)). |
| "More cores = faster" | Mutex contention and cache thrashing can make threaded code *slower*. |
| "This can't be optimized more" | 10x speedup is often possible by changing data layout (Data-Oriented Design). |

## Use Cases

### 1. Development Workflow
- **Find hotspots**: Identify which 20% of code takes 80% of time
- **Validate optimizations**: Measure before/after to confirm improvement
- **Catch regressions**: Detect when changes slow down code
- **Guide decisions**: Choose algorithms based on actual measurements

### 2. Production Diagnostics
- **Debug slow requests**: Identify why specific requests are slow
- **Capacity planning**: Understand resource usage patterns
- **Optimize critical paths**: Focus on code that actually matters
- **Memory leaks**: Track allocation patterns over time

### 3. Algorithm Selection
- **Compare implementations**: Measure Vec vs LinkedList vs custom structure
- **Scaling analysis**: How does performance change with input size?
- **Cache behavior**: Understand cache-friendliness of data structures
- **Allocation patterns**: Identify unnecessary allocations

### 4. Educational Tool
- **Understand performance**: See actual cost of operations
- **Learn optimization**: Measure impact of techniques
- **Debug performance**: Find unexpected bottlenecks
- **Benchmark comprehension**: Interpret profiling data

---

## Building the Project

### Milestone 1: CPU Time Profiler

**Goal**: Build a basic CPU profiler that tracks time spent in each function using function entry/exit hooks.

**Why we start here**: CPU profiling is the foundation—knowing where time is spent drives all optimization decisions.

#### Architecture

**Structs:**
- `Profiler` - Main profiling engine
  - **Field**: `call_stack: Vec<CallFrame>` - Current call stack
  - **Field**: `function_stats: HashMap<String, FunctionStats>` - Per-function statistics
  - **Field**: `start_time: Instant` - Profiling session start
  - **Field**: `enabled: bool` - Whether profiling is active

- `CallFrame` - One function call on the stack
  - **Field**: `function_name: String` - Function being called
  - **Field**: `entry_time: Instant` - When function was entered
  - **Field**: `parent_index: Option<usize>` - Parent frame index

- `FunctionStats` - Statistics for one function
  - **Field**: `total_time: Duration` - Total time across all calls
  - **Field**: `self_time: Duration` - Time excluding children
  - **Field**: `call_count: usize` - Number of times called
  - **Field**: `avg_time: Duration` - Average time per call

**Functions:**
- `new() -> Profiler` - Create profiler
- `enter_function(&mut self, name: &str)` - Record function entry
- `exit_function(&mut self)` - Record function exit
- `get_stats(&self) -> Vec<FunctionStats>` - Get sorted statistics
- `reset(&mut self)` - Clear all collected data

**Starter Code**:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
        // TODO: Initialize profiler
        todo!("Create profiler")
    }

    pub fn enter_function(&mut self, name: &str) {
        // TODO: Push frame onto call stack
        // TODO: Record entry time
        todo!("Enter function")
    }

    pub fn exit_function(&mut self) {
        // TODO: Pop frame from call stack
        // TODO: Calculate duration
        // TODO: Update function_stats
        // TODO: Update parent's self_time
        todo!("Exit function")
    }

    pub fn get_stats(&self) -> Vec<FunctionStats> {
        // TODO: Collect all stats
        // TODO: Sort by total_time descending
        todo!("Get statistics")
    }

    pub fn reset(&mut self) {
        // TODO: Clear call stack
        // TODO: Clear function stats
        todo!("Reset profiler")
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

// Convenience macros for profiling
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _guard = ProfileGuard::new($name);
    };
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
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn slow_function() {
        profile_scope!("slow_function");
        thread::sleep(Duration::from_millis(10));
    }

    fn fast_function() {
        profile_scope!("fast_function");
        thread::sleep(Duration::from_millis(1));
    }

    fn outer_function() {
        profile_scope!("outer_function");
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

        // Should have stats for outer, fast, and slow
        assert_eq!(stats.len(), 3);

        // Find outer function stats
        let outer = stats.iter().find(|s| s.function_name == "outer_function").unwrap();
        let slow = stats.iter().find(|s| s.function_name == "slow_function").unwrap();

        // Outer should include time of children
        assert!(outer.total_time > slow.total_time);
        // But self_time should be small
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
}
```

**Check Your Understanding**:
- Why use thread-local storage for the profiler?
- How do we distinguish total_time from self_time?
- What happens if exit_function() is called without matching enter_function()?

---

#### Why Milestone 1 Isn't Enough

**Limitation**: CPU time profiling shows where time is spent, but doesn't reveal memory allocation patterns—often the actual bottleneck.

**What we're adding**: Memory allocation tracking to identify allocation hotspots and excessive allocations.

**Improvement**:
- **Allocation tracking**: See where allocations happen
- **Size tracking**: Identify large allocations
- **Frequency analysis**: Find allocation-heavy loops
- **Actionable data**: Know what to optimize for memory

---

### Milestone 2: Memory Allocation Tracker

**Goal**: Track all heap allocations to identify allocation hotspots and patterns.

**Why this matters**: Allocations are often 10-100x slower than stack operations. Reducing allocations can yield dramatic speedups.

#### Architecture

**Structs:**
- `AllocationTracker` - Tracks memory allocations
  - **Field**: `allocations: HashMap<usize, AllocationInfo>` - Active allocations
  - **Field**: `allocation_stats: HashMap<String, AllocStats>` - Per-location stats
  - **Field**: `total_allocated: usize` - Total bytes allocated
  - **Field**: `total_freed: usize` - Total bytes freed

- `AllocationInfo` - Information about one allocation
  - **Field**: `size: usize` - Bytes allocated
  - **Field**: `location: String` - Where allocated (function name)
  - **Field**: `timestamp: Instant` - When allocated

- `AllocStats` - Statistics for allocations at one location
  - **Field**: `count: usize` - Number of allocations
  - **Field**: `total_bytes: usize` - Total bytes allocated
  - **Field**: `peak_bytes: usize` - Peak simultaneous bytes
  - **Field**: `avg_size: usize` - Average allocation size

**Functions:**
- `track_allocation(&mut self, ptr: usize, size: usize, location: &str)` - Record allocation
- `track_deallocation(&mut self, ptr: usize)` - Record free
- `get_hotspots(&self) -> Vec<AllocStats>` - Get top allocation sites
- `get_live_allocations(&self) -> Vec<AllocationInfo>` - Get memory leaks
- `get_total_allocated(&self) -> usize` - Total allocation size

**Starter Code**:

```rust
use std::collections::HashMap;
use std::time::Instant;
use std::sync::Mutex;

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
        // TODO: Initialize tracker
        todo!("Create allocation tracker")
    }

    pub fn track_allocation(&mut self, ptr: usize, size: usize, location: &str) {
        // TODO: Record allocation in allocations map
        // TODO: Update allocation_stats for this location
        // TODO: Update total_allocated
        // TODO: Update peak_bytes if necessary
        todo!("Track allocation")
    }

    pub fn track_deallocation(&mut self, ptr: usize) {
        // TODO: Look up allocation
        // TODO: Update allocation_stats
        // TODO: Update total_freed
        // TODO: Remove from allocations map
        todo!("Track deallocation")
    }

    pub fn get_hotspots(&self) -> Vec<AllocStats> {
        // TODO: Collect all AllocStats
        // TODO: Sort by total_bytes descending
        // TODO: Return top N
        todo!("Get allocation hotspots")
    }

    pub fn get_live_allocations(&self) -> Vec<AllocationInfo> {
        // TODO: Return all current allocations
        // TODO: Potential memory leaks if this list is large
        todo!("Get live allocations")
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

// Macro to track allocations in a scope
#[macro_export]
macro_rules! track_allocations {
    ($name:expr, $block:block) => {{
        let before = get_memory_usage();
        let result = $block;
        let after = get_memory_usage();
        println!("{}: allocated {} bytes", $name, after.saturating_sub(before));
        result
    }};
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

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

        // Allocate from different locations
        for i in 0..10 {
            tracker.track_allocation(i, 100, "hot_function");
        }

        for i in 10..12 {
            tracker.track_allocation(i, 50, "cold_function");
        }

        let hotspots = tracker.get_hotspots();

        // hot_function should be #1 hotspot
        assert_eq!(hotspots[0].location, "hot_function");
        assert_eq!(hotspots[0].count, 10);
        assert_eq!(hotspots[0].total_bytes, 1000);
    }

    #[test]
    fn test_memory_leak_detection() {
        let mut tracker = AllocationTracker::new();

        tracker.track_allocation(0x1000, 100, "leak_function");
        tracker.track_allocation(0x2000, 200, "leak_function");
        tracker.track_deallocation(0x1000);  // Only deallocate one

        let leaks = tracker.get_live_allocations();

        // Should have one leaked allocation
        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].size, 200);
    }

    #[test]
    fn test_allocation_stats() {
        let mut tracker = AllocationTracker::new();

        // Multiple allocations of different sizes
        tracker.track_allocation(0x1000, 100, "func");
        tracker.track_allocation(0x2000, 200, "func");
        tracker.track_allocation(0x3000, 300, "func");

        let hotspots = tracker.get_hotspots();
        let stats = &hotspots[0];

        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_bytes, 600);
        assert_eq!(stats.avg_size, 200);
    }
}
```

---

#### Why Milestone 2 Isn't Enough

**Limitation**: We collect profiling data but have no way to visualize it. Raw numbers are hard to interpret.

**What we're adding**: Flamegraph generation to visualize where time is spent in an intuitive, interactive format.

**Improvement**:
- **Visualization**: See hotspots at a glance
- **Call hierarchy**: Understand caller/callee relationships
- **Proportional display**: Width shows relative time
- **Interactive**: Click to zoom, explore call paths

---

### Milestone 3: Flamegraph Generation

**Goal**: Generate SVG flamegraphs that visualize profiling data.

**Why this matters**: Flamegraphs make performance bottlenecks immediately obvious. A wide bar = expensive function.

#### Architecture

**Structs:**
- `Flamegraph` - Flamegraph generator
  - **Field**: `call_tree: CallTree` - Hierarchical call data
  - **Field**: `max_depth: usize` - Maximum stack depth

- `CallTree` - Hierarchical representation of calls
  - **Field**: `name: String` - Function name
  - **Field**: `total_time: Duration` - Time including children
  - **Field**: `children: Vec<CallTree>` - Child function calls

**Functions:**
- `build_call_tree(stats: Vec<FunctionStats>) -> CallTree` - Build hierarchy
- `generate_svg(&self) -> String` - Generate SVG flamegraph
- `render_node(&self, node: &CallTree, x: f64, y: f64, width: f64) -> String` - Render one node

**Starter Code**:

```rust
use std::time::Duration;

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
        // TODO: Calculate max_depth
        todo!("Create flamegraph")
    }

    pub fn generate_svg(&self) -> String {
        // TODO: Generate SVG header
        // TODO: Calculate dimensions
        // TODO: Render call tree recursively
        // TODO: Add tooltips and interactivity
        todo!("Generate SVG")
    }

    fn render_node(&self, node: &CallTree, x: f64, y: f64, width: f64, height: f64) -> String {
        // TODO: Create SVG rect element
        // TODO: Calculate color based on function name hash
        // TODO: Add text label
        // TODO: Add tooltip with timing info
        // TODO: Recursively render children
        todo!("Render flamegraph node")
    }

    fn calculate_max_depth(node: &CallTree) -> usize {
        // TODO: Recursively find maximum depth
        todo!("Calculate max depth")
    }

    fn hash_color(name: &str) -> String {
        // TODO: Generate consistent color from function name
        // TODO: Use HSL color space for better visibility
        todo!("Generate color for function")
    }
}

pub fn build_call_tree_from_stats(stats: &[FunctionStats]) -> CallTree {
    // TODO: Reconstruct call hierarchy from flat stats
    // TODO: This requires tracking parent-child relationships
    todo!("Build call tree")
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

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

        // Should contain SVG elements
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));

        // Should contain function names
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
        // Same function name should always get same color
        let color1 = Flamegraph::hash_color("test_function");
        let color2 = Flamegraph::hash_color("test_function");

        assert_eq!(color1, color2);
    }
}
```

---

#### Why Milestone 3 Isn't Enough

**Limitation**: Manual instrumentation is tedious and error-prone. Developers must remember to add profile_scope!() everywhere.

**What we're adding**: Automatic instrumentation via procedural macros that instrument all functions transparently.

**Improvement**:
- **Automation**: No manual instrumentation needed
- **Completeness**: Never miss a function
- **Maintainability**: No scattered profiling code
- **Toggle-able**: Enable/disable profiling with feature flags

---

### Milestone 4: Automatic Instrumentation with Proc Macros

**Goal**: Create a procedural macro that automatically instruments functions for profiling.

**Why this matters**: Manual instrumentation is tedious and incomplete. Automatic instrumentation ensures comprehensive profiling.

#### Architecture

**Proc Macro:**
- `#[profile]` - Attribute macro for functions
  - Wraps function body in profiling code
  - Preserves function signature
  - Only active when profiling feature enabled

**Functions:**
- `profile_impl(item: TokenStream) -> TokenStream` - Macro implementation
- `instrument_function(func: ItemFn) -> TokenStream` - Add profiling to function

**Starter Code**:

```rust
// In a separate crate: profiler-macros

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn profile(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // TODO: Extract function name
    // TODO: Generate profiling code
    // TODO: Wrap original function body
    // TODO: Preserve function signature and attributes

    instrument_function(input_fn)
}

fn instrument_function(func: ItemFn) -> TokenStream {
    let func_name = &func.sig.ident;
    let func_name_str = func_name.to_string();
    let block = &func.block;
    let sig = &func.sig;
    let vis = &func.vis;
    let attrs = &func.attrs;

    let instrumented = quote! {
        #(#attrs)*
        #vis #sig {
            #[cfg(feature = "profiling")]
            let _guard = crate::profiler::ProfileGuard::new(#func_name_str);

            #block
        }
    };

    TokenStream::from(instrumented)
}
```

**Usage Example**:

```rust
// In main crate
use profiler_macros::profile;

#[profile]
fn expensive_function(n: usize) -> usize {
    let mut sum = 0;
    for i in 0..n {
        sum += i;
    }
    sum
}

#[profile]
fn another_function() {
    expensive_function(1000);
}

fn main() {
    another_function();

    let stats = get_profile_stats();
    for stat in stats {
        println!("{}: {:?}", stat.function_name, stat.total_time);
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[profile]
    fn test_function() -> i32 {
        42
    }

    #[test]
    fn test_macro_preserves_behavior() {
        assert_eq!(test_function(), 42);
    }

    #[test]
    fn test_profiling_enabled() {
        reset_profiler();

        test_function();

        let stats = get_profile_stats();

        #[cfg(feature = "profiling")]
        assert_eq!(stats.len(), 1);

        #[cfg(not(feature = "profiling"))]
        assert_eq!(stats.len(), 0);
    }
}
```

---

#### Why Milestone 4 Isn't Enough

**Limitation**: Profiling data is only useful if we can analyze it and provide actionable recommendations.

**What we're adding**: Automated analysis that detects performance anti-patterns and suggests optimizations.

**Improvement**:
- **Intelligence**: Automatically identify problems
- **Actionable**: Concrete optimization suggestions
- **Prioritized**: Focus on high-impact optimizations
- **Educational**: Learn performance patterns

---

### Milestone 5: Automated Performance Analysis

**Goal**: Analyze profiling data to automatically detect performance issues and recommend optimizations.

**Why this matters**: Raw profiling data requires expertise to interpret. Automated analysis democratizes performance optimization.

#### Architecture

**Structs:**
- `PerformanceAnalyzer` - Analyzes profiling data
  - **Field**: `cpu_stats: Vec<FunctionStats>` - CPU profiling data
  - **Field**: `alloc_stats: Vec<AllocStats>` - Allocation data

- `PerformanceIssue` - One detected issue
  - **Field**: `severity: Severity` - Critical/High/Medium/Low
  - **Field**: `category: Category` - Type of issue
  - **Field**: `description: String` - What's wrong
  - **Field**: `recommendation: String` - How to fix
  - **Field**: `location: String` - Where it occurs

**Enums:**
- `Severity` - Issue importance
  - **Variants**: `Critical`, `High`, `Medium`, `Low`

- `Category` - Type of performance issue
  - **Variants**: `ExcessiveAllocation`, `HotLoop`, `LargeAllocation`, `FrequentAllocation`, `DeepCallStack`, `SlowFunction`

**Functions:**
- `analyze(&self) -> Vec<PerformanceIssue>` - Find all issues
- `detect_allocation_issues(&self) -> Vec<PerformanceIssue>` - Allocation problems
- `detect_cpu_issues(&self) -> Vec<PerformanceIssue>` - CPU problems
- `generate_report(&self) -> String` - Human-readable report

**Starter Code**:

```rust
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
        // TODO: Initialize analyzer
        todo!("Create analyzer")
    }

    pub fn analyze(&self) -> Vec<PerformanceIssue> {
        // TODO: Run all detection methods
        // TODO: Combine and sort by severity
        let mut issues = Vec::new();

        issues.extend(self.detect_allocation_issues());
        issues.extend(self.detect_cpu_issues());
        issues.extend(self.detect_hotloops());

        // Sort by severity
        issues.sort_by_key(|issue| issue.severity);

        issues
    }

    fn detect_allocation_issues(&self) -> Vec<PerformanceIssue> {
        // TODO: Find functions allocating excessively
        // TODO: Find large single allocations
        // TODO: Find frequent small allocations
        // TODO: Suggest using Vec::with_capacity, SmallVec, etc.
        todo!("Detect allocation issues")
    }

    fn detect_cpu_issues(&self) -> Vec<PerformanceIssue> {
        // TODO: Find functions taking >50% total time
        // TODO: Identify functions called very frequently
        // TODO: Suggest algorithm improvements
        todo!("Detect CPU issues")
    }

    fn detect_hotloops(&self) -> Vec<PerformanceIssue> {
        // TODO: Find functions with high call count
        // TODO: Check if called in loops
        // TODO: Suggest loop hoisting, precomputation
        todo!("Detect hot loops")
    }

    pub fn generate_report(&self) -> String {
        // TODO: Create human-readable report
        // TODO: Group by severity
        // TODO: Include statistics
        // TODO: Provide code examples
        todo!("Generate report")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_excessive_allocation() {
        let alloc_stats = vec![
            AllocStats {
                location: "hot_loop".to_string(),
                count: 10000,  // Many allocations
                total_bytes: 1000000,
                peak_bytes: 100000,
                avg_size: 100,
                current_bytes: 0,
            },
        ];

        let analyzer = PerformanceAnalyzer::new(vec![], alloc_stats);
        let issues = analyzer.detect_allocation_issues();

        // Should detect excessive allocation
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

        // Should identify slow function
        assert!(issues.iter().any(|i| {
            i.location == "slow" && i.category == Category::SlowFunction
        }));
    }

    #[test]
    fn test_severity_prioritization() {
        let cpu_stats = vec![
            FunctionStats {
                function_name: "critical".to_string(),
                total_time: Duration::from_secs(100),  // 100s - critical!
                self_time: Duration::from_secs(100),
                call_count: 1,
                avg_time: Duration::from_secs(100),
            },
        ];

        let analyzer = PerformanceAnalyzer::new(cpu_stats, vec![]);
        let issues = analyzer.analyze();

        // Critical issues should be first
        assert!(issues[0].severity == Severity::Critical);
    }

    #[test]
    fn test_generate_report() {
        let analyzer = PerformanceAnalyzer::new(vec![], vec![]);
        let report = analyzer.generate_report();

        // Should have structured sections
        assert!(report.contains("Performance Analysis"));
        assert!(report.contains("Issues Found") || report.contains("No issues"));
    }
}
```

---

#### Why Milestone 5 Isn't Enough

**Limitation**: We can identify issues but can't validate that optimizations actually helped. Need before/after comparison.

**What we're adding**: Optimization validation framework that compares performance before and after changes.

**Improvement**:
- **Validation**: Prove optimizations work
- **Regression detection**: Catch slowdowns
- **Quantification**: Measure exact speedup
- **Confidence**: Know optimization was worth it

---

### Milestone 6: Optimization Validation and Comparison

**Goal**: Compare profiling data before and after optimizations to validate improvements.

**Why this matters**: Without measurement, you don't know if optimizations helped. Comparison proves ROI.

#### Architecture

**Structs:**
- `ProfileComparison` - Compares two profiling sessions
  - **Field**: `before: ProfileSnapshot` - Baseline performance
  - **Field**: `after: ProfileSnapshot` - Optimized performance

- `ProfileSnapshot` - One profiling session
  - **Field**: `name: String` - Session name
  - **Field**: `cpu_stats: Vec<FunctionStats>` - CPU data
  - **Field**: `alloc_stats: Vec<AllocStats>` - Allocation data
  - **Field**: `total_time: Duration` - Total runtime

- `Improvement` - Performance change
  - **Field**: `function: String` - What changed
  - **Field**: `metric: Metric` - What metric
  - **Field**: `before_value: f64` - Original value
  - **Field**: `after_value: f64` - New value
  - **Field**: `percent_change: f64` - Percentage improvement

**Functions:**
- `compare(before: ProfileSnapshot, after: ProfileSnapshot) -> ProfileComparison` - Compare snapshots
- `find_improvements(&self) -> Vec<Improvement>` - Find what improved
- `find_regressions(&self) -> Vec<Improvement>` - Find what got worse
- `generate_comparison_report(&self) -> String` - Summary report

**Starter Code**:

```rust
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
        // TODO: Compare CPU stats
        // TODO: Compare allocation stats
        // TODO: Calculate percentage changes
        // TODO: Filter for improvements (negative % = better)
        todo!("Find improvements")
    }

    pub fn find_regressions(&self) -> Vec<Improvement> {
        // TODO: Same as improvements but filter for worse performance
        todo!("Find regressions")
    }

    pub fn overall_speedup(&self) -> f64 {
        // TODO: Calculate total runtime ratio
        let before_ms = self.before.total_time.as_secs_f64() * 1000.0;
        let after_ms = self.after.total_time.as_secs_f64() * 1000.0;

        before_ms / after_ms
    }

    pub fn generate_comparison_report(&self) -> String {
        // TODO: Create detailed comparison report
        // TODO: Show overall speedup
        // TODO: List top improvements
        // TODO: Warn about regressions
        // TODO: Include before/after flamegraphs
        todo!("Generate comparison report")
    }
}

// Helper to capture a profile snapshot
pub fn capture_snapshot(name: &str) -> ProfileSnapshot {
    ProfileSnapshot {
        name: name.to_string(),
        cpu_stats: get_profile_stats(),
        alloc_stats: get_allocation_hotspots(),
        total_time: Duration::from_secs(0), // Calculate from stats
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(speedup, 2.0);  // 2x faster
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
                    total_time: Duration::from_millis(500),  // 2x faster!
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
        assert!(improvements[0].percent_change < 0.0);  // Negative = improvement
    }

    #[test]
    fn test_detect_regression() {
        let before = create_test_snapshot("before", 100);
        let after = create_test_snapshot("after", 200);  // Slower!

        let comparison = ProfileComparison::new(before, after);
        let regressions = comparison.find_regressions();

        assert!(regressions.len() > 0);
    }
}
```

---

## Testing Strategies

### 1. Unit Tests
- Test each profiling component independently
- Verify statistics calculations
- Validate allocation tracking

### 2. Integration Tests
- Profile real functions end-to-end
- Generate actual flamegraphs
- Validate analysis accuracy

### 3. Benchmark Tests
- Measure profiling overhead (should be <5%)
- Test with large programs
- Verify memory usage of profiler itself

### 4. Real-World Tests
- Profile actual applications
- Validate optimizations lead to speedups
- Compare with production profilers (perf, Instruments)

---

## Complete Working Example

See the generated source files for full implementation. The toolkit demonstrates:
- **CPU profiling**: Track time spent in each function
- **Memory tracking**: Identify allocation hotspots
- **Visualization**: Generate interactive flamegraphs
- **Automation**: Procedural macros for easy instrumentation
- **Analysis**: Automated performance issue detection
- **Validation**: Before/after optimization comparison

This comprehensive profiling toolkit teaches performance measurement, optimization techniques, and data-driven development practices essential for building high-performance Rust applications.
