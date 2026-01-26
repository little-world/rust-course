// Anti-Patterns Part 1: Common Pitfalls
// Demonstrates common mistakes and their correct solutions.

use rayon::prelude::*;
use std::sync::Arc;
use std::thread;

// ============================================================================
// Anti-Pattern: Excessive Cloning
// ============================================================================

mod excessive_cloning {
    // ANTI-PATTERN: Clone to avoid borrow checker errors
    #[allow(dead_code)]
    fn process_data_bad(data: Vec<String>) {
        let copy1 = data.clone(); // Unnecessary
        print_data_bad(copy1);

        let copy2 = data.clone(); // Unnecessary
        let _ = transform_data_bad(copy2);

        let copy3 = data.clone(); // Unnecessary
        save_data(copy3);
    }

    fn print_data_bad(data: Vec<String>) {
        for item in &data {
            println!("{}", item);
        }
    }

    fn transform_data_bad(data: Vec<String>) -> Vec<String> {
        data.iter().map(|s| s.to_uppercase()).collect()
    }

    fn save_data(_data: Vec<String>) {
        // Save to database
    }

    // CORRECT: Use borrowing appropriately
    pub fn process_data_good(data: Vec<String>) {
        // Borrow for read-only access
        print_data_good(&data);

        // Clone only when you need to modify and keep original
        let _transformed = transform_data_good(&data);

        // Move ownership when done with original
        save_data(data);
    }

    pub fn print_data_good(data: &[String]) {
        for item in data {
            println!("{}", item);
        }
    }

    pub fn transform_data_good(data: &[String]) -> Vec<String> {
        data.iter().map(|s| s.to_uppercase()).collect()
    }

    pub fn demo() {
        println!("=== Excessive Cloning Anti-Pattern ===");
        let data = vec!["hello".to_string(), "world".to_string()];
        process_data_good(data);
    }
}

// ============================================================================
// Anti-Pattern: Overusing Rc/Arc Without Need
// ============================================================================

mod overusing_rc_arc {
    use std::sync::Arc;
    use std::thread;

    #[derive(Clone)]
    pub struct Config {
        pub name: String,
    }

    impl Config {
        pub fn load() -> Self {
            Config {
                name: "default".to_string(),
            }
        }
    }

    pub struct Logger;

    impl Logger {
        pub fn log(&self, msg: &str) {
            println!("LOG: {}", msg);
        }
    }

    #[allow(dead_code)]
    pub struct Cache;

    // CORRECT: Use references with lifetimes
    pub struct DataProcessor<'a> {
        config: &'a Config,
        logger: &'a Logger,
    }

    impl<'a> DataProcessor<'a> {
        pub fn new(config: &'a Config, logger: &'a Logger) -> Self {
            Self { config, logger }
        }

        pub fn process(&self, data: &str) {
            self.logger.log(&format!("Processing {} with config {}", data, self.config.name));
        }
    }

    // Only use Arc when truly sharing across owners (threads)
    pub fn process_with_config(config: &Arc<Config>) {
        println!("Processing with config: {}", config.name);
    }

    pub fn multiple_threads_need_shared_data() {
        let config = Arc::new(Config::load());

        let config1 = Arc::clone(&config);
        let handle1 = thread::spawn(move || process_with_config(&config1));

        let config2 = Arc::clone(&config);
        let handle2 = thread::spawn(move || process_with_config(&config2));

        handle1.join().unwrap();
        handle2.join().unwrap();
    }

    pub fn demo() {
        println!("\n=== Overusing Rc/Arc Anti-Pattern ===");

        // Using references (correct for single-threaded)
        let config = Config::load();
        let logger = Logger;
        let processor = DataProcessor::new(&config, &logger);
        processor.process("test data");

        // Using Arc for multi-threaded (correct)
        println!("Multi-threaded with Arc:");
        multiple_threads_need_shared_data();
    }
}

// ============================================================================
// Anti-Pattern: Ignoring Iterator Combinators
// ============================================================================

mod ignoring_iterators {
    use rayon::prelude::*;

    // ANTI-PATTERN: Manual loops instead of iterators
    #[allow(dead_code)]
    fn process_numbers_bad(numbers: &[i32]) -> Vec<i32> {
        let mut result = Vec::new();
        for &num in numbers {
            if num % 2 == 0 {
                result.push(num * 2);
            }
        }
        result
    }

    #[allow(dead_code)]
    fn find_first_large_bad(numbers: &[i32]) -> Option<i32> {
        for &num in numbers {
            if num > 100 {
                return Some(num);
            }
        }
        None
    }

    #[allow(dead_code)]
    fn sum_squares_bad(numbers: &[i32]) -> i32 {
        let mut sum = 0;
        for &num in numbers {
            sum += num * num;
        }
        sum
    }

    // CORRECT: Use iterator combinators
    pub fn process_numbers(numbers: &[i32]) -> Vec<i32> {
        numbers
            .iter()
            .filter(|&&num| num % 2 == 0)
            .map(|&num| num * 2)
            .collect()
    }

    pub fn find_first_large(numbers: &[i32]) -> Option<i32> {
        numbers.iter().find(|&&num| num > 100).copied()
    }

    pub fn sum_squares(numbers: &[i32]) -> i32 {
        numbers.iter().map(|&num| num * num).sum()
    }

    // Easy to make parallel with rayon
    pub fn parallel_sum_squares(numbers: &[i32]) -> i32 {
        numbers.par_iter().map(|&num| num * num).sum()
    }

    pub fn demo() {
        println!("\n=== Ignoring Iterator Combinators Anti-Pattern ===");
        let numbers = vec![1, 2, 3, 4, 5, 100, 101, 102];

        println!("process_numbers: {:?}", process_numbers(&numbers));
        println!("find_first_large: {:?}", find_first_large(&numbers));
        println!("sum_squares: {}", sum_squares(&numbers));
        println!("parallel_sum_squares: {}", parallel_sum_squares(&numbers));
    }
}

// ============================================================================
// Anti-Pattern: Deref Coercion Abuse
// ============================================================================

mod deref_abuse {
    // ANTI-PATTERN: Using Deref for inheritance-like behavior
    // This is shown as an example of what NOT to do

    #[derive(Clone)]
    pub struct Employee {
        pub name: String,
        pub id: u32,
    }

    // CORRECT: Explicit delegation
    pub struct Manager {
        employee: Employee,
        pub team_size: usize,
    }

    impl Manager {
        pub fn new(name: String, id: u32, team_size: usize) -> Self {
            Manager {
                employee: Employee { name, id },
                team_size,
            }
        }

        pub fn employee(&self) -> &Employee {
            &self.employee
        }

        // Delegate specific methods explicitly
        pub fn name(&self) -> &str {
            &self.employee.name
        }

        pub fn id(&self) -> u32 {
            self.employee.id
        }
    }

    pub fn print_employee_info(emp: &Employee) {
        println!("{}: {}", emp.id, emp.name);
    }

    pub fn demo() {
        println!("\n=== Deref Coercion Abuse Anti-Pattern ===");
        let manager = Manager::new("Alice".to_string(), 1, 5);

        // Clear and explicit
        print_employee_info(manager.employee());
        println!("Manager {} has team size: {}", manager.name(), manager.team_size);
    }
}

// ============================================================================
// Anti-Pattern: String vs &str Confusion
// ============================================================================

mod string_confusion {
    // ANTI-PATTERN: Unnecessary allocations
    #[allow(dead_code)]
    fn greet_bad(name: String) -> String {
        format!("Hello, {}", name)
    }

    // CORRECT: Accept &str, return String when needed
    pub fn greet(name: &str) -> String {
        format!("Hello, {}", name)
    }

    pub fn process_names(names: &[&str]) {
        for &name in names {
            let greeting = greet(name); // No unnecessary allocation
            println!("{}", greeting);
        }
    }

    // For more flexibility, use generic trait bounds
    pub fn greet_generic<S: AsRef<str>>(name: S) -> String {
        format!("Hello, {}", name.as_ref())
    }

    pub fn demo() {
        println!("\n=== String vs &str Confusion Anti-Pattern ===");

        // Works with literals and owned strings
        println!("{}", greet("Alice")); // No allocation needed
        let owned = String::from("Bob");
        println!("{}", greet(&owned)); // Also works

        // Generic version works with both
        println!("{}", greet_generic("Charlie"));
        println!("{}", greet_generic(String::from("Diana")));

        // Process multiple names
        let names = vec!["Eve", "Frank", "Grace"];
        process_names(&names);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_numbers() {
        let numbers = vec![1, 2, 3, 4, 5, 6];
        let result = ignoring_iterators::process_numbers(&numbers);
        assert_eq!(result, vec![4, 8, 12]); // 2*2, 4*2, 6*2
    }

    #[test]
    fn test_find_first_large() {
        let numbers = vec![1, 50, 101, 200];
        assert_eq!(ignoring_iterators::find_first_large(&numbers), Some(101));

        let small = vec![1, 2, 3];
        assert_eq!(ignoring_iterators::find_first_large(&small), None);
    }

    #[test]
    fn test_sum_squares() {
        let numbers = vec![1, 2, 3];
        assert_eq!(ignoring_iterators::sum_squares(&numbers), 14); // 1 + 4 + 9
    }

    #[test]
    fn test_parallel_sum_squares() {
        let numbers: Vec<i32> = (1..=100).collect();
        let sequential = ignoring_iterators::sum_squares(&numbers);
        let parallel = ignoring_iterators::parallel_sum_squares(&numbers);
        assert_eq!(sequential, parallel);
    }

    #[test]
    fn test_greet() {
        assert_eq!(string_confusion::greet("World"), "Hello, World");
    }

    #[test]
    fn test_greet_generic() {
        assert_eq!(string_confusion::greet_generic("Alice"), "Hello, Alice");
        assert_eq!(
            string_confusion::greet_generic(String::from("Bob")),
            "Hello, Bob"
        );
    }

    #[test]
    fn test_manager_delegation() {
        let manager = deref_abuse::Manager::new("Alice".to_string(), 1, 5);
        assert_eq!(manager.name(), "Alice");
        assert_eq!(manager.id(), 1);
        assert_eq!(manager.team_size, 5);
    }

    #[test]
    fn test_transform_data() {
        let data = vec!["hello".to_string(), "world".to_string()];
        let result = excessive_cloning::transform_data_good(&data);
        assert_eq!(result, vec!["HELLO", "WORLD"]);
    }

    #[test]
    fn test_data_processor() {
        let config = overusing_rc_arc::Config::load();
        let logger = overusing_rc_arc::Logger;
        let processor = overusing_rc_arc::DataProcessor::new(&config, &logger);
        // Just verify it doesn't panic
        processor.process("test");
    }

    #[test]
    fn test_arc_sharing() {
        let config = Arc::new(overusing_rc_arc::Config::load());
        let config_clone = Arc::clone(&config);

        let handle = thread::spawn(move || {
            assert_eq!(config_clone.name, "default");
        });

        handle.join().unwrap();
    }
}

fn main() {
    println!("Anti-Patterns Part 1: Common Pitfalls");
    println!("=====================================\n");

    excessive_cloning::demo();
    overusing_rc_arc::demo();
    ignoring_iterators::demo();
    deref_abuse::demo();
    string_confusion::demo();
}
