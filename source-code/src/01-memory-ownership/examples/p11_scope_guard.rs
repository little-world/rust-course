// Pattern 11: Scope Guard for Rollback
struct ScopeGuard<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
    fn new(cleanup: F) -> Self {
        ScopeGuard { cleanup: Some(cleanup) }
    }

    fn disarm(mut self) {
        self.cleanup = None;
    }
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

// Usage: Rollback runs unless explicitly disarmed
fn transaction() {
    let guard = ScopeGuard::new(|| println!("Rolling back"));
    // ... do work ...
    guard.disarm(); // Success! No rollback
    println!("Transaction committed successfully");
}

fn failed_transaction() {
    let _guard = ScopeGuard::new(|| println!("Rolling back failed transaction"));
    // Simulating failure - guard not disarmed
} // Rollback runs here

fn main() {
    transaction();
    failed_transaction();
    println!("Scope guard example completed");
}
