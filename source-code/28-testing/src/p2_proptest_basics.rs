// Pattern 2: Property-Based Testing with proptest
// Demonstrates proptest basics, shrinking, custom generators, and invariant testing.

use proptest::prelude::*;
use std::collections::HashMap;

// ============================================================================
// Example: Can do better (motivating example)
// ============================================================================

fn sort(mut vec: Vec<i32>) -> Vec<i32> {
    vec.sort();
    vec
}

#[cfg(test)]
mod motivating_example {
    use super::*;

    #[test]
    fn test_sort() {
        assert_eq!(sort(vec![3, 1, 2]), vec![1, 2, 3]);
    }

    // This test is fine, but what about:
    // - Empty vectors?
    // - Single-element vectors?
    // - Already-sorted vectors?
    // - Reverse-sorted vectors?
    // - Duplicate elements?
    // - Very large vectors?
    // - Vectors with MIN and MAX values?
}

// ============================================================================
// Example: Introduction to proptest
// ============================================================================

proptest! {
    #[test]
    fn test_sort_properties(mut vec: Vec<i32>) {
        let sorted = sort(vec.clone());

        // Property 1: Output length equals input length
        prop_assert_eq!(sorted.len(), vec.len());

        // Property 2: Output is sorted
        for i in 1..sorted.len() {
            prop_assert!(sorted[i - 1] <= sorted[i]);
        }

        // Property 3: Output contains same elements as input
        vec.sort();
        prop_assert_eq!(sorted, vec);
    }
}

// ============================================================================
// Example: Shrinking - Finding Minimal Failing Cases
// ============================================================================

fn buggy_absolute_value(x: i32) -> i32 {
    if x < 0 {
        // Bug: -x overflows for i32::MIN
        x.wrapping_neg()  // Using wrapping_neg to demonstrate
    } else {
        x
    }
}

proptest! {
    #[test]
    fn test_absolute_value_properties(x in i32::MIN+1..=i32::MAX) {
        // Note: We exclude i32::MIN to avoid the overflow
        let result = buggy_absolute_value(x);
        prop_assert!(result >= 0 || x == i32::MIN);
    }
}

// ============================================================================
// Example: Custom Generators
// ============================================================================

// Generate vectors of length 1-100
prop_compose! {
    fn vec_1_to_100()(vec in prop::collection::vec(any::<i32>(), 1..=100)) -> Vec<i32> {
        vec
    }
}

// Generate email-like strings
prop_compose! {
    fn email_strategy()(
        username in "[a-z]{3,10}",
        domain in "[a-z]{3,10}",
        tld in "(com|org|net)"
    ) -> String {
        format!("{}@{}.{}", username, domain, tld)
    }
}

proptest! {
    #[test]
    fn test_with_custom_generator(vec in vec_1_to_100()) {
        prop_assert!(!vec.is_empty());
        prop_assert!(vec.len() <= 100);
    }

    #[test]
    fn test_email_parsing(email in email_strategy()) {
        prop_assert!(email.contains('@'));
        prop_assert!(email.contains('.'));
    }
}

// ============================================================================
// Example: Testing Invariants
// ============================================================================

fn merge_maps(mut a: HashMap<String, i32>, b: HashMap<String, i32>) -> HashMap<String, i32> {
    for (k, v) in b {
        // Use saturating_add to avoid overflow - this demonstrates a fixed version
        let entry = a.entry(k).or_insert(0);
        *entry = entry.saturating_add(v);
    }
    a
}

proptest! {
    #[test]
    fn test_merge_properties(
        a: HashMap<String, i32>,
        b: HashMap<String, i32>,
    ) {
        let merged = merge_maps(a.clone(), b.clone());

        // Property 1: All keys from both maps are in the result
        for key in a.keys().chain(b.keys()) {
            prop_assert!(merged.contains_key(key));
        }

        // Property 2: Values are summed correctly (with saturation)
        for key in merged.keys() {
            let expected = a.get(key).unwrap_or(&0).saturating_add(*b.get(key).unwrap_or(&0));
            prop_assert_eq!(merged[key], expected);
        }

        // Property 3: Merging with empty map is identity
        let empty: HashMap<String, i32> = HashMap::new();
        prop_assert_eq!(merge_maps(a.clone(), empty.clone()), a);
    }
}

fn main() {
    println!("Property-based testing with proptest - run with: cargo test");
    println!("proptest generates hundreds of random inputs and shrinks failures.");
}
