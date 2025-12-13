use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};

/* ============================================================
 * Shared domain types
 * ============================================================
 */

#[derive(Debug, Clone, PartialEq)]
struct Item {
    product_id: u64,
    name: String,
    price: f64,
}

/* ============================================================
 * Milestone 1 + 2: Enum-based runtime state machine
 * ============================================================
 */

#[derive(Debug, Clone, PartialEq)]
enum OrderState {
    Pending {
        items: Vec<Item>,
        customer_id: u64,
    },
    Paid {
        order_id: u64,
        payment_id: String,
        amount: f64,
    },
    Shipped {
        order_id: u64,
        tracking_number: String,
    },
    Delivered {
        order_id: u64,
        delivered_at: u64,
    },
    Cancelled {
        order_id: u64,
        reason: String,
    },
}

impl OrderState {
    fn new_pending(items: Vec<Item>, customer_id: u64) -> Self {
        OrderState::Pending { items, customer_id }
    }

    fn status_string(&self) -> &str {
        match self {
            OrderState::Pending { .. } => "Pending",
            OrderState::Paid { .. } => "Paid",
            OrderState::Shipped { .. } => "Shipped",
            OrderState::Delivered { .. } => "Delivered",
            OrderState::Cancelled { .. } => "Cancelled",
        }
    }

    fn pay(self, payment_id: String) -> Result<Self, String> {
        match self {
            OrderState::Pending { items, customer_id } => {
                if items.is_empty() {
                    return Err("Cannot pay for empty order".into());
                }

                let amount: f64 = items.iter().map(|i| i.price).sum();

                Ok(OrderState::Paid {
                    order_id: customer_id,
                    payment_id,
                    amount,
                })
            }
            _ => Err("Can only pay for pending orders".into()),
        }
    }

    fn ship(self, tracking_number: String) -> Result<Self, String> {
        match self {
            OrderState::Paid { order_id, .. } => Ok(OrderState::Shipped {
                order_id,
                tracking_number,
            }),
            _ => Err("Can only ship paid orders".into()),
        }
    }

    fn deliver(self) -> Result<Self, String> {
        match self {
            OrderState::Shipped { order_id, .. } => Ok(OrderState::Delivered {
                order_id,
                delivered_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }),
            _ => Err("Can only deliver shipped orders".into()),
        }
    }

    fn cancel(self, reason: String) -> Result<Self, String> {
        match self {
            OrderState::Pending { customer_id, .. }
            | OrderState::Paid { order_id: customer_id, .. } => Ok(OrderState::Cancelled {
                order_id: customer_id,
                reason,
            }),
            _ => Err("Cannot cancel after shipping".into()),
        }
    }

    fn can_cancel(&self) -> bool {
        matches!(
            self,
            OrderState::Pending { .. } | OrderState::Paid { .. }
        )
    }
}

/* ============================================================
 * Milestone 3: Typestate pattern (compile-time safety)
 * ============================================================
 */

struct Pending;
struct Paid;
struct Shipped;
struct Delivered;
struct Cancelled;

struct Order<State> {
    id: u64,
    customer_id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,
}

impl Order<Pending> {
    fn new(customer_id: u64, items: Vec<Item>) -> Result<Self, String> {
        if items.is_empty() {
            return Err("Order must contain at least one item".into());
        }

        Ok(Self {
            id: customer_id,
            customer_id,
            items,
            _state: PhantomData,
        })
    }

    fn pay(self, _payment_id: String) -> Result<Order<Paid>, String> {
        Ok(Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        })
    }

    fn cancel(self, _reason: String) -> Order<Cancelled> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        }
    }
}

impl Order<Paid> {
    fn ship(self, _tracking_number: String) -> Order<Shipped> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        }
    }

    fn cancel(self, _reason: String) -> Order<Cancelled> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        }
    }
}

impl Order<Shipped> {
    fn deliver(self) -> Order<Delivered> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        }
    }
}

impl<State> Order<State> {
    fn id(&self) -> u64 {
        self.id
    }

    fn customer_id(&self) -> u64 {
        self.customer_id
    }

    fn items(&self) -> &[Item] {
        &self.items
    }
}

/* ============================================================
 * Demo (cargo run)
 * ============================================================
 */

fn main() {
    let items = vec![Item {
        product_id: 1,
        name: "Widget".into(),
        price: 9.99,
    }];

    println!("== Runtime enum state machine ==");
    let order = OrderState::new_pending(items.clone(), 42);
    let order = order.pay("PAY123".into()).unwrap();
    let order = order.ship("TRACK123".into()).unwrap();
    let order = order.deliver().unwrap();
    println!("Final state: {}", order.status_string());

    println!("\n== Typestate machine ==");
    let order = Order::<Pending>::new(42, items).unwrap();
    let order = order.pay("PAY123".into()).unwrap();
    let order = order.ship("TRACK123".into());
    let order = order.deliver();
    println!("Delivered order for customer {}", order.customer_id());
}

/* ============================================================
 * Tests (cargo test)
 * ============================================================
 */

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items() -> Vec<Item> {
        vec![Item {
            product_id: 1,
            name: "Widget".into(),
            price: 9.99,
        }]
    }

    #[test]
    fn test_enum_valid_transitions() {
        let order = OrderState::new_pending(sample_items(), 1);
        let order = order.pay("pay".into()).unwrap();
        let order = order.ship("track".into()).unwrap();
        let order = order.deliver().unwrap();

        assert_eq!(order.status_string(), "Delivered");
    }

    #[test]
    fn test_enum_invalid_transitions() {
        let order = OrderState::new_pending(sample_items(), 1);
        assert!(order.clone().ship("track".into()).is_err());

        let order = order.pay("pay".into()).unwrap();
        assert!(order.clone().pay("pay2".into()).is_err());
    }

    #[test]
    fn test_enum_cancellation_rules() {
        let order = OrderState::new_pending(sample_items(), 1);
        assert!(order.can_cancel());

        let order = order.pay("pay".into()).unwrap();
        assert!(order.can_cancel());

        let order = order.ship("track".into()).unwrap();
        assert!(!order.can_cancel());
    }

    #[test]
    fn test_typestate_valid_flow() {
        let order = Order::<Pending>::new(1, sample_items()).unwrap();
        let order = order.pay("pay".into()).unwrap();
        let order = order.ship("track".into());
        let order = order.deliver();

        assert_eq!(order.customer_id(), 1);
    }

    #[test]
    fn test_typestate_common_methods() {
        let order = Order::<Pending>::new(1, sample_items()).unwrap();
        assert_eq!(order.items().len(), 1);
        assert_eq!(order.customer_id(), 1);
    }
}