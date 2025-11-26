## Project 2: Order Processing State Machine with Enums

### Problem Statement

Build a type-safe order processing system that uses enums to model state transitions. You'll start with basic enum variants, add exhaustive pattern matching for state transitions, then implement compile-time state checking using the typestate pattern.

### Why It Matters

**Real-World Impact**: State management bugs are expensive:

**The State Confusion Problem**:
- **Amazon (2006)**: Order processing bug caused incorrect shipments, millions in losses
- **Payment processors**: Process same payment twice due to state confusion
- **E-commerce**: Ship orders that were cancelled, refund orders not yet paid
- **Booking systems**: Double-book resources, allow modifications after confirmation

**Without Type-Safe States**:
```rust
struct Order {
    id: u64,
    state: String,  // "pending", "paid", "shipped"???
    tracking_number: Option<String>,
    payment_id: Option<String>,
}

fn ship_order(order: &mut Order) {
    // Runtime checks everywhere!
    if order.state == "paid" {  // String comparison prone to typos
        order.tracking_number = Some(generate_tracking());
        order.state = "shipped".to_string();  // Typo: "shiped"?
    }
    // What if state was "cancelled"? Silent failure!
}
```

**With Enum State Machine**:
```rust
enum OrderState {
    Pending { items: Vec<Item> },
    Paid { payment_id: String },
    Shipped { tracking: String },
    Cancelled { reason: String },
}

impl OrderState {
    fn ship(self) -> Result<OrderState, String> {
        match self {
            OrderState::Paid { payment_id } => {
                Ok(OrderState::Shipped {
                    tracking: generate_tracking()
                })
            }
            _ => Err("Can only ship paid orders")
        }
    }
}
```

**Performance & Safety Benefits**:
- **Exhaustive matching**: Compiler forces handling of all states
- **Impossible states**: Can't have both `tracking_number` and be unpaid
- **Zero runtime cost**: Enum same size as largest variant + 1-byte discriminant
- **Self-documenting**: All valid states visible in type definition

**Real Production Examples**:
- **Payment gateways**: Pending → Authorized → Captured → Settled states
- **Workflow engines**: Draft → Review → Approved → Published transitions
- **Connection pools**: Idle → Active → Closing → Closed states
- **HTTP clients**: Connecting → Connected → Reading → Complete states

### Use Cases

**When you need this pattern**:
1. **Order/Payment processing**: Pending → Paid → Shipped → Delivered
2. **Document workflows**: Draft → Review → Approved → Published
3. **Network connections**: Connecting → Connected → Closing → Closed
4. **Game states**: Menu → Playing → Paused → GameOver
5. **User authentication**: Anonymous → LoggedIn → Verified → Admin
6. **File uploads**: Validating → Uploading → Processing → Complete

**Enum State Machines Prevent**:
- Shipping unpaid orders
- Charging paid orders twice
- Cancelling already shipped orders
- Accessing data from wrong state
- Forgetting to handle edge cases

### Learning Goals

- Use enums to model state machines with exhaustive matching
- Implement state transitions that consume and transform states
- Understand pattern matching for compile-time guarantees
- Build typestate pattern for impossible-states-as-unrepresentable
- Compare runtime state checking vs compile-time state checking
- Handle associated data per state variant

---

### Milestone 1: Basic Order Enum with States

**Goal**: Define an enum representing different order states with associated data.

**Starter Code**:
```rust
use std::time::Instant;

#[derive(Debug, Clone)]
struct Item {
    product_id: u64,
    name: String,
    price: f64,
}

// TODO: Define OrderState enum with variants:
// - Pending { items: Vec<Item>, customer_id: u64 }
// - Paid { order_id: u64, payment_id: String, amount: f64 }
// - Shipped { order_id: u64, tracking_number: String }
// - Delivered { order_id: u64, delivered_at: Instant }
// - Cancelled { order_id: u64, reason: String }

#[derive(Debug, Clone)]
enum OrderState {
    // TODO: Add variants here
}

impl OrderState {
    fn new_pending(items: Vec<Item>, customer_id: u64) -> Self {
        // TODO: Create Pending variant
        todo!()
    }

    fn status_string(&self) -> &str {
        // TODO: Match on self and return appropriate status string
        // Hint: "Pending", "Paid", "Shipped", "Delivered", "Cancelled"
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_create_pending_order() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];
    let order = OrderState::new_pending(items, 123);

    assert_eq!(order.status_string(), "Pending");
}

#[test]
fn test_all_states() {
    // TODO: Test creating each state variant
    let pending = OrderState::Pending {
        items: vec![],
        customer_id: 1,
    };

    let paid = OrderState::Paid {
        order_id: 1,
        payment_id: "pay_123".to_string(),
        amount: 99.99,
    };

    // Test pattern matching works
    match pending {
        OrderState::Pending { .. } => { /* OK */ }
        _ => panic!("Should be pending"),
    }
}
```

**Check Your Understanding**:
- Why does each variant have different associated data?
- What prevents you from accessing `payment_id` on a `Pending` order?
- How does this compare to having optional fields on a single struct?

---

### 🔄 Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitations**:
1. **No state transitions**: Can create any state, but can't safely transition between them
2. **Validation missing**: Nothing prevents creating `Paid` order with negative amount
3. **No business rules**: Could go directly from Pending to Delivered, skipping payment
4. **Manual state checking**: Users must pattern match everywhere to get data

**What we're adding**: **State transition methods** that:
- Consume the current state and return a new state
- Enforce valid transitions (can't ship before paying)
- Validate business rules (can't cancel after delivery)
- Use `Result` for error handling

**Improvements**:
- **Type-safe transitions**: `order.pay()` consumes `Pending`, returns `Paid`
- **Exhaustive matching**: Compiler ensures all current states handled
- **Business logic**: Validation happens in transition methods
- **Self-documenting**: Method names show valid transitions

---

### Milestone 2: State Transitions with Pattern Matching

**Goal**: Implement methods that transition between states with validation.

**Starter Code**:
```rust
impl OrderState {
    fn pay(self, payment_id: String) -> Result<Self, String> {
        // TODO: Match on self
        // - If Pending: validate items not empty, calculate total, return Paid
        // - Otherwise: return Err saying order is not in pending state
        match self {
            OrderState::Pending { items, customer_id } => {
                // TODO: Validate items not empty
                // TODO: Calculate total amount
                // TODO: Return Paid variant with order_id, payment_id, amount
                todo!()
            }
            _ => Err("Can only pay for pending orders".to_string()),
        }
    }

    fn ship(self, tracking_number: String) -> Result<Self, String> {
        // TODO: Match on self
        // - If Paid: return Shipped with order_id and tracking_number
        // - Otherwise: return Err
        todo!()
    }

    fn deliver(self) -> Result<Self, String> {
        // TODO: Match on self
        // - If Shipped: return Delivered with order_id and Instant::now()
        // - Otherwise: return Err
        todo!()
    }

    fn cancel(self, reason: String) -> Result<Self, String> {
        // TODO: Match on self
        // - If Pending or Paid: return Cancelled with reason
        // - If Shipped or Delivered: return Err (can't cancel after shipping)
        // Hint: Use | to match multiple variants
        todo!()
    }

    fn can_cancel(&self) -> bool {
        // TODO: Return true only for Pending or Paid states
        // Hint: Use matches! macro
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_valid_transitions() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let order = OrderState::new_pending(items, 123);
    let order = order.pay("payment_123".to_string()).unwrap();
    assert_eq!(order.status_string(), "Paid");

    let order = order.ship("TRACK123".to_string()).unwrap();
    assert_eq!(order.status_string(), "Shipped");

    let order = order.deliver().unwrap();
    assert_eq!(order.status_string(), "Delivered");
}

#[test]
fn test_invalid_transitions() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let order = OrderState::new_pending(items, 123);

    // Can't ship before paying
    assert!(order.clone().ship("TRACK123".to_string()).is_err());

    // Can pay
    let order = order.pay("payment_123".to_string()).unwrap();

    // Can't pay again
    assert!(order.clone().pay("payment_456".to_string()).is_err());
}

#[test]
fn test_cancellation_rules() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    // Can cancel pending order
    let order = OrderState::new_pending(items.clone(), 123);
    assert!(order.can_cancel());
    assert!(order.cancel("Customer request".to_string()).is_ok());

    // Can cancel paid order
    let order = OrderState::new_pending(items.clone(), 123)
        .pay("payment_123".to_string())
        .unwrap();
    assert!(order.can_cancel());

    // Cannot cancel shipped order
    let order = OrderState::new_pending(items, 123)
        .pay("payment_123".to_string())
        .unwrap()
        .ship("TRACK123".to_string())
        .unwrap();
    assert!(!order.can_cancel());
    assert!(order.cancel("Too late".to_string()).is_err());
}
```

**Check Your Understanding**:
- Why do transition methods take `self` (ownership) instead of `&self`?
- How does the compiler help you handle all possible current states?
- What happens if you forget to handle a variant in a match?
- Why is returning `Result` better than panicking on invalid transitions?

---

### 🔄 Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Remaining Issues**:
1. **Runtime checks**: Still possible to call wrong method at runtime, just returns `Err`
2. **API not self-documenting**: IDE doesn't show which methods available for current state
3. **Enum still mutable**: Someone could manually construct invalid transitions
4. **No compile-time guarantees**: `order.ship()` compiles even on `Pending` order

**What we're adding**: **Typestate pattern** - use the type system to encode states:
- Each state is a separate type: `struct Pending`, `struct Paid`, etc.
- Generic wrapper: `Order<State>` where `State` is the current state type
- Methods only available on appropriate state: `Order<Pending>` can't call `ship()`
- Transitions return new type: `pay()` consumes `Order<Pending>`, returns `Order<Paid>`

**Improvements**:
- **Compile-time checking**: `pending_order.ship()` doesn't compile!
- **Zero runtime cost**: State stored in type, not value
- **IDE support**: Autocomplete only shows valid methods for current state
- **Impossible states impossible**: Can't have `Order<Shipped>` without going through payment

**Trade-offs**:
- **More complex**: More types and trait bounds
- **Less dynamic**: Can't store mixed states in Vec without trait objects
- **Worth it when**: State transitions known at compile-time

---

### Milestone 3: Typestate Pattern for Compile-Time Safety

**Goal**: Use phantom types to enforce state transitions at compile-time.

**Starter Code**:
```rust
use std::marker::PhantomData;

// State marker types (zero-sized)
struct Pending;
struct Paid;
struct Shipped;
struct Delivered;
struct Cancelled;

// Order with typestate
struct Order<State> {
    id: u64,
    customer_id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,
}

// Methods available only on Pending orders
impl Order<Pending> {
    fn new(customer_id: u64, items: Vec<Item>) -> Result<Self, String> {
        // TODO: Validate items not empty
        // TODO: Create Order<Pending> with generated id
        // Hint: Use PhantomData for _state field
        todo!()
    }

    fn pay(self, payment_id: String) -> Result<Order<Paid>, String> {
        // TODO: Validate items, calculate total
        // TODO: Process payment (simulate)
        // TODO: Return Order<Paid> with same id, customer_id, items
        // Hint: Order { id: self.id, customer_id: self.customer_id, items: self.items, _state: PhantomData }
        todo!()
    }

    fn cancel(self, reason: String) -> Order<Cancelled> {
        // TODO: Convert to Cancelled state
        // Note: Doesn't return Result since always allowed
        todo!()
    }
}

// Methods available only on Paid orders
impl Order<Paid> {
    fn ship(self, tracking_number: String) -> Order<Shipped> {
        // TODO: Return Order<Shipped>
        // In real system, would store tracking_number
        // For now, just transition state
        todo!()
    }

    fn cancel(self, reason: String) -> Order<Cancelled> {
        // TODO: Convert to Cancelled state
        todo!()
    }
}

// Methods available only on Shipped orders
impl Order<Shipped> {
    fn deliver(self) -> Order<Delivered> {
        // TODO: Return Order<Delivered>
        todo!()
    }

    // Note: No cancel method! Can't cancel after shipping
}

// Delivered orders have no more transitions (terminal state)
impl Order<Delivered> {
    // No state transition methods
}

// Common methods available in all states
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
```

**Checkpoint Tests**:
```rust
#[test]
fn test_typestate_valid_flow() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let order = Order::<Pending>::new(1, items).unwrap();
    let order = order.pay("payment_123".to_string()).unwrap();
    let order = order.ship("TRACK123".to_string());
    let order = order.deliver();

    // order is now Order<Delivered>
    assert_eq!(order.customer_id(), 1);
}

#[test]
fn test_compile_time_enforcement() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let pending_order = Order::<Pending>::new(1, items).unwrap();

    // These won't compile! Uncomment to see errors:
    // pending_order.ship("TRACK123".to_string()); // ❌ No ship method on Pending
    // pending_order.deliver(); // ❌ No deliver method on Pending

    let paid_order = pending_order.pay("payment_123".to_string()).unwrap();
    // paid_order.pay("payment_456".to_string()); // ❌ No pay method on Paid (consumed)

    let shipped_order = paid_order.ship("TRACK123".to_string());
    // shipped_order.cancel("Oops".to_string()); // ❌ No cancel method on Shipped!
}

#[test]
fn test_cancellation_only_early_states() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    // Can cancel pending
    let order = Order::<Pending>::new(1, items.clone()).unwrap();
    let _cancelled = order.cancel("Customer request".to_string());

    // Can cancel paid
    let order = Order::<Pending>::new(1, items).unwrap();
    let order = order.pay("payment_123".to_string()).unwrap();
    let _cancelled = order.cancel("Changed mind".to_string());

    // Shipped orders don't have cancel method - compile-time enforcement!
}

#[test]
fn test_common_methods_all_states() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let pending = Order::<Pending>::new(1, items.clone()).unwrap();
    assert_eq!(pending.customer_id(), 1);

    let paid = Order::<Pending>::new(1, items).unwrap()
        .pay("payment_123".to_string())
        .unwrap();
    assert_eq!(paid.customer_id(), 1);

    // Common methods available in all states
}
```

**Check Your Understanding**:
- Why is `_state: PhantomData<State>` needed?
- What's the memory size of `Order<Pending>` vs `Order<Paid>`? (Hint: same!)
- Why can't you store `Vec<Order<??>>` with mixed states?
- How does IDE autocomplete know which methods are available?
- When would you prefer runtime enum states vs compile-time typestates?

---

### Complete Project Summary

**What You Built**:
1. Enum-based state machine with associated data per state
2. State transition methods with exhaustive pattern matching
3. Typestate pattern for compile-time state checking
4. Comparison of runtime vs compile-time state enforcement

**Key Concepts Practiced**:
- Enum variants with different associated data
- Exhaustive pattern matching
- Consuming transitions (taking `self`)
- Phantom types and zero-sized types
- Compile-time state machines

**Runtime vs Compile-Time Comparison**:

| Aspect | Enum States (Runtime) | Typestate (Compile-Time) |
|--------|----------------------|--------------------------|
| **Flexibility** | Can store mixed states in `Vec<OrderState>` | Can't store mixed states easily |
| **Validation** | Returns `Result`, checked at runtime | Compile error for invalid transitions |
| **IDE Support** | Shows all methods on all states | Shows only valid methods for current state |
| **Memory** | Size = largest variant + discriminant | Size = data only, state in type |
| **Complexity** | Simpler, one type | More complex, multiple types |
| **Use Case** | Dynamic state, runtime decisions | Known state flow, API safety |

**Real-World Applications**:
- **Enum approach**: Payment processors, workflow engines with dynamic states
- **Typestate approach**: Database connections, file handles, builder patterns

