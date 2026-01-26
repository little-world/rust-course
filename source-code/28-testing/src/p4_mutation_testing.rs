// Pattern 4: Mutation Testing
// NOTE: Real mutation testing requires cargo-mutants or mutagen.
// This file demonstrates functions that would be good mutation testing targets.

// ============================================================================
// Example: Functions suitable for mutation testing
// ============================================================================

/// Age eligibility check - a common mutation testing target
/// Mutants would change >= to >, 18 to 17 or 19, etc.
pub fn is_eligible(age: u8) -> bool {
    age >= 18
}

/// Price calculation with discount - mutation testing reveals weak assertions
pub fn calculate_price(base_price: f64, quantity: u32, discount_percent: f64) -> f64 {
    if quantity == 0 {
        return 0.0;
    }

    let subtotal = base_price * quantity as f64;

    // Mutants might change > to >=, 100.0 to other values
    let discount = if subtotal > 100.0 {
        subtotal * (discount_percent / 100.0)
    } else {
        0.0
    };

    subtotal - discount
}

/// Tax calculation - financial code benefits from mutation testing
pub fn calculate_tax(amount: f64, tax_rate: f64) -> f64 {
    if amount <= 0.0 {
        return 0.0;
    }
    amount * (tax_rate / 100.0)
}

/// Comparison function - operators are prime mutation targets
pub fn compare_scores(a: i32, b: i32) -> i32 {
    if a > b {
        1
    } else if a < b {
        -1
    } else {
        0
    }
}

/// Range check - boundary conditions are mutation-prone
pub fn is_in_range(value: i32, min: i32, max: i32) -> bool {
    value >= min && value <= max
}

/// Grade calculation - multiple branches = many mutation opportunities
pub fn calculate_grade(score: u32) -> char {
    if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else if score >= 70 {
        'C'
    } else if score >= 60 {
        'D'
    } else {
        'F'
    }
}

// ============================================================================
// Tests that would catch (or miss) mutants
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- is_eligible tests ----

    #[test]
    fn test_eligible_at_18() {
        assert!(is_eligible(18));
    }

    #[test]
    fn test_eligible_above_18() {
        assert!(is_eligible(25));
    }

    #[test]
    fn test_not_eligible_at_17() {
        assert!(!is_eligible(17));
    }

    // This test catches the mutant that changes >= to >
    #[test]
    fn test_boundary_exactly_18() {
        assert!(is_eligible(18));
        assert!(!is_eligible(17));
    }

    // ---- calculate_price tests ----

    #[test]
    fn test_price_no_discount_under_100() {
        let price = calculate_price(10.0, 5, 10.0);
        assert_eq!(price, 50.0); // 50 <= 100, no discount
    }

    #[test]
    fn test_price_with_discount_over_100() {
        let price = calculate_price(50.0, 3, 10.0);
        // 150 > 100, 10% discount = 15
        assert!((price - 135.0).abs() < 0.01);
    }

    #[test]
    fn test_price_zero_quantity() {
        assert_eq!(calculate_price(100.0, 0, 50.0), 0.0);
    }

    // Boundary test - catches mutant changing > to >=
    #[test]
    fn test_price_exactly_at_100() {
        let price = calculate_price(50.0, 2, 10.0);
        assert_eq!(price, 100.0); // exactly 100, no discount
    }

    #[test]
    fn test_price_just_over_100() {
        let price = calculate_price(50.5, 2, 10.0);
        // 101 > 100, gets 10% discount
        assert!((price - 90.9).abs() < 0.01);
    }

    // ---- calculate_tax tests ----

    #[test]
    fn test_tax_positive_amount() {
        let tax = calculate_tax(100.0, 8.0);
        assert!((tax - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_tax_zero_amount() {
        assert_eq!(calculate_tax(0.0, 8.0), 0.0);
    }

    #[test]
    fn test_tax_negative_amount() {
        assert_eq!(calculate_tax(-50.0, 8.0), 0.0);
    }

    // ---- compare_scores tests ----

    #[test]
    fn test_compare_greater() {
        assert_eq!(compare_scores(10, 5), 1);
    }

    #[test]
    fn test_compare_lesser() {
        assert_eq!(compare_scores(5, 10), -1);
    }

    #[test]
    fn test_compare_equal() {
        assert_eq!(compare_scores(5, 5), 0);
    }

    // ---- is_in_range tests ----

    #[test]
    fn test_in_range_middle() {
        assert!(is_in_range(5, 1, 10));
    }

    #[test]
    fn test_in_range_at_min() {
        assert!(is_in_range(1, 1, 10));
    }

    #[test]
    fn test_in_range_at_max() {
        assert!(is_in_range(10, 1, 10));
    }

    #[test]
    fn test_below_range() {
        assert!(!is_in_range(0, 1, 10));
    }

    #[test]
    fn test_above_range() {
        assert!(!is_in_range(11, 1, 10));
    }

    // ---- calculate_grade tests ----

    #[test]
    fn test_grade_a() {
        assert_eq!(calculate_grade(95), 'A');
        assert_eq!(calculate_grade(90), 'A');
    }

    #[test]
    fn test_grade_b() {
        assert_eq!(calculate_grade(85), 'B');
        assert_eq!(calculate_grade(80), 'B');
    }

    #[test]
    fn test_grade_c() {
        assert_eq!(calculate_grade(75), 'C');
        assert_eq!(calculate_grade(70), 'C');
    }

    #[test]
    fn test_grade_d() {
        assert_eq!(calculate_grade(65), 'D');
        assert_eq!(calculate_grade(60), 'D');
    }

    #[test]
    fn test_grade_f() {
        assert_eq!(calculate_grade(55), 'F');
        assert_eq!(calculate_grade(0), 'F');
    }

    // Boundary tests - catch mutants changing thresholds
    #[test]
    fn test_grade_boundaries() {
        assert_eq!(calculate_grade(89), 'B'); // Just below A
        assert_eq!(calculate_grade(90), 'A'); // Exactly A
        assert_eq!(calculate_grade(79), 'C'); // Just below B
        assert_eq!(calculate_grade(80), 'B'); // Exactly B
    }
}

fn main() {
    println!("Pattern 4: Mutation Testing Demonstration");
    println!("==========================================");
    println!();
    println!("This demonstrates functions suitable for mutation testing.");
    println!("Real mutation testing requires: cargo install cargo-mutants");
    println!();
    println!("Example mutations that tools would generate:");
    println!("  - is_eligible: change `>= 18` to `> 18` or `>= 17`");
    println!("  - calculate_price: change `> 100.0` to `>= 100.0`");
    println!("  - compare_scores: change `>` to `>=` or swap return values");
    println!();

    // Demo the functions
    println!("Function demos:");
    println!("  is_eligible(17) = {}", is_eligible(17));
    println!("  is_eligible(18) = {}", is_eligible(18));
    println!("  calculate_price(50, 3, 10%) = ${:.2}", calculate_price(50.0, 3, 10.0));
    println!("  calculate_grade(85) = {}", calculate_grade(85));

    println!();
    println!("Run tests with: cargo test --bin p4_mutation_testing");
    println!("Run mutation testing with: cargo mutants --bin p4_mutation_testing");
}
