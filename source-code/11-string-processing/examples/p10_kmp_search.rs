//! Pattern 10: Knuth-Morris-Pratt (KMP) String Search
//! O(N+M) Guaranteed String Matching
//!
//! Run with: cargo run --example p10_kmp_search

fn main() {
    println!("=== KMP String Search ===\n");

    // Basic example
    let pattern = "ABABC";
    let text = "ABABDABACDABABCABAB";

    println!("Pattern: '{}'", pattern);
    println!("Text:    '{}'", text);

    let matcher = KmpMatcher::new(pattern);
    let matches = matcher.find_all(text);

    println!("\nPattern found at positions: {:?}", matches);
    for pos in &matches {
        println!("  Position {}: '{}'", pos, &text[*pos..*pos + pattern.len()]);
    }

    // Show failure function
    println!("\n=== Failure Function ===\n");
    println!("Pattern: {}", pattern);
    println!("Failure: {:?}", matcher.failure);
    println!("\nExplanation:");
    println!("  failure[0] = 0: No proper prefix/suffix for 'A'");
    println!("  failure[1] = 0: 'AB' has no matching prefix/suffix");
    println!("  failure[2] = 1: 'ABA' has 'A' as both prefix and suffix");
    println!("  failure[3] = 2: 'ABAB' has 'AB' as both prefix and suffix");
    println!("  failure[4] = 0: 'ABABC' has no matching prefix/suffix");

    // Multiple occurrences
    println!("\n=== Multiple Occurrences ===\n");

    let pattern2 = "the";
    let text2 = "the cat sat on the mat with the bat";

    let matcher2 = KmpMatcher::new(pattern2);
    let matches2 = matcher2.find_all(text2);

    println!("Pattern: '{}'", pattern2);
    println!("Text: '{}'", text2);
    println!("Found at positions: {:?}", matches2);

    // Contains check
    println!("\n=== Contains Check ===\n");

    let texts = ["hello world", "goodbye world", "hello there"];
    let search_pattern = "hello";
    let matcher3 = KmpMatcher::new(search_pattern);

    for text in texts {
        println!("'{}' contains '{}': {}", text, search_pattern, matcher3.contains(text));
    }

    println!("\n=== Key Points ===");
    println!("1. O(M) preprocessing to build failure function");
    println!("2. O(N) search - each text character examined once");
    println!("3. No backtracking in text - linear time guaranteed");
    println!("4. Works well when pattern has internal repetition");
}

struct KmpMatcher {
    pattern: Vec<char>,
    failure: Vec<usize>,
}

impl KmpMatcher {
    fn new(pattern: &str) -> Self {
        let pattern: Vec<char> = pattern.chars().collect();
        let failure = Self::compute_failure(&pattern);

        KmpMatcher { pattern, failure }
    }

    fn compute_failure(pattern: &[char]) -> Vec<usize> {
        let mut failure = vec![0; pattern.len()];
        let mut j = 0;

        for i in 1..pattern.len() {
            while j > 0 && pattern[i] != pattern[j] {
                j = failure[j - 1];
            }

            if pattern[i] == pattern[j] {
                j += 1;
            }

            failure[i] = j;
        }

        failure
    }

    fn find_all(&self, text: &str) -> Vec<usize> {
        let text: Vec<char> = text.chars().collect();
        let mut matches = Vec::new();
        let mut j = 0;

        for (i, &ch) in text.iter().enumerate() {
            while j > 0 && ch != self.pattern[j] {
                j = self.failure[j - 1];
            }

            if ch == self.pattern[j] {
                j += 1;
            }

            if j == self.pattern.len() {
                matches.push(i + 1 - j);
                j = self.failure[j - 1];
            }
        }

        matches
    }

    fn contains(&self, text: &str) -> bool {
        !self.find_all(text).is_empty()
    }
}
