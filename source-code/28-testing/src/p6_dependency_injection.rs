// Pattern 6: Mock and Stub Patterns - Dependency Injection
// Demonstrates trait-based dependency injection for testability.

// ============================================================================
// Example: Dependency Injection Patterns
// ============================================================================

// Better: Dependency injection via traits
trait PaymentGateway {
    fn charge(&self, amount: f64) -> Result<String, String>;
}

struct PaymentProcessor<G: PaymentGateway> {
    gateway: G,
}

impl<G: PaymentGateway> PaymentProcessor<G> {
    fn new(gateway: G) -> Self {
        PaymentProcessor { gateway }
    }

    fn process_payment(&self, amount: f64) -> Result<(), String> {
        let transaction_id = self.gateway.charge(amount)?;
        println!("Processed payment: {}", transaction_id);
        Ok(())
    }
}

// Real gateway (for production)
struct StripeGateway {
    api_key: String,
}

impl PaymentGateway for StripeGateway {
    fn charge(&self, amount: f64) -> Result<String, String> {
        // In real code, this would call Stripe API
        println!("Charging ${:.2} via Stripe (key: {}...)", amount, &self.api_key[..4]);
        Ok(format!("stripe_txn_{}", amount as u64))
    }
}

// Mock gateway for testing
struct MockGateway {
    should_succeed: bool,
}

impl PaymentGateway for MockGateway {
    fn charge(&self, amount: f64) -> Result<String, String> {
        if self.should_succeed {
            Ok(format!("mock_txn_{}", amount))
        } else {
            Err("Payment failed".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_payment() {
        let gateway = MockGateway { should_succeed: true };
        let processor = PaymentProcessor::new(gateway);

        assert!(processor.process_payment(99.99).is_ok());
    }

    #[test]
    fn test_failed_payment() {
        let gateway = MockGateway { should_succeed: false };
        let processor = PaymentProcessor::new(gateway);

        assert!(processor.process_payment(99.99).is_err());
    }
}

fn main() {
    // Production use
    let stripe = StripeGateway {
        api_key: "sk_live_abc123xyz".to_string(),
    };
    let processor = PaymentProcessor::new(stripe);

    println!("Production payment:");
    let _ = processor.process_payment(49.99);

    // Test use
    let mock = MockGateway { should_succeed: true };
    let test_processor = PaymentProcessor::new(mock);

    println!("\nTest payment (success):");
    let _ = test_processor.process_payment(49.99);

    let mock_fail = MockGateway { should_succeed: false };
    let fail_processor = PaymentProcessor::new(mock_fail);

    println!("\nTest payment (failure):");
    match fail_processor.process_payment(49.99) {
        Ok(_) => println!("Payment succeeded"),
        Err(e) => println!("Payment failed: {}", e),
    }
}
