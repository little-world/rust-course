// Pattern 5: Formal Verification with Kani
// NOTE: Real verification requires Kani model checker (cargo kani).
// This file demonstrates the patterns with equivalent property-based tests.

// ============================================================================
// Example: Verifying a Safe Add
// ============================================================================

/// Checked addition that returns None on overflow
pub fn checked_add(a: u32, b: u32) -> Option<u32> {
    a.checked_add(b)
}

/// The property we want to prove:
/// If checked_add returns Some(sum), then sum >= a && sum >= b
/// This is what Kani would verify for ALL u32 values.
#[cfg(test)]
fn verify_checked_add_property(a: u32, b: u32) -> bool {
    if let Some(sum) = checked_add(a, b) {
        sum >= a && sum >= b
    } else {
        // None case is fine - overflow was detected
        true
    }
}

// ============================================================================
// Example: Proving State Machines
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Locked,
    Unlocked,
}

/// State machine transition function
pub fn next_door_state(state: DoorState, code_entered: bool) -> DoorState {
    match (state, code_entered) {
        (DoorState::Locked, true) => DoorState::Unlocked,
        (DoorState::Unlocked, false) => DoorState::Locked,
        _ => state,
    }
}

/// Property: The door is always in a valid state (Locked or Unlocked)
/// Kani would verify this for all boolean input combinations
#[cfg(test)]
fn verify_door_state_property(code1: bool, code2: bool) -> bool {
    let s1 = next_door_state(DoorState::Locked, code1);
    let s2 = next_door_state(s1, code2);

    matches!(s2, DoorState::Locked | DoorState::Unlocked)
}

// ============================================================================
// Example: Verifying array bounds
// ============================================================================

/// Safe array access that returns None for out-of-bounds
pub fn safe_get<T: Clone>(arr: &[T], index: usize) -> Option<T> {
    arr.get(index).cloned()
}

/// Property: safe_get never panics and returns Some only for valid indices
#[cfg(test)]
fn verify_safe_get_property<T: Clone>(arr: &[T], index: usize) -> bool {
    match safe_get(arr, index) {
        Some(_) => index < arr.len(),
        None => index >= arr.len(),
    }
}

// ============================================================================
// Example: Verifying invariants
// ============================================================================

/// A bounded counter that stays within [0, max]
#[derive(Debug, Clone)]
pub struct BoundedCounter {
    value: u32,
    max: u32,
}

impl BoundedCounter {
    pub fn new(max: u32) -> Self {
        BoundedCounter { value: 0, max }
    }

    pub fn increment(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }

    pub fn decrement(&mut self) {
        if self.value > 0 {
            self.value -= 1;
        }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    /// Invariant: value is always <= max
    pub fn check_invariant(&self) -> bool {
        self.value <= self.max
    }
}

// ============================================================================
// Tests that simulate what Kani would prove
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Using proptest to approximate what Kani does symbolically

    proptest! {
        // Simulating: #[kani::proof] fn checked_add_never_wraps()
        #[test]
        fn test_checked_add_property(a: u32, b: u32) {
            prop_assert!(verify_checked_add_property(a, b));
        }

        // Simulating: #[kani::proof] fn door_never_skips_locked_state()
        #[test]
        fn test_door_state_property(code1: bool, code2: bool) {
            prop_assert!(verify_door_state_property(code1, code2));
        }

        // Simulating bounds checking proof
        #[test]
        fn test_safe_get_property(
            arr in prop::collection::vec(any::<i32>(), 0..100),
            index: usize
        ) {
            prop_assert!(verify_safe_get_property(&arr, index));
        }
    }

    #[test]
    fn test_checked_add_no_overflow() {
        assert_eq!(checked_add(100, 200), Some(300));
        assert!(verify_checked_add_property(100, 200));
    }

    #[test]
    fn test_checked_add_overflow() {
        assert_eq!(checked_add(u32::MAX, 1), None);
        assert!(verify_checked_add_property(u32::MAX, 1));
    }

    #[test]
    fn test_door_transitions() {
        // Locked -> enter code -> Unlocked
        assert_eq!(next_door_state(DoorState::Locked, true), DoorState::Unlocked);

        // Unlocked -> no code -> Locked
        assert_eq!(next_door_state(DoorState::Unlocked, false), DoorState::Locked);

        // Locked -> no code -> stays Locked
        assert_eq!(next_door_state(DoorState::Locked, false), DoorState::Locked);

        // Unlocked -> enter code -> stays Unlocked
        assert_eq!(next_door_state(DoorState::Unlocked, true), DoorState::Unlocked);
    }

    #[test]
    fn test_bounded_counter_invariant() {
        let mut counter = BoundedCounter::new(5);

        // Increment to max
        for _ in 0..10 {
            counter.increment();
            assert!(counter.check_invariant());
        }
        assert_eq!(counter.value(), 5);

        // Decrement to zero
        for _ in 0..10 {
            counter.decrement();
            assert!(counter.check_invariant());
        }
        assert_eq!(counter.value(), 0);
    }

    // All possible state combinations for door (exhaustive for small state space)
    #[test]
    fn test_door_exhaustive() {
        for code1 in [true, false] {
            for code2 in [true, false] {
                assert!(verify_door_state_property(code1, code2));
            }
        }
    }
}

fn main() {
    println!("Pattern 5: Formal Verification Demonstration");
    println!("=============================================");
    println!();
    println!("This demonstrates patterns used with Kani model checker.");
    println!("Real verification requires: cargo install kani-verifier");
    println!();
    println!("Kani proofs would look like:");
    println!();
    println!("  #[kani::proof]");
    println!("  fn checked_add_never_wraps() {{");
    println!("      let a = kani::any::<u32>();");
    println!("      let b = kani::any::<u32>();");
    println!("      if let Some(sum) = checked_add(a, b) {{");
    println!("          assert!(sum >= a && sum >= b);");
    println!("      }}");
    println!("  }}");
    println!();

    // Demo the functions
    println!("Function demos:");
    println!("  checked_add(100, 200) = {:?}", checked_add(100, 200));
    println!("  checked_add(u32::MAX, 1) = {:?}", checked_add(u32::MAX, 1));
    println!();
    println!("  Door: Locked + enter_code -> {:?}", next_door_state(DoorState::Locked, true));
    println!("  Door: Unlocked + no_code -> {:?}", next_door_state(DoorState::Unlocked, false));

    let mut counter = BoundedCounter::new(3);
    println!();
    println!("  BoundedCounter(max=3):");
    for _ in 0..5 {
        counter.increment();
        println!("    after increment: {} (invariant: {})", counter.value(), counter.check_invariant());
    }

    println!();
    println!("Run tests with: cargo test --bin p5_formal_verification");
}
