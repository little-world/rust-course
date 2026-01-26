// Anti-Patterns Part 2: Performance Anti-Patterns
// Demonstrates performance mistakes and their correct solutions.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

// ============================================================================
// Anti-Pattern: Collecting Iterators Unnecessarily
// ============================================================================

mod collecting_unnecessarily {
    // ANTI-PATTERN: Unnecessary intermediate collections
    #[allow(dead_code)]
    fn process_data_bad(numbers: &[i32]) -> i32 {
        let evens: Vec<i32> = numbers
            .iter()
            .filter(|&&x| x % 2 == 0)
            .copied()
            .collect(); // Unnecessary allocation

        let doubled: Vec<i32> = evens.iter().map(|&x| x * 2).collect(); // Another unnecessary allocation

        doubled.iter().sum()
    }

    // CORRECT: Single iterator chain
    pub fn process_data(numbers: &[i32]) -> i32 {
        numbers
            .iter()
            .filter(|&&x| x % 2 == 0)
            .map(|&x| x * 2)
            .sum()
        // Zero allocations, single pass through data
    }

    // Only collect when you need to reuse the result
    pub fn process_data_reusable(numbers: &[i32]) -> Vec<i32> {
        numbers
            .iter()
            .filter(|&&x| x % 2 == 0)
            .map(|&x| x * 2)
            .collect() // Now justified: we return the collection
    }

    pub fn demo() {
        println!("=== Collecting Iterators Unnecessarily Anti-Pattern ===");
        let numbers: Vec<i32> = (1..=10).collect();
        println!("Numbers: {:?}", numbers);
        println!("Sum of doubled evens: {}", process_data(&numbers));
        println!("Doubled evens: {:?}", process_data_reusable(&numbers));
    }
}

// ============================================================================
// Anti-Pattern: Vec<T> When Array Suffices
// ============================================================================

mod vec_vs_array {
    // ANTI-PATTERN: Heap allocation for fixed-size data
    #[allow(dead_code)]
    fn get_rgb_channels_bad(pixel: u32) -> Vec<u8> {
        vec![
            ((pixel >> 16) & 0xFF) as u8,
            ((pixel >> 8) & 0xFF) as u8,
            (pixel & 0xFF) as u8,
        ]
    }

    // CORRECT: Stack-allocated arrays
    pub fn get_rgb_channels(pixel: u32) -> [u8; 3] {
        [
            ((pixel >> 16) & 0xFF) as u8,
            ((pixel >> 8) & 0xFF) as u8,
            (pixel & 0xFF) as u8,
        ]
    }

    pub fn multiply_3x3(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        result
        // All on stack, no allocations
    }

    // Use Vec only when size is truly dynamic
    pub fn get_pixels(count: usize) -> Vec<[u8; 3]> {
        vec![[0, 0, 0]; count]
    }

    pub fn demo() {
        println!("\n=== Vec<T> When Array Suffices Anti-Pattern ===");

        let pixel = 0xFF8040u32; // Red=255, Green=128, Blue=64
        let [r, g, b] = get_rgb_channels(pixel);
        println!("RGB: ({}, {}, {})", r, g, b);

        let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let result = multiply_3x3(identity, identity);
        println!("Identity * Identity = {:?}", result);

        let pixels = get_pixels(3);
        println!("Dynamic pixels: {:?}", pixels);
    }
}

// ============================================================================
// Anti-Pattern: HashMap for Small Key Sets
// ============================================================================

mod hashmap_small_sets {
    use super::*;

    // ANTI-PATTERN: HashMap for 3 items
    #[allow(dead_code)]
    fn get_status_code_bad(status: &str) -> u16 {
        let mut codes = HashMap::new();
        codes.insert("ok", 200);
        codes.insert("not_found", 404);
        codes.insert("error", 500);

        *codes.get(status).unwrap_or(&500)
        // Recreating HashMap on every call!
    }

    // CORRECT: Match for small known sets
    pub fn get_status_code(status: &str) -> u16 {
        match status {
            "ok" => 200,
            "not_found" => 404,
            "error" => 500,
            _ => 500,
        }
        // Compiles to jump table or if-chain, no allocation
    }

    // Or array of tuples for linear search
    pub fn get_status_code_array(status: &str) -> u16 {
        const CODES: &[(&str, u16)] = &[("ok", 200), ("not_found", 404), ("error", 500)];

        CODES
            .iter()
            .find(|(key, _)| *key == status)
            .map(|(_, code)| *code)
            .unwrap_or(500)
    }

    // Use HashMap only for larger, dynamic collections
    static LARGE_CODES: LazyLock<HashMap<&'static str, u16>> = LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("ok", 200);
        map.insert("created", 201);
        map.insert("accepted", 202);
        map.insert("no_content", 204);
        map.insert("moved_permanently", 301);
        map.insert("found", 302);
        map.insert("not_modified", 304);
        map.insert("bad_request", 400);
        map.insert("unauthorized", 401);
        map.insert("forbidden", 403);
        map.insert("not_found", 404);
        map.insert("method_not_allowed", 405);
        map.insert("internal_server_error", 500);
        map.insert("bad_gateway", 502);
        map.insert("service_unavailable", 503);
        map
    });

    pub fn get_status_code_large(status: &str) -> u16 {
        *LARGE_CODES.get(status).unwrap_or(&500)
    }

    pub fn demo() {
        println!("\n=== HashMap for Small Key Sets Anti-Pattern ===");
        println!("Match-based lookup:");
        println!("  ok -> {}", get_status_code("ok"));
        println!("  not_found -> {}", get_status_code("not_found"));
        println!("  unknown -> {}", get_status_code("unknown"));

        println!("Array-based lookup:");
        println!("  ok -> {}", get_status_code_array("ok"));
        println!("  error -> {}", get_status_code_array("error"));

        println!("HashMap for large sets:");
        println!("  unauthorized -> {}", get_status_code_large("unauthorized"));
        println!("  bad_gateway -> {}", get_status_code_large("bad_gateway"));
    }
}

// ============================================================================
// Anti-Pattern: Premature String Allocation
// ============================================================================

mod premature_allocation {
    use super::*;

    // ANTI-PATTERN: Allocate early, use late
    #[allow(dead_code)]
    fn process_log_line_bad(line: &str) -> Option<String> {
        let owned = line.to_string(); // Allocate immediately

        if !owned.starts_with("ERROR") {
            return None; // Wasted allocation
        }

        Some(owned.to_uppercase()) // Another allocation
    }

    // CORRECT: Delay allocation
    pub fn process_log_line(line: &str) -> Option<String> {
        if !line.starts_with("ERROR") {
            return None; // No allocation for filtered lines
        }

        Some(line.to_uppercase()) // Single allocation only when needed
    }

    pub fn extract_field<'a>(data: &'a str, field: &str) -> &'a str {
        data.split(',')
            .find(|s| s.starts_with(field))
            .unwrap_or("")
        // No allocations at all, returns slice of input
    }

    // If ownership needed, use Cow for conditional allocation
    pub fn normalize(s: &str) -> Cow<'_, str> {
        if s.chars().any(|c| c.is_uppercase()) {
            Cow::Owned(s.to_lowercase()) // Allocate only if needed
        } else {
            Cow::Borrowed(s) // Zero-cost
        }
    }

    pub fn demo() {
        println!("\n=== Premature String Allocation Anti-Pattern ===");

        let lines = vec![
            "INFO: Starting service",
            "ERROR: Connection failed",
            "DEBUG: Checking config",
            "ERROR: Timeout occurred",
        ];

        println!("Processing log lines:");
        for line in &lines {
            if let Some(processed) = process_log_line(line) {
                println!("  Processed: {}", processed);
            }
        }

        let data = "name:Alice,age:30,city:NYC";
        println!("\nExtract field 'age': '{}'", extract_field(data, "age"));

        println!("\nNormalize strings:");
        let s1 = "hello";
        let s2 = "HELLO";
        println!("  '{}' -> '{}' (borrowed: {})", s1, normalize(s1), matches!(normalize(s1), Cow::Borrowed(_)));
        println!("  '{}' -> '{}' (borrowed: {})", s2, normalize(s2), matches!(normalize(s2), Cow::Borrowed(_)));
    }
}

// ============================================================================
// Anti-Pattern: Boxed Trait Objects Everywhere
// ============================================================================

mod boxed_trait_objects {
    // ANTI-PATTERN: Dynamic dispatch when static suffices
    pub trait Processor {
        fn process(&self, data: &str) -> String;
    }

    pub struct Uppercase;
    impl Processor for Uppercase {
        fn process(&self, data: &str) -> String {
            data.to_uppercase()
        }
    }

    pub struct Reverse;
    impl Processor for Reverse {
        fn process(&self, data: &str) -> String {
            data.chars().rev().collect()
        }
    }

    #[allow(dead_code)]
    fn pipeline_bad(processors: Vec<Box<dyn Processor>>, data: &str) -> String {
        let mut result = data.to_string();
        for processor in processors {
            result = processor.process(&result); // Virtual call overhead
        }
        result
    }

    // CORRECT: Static dispatch with generics
    pub fn pipeline<P1, P2>(p1: P1, p2: P2, data: &str) -> String
    where
        P1: Processor,
        P2: Processor,
    {
        let result = p1.process(data);
        p2.process(&result)
        // All calls inlined, no heap allocations
    }

    // Or use impl Trait for flexibility
    pub fn process_twice(data: &str, processor: impl Processor) -> String {
        let once = processor.process(data);
        processor.process(&once)
    }

    // Only use dyn when you truly need runtime polymorphism
    pub fn dynamic_pipeline(processors: Vec<Box<dyn Processor>>, data: &str) -> String {
        // Justified: processors unknown at compile time
        let mut result = data.to_string();
        for processor in processors {
            result = processor.process(&result);
        }
        result
    }

    pub fn demo() {
        println!("\n=== Boxed Trait Objects Everywhere Anti-Pattern ===");

        let data = "hello";

        // Static dispatch (faster)
        println!("Static pipeline: {}", pipeline(Uppercase, Reverse, data));

        // impl Trait
        println!("Process twice: {}", process_twice(data, Uppercase));

        // Dynamic dispatch (only when needed)
        let processors: Vec<Box<dyn Processor>> = vec![Box::new(Uppercase), Box::new(Reverse)];
        println!("Dynamic pipeline: {}", dynamic_pipeline(processors, data));
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data() {
        let numbers = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(collecting_unnecessarily::process_data(&numbers), 24); // (2+4+6)*2 = 24
    }

    #[test]
    fn test_process_data_reusable() {
        let numbers = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(
            collecting_unnecessarily::process_data_reusable(&numbers),
            vec![4, 8, 12]
        );
    }

    #[test]
    fn test_rgb_channels() {
        let pixel = 0xFF8040u32;
        let [r, g, b] = vec_vs_array::get_rgb_channels(pixel);
        assert_eq!(r, 255);
        assert_eq!(g, 128);
        assert_eq!(b, 64);
    }

    #[test]
    fn test_multiply_3x3_identity() {
        let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let result = vec_vs_array::multiply_3x3(identity, identity);
        for i in 0..3 {
            for j in 0..3 {
                if i == j {
                    assert!((result[i][j] - 1.0).abs() < 0.0001);
                } else {
                    assert!((result[i][j] - 0.0).abs() < 0.0001);
                }
            }
        }
    }

    #[test]
    fn test_status_code_match() {
        assert_eq!(hashmap_small_sets::get_status_code("ok"), 200);
        assert_eq!(hashmap_small_sets::get_status_code("not_found"), 404);
        assert_eq!(hashmap_small_sets::get_status_code("unknown"), 500);
    }

    #[test]
    fn test_status_code_array() {
        assert_eq!(hashmap_small_sets::get_status_code_array("ok"), 200);
        assert_eq!(hashmap_small_sets::get_status_code_array("error"), 500);
    }

    #[test]
    fn test_status_code_large() {
        assert_eq!(hashmap_small_sets::get_status_code_large("unauthorized"), 401);
        assert_eq!(hashmap_small_sets::get_status_code_large("bad_gateway"), 502);
    }

    #[test]
    fn test_process_log_line() {
        assert_eq!(
            premature_allocation::process_log_line("ERROR: test"),
            Some("ERROR: TEST".to_string())
        );
        assert_eq!(premature_allocation::process_log_line("INFO: test"), None);
    }

    #[test]
    fn test_extract_field() {
        let data = "name:Alice,age:30,city:NYC";
        assert_eq!(premature_allocation::extract_field(data, "age"), "age:30");
        assert_eq!(premature_allocation::extract_field(data, "missing"), "");
    }

    #[test]
    fn test_normalize() {
        let s1 = "hello";
        let s2 = "HELLO";

        let cow1 = premature_allocation::normalize(s1);
        let cow2 = premature_allocation::normalize(s2);

        assert!(matches!(cow1, Cow::Borrowed(_)));
        assert!(matches!(cow2, Cow::Owned(_)));
        assert_eq!(cow1, "hello");
        assert_eq!(cow2, "hello");
    }

    #[test]
    fn test_static_pipeline() {
        let result = boxed_trait_objects::pipeline(
            boxed_trait_objects::Uppercase,
            boxed_trait_objects::Reverse,
            "hello",
        );
        assert_eq!(result, "OLLEH");
    }

    #[test]
    fn test_dynamic_pipeline() {
        let processors: Vec<Box<dyn boxed_trait_objects::Processor>> = vec![
            Box::new(boxed_trait_objects::Uppercase),
            Box::new(boxed_trait_objects::Reverse),
        ];
        let result = boxed_trait_objects::dynamic_pipeline(processors, "hello");
        assert_eq!(result, "OLLEH");
    }
}

fn main() {
    println!("Anti-Patterns Part 2: Performance Anti-Patterns");
    println!("================================================\n");

    collecting_unnecessarily::demo();
    vec_vs_array::demo();
    hashmap_small_sets::demo();
    premature_allocation::demo();
    boxed_trait_objects::demo();
}
