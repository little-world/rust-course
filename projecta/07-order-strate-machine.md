## Project 7: Order Processing State Machine with Enums

### Problem Statement

Build a type-safe order processing system that uses enums to model state transitions. You'll start with basic enum variants, add exhaustive pattern matching for state transitions, then implement compile-time state checking using the typestate pattern.

### Use Cases

**When you need this pattern**:
1. **Order/Payment processing**: Pending → Paid → Shipped → Delivered
2. **Document workflows**: Draft → Review → Approved → Published
3. **Network connections**: Connecting → Connected → Closing → Closed
4. **Game states**: Menu → Playing → Paused → GameOver
5. **User authentication**: Anonymous → LoggedIn → Verified → Admin
6. **File uploads**: Validating → Uploading → Processing → Complete


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


**Enum State Machines Prevent**:
- Shipping unpaid orders
- Charging paid orders twice
- Cancelling already shipped orders
- Accessing data from wrong state
- Forgetting to handle edge cases


### Milestone 1: Basic Order Enum with States

**Goal**: Define an enum representing different order states, where each variant carries state-specific data.

**Why This Milestone Matters**:

This milestone introduces **enums as state machines**—one of Rust's most powerful patterns. Unlike structs (which group related data), enums represent **alternatives**—a value is exactly one variant at any time.

**Structs vs Enums**:

```rust
// Struct: Has ALL fields at once (AND)
struct User {
    name: String,     // AND
    email: String,    // AND
    age: u32,        // AND
}

// Enum: Is EXACTLY ONE variant (OR)
enum LoginState {
    Anonymous,              // OR
    LoggedIn { user_id: u64 },  // OR
    Admin { user_id: u64, permissions: Vec<String> },  // OR
}
```

**Enum Syntax**:

```rust
enum OrderState {
    // Unit variant (no data)
    Simple,

    // Tuple variant (unnamed fields)
    WithData(String, u64),

    // Struct variant (named fields) - most common
    WithNamedData {
        field1: String,
        field2: u64,
    },
}
```

We use **struct variants** for clarity—named fields are self-documenting.




**Pattern Matching**:

The primary way to work with enums is pattern matching:

```rust
let order = OrderState::Pending {
    items: vec![...],
    customer_id: 123,
};

match order {
    OrderState::Pending { items, customer_id } => {
        println!("Order for customer {} has {} items", customer_id, items.len());
    }
    OrderState::Paid { payment_id, amount, .. } => {
        println!("Payment {} for ${}", payment_id, amount);
    }
    OrderState::Shipped { tracking_number, .. } => {
        println!("Shipped: {}", tracking_number);
    }
    OrderState::Delivered { .. } => {
        println!("Delivered!");
    }
    OrderState::Cancelled { reason, .. } => {
        println!("Cancelled: {}", reason);
    }
}
```

**Exhaustive Matching**:

The compiler **forces** you to handle all variants:

```rust
match order {
    OrderState::Pending { .. } => { },
    OrderState::Paid { .. } => { },
    // ❌ Compile error: missing Shipped, Delivered, Cancelled!
}
```

This prevents bugs where you forget to handle a case.

**Memory Layout**:

Enums use a **discriminant** (tag) to track which variant is active:

```rust
enum OrderState {
    Pending { items: Vec<Item>, customer_id: u64 },  // 32 bytes
    Paid { order_id: u64, payment_id: String, amount: f64 },  // 40 bytes
    Shipped { order_id: u64, tracking_number: String },  // 32 bytes
}
```

**Memory size**: `max(all variants) + discriminant`
- Largest variant: `Paid` (40 bytes)
- Discriminant: 1 byte (actually 8 bytes with alignment)
- **Total**: ~48 bytes

All variants share the same memory space. Only one is active at a time.

**Why Use Enum for State Machines?**

**State machines** have:
1. **Finite states**: Known, fixed set of states
2. **Transitions**: Rules for moving between states
3. **State-specific data**: Each state needs different information
4. **Exclusive states**: Can't be in two states at once

Enums are **perfect** for this! Each variant = one state, pattern matching = transitions.

**What We're Building**:

Five order states representing the complete order lifecycle:

1. **`Pending`**: Order created, awaiting payment
   - Contains: `items`, `customer_id`
   - Can transition to: `Paid`, `Cancelled`

2. **`Paid`**: Payment received, awaiting shipment
   - Contains: `order_id`, `payment_id`, `amount`
   - Can transition to: `Shipped`, `Cancelled`

3. **`Shipped`**: Package shipped, in transit
   - Contains: `order_id`, `tracking_number`
   - Can transition to: `Delivered`

4. **`Delivered`**: Package delivered to customer
   - Contains: `order_id`, `delivered_at`
   - Terminal state (no further transitions)

5. **`Cancelled`**: Order cancelled
   - Contains: `order_id`, `reason`
   - Terminal state


**Starter Code**:
```rust
use std::time::Instant;

// Item: Represents a product in an order
// Role: Stores product details for order line items
#[derive(Debug, Clone)]
struct Item {
   // TODO: add the fields: 
   // Unique identifier for the product
   // Product display name
   // Product price in dollars
}

// OrderState: Enum representing all possible order states
// Role: Type-safe state machine where each variant has state-specific data
#[derive(Debug, Clone)]
enum OrderState {
   //TODO: Define variants: - Pending - Paid  - Shipped - Delivered - Cancelled
}

impl OrderState {
    // new_pending: Creates a new order in Pending state
    // Role: Constructor for initial order state with items and customer
    fn new_pending(items: Vec<Item>, customer_id: u64) -> Self {
        // TODO: Create Pending variant
        todo!()
    }

    // status_string: Returns human-readable status
    // Role: Provides string representation of current state for display
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

###  Why Milestone 1 Isn't Enough 

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

**Goal**: Implement methods that transition between states with validation, using pattern matching to consume states and enforce valid transitions.

**Issues**:
1. **No validation**: Can create `Paid` state without actually processing payment
2. **Skip steps**: Can go directly from `Pending` to `Shipped`, bypassing payment
3. **No business logic**: Amount calculation, inventory checks, fraud detection—all missing
4. **Manual construction**: Easy to forget required fields or use wrong data

**Solution: Controlled Transition**:

```rust
impl OrderState {
    // Only way to go from Pending to Paid
    fn pay(self, payment_id: String) -> Result<Self, String> {
        match self {
            OrderState::Pending { items, customer_id } => {
                // ✅ Validate items
                if items.is_empty() {
                    return Err("Cannot pay for empty order".to_string());
                }

                // ✅ Calculate total
                let amount: f64 = items.iter().map(|i| i.price).sum();

                // ✅ Process payment (in real system)
                // payment_processor.charge(payment_id, amount)?;

                // ✅ Transition to Paid state
                Ok(OrderState::Paid {
                    order_id: customer_id,
                    payment_id,
                    amount,
                })
            }
            // ✅ Reject invalid transitions
            _ => Err("Can only pay for pending orders".to_string()),
        }
    }
}

// Now the only way to get Paid state:
let order = OrderState::new_pending(items, 123);
let order = order.pay("PAY_123".to_string())?;  // Validated!
```

**What We're Building**:

Four transition methods representing the order lifecycle:

1. **`pay(self, payment_id) -> Result<OrderState, String>`**
    - Consumes: `Pending`
    - Produces: `Paid` or error
    - Validates: Items not empty, calculates total amount
    - Business rule: Must have items to pay for

2. **`ship(self, tracking_number) -> Result<OrderState, String>`**
    - Consumes: `Paid`
    - Produces: `Shipped` or error
    - Business rule: Can't ship unpaid orders

3. **`deliver(self) -> Result<OrderState, String>`**
    - Consumes: `Shipped`
    - Produces: `Delivered` or error
    - Adds: Delivery timestamp
    - Business rule: Can't deliver unshipped orders

4. **`cancel(self, reason) -> Result<OrderState, String>`**
    - Consumes: `Pending` or `Paid` only
    - Produces: `Cancelled` or error
    - Business rule: Can't cancel after shipping

**Why Take `self` (Ownership)?**
**Consuming transitions** prevent:
- Using old states after transition
- Paying for same order twice
- Concurrent access to transitioning state
- Forgetting to use the new state

**Memory and Performance**:

- **No allocation overhead**: Transitions just move data, no extra allocations
- **No copying**: `self` moved by value, not copied
- **Same memory size**: `OrderState` always `max(variants) + discriminant`
- **Stack-based**: Entire state machine lives on stack

**State Machine Guarantees**:

✅ **Compile-time guarantees**:
- All variants handled in match (exhaustiveness)
- Type-correct data in each variant
- Can't create variant without required fields

✅ **Runtime guarantees** (via transitions):
- Can't skip payment and go straight to shipped
- Can't pay for empty order
- Can't cancel after shipping
- Can't pay for same order twice (consuming `self`)

❌ **Still possible** (will fix in Milestone 3):
- Calling `order.ship()` on `Pending` order (returns `Err` at runtime)
- IDE shows all methods on all states (no compile-time filtering)
- Can store mixed states in collections but lose type info


**Starter Code**:
```rust
impl OrderState {
    // pay: Transitions from Pending to Paid state
    // Role: Processes payment, validates items, calculates total
    // Consumes Pending state, returns Paid state or error
    fn pay(self, payment_id: String) -> Result<Self, String> {
        // Match on current state to enforce valid transitions
        match self {
            OrderState::Pending { items, customer_id } => {
                // TODO: Validate items not empty
                // TODO: Calculate total amount by summing item prices
                // TODO: Return Paid variant with order_id, payment_id, amount
                // Hint: Generate order_id from customer_id (e.g., customer_id as order_id)
                todo!()
            }
            _ => Err("Can only pay for pending orders".to_string()),
        }
    }

    // ship: Transitions from Paid to Shipped state
    // Role: Records shipment with tracking number
    // Consumes Paid state, returns Shipped state or error
    fn ship(self, tracking_number: String) -> Result<Self, String> {
        // TODO: Match on self
        // - If Paid: extract order_id, return Shipped with order_id and tracking_number
        // - Otherwise: return Err("Can only ship paid orders")
        todo!()
    }

    // deliver: Transitions from Shipped to Delivered state
    // Role: Marks order as delivered with timestamp
    // Consumes Shipped state, returns Delivered state or error
    fn deliver(self) -> Result<Self, String> {
        // TODO: Match on self
        // - If Shipped: extract order_id, return Delivered with order_id and Instant::now()
        // - Otherwise: return Err("Can only deliver shipped orders")
        todo!()
    }

    // cancel: Transitions to Cancelled state (only from Pending/Paid)
    // Role: Cancels order with reason, enforces business rules
    // Consumes current state, returns Cancelled state or error
    fn cancel(self, reason: String) -> Result<Self, String> {
        // TODO: Match on self
        // - If Pending or Paid: extract order_id (or use customer_id), return Cancelled
        // - If Shipped or Delivered: return Err("Cannot cancel after shipping")
        // Hint: Use | to match multiple variants: OrderState::Pending { .. } | OrderState::Paid { .. }
        todo!()
    }

    // can_cancel: Checks if order can be cancelled
    // Role: Query method that doesn't consume state
    fn can_cancel(&self) -> bool {
        // TODO: Return true only for Pending or Paid states
        // Hint: Use matches! macro: matches!(self, OrderState::Pending { .. } | OrderState::Paid { .. })
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

### Why Milestone 2 Isn't Enough 

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

**Goal**: Use phantom types to encode states in the type system, making invalid state transitions impossible to compile rather than just returning errors at runtime.

**Problems with runtime checking**:
1. **Late error detection**: Find bugs when testing, not when coding
2. **Poor IDE support**: Autocomplete shows all methods on all states
3. **Defensive programming**: Must handle all `Result::Err` cases
4. **Lost type information**: `Vec<OrderState>` loses which specific state each order is in
5. **Runtime cost**: Every transition checks state discriminant

**What We're Building**:

Five marker types and one generic struct:

**State Markers** (zero-sized types):
```rust
struct Pending;    // 0 bytes
struct Paid;       // 0 bytes
struct Shipped;    // 0 bytes
struct Delivered;  // 0 bytes
struct Cancelled;  // 0 bytes
```

**Generic Order**:
```rust
struct Order<State> {
    id: u64,
    customer_id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,  // Zero-sized!
}
```

**Memory layout**: All `Order<State>` variants are **exactly the same size**!

**What is `PhantomData<State>`?**
`PhantomData` is a **zero-sized type** that tells the compiler "this struct owns a `State` type, even though we don't actually store it":

```rust
use std::marker::PhantomData;

struct Order<State> {
    id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,  // "Pretend" we have a State
}

// Why PhantomData is needed:
struct OrderBroken<State> {  // ❌ Error: parameter `State` is never used
    id: u64,
    items: Vec<Item>,
}

struct OrderFixed<State> {   // ✅ OK: State appears in PhantomData
    id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,
}
```

**PhantomData properties**:
- **Size**: 0 bytes (optimized away at compile-time)
- **Purpose**: Make generic parameter `State` "used" so compiler accepts it
- **Ownership**: Tells compiler about ownership/lifetime relationships
- **Convention**: Field name starts with `_` to indicate "unused at runtime"


**Advantages of Typestate Pattern**:

✅ **Compile-time safety**: Invalid transitions caught before runtime
✅ **Better IDE support**: Autocomplete shows only valid methods for current state
✅ **Self-documenting**: Type signatures show state flow
✅ **Zero runtime cost**: State stored in type, not value
✅ **Impossible states impossible**: Can't have `Order<Shipped>` without paying first
✅ **Clearer error messages**: Compiler explains what went wrong and suggests fixes

**Disadvantages of Typestate Pattern**:

❌ **More complex**: More types and `impl` blocks than enum approach
❌ **Less dynamic**: Can't store `Vec<Order<?>>` with mixed states easily
❌ **Verbose generics**: Type signatures get longer: `Order<Pending>` vs `OrderState`
❌ **Trait objects difficult**: Need trait bounds for dynamic dispatch
❌ **Learning curve**: PhantomData and type-level programming are advanced concepts

**When to Use Typestate vs Enum States**:

| Use Typestate When... | Use Enum When... |
|----------------------|------------------|
| State flow is known at compile-time | State changes based on runtime data |
| Want maximum compile-time safety | Need to store mixed states (`Vec<OrderState>`) |
| Building APIs where mistakes are costly | Building flexible workflow engines |
| IDE support is critical | Dynamic state transitions (e.g., config-driven) |
| Zero runtime cost is important | Simplicity is more important than type safety |



**Starter Code**:
```rust
use std::marker::PhantomData;

// State marker types (zero-sized types)
// Role: Compile-time type markers that carry no runtime data
struct Pending;    // Order created, awaiting payment
struct Paid;       // Payment received, awaiting shipment
struct Shipped;    // Order shipped, in transit
struct Delivered;  // Order delivered to customer
struct Cancelled;  // Order cancelled (terminal state)

// Order<State>: Generic order struct parameterized by state
// Role: Holds order data, state encoded in type parameter
struct Order<State> {
    // TODO: Unique order identifier
    // TODO: Customer who placed order
    // TODO: Items in the order
    // TODO: Zero-sized type marker for compile-time state
}

// Pending state implementation
// Role: Methods available only when Order is in Pending state
impl Order<Pending> {
    // new: Creates a new order in Pending state
    // Role: Constructor validating items and initializing order
    fn new(customer_id: u64, items: Vec<Item>) -> Result<Self, String> {
        // TODO: Validate items not empty
        // TODO: Create Order<Pending> with generated id (e.g., customer_id)
        todo!()
    }

    // pay: Transitions from Pending to Paid
    // Role: Processes payment, consumes Order<Pending>, returns Order<Paid>
    fn pay(self, payment_id: String) -> Result<Order<Paid>, String> {
        // TODO: Validate items, calculate total amount
        // TODO: Simulate payment processing
        // TODO: Return Order<Paid> with same id, customer_id, items
        todo!()
    }

    // cancel: Transitions from Pending to Cancelled
    // Role: Cancels order before payment
    fn cancel(self, reason: String) -> Order<Cancelled> {
        todo!()
    }
}

// Paid state implementation
// Role: Methods available only when Order is in Paid state
impl Order<Paid> {
    // ship: Transitions from Paid to Shipped
    // Role: Marks order as shipped with tracking number
    fn ship(self, tracking_number: String) -> Order<Shipped> {
        // Note: In real system, would store tracking_number in Order struct
        // For this exercise, just transition the state
        todo!()
    }

    // cancel: Transitions from Paid to Cancelled
    // Role: Cancels order after payment but before shipping
    fn cancel(self, reason: String) -> Order<Cancelled> {
        todo!()
    }
}

// Shipped state implementation
// Role: Methods available only when Order is in Shipped state
impl Order<Shipped> {
    // deliver: Transitions from Shipped to Delivered
    // Role: Marks order as delivered (terminal state)
    fn deliver(self) -> Order<Delivered> {
        todo!()
    }

    // Note: No cancel method! Can't cancel after shipping - enforced at compile-time
}

// Delivered state implementation (terminal state)
// Role: No state transitions available from Delivered
impl Order<Delivered> {
    // No state transition methods - terminal state
}

// Common methods available in all states
// Role: Generic implementation over any state type
impl<State> Order<State> {
    // id: Returns order ID
    // Role: Accessor available in all states
    fn id(&self) -> u64 {
       todo!()
    }

    // customer_id: Returns customer ID
    // Role: Accessor available in all states
    fn customer_id(&self) -> u64 {
        todo!()
    }

    // items: Returns reference to order items
    // Role: Accessor available in all states
    fn items(&self) -> &[Item] {
       todo!()
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

---

### Complete Working Example

Here's the fully implemented solution for both approaches:

```rust
use std::time::Instant;
use std::marker::PhantomData;

// ============================================================================
// MILESTONE 1 & 2: Enum-Based State Machine
// ============================================================================

#[derive(Debug, Clone)]
struct Item {
    product_id: u64,
    name: String,
    price: f64,
}

#[derive(Debug, Clone)]
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
        delivered_at: Instant,
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

    // State transition: Pending -> Paid
    fn pay(self, payment_id: String) -> Result<Self, String> {
        match self {
            OrderState::Pending { items, customer_id } => {
                if items.is_empty() {
                    return Err("Cannot pay for order with no items".to_string());
                }
                let amount: f64 = items.iter().map(|item| item.price).sum();
                Ok(OrderState::Paid {
                    order_id: customer_id, // Using customer_id as order_id
                    payment_id,
                    amount,
                })
            }
            _ => Err("Can only pay for pending orders".to_string()),
        }
    }

    // State transition: Paid -> Shipped
    fn ship(self, tracking_number: String) -> Result<Self, String> {
        match self {
            OrderState::Paid { order_id, .. } => {
                Ok(OrderState::Shipped {
                    order_id,
                    tracking_number,
                })
            }
            _ => Err("Can only ship paid orders".to_string()),
        }
    }

    // State transition: Shipped -> Delivered
    fn deliver(self) -> Result<Self, String> {
        match self {
            OrderState::Shipped { order_id, .. } => {
                Ok(OrderState::Delivered {
                    order_id,
                    delivered_at: Instant::now(),
                })
            }
            _ => Err("Can only deliver shipped orders".to_string()),
        }
    }

    // State transition: Pending/Paid -> Cancelled
    fn cancel(self, reason: String) -> Result<Self, String> {
        match self {
            OrderState::Pending { customer_id, .. } => {
                Ok(OrderState::Cancelled {
                    order_id: customer_id,
                    reason,
                })
            }
            OrderState::Paid { order_id, .. } => {
                Ok(OrderState::Cancelled { order_id, reason })
            }
            OrderState::Shipped { .. } | OrderState::Delivered { .. } => {
                Err("Cannot cancel after shipping".to_string())
            }
            OrderState::Cancelled { .. } => {
                Err("Order already cancelled".to_string())
            }
        }
    }

    fn can_cancel(&self) -> bool {
        matches!(self, OrderState::Pending { .. } | OrderState::Paid { .. })
    }
}

// ============================================================================
// MILESTONE 3: Typestate Pattern
// ============================================================================

// Zero-sized state marker types
struct Pending;
struct Paid;
struct Shipped;
struct Delivered;
struct Cancelled;

// Generic Order with typestate
struct Order<State> {
    id: u64,
    customer_id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,
}

impl Order<Pending> {
    fn new(customer_id: u64, items: Vec<Item>) -> Result<Self, String> {
        if items.is_empty() {
            return Err("Cannot create order with no items".to_string());
        }
        Ok(Order {
            id: customer_id, // Using customer_id as order id
            customer_id,
            items,
            _state: PhantomData,
        })
    }

    fn pay(self, _payment_id: String) -> Result<Order<Paid>, String> {
        let total: f64 = self.items.iter().map(|item| item.price).sum();
        if total <= 0.0 {
            return Err("Order total must be positive".to_string());
        }
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
    // Note: No cancel method - enforced at compile time!
}

impl Order<Delivered> {
    // Terminal state - no transitions
}

impl Order<Cancelled> {
    // Terminal state - no transitions
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

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    println!("=== Enum-Based State Machine ===\n");

    // Example 1: Complete order flow
    let items = vec![
        Item {
            product_id: 1,
            name: "Rust Book".to_string(),
            price: 39.99,
        },
        Item {
            product_id: 2,
            name: "Mechanical Keyboard".to_string(),
            price: 129.99,
        },
    ];

    let order = OrderState::new_pending(items.clone(), 12345);
    println!("Order status: {}", order.status_string());

    let order = order.pay("PAY_ABC123".to_string()).unwrap();
    println!("Order status after payment: {}", order.status_string());

    let order = order.ship("TRACK_XYZ789".to_string()).unwrap();
    println!("Order status after shipping: {}", order.status_string());

    let order = order.deliver().unwrap();
    println!("Order status after delivery: {}\n", order.status_string());

    // Example 2: Cancellation flow
    let order2 = OrderState::new_pending(items.clone(), 67890);
    let order2 = order2.pay("PAY_DEF456".to_string()).unwrap();
    println!("Can cancel paid order: {}", order2.can_cancel());

    let order2 = order2.cancel("Customer changed mind".to_string()).unwrap();
    println!("Order cancelled: {}\n", order2.status_string());

    // Example 3: Invalid transitions (caught at runtime)
    let order3 = OrderState::new_pending(items.clone(), 11111);
    match order3.ship("INVALID".to_string()) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Error (expected): {}\n", e),
    }

    println!("=== Typestate Pattern (Compile-Time Safety) ===\n");

    // Example 4: Typestate complete flow
    let order = Order::<Pending>::new(22222, items.clone()).unwrap();
    println!("Created order ID: {}", order.id());

    let order = order.pay("PAY_GHI789".to_string()).unwrap();
    println!("Payment processed");

    let order = order.ship("TRACK_JKL012".to_string());
    println!("Order shipped");

    let order = order.deliver();
    println!("Order delivered, ID: {}\n", order.id());

    // Example 5: Typestate cancellation
    let order = Order::<Pending>::new(33333, items.clone()).unwrap();
    let order = order.pay("PAY_MNO345".to_string()).unwrap();
    let order = order.cancel("Refund requested".to_string());
    println!("Order cancelled, ID: {}", order.id());

    // Example 6: These won't compile! (Uncomment to see compile errors)
    // let pending = Order::<Pending>::new(44444, items).unwrap();
    // pending.ship("ERROR".to_string()); // ❌ Compile error: no ship method on Pending
    // pending.deliver(); // ❌ Compile error: no deliver method on Pending

    // let shipped = pending.pay("PAY".to_string()).unwrap().ship("TRACK".to_string());
    // shipped.cancel("Too late".to_string()); // ❌ Compile error: no cancel on Shipped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_state_machine_complete_flow() {
        let items = vec![Item {
            product_id: 1,
            name: "Test".to_string(),
            price: 10.0,
        }];

        let order = OrderState::new_pending(items, 1);
        let order = order.pay("payment".to_string()).unwrap();
        let order = order.ship("tracking".to_string()).unwrap();
        let order = order.deliver().unwrap();

        assert_eq!(order.status_string(), "Delivered");
    }

    #[test]
    fn test_enum_invalid_transitions() {
        let items = vec![Item {
            product_id: 1,
            name: "Test".to_string(),
            price: 10.0,
        }];

        let order = OrderState::new_pending(items, 1);

        // Can't ship before paying
        assert!(order.clone().ship("track".to_string()).is_err());

        let order = order.pay("payment".to_string()).unwrap();
        let order = order.ship("track".to_string()).unwrap();

        // Can't cancel after shipping
        assert!(order.cancel("reason".to_string()).is_err());
    }

    #[test]
    fn test_typestate_complete_flow() {
        let items = vec![Item {
            product_id: 1,
            name: "Test".to_string(),
            price: 10.0,
        }];

        let order = Order::<Pending>::new(1, items).unwrap();
        let order = order.pay("payment".to_string()).unwrap();
        let order = order.ship("tracking".to_string());
        let order = order.deliver();

        assert_eq!(order.id(), 1);
    }

    #[test]
    fn test_typestate_cancellation() {
        let items = vec![Item {
            product_id: 1,
            name: "Test".to_string(),
            price: 10.0,
        }];

        let order = Order::<Pending>::new(1, items.clone()).unwrap();
        let _cancelled = order.cancel("reason".to_string());

        let order = Order::<Pending>::new(1, items).unwrap();
        let order = order.pay("payment".to_string()).unwrap();
        let _cancelled = order.cancel("reason".to_string());

        // Shipped orders can't be cancelled - enforced by type system
    }
}
```

**Key Takeaways from Complete Example**:

**Enum Approach (Runtime)**:
1. **Flexibility**: Single type can represent all states
2. **Dynamic**: Can store `Vec<OrderState>` with mixed states
3. **Runtime checks**: Invalid transitions return `Err` at runtime
4. **Simpler**: One enum definition with pattern matching

**Typestate Approach (Compile-Time)**:
1. **Type safety**: Invalid transitions won't compile
2. **Zero runtime cost**: State in type system, not data
3. **IDE support**: Autocomplete shows only valid methods
4. **More complex**: Multiple types and implementations

**When to Use Each**:
- **Enum**: When state is determined at runtime, need to store mixed states
- **Typestate**: When state flow is known, want maximum compile-time safety

---
