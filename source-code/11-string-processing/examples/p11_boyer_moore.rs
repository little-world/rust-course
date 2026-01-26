//! Pattern 11: Boyer-Moore String Search
//! O(N/M) Best Case String Matching
//!
//! Run with: cargo run --example p11_boyer_moore

use std::collections::HashMap;

fn main() {
    println!("=== Boyer-Moore String Search ===\n");

    // Basic example
    let pattern = "EXAMPLE";
    let text = "THIS IS A SIMPLE EXAMPLE FOR EXAMPLE MATCHING";

    println!("Pattern: '{}'", pattern);
    println!("Text:    '{}'", text);

    let bm = BoyerMoore::new(pattern);
    let matches = bm.find_all(text);

    println!("\nMatches at: {:?}", matches);
    for pos in &matches {
        println!("  Position {}: '{}'", pos, &text[*pos..*pos + pattern.len()]);
    }

    // Show bad character table
    println!("\n=== Bad Character Table ===\n");
    println!("Pattern: {}", pattern);
    println!("Table: {:?}", bm.bad_char);
    println!("\nExplanation:");
    println!("  Each character maps to its rightmost position in pattern");
    println!("  On mismatch, we can shift to align the mismatched char");

    // Longer pattern - demonstrates sublinear behavior
    println!("\n=== Long Pattern Benefits ===\n");

    let long_pattern = "ALGORITHM";
    let long_text = "THE SEARCH ALGORITHM IS AN EFFICIENT ALGORITHM FOR FINDING PATTERNS IN TEXT";

    let bm2 = BoyerMoore::new(long_pattern);
    let matches2 = bm2.find_all(long_text);

    println!("Pattern: '{}' ({} chars)", long_pattern, long_pattern.len());
    println!("Text length: {}", long_text.len());
    println!("Found at positions: {:?}", matches2);
    println!("Potential best case: examine only ~{} chars (N/M)", long_text.len() / long_pattern.len());

    // Compare with no matches
    println!("\n=== No Match Case ===\n");

    let no_match_text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let bm3 = BoyerMoore::new("XYZ");
    let matches3 = bm3.find_all(no_match_text);

    println!("Searching for 'XYZ' in '{}...'", &no_match_text[..10]);
    println!("Matches: {:?}", matches3);
    println!("Boyer-Moore can skip ahead on each mismatch");

    println!("\n=== Key Points ===");
    println!("1. Right-to-left scanning for maximum information");
    println!("2. Bad character rule enables large forward jumps");
    println!("3. Best case O(N/M) - sublinear!");
    println!("4. Used in grep, vim, and text editors");
}

struct BoyerMoore {
    pattern: Vec<char>,
    bad_char: HashMap<char, usize>,
}

impl BoyerMoore {
    fn new(pattern: &str) -> Self {
        let pattern: Vec<char> = pattern.chars().collect();
        let bad_char = Self::build_bad_char_table(&pattern);

        BoyerMoore { pattern, bad_char }
    }

    fn build_bad_char_table(pattern: &[char]) -> HashMap<char, usize> {
        let mut table = HashMap::new();

        for (i, &ch) in pattern.iter().enumerate() {
            table.insert(ch, i);
        }

        table
    }

    fn find_all(&self, text: &str) -> Vec<usize> {
        let text: Vec<char> = text.chars().collect();
        let mut matches = Vec::new();
        let m = self.pattern.len();
        let n = text.len();

        if m > n {
            return matches;
        }

        let mut s = 0;  // Shift of pattern relative to text

        while s <= n - m {
            let mut j = m;

            // Scan from right to left
            while j > 0 && self.pattern[j - 1] == text[s + j - 1] {
                j -= 1;
            }

            if j == 0 {
                // Match found
                matches.push(s);

                // Shift pattern
                if s + m < n {
                    let next_char = text[s + m];
                    let skip = self.bad_char.get(&next_char).unwrap_or(&0);
                    s += m - skip;
                } else {
                    s += 1;
                }
            } else {
                // Mismatch: use bad character rule
                let bad_char = text[s + j - 1];
                let shift = if let Some(&pos) = self.bad_char.get(&bad_char) {
                    if pos < j - 1 {
                        j - 1 - pos
                    } else {
                        1
                    }
                } else {
                    j
                };

                s += shift;
            }
        }

        matches
    }
}
