//! Pattern 4: Enum Design Patterns
//! Example: Enum State Machines
//!
//! Run with: cargo run --example p4_state_machine

use std::time::Instant;

enum OrderStatus {
    Pending {
        items: Vec<String>,
        customer_id: u64,
    },
    Processing {
        order_id: u64,
        started_at: Instant,
    },
    Shipped {
        order_id: u64,
        tracking_number: String,
    },
    Delivered {
        order_id: u64,
        signature: Option<String>,
    },
    Cancelled {
        order_id: u64,
        reason: String,
    },
}

impl OrderStatus {
    fn process(self) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Pending { items, .. } => {
                if items.is_empty() {
                    return Err("Cannot process empty order".to_string());
                }
                Ok(OrderStatus::Processing {
                    order_id: 12345,
                    started_at: Instant::now(),
                })
            }
            _ => Err("Order is not in pending state".to_string()),
        }
    }

    fn ship(self, tracking: String) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Processing { order_id, .. } => Ok(OrderStatus::Shipped {
                order_id,
                tracking_number: tracking,
            }),
            _ => Err("Order must be processing to ship".to_string()),
        }
    }

    fn deliver(self, signature: Option<String>) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Shipped { order_id, .. } => Ok(OrderStatus::Delivered {
                order_id,
                signature,
            }),
            _ => Err("Order must be shipped to deliver".to_string()),
        }
    }

    fn cancel(self, reason: String) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Pending { .. } | OrderStatus::Processing { order_id: _, .. } => {
                Ok(OrderStatus::Cancelled {
                    order_id: 12345,
                    reason,
                })
            }
            _ => Err("Cannot cancel order in current state".to_string()),
        }
    }

    fn can_cancel(&self) -> bool {
        matches!(
            self,
            OrderStatus::Pending { .. } | OrderStatus::Processing { .. }
        )
    }

    fn status_name(&self) -> &'static str {
        match self {
            OrderStatus::Pending { .. } => "Pending",
            OrderStatus::Processing { .. } => "Processing",
            OrderStatus::Shipped { .. } => "Shipped",
            OrderStatus::Delivered { .. } => "Delivered",
            OrderStatus::Cancelled { .. } => "Cancelled",
        }
    }
}

fn main() {
    // Usage: State transitions consume self and return new state.
    println!("=== Order State Machine Demo ===\n");

    // Create a pending order
    let order = OrderStatus::Pending {
        items: vec!["Book".to_string(), "Pen".to_string()],
        customer_id: 42,
    };
    println!("Initial state: {}", order.status_name());
    println!("Can cancel: {}", order.can_cancel());

    // Process the order
    let order = order.process().unwrap();
    println!("\nAfter process: {}", order.status_name());
    assert!(order.can_cancel());

    // Ship the order
    let order = order.ship("TRACK123456".to_string()).unwrap();
    println!("After ship: {}", order.status_name());
    println!("Can cancel: {}", order.can_cancel()); // false now

    // Deliver the order
    let order = order.deliver(Some("J. Doe".to_string())).unwrap();
    println!("After deliver: {}", order.status_name());

    // Try invalid transition
    println!("\n=== Testing Invalid Transitions ===");
    let new_order = OrderStatus::Pending {
        items: vec![],
        customer_id: 1,
    };
    match new_order.process() {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {}", e),
    }
}
