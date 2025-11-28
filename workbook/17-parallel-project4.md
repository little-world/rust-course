# Project: Event-Driven Messaging System - From Observer to Kafka

## Problem Statement

Modern distributed systems rely on event-driven architectures to handle millions of messages per second. This project guides you from basic Observer pattern to building a production-grade distributed messaging system similar to Apache Kafka.

**Real-World Use Cases:**
- **E-commerce**: Order events flow from checkout → inventory → shipping → notifications
- **Financial Systems**: Trade executions → risk analysis → reporting → compliance
- **IoT Platforms**: Sensor data → processing → alerting → storage
- **Microservices**: Service-to-service communication with guaranteed delivery

**Why This Matters:**
- Learn event-driven architecture patterns used by Kafka, RabbitMQ, and cloud messaging services
- Understand trade-offs between simplicity, scalability, and fault tolerance
- Master concurrent data structures and distributed systems concepts
- Build systems that handle high throughput with strong delivery guarantees

## Learning Objectives

By completing this project, you will:
- Implement the Observer pattern and understand its limitations
- Build thread-safe pub-sub systems using Arc/Mutex and channels
- Design topic-based routing with partitioning strategies
- Implement consumer groups for load balancing
- Create persistent logs with offset tracking for durability
- Build distributed systems with replication and leader election

---

## Milestone 1: Sequential Observer Pattern

**Goal:** Implement the classic Observer pattern for event notification.

### Why Start Here?

The Observer pattern is the foundation of all event-driven systems. By starting with a single-threaded implementation, you'll understand:
- The core publish-subscribe mechanism
- How observers register for events
- How events are delivered to subscribers
- The limitations that motivate more sophisticated designs

**Limitations we'll address later:**
- Not thread-safe (can't publish from multiple threads)
- Blocking delivery (slow observers block fast ones)
- No message persistence (events are lost if observer is down)
- No routing (all observers get all events)

### Architecture

```rust
pub struct EventBus {
    observers: Vec<Box<dyn Observer>>,
}

pub trait Observer {
    fn on_event(&self, event: &Event);
}

pub struct Event {
    pub event_type: String,
    pub data: String,
    pub timestamp: u64,
}
```

**Key Concepts:**
1. **EventBus**: Central hub that maintains list of observers
2. **Observer trait**: Interface for receiving events
3. **Event**: Message containing type, data, and metadata
4. **Synchronous delivery**: Events delivered immediately in order

### Your Task

Implement a simple event bus with:

```rust
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub data: String,
    pub timestamp: u64,
}

impl Event {
    pub fn new(event_type: impl Into<String>, data: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Event {
            event_type: event_type.into(),
            data: data.into(),
            timestamp,
        }
    }
}

pub trait Observer: Send + Sync {
    fn on_event(&self, event: &Event);
}

pub struct EventBus {
    // TODO: Store observers
    // Hint: Vec<Box<dyn Observer>>
}

impl EventBus {
    pub fn new() -> Self {
        todo!("Create empty observer list")
    }

    pub fn subscribe(&mut self, observer: Box<dyn Observer>) {
        todo!("Add observer to list")
    }

    pub fn publish(&self, event: Event) {
        todo!("Deliver event to all observers")
    }
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct TestObserver {
        events: Rc<RefCell<Vec<Event>>>,
    }

    impl TestObserver {
        fn new() -> (Self, Rc<RefCell<Vec<Event>>>) {
            let events = Rc::new(RefCell::new(Vec::new()));
            (TestObserver { events: events.clone() }, events)
        }
    }

    impl Observer for TestObserver {
        fn on_event(&self, event: &Event) {
            self.events.borrow_mut().push(event.clone());
        }
    }

    #[test]
    fn test_single_observer() {
        let mut bus = EventBus::new();
        let (observer, events) = TestObserver::new();

        bus.subscribe(Box::new(observer));
        bus.publish(Event::new("test", "data1"));

        assert_eq!(events.borrow().len(), 1);
        assert_eq!(events.borrow()[0].event_type, "test");
    }

    #[test]
    fn test_multiple_observers() {
        let mut bus = EventBus::new();
        let (obs1, events1) = TestObserver::new();
        let (obs2, events2) = TestObserver::new();

        bus.subscribe(Box::new(obs1));
        bus.subscribe(Box::new(obs2));
        bus.publish(Event::new("broadcast", "data"));

        assert_eq!(events1.borrow().len(), 1);
        assert_eq!(events2.borrow().len(), 1);
    }

    #[test]
    fn test_multiple_events() {
        let mut bus = EventBus::new();
        let (observer, events) = TestObserver::new();

        bus.subscribe(Box::new(observer));
        bus.publish(Event::new("event1", "data1"));
        bus.publish(Event::new("event2", "data2"));
        bus.publish(Event::new("event3", "data3"));

        assert_eq!(events.borrow().len(), 3);
    }

    #[test]
    fn test_event_ordering() {
        let mut bus = EventBus::new();
        let (observer, events) = TestObserver::new();

        bus.subscribe(Box::new(observer));

        for i in 0..10 {
            bus.publish(Event::new("seq", format!("{}", i)));
        }

        let captured = events.borrow();
        for i in 0..10 {
            assert_eq!(captured[i].data, format!("{}", i));
        }
    }
}
```

**Expected Output:**
```
Running observer pattern tests...
✓ Single observer receives event
✓ Multiple observers receive same event
✓ Observers receive multiple events
✓ Events delivered in order
```

---

## Milestone 2: Multi-Threaded Observer with Arc/Mutex

**Goal:** Make the event bus thread-safe so multiple threads can publish and subscribe concurrently.

### Why Milestone 1 Isn't Enough

The sequential observer has critical limitations:
1. **Not thread-safe**: Multiple publishers would cause data races
2. **Blocking**: Long-running observers block the publisher
3. **No parallelism**: Can't leverage multiple CPU cores

**Real-world scenario:** An e-commerce platform where:
- Order service publishes order events
- Inventory service publishes stock updates
- Payment service publishes payment confirmations
- All happening concurrently from different threads

### Architecture

```rust
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};

pub struct ThreadSafeEventBus {
    observers: Arc<Mutex<Vec<Sender<Event>>>>,
}
```

**Key Design Decisions:**

1. **Arc<Mutex<Vec>>**: Shared ownership with exclusive access for registration
2. **MPSC Channels**: Non-blocking delivery - publisher sends to channel, observers receive asynchronously
3. **Background threads**: Each observer runs in its own thread

**Performance Characteristics:**
- Publishers: O(n) to send to all channels, but non-blocking
- Observers: Independent processing, no blocking
- Registration: O(1) with mutex contention

### Your Task

Implement a thread-safe event bus:

```rust
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

pub struct ThreadSafeEventBus {
    // TODO: Store senders to observer channels
    // Hint: Arc<Mutex<Vec<Sender<Event>>>>
}

impl ThreadSafeEventBus {
    pub fn new() -> Self {
        todo!("Initialize empty observer list")
    }

    pub fn subscribe<F>(&self, handler: F) -> ObserverHandle
    where
        F: Fn(Event) + Send + 'static,
    {
        todo!("
        1. Create channel (Sender, Receiver)
        2. Spawn thread that receives events and calls handler
        3. Add sender to observers list
        4. Return handle for cleanup
        ")
    }

    pub fn publish(&self, event: Event) {
        todo!("
        1. Lock observers list
        2. Send event to all senders
        3. Remove disconnected observers
        ")
    }
}

pub struct ObserverHandle {
    // TODO: Store thread handle for joining
}

impl Drop for ObserverHandle {
    fn drop(&mut self) {
        // Handle will automatically close channel when dropped
    }
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn test_concurrent_publishers() {
        let bus = Arc::new(ThreadSafeEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _handle = bus.subscribe(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut handles = vec![];
        for i in 0..10 {
            let bus_clone = bus.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    bus_clone.publish(Event::new("test", format!("{}:{}", i, j)));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        thread::sleep(Duration::from_millis(100));
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_multiple_observers_concurrent() {
        let bus = Arc::new(ThreadSafeEventBus::new());

        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        let c1 = counter1.clone();
        let c2 = counter2.clone();

        let _h1 = bus.subscribe(move |_| { c1.fetch_add(1, Ordering::SeqCst); });
        let _h2 = bus.subscribe(move |_| { c2.fetch_add(1, Ordering::SeqCst); });

        for i in 0..50 {
            bus.publish(Event::new("test", format!("{}", i)));
        }

        thread::sleep(Duration::from_millis(100));
        assert_eq!(counter1.load(Ordering::SeqCst), 50);
        assert_eq!(counter2.load(Ordering::SeqCst), 50);
    }

    #[test]
    fn test_observer_isolation() {
        let bus = Arc::new(ThreadSafeEventBus::new());

        let fast_counter = Arc::new(AtomicUsize::new(0));
        let slow_counter = Arc::new(AtomicUsize::new(0));

        let fc = fast_counter.clone();
        let sc = slow_counter.clone();

        let _fast = bus.subscribe(move |_| {
            fc.fetch_add(1, Ordering::SeqCst);
        });

        let _slow = bus.subscribe(move |_| {
            thread::sleep(Duration::from_millis(10));
            sc.fetch_add(1, Ordering::SeqCst);
        });

        let start = std::time::Instant::now();
        for i in 0..10 {
            bus.publish(Event::new("test", format!("{}", i)));
        }
        let duration = start.elapsed();

        // Publishing should be fast (not blocked by slow observer)
        assert!(duration < Duration::from_millis(100));

        thread::sleep(Duration::from_millis(200));
        assert_eq!(fast_counter.load(Ordering::SeqCst), 10);
        assert_eq!(slow_counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_observer_cleanup() {
        let bus = Arc::new(ThreadSafeEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let c = counter.clone();
            let _handle = bus.subscribe(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            });

            bus.publish(Event::new("test", "1"));
            thread::sleep(Duration::from_millis(50));
            // Handle dropped here
        }

        bus.publish(Event::new("test", "2"));
        thread::sleep(Duration::from_millis(50));

        // Should only receive first event
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
```

**Performance Benchmark:**

```rust
#[test]
fn benchmark_threaded_vs_sequential() {
    use std::time::Instant;

    // Sequential
    let mut seq_bus = crate::milestone1::EventBus::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    struct SeqObserver { counter: Arc<AtomicUsize> }
    impl crate::milestone1::Observer for SeqObserver {
        fn on_event(&self, _: &Event) {
            thread::sleep(Duration::from_micros(100));
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    seq_bus.subscribe(Box::new(SeqObserver { counter: c }));

    let start = Instant::now();
    for i in 0..100 {
        seq_bus.publish(Event::new("test", format!("{}", i)));
    }
    let seq_duration = start.elapsed();

    // Threaded
    let thread_bus = Arc::new(ThreadSafeEventBus::new());
    let counter2 = Arc::new(AtomicUsize::new(0));
    let c2 = counter2.clone();

    let _handle = thread_bus.subscribe(move |_| {
        thread::sleep(Duration::from_micros(100));
        c2.fetch_add(1, Ordering::SeqCst);
    });

    let start = Instant::now();
    for i in 0..100 {
        thread_bus.publish(Event::new("test", format!("{}", i)));
    }
    let thread_duration = start.elapsed();

    println!("Sequential: {:?}", seq_duration);
    println!("Threaded:   {:?}", thread_duration);
    println!("Speedup:    {:.2}x", seq_duration.as_secs_f64() / thread_duration.as_secs_f64());

    // Threaded should be much faster (non-blocking)
    assert!(thread_duration < seq_duration / 10);
}
```

**Expected Output:**
```
Sequential: 10.2s (blocking)
Threaded:   15ms (non-blocking)
Speedup:    680x
```

---

## Milestone 3: Topic-Based Message Routing

**Goal:** Add topic-based routing so observers only receive events they're interested in.

### Why Milestone 2 Isn't Enough

The threaded event bus broadcasts all events to all observers:
1. **Bandwidth waste**: Observers receive irrelevant events
2. **CPU waste**: Observers must filter events themselves
3. **No organization**: Can't route events by category

**Real-world scenario:** A microservices platform where:
- `orders.created` → Inventory service
- `payments.completed` → Billing service
- `users.registered` → Email service
- Each service only cares about specific topics

### Architecture

```rust
pub struct TopicEventBus {
    topics: Arc<RwLock<HashMap<String, Vec<Sender<Event>>>>>,
}
```

**Key Design Decisions:**

1. **Topic hierarchy**: Use dot-separated names like `orders.created`, `orders.cancelled`
2. **Pattern matching**: Support wildcards like `orders.*` or `*.error`
3. **RwLock**: Many readers (publishers check topics), few writers (registration)
4. **Per-topic channels**: Observers subscribe to specific topics only

**Routing Strategies:**
- **Exact match**: `orders.created` matches only `orders.created`
- **Wildcard suffix**: `orders.*` matches `orders.created`, `orders.cancelled`
- **Wildcard prefix**: `*.error` matches `payment.error`, `shipping.error`

### Your Task

Implement topic-based routing:

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

pub struct TopicEventBus {
    // TODO: Map from topic name to list of subscribers
    // Hint: Arc<RwLock<HashMap<String, Vec<Sender<Event>>>>>
}

impl TopicEventBus {
    pub fn new() -> Self {
        todo!("Initialize empty topic map")
    }

    pub fn subscribe<F>(&self, topic_pattern: &str, handler: F) -> SubscriptionHandle
    where
        F: Fn(Event) + Send + 'static,
    {
        todo!("
        1. Create channel
        2. Spawn handler thread
        3. Add sender to topic's subscriber list
        4. Return handle
        ")
    }

    pub fn publish(&self, topic: &str, event: Event) {
        todo!("
        1. Read lock topics map
        2. Find matching topics (exact match + wildcards)
        3. Send event to all matching subscribers
        4. Clean up disconnected subscribers
        ")
    }

    fn matches_pattern(pattern: &str, topic: &str) -> bool {
        todo!("
        Implement wildcard matching:
        - 'orders.*' matches 'orders.created', 'orders.cancelled'
        - '*.error' matches 'payment.error', 'shipping.error'
        - '*' matches everything
        - 'orders.created' matches exactly 'orders.created'
        ")
    }
}

pub struct SubscriptionHandle {
    topic: String,
    // TODO: Add sender to signal unsubscribe
}

impl Drop for SubscriptionHandle {
    fn drop(&mut self) {
        // Channel closes automatically
    }
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn test_exact_topic_match() {
        let bus = Arc::new(TopicEventBus::new());

        let orders_count = Arc::new(AtomicUsize::new(0));
        let payments_count = Arc::new(AtomicUsize::new(0));

        let oc = orders_count.clone();
        let pc = payments_count.clone();

        let _h1 = bus.subscribe("orders.created", move |_| {
            oc.fetch_add(1, Ordering::SeqCst);
        });

        let _h2 = bus.subscribe("payments.completed", move |_| {
            pc.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish("orders.created", Event::new("orders.created", "order1"));
        bus.publish("orders.created", Event::new("orders.created", "order2"));
        bus.publish("payments.completed", Event::new("payments.completed", "pay1"));

        thread::sleep(Duration::from_millis(50));

        assert_eq!(orders_count.load(Ordering::SeqCst), 2);
        assert_eq!(payments_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_wildcard_suffix() {
        let bus = Arc::new(TopicEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let _handle = bus.subscribe("orders.*", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish("orders.created", Event::new("orders.created", "1"));
        bus.publish("orders.cancelled", Event::new("orders.cancelled", "2"));
        bus.publish("orders.shipped", Event::new("orders.shipped", "3"));
        bus.publish("payments.completed", Event::new("payments.completed", "4"));

        thread::sleep(Duration::from_millis(50));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_wildcard_prefix() {
        let bus = Arc::new(TopicEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let _handle = bus.subscribe("*.error", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish("payment.error", Event::new("payment.error", "1"));
        bus.publish("shipping.error", Event::new("shipping.error", "2"));
        bus.publish("orders.created", Event::new("orders.created", "3"));

        thread::sleep(Duration::from_millis(50));
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_multiple_patterns_per_subscriber() {
        let bus = Arc::new(TopicEventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        let c1 = counter.clone();
        let c2 = counter.clone();

        let _h1 = bus.subscribe("orders.*", move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        let _h2 = bus.subscribe("*.error", move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish("orders.error", Event::new("orders.error", "1"));

        thread::sleep(Duration::from_millis(50));
        // Should match both patterns
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_topic_isolation() {
        let bus = Arc::new(TopicEventBus::new());

        let slow_count = Arc::new(AtomicUsize::new(0));
        let fast_count = Arc::new(AtomicUsize::new(0));

        let sc = slow_count.clone();
        let fc = fast_count.clone();

        let _slow = bus.subscribe("slow.topic", move |_| {
            thread::sleep(Duration::from_millis(100));
            sc.fetch_add(1, Ordering::SeqCst);
        });

        let _fast = bus.subscribe("fast.topic", move |_| {
            fc.fetch_add(1, Ordering::SeqCst);
        });

        let start = std::time::Instant::now();

        bus.publish("slow.topic", Event::new("slow.topic", "1"));
        for i in 0..100 {
            bus.publish("fast.topic", Event::new("fast.topic", format!("{}", i)));
        }

        let publish_duration = start.elapsed();

        thread::sleep(Duration::from_millis(50));

        // Fast topic should process all events quickly
        assert_eq!(fast_count.load(Ordering::SeqCst), 100);
        // Publishing shouldn't be blocked
        assert!(publish_duration < Duration::from_millis(50));
    }
}
```

**Expected Output:**
```
Topic routing tests:
✓ Exact topic matching works
✓ Wildcard suffix (orders.*) works
✓ Wildcard prefix (*.error) works
✓ Multiple patterns per subscriber
✓ Topics are isolated (slow doesn't block fast)
```

---

## Milestone 4: Partitioned Topics with Consumer Groups

**Goal:** Add partitioning for horizontal scaling and consumer groups for load balancing.

### Why Milestone 3 Isn't Enough

Topic-based routing has scalability limits:
1. **Single consumer bottleneck**: One slow consumer can't keep up with high throughput
2. **No parallelism within topic**: Can't process messages in parallel
3. **No load balancing**: Can't distribute work across multiple instances

**Real-world scenario:** Processing 1M events/sec on `orders.created` topic:
- Single consumer: 1,000 msgs/sec → 1,000 seconds behind
- 10 consumers with partitions: 100,000 msgs/sec each → real-time

### Architecture

```rust
pub struct PartitionedBus {
    topics: Arc<RwLock<HashMap<String, Topic>>>,
}

struct Topic {
    partitions: Vec<Partition>,
    consumer_groups: HashMap<String, ConsumerGroup>,
}

struct Partition {
    id: usize,
    sender: Sender<Event>,
}

struct ConsumerGroup {
    name: String,
    members: Vec<ConsumerMember>,
    // Load balancing strategy
}
```

**Key Concepts:**

1. **Partitions**: Divide topic into N independent queues
2. **Partition Key**: Hash of event key determines partition (e.g., hash(order_id) % N)
3. **Consumer Groups**: Multiple consumers with same group_id share partitions
4. **Load Balancing**: Each partition assigned to one consumer in group

**Guarantees:**
- Messages with same key go to same partition (ordering preserved per key)
- Each partition consumed by at most one consumer per group
- Partitions distributed evenly across consumers

### Your Task

Implement partitioned topics with consumer groups:

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct PartitionedBus {
    // TODO: Map from topic to Topic metadata
}

struct Topic {
    num_partitions: usize,
    partitions: Vec<Partition>,
    consumer_groups: HashMap<String, ConsumerGroup>,
}

struct Partition {
    id: usize,
    sender: Sender<Event>,
}

struct ConsumerGroup {
    name: String,
    consumers: Vec<ConsumerHandle>,
    current_assignment: HashMap<usize, usize>, // partition_id -> consumer_index
}

impl PartitionedBus {
    pub fn new() -> Self {
        todo!("Initialize empty topics map")
    }

    pub fn create_topic(&self, name: &str, num_partitions: usize) {
        todo!("
        1. Create N partition channels
        2. Store in topics map
        ")
    }

    pub fn subscribe<F>(
        &self,
        topic: &str,
        consumer_group: &str,
        handler: F,
    ) -> ConsumerHandle
    where
        F: Fn(Event) + Send + 'static,
    {
        todo!("
        1. Get or create consumer group
        2. Add new consumer to group
        3. Rebalance partitions across all consumers
        4. Connect consumer to assigned partitions
        ")
    }

    pub fn publish(&self, topic: &str, key: &str, event: Event) {
        todo!("
        1. Hash the key
        2. Determine partition: hash % num_partitions
        3. Send to partition's channel
        ")
    }

    fn partition_for_key(&self, key: &str, num_partitions: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % num_partitions
    }

    fn rebalance(&mut self, topic: &str, group: &str) {
        todo!("
        Distribute partitions evenly across consumers:
        - 4 partitions, 2 consumers: [0,1], [2,3]
        - 4 partitions, 3 consumers: [0,1], [2], [3]
        ")
    }
}

pub struct ConsumerHandle {
    // TODO: Handle for cleanup
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use std::collections::HashSet;

    #[test]
    fn test_partition_distribution() {
        let bus = Arc::new(PartitionedBus::new());
        bus.create_topic("orders", 4);

        let mut partition_counts = vec![
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
        ];

        let mut handles = vec![];

        // Each consumer tracks which partition it's reading from
        for i in 0..4 {
            let counter = partition_counts[i].clone();
            let handle = bus.subscribe("orders", &format!("group-{}", i), move |_event| {
                counter.fetch_add(1, Ordering::SeqCst);
            });
            handles.push(handle);
        }

        // Publish 100 events with different keys
        for i in 0..100 {
            bus.publish("orders", &format!("order-{}", i),
                       Event::new("orders", format!("{}", i)));
        }

        thread::sleep(Duration::from_millis(100));

        // Each partition should receive some events
        for count in &partition_counts {
            let val = count.load(Ordering::SeqCst);
            assert!(val > 0, "Partition should receive events");
        }
    }

    #[test]
    fn test_same_key_same_partition() {
        let bus = Arc::new(PartitionedBus::new());
        bus.create_topic("orders", 4);

        let partitions_seen = Arc::new(Mutex::new(HashSet::new()));
        let ps = partitions_seen.clone();

        let _handle = bus.subscribe("orders", "group1", move |event| {
            // Track which partitions we see for this key
            ps.lock().unwrap().insert(event.data.clone());
        });

        // Publish multiple events with same key
        for i in 0..20 {
            bus.publish("orders", "same-key", Event::new("orders", format!("{}", i)));
        }

        thread::sleep(Duration::from_millis(100));

        // All should go to exactly one partition
        // (If we had per-partition tracking, it would be single partition)
    }

    #[test]
    fn test_consumer_group_load_balancing() {
        let bus = Arc::new(PartitionedBus::new());
        bus.create_topic("orders", 8);

        let consumer1_count = Arc::new(AtomicUsize::new(0));
        let consumer2_count = Arc::new(AtomicUsize::new(0));

        let c1 = consumer1_count.clone();
        let c2 = consumer2_count.clone();

        // Two consumers in same group should split partitions
        let _h1 = bus.subscribe("orders", "group1", move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        let _h2 = bus.subscribe("orders", "group1", move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        // Publish to all partitions
        for i in 0..800 {
            bus.publish("orders", &format!("key-{}", i),
                       Event::new("orders", format!("{}", i)));
        }

        thread::sleep(Duration::from_millis(200));

        let count1 = consumer1_count.load(Ordering::SeqCst);
        let count2 = consumer2_count.load(Ordering::SeqCst);

        // Should be roughly balanced (within 20%)
        assert!(count1 > 300 && count1 < 500);
        assert!(count2 > 300 && count2 < 500);
        assert_eq!(count1 + count2, 800);
    }

    #[test]
    fn test_different_consumer_groups_independent() {
        let bus = Arc::new(PartitionedBus::new());
        bus.create_topic("orders", 4);

        let group1_count = Arc::new(AtomicUsize::new(0));
        let group2_count = Arc::new(AtomicUsize::new(0));

        let g1 = group1_count.clone();
        let g2 = group2_count.clone();

        // Different groups should both receive all messages
        let _h1 = bus.subscribe("orders", "analytics", move |_| {
            g1.fetch_add(1, Ordering::SeqCst);
        });

        let _h2 = bus.subscribe("orders", "billing", move |_| {
            g2.fetch_add(1, Ordering::SeqCst);
        });

        for i in 0..100 {
            bus.publish("orders", &format!("key-{}", i),
                       Event::new("orders", format!("{}", i)));
        }

        thread::sleep(Duration::from_millis(100));

        assert_eq!(group1_count.load(Ordering::SeqCst), 100);
        assert_eq!(group2_count.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_rebalancing_on_new_consumer() {
        let bus = Arc::new(PartitionedBus::new());
        bus.create_topic("orders", 4);

        let consumer1_count = Arc::new(AtomicUsize::new(0));
        let c1 = consumer1_count.clone();

        let _h1 = bus.subscribe("orders", "group1", move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        // First consumer gets all partitions
        for i in 0..100 {
            bus.publish("orders", &format!("key-{}", i),
                       Event::new("orders", format!("{}", i)));
        }

        thread::sleep(Duration::from_millis(50));
        assert_eq!(consumer1_count.load(Ordering::SeqCst), 100);

        // Add second consumer - should trigger rebalance
        let consumer2_count = Arc::new(AtomicUsize::new(0));
        let c2 = consumer2_count.clone();

        let _h2 = bus.subscribe("orders", "group1", move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        // New messages should be split
        for i in 100..200 {
            bus.publish("orders", &format!("key-{}", i),
                       Event::new("orders", format!("{}", i)));
        }

        thread::sleep(Duration::from_millis(100));

        // Both should have received some of the new messages
        let total = consumer1_count.load(Ordering::SeqCst) +
                   consumer2_count.load(Ordering::SeqCst);
        assert_eq!(total, 200);
    }
}
```

**Performance Benchmark:**

```rust
#[test]
fn benchmark_partitioned_throughput() {
    let bus = Arc::new(PartitionedBus::new());

    // Test with different partition counts
    for num_partitions in [1, 2, 4, 8, 16] {
        let topic = format!("orders-{}", num_partitions);
        bus.create_topic(&topic, num_partitions);

        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        // One consumer per partition
        for _ in 0..num_partitions {
            let c = counter.clone();
            let h = bus.subscribe(&topic, "group1", move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            });
            handles.push(h);
        }

        let start = std::time::Instant::now();

        // Publish 10,000 events
        for i in 0..10_000 {
            bus.publish(&topic, &format!("key-{}", i),
                       Event::new(&topic, format!("{}", i)));
        }

        // Wait for processing
        while counter.load(Ordering::SeqCst) < 10_000 {
            thread::sleep(Duration::from_millis(10));
        }

        let duration = start.elapsed();
        let throughput = 10_000.0 / duration.as_secs_f64();

        println!("Partitions: {}, Throughput: {:.0} msgs/sec",
                 num_partitions, throughput);
    }
}
```

**Expected Output:**
```
Partitions: 1,  Throughput: 50,000 msgs/sec
Partitions: 2,  Throughput: 95,000 msgs/sec
Partitions: 4,  Throughput: 180,000 msgs/sec
Partitions: 8,  Throughput: 320,000 msgs/sec
Partitions: 16, Throughput: 550,000 msgs/sec
```

---

## Milestone 5: Persistent Log with Offset Tracking

**Goal:** Add durability by persisting messages to disk and tracking consumer offsets.

### Why Milestone 4 Isn't Enough

In-memory channels have critical limitations:
1. **No durability**: Messages lost if process crashes
2. **No replay**: Can't reprocess historical data
3. **No recovery**: Consumer failures lose messages permanently
4. **No time travel**: Can't debug by replaying production events

**Real-world scenario:** Financial trading system:
- Must persist all trades for audit compliance
- Must replay from any point for debugging
- Must guarantee no message loss (at-least-once delivery)

### Architecture

```rust
pub struct PersistentBus {
    topics: Arc<RwLock<HashMap<String, PersistentTopic>>>,
    data_dir: PathBuf,
}

struct PersistentTopic {
    partitions: Vec<PersistentPartition>,
}

struct PersistentPartition {
    id: usize,
    log: CommitLog,
    offsets: HashMap<String, u64>, // consumer_group -> offset
}

struct CommitLog {
    segments: Vec<Segment>,
    active_segment: Segment,
}

struct Segment {
    base_offset: u64,
    file: BufWriter<File>,
}
```

**Key Concepts:**

1. **Commit Log**: Append-only log of all messages in partition
2. **Offset**: Monotonically increasing ID for each message
3. **Consumer Offset**: Last successfully processed offset per consumer group
4. **Segment Files**: Log split into segments for efficient compaction
5. **Checkpointing**: Periodically save consumer offsets to disk

**Storage Layout:**
```
data/
  orders/
    partition-0/
      00000000000000000000.log  # Segment starting at offset 0
      00000000000000001000.log  # Segment starting at offset 1000
      00000000000000002000.log  # Active segment
      consumer-offsets.json     # {group1: 1850, group2: 2000}
    partition-1/
      ...
```

### Your Task

Implement persistent log with offset tracking:

```rust
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistedEvent {
    pub offset: u64,
    pub timestamp: u64,
    pub key: String,
    pub event: Event,
}

pub struct CommitLog {
    dir: PathBuf,
    active_segment: Segment,
    next_offset: u64,
    segment_size_bytes: usize,
}

struct Segment {
    base_offset: u64,
    writer: BufWriter<File>,
    size_bytes: usize,
}

impl CommitLog {
    pub fn open(dir: impl AsRef<Path>) -> std::io::Result<Self> {
        todo!("
        1. Create directory if not exists
        2. Find existing segments
        3. Open or create active segment
        4. Determine next_offset from last segment
        ")
    }

    pub fn append(&mut self, key: &str, event: Event) -> std::io::Result<u64> {
        todo!("
        1. Create PersistedEvent with next_offset
        2. Serialize to JSON + newline
        3. Write to active segment
        4. Flush
        5. If segment full, create new segment
        6. Return offset
        ")
    }

    pub fn read_from(&self, start_offset: u64) -> std::io::Result<Vec<PersistedEvent>> {
        todo!("
        1. Find segment containing start_offset
        2. Open segment file
        3. Skip to start_offset
        4. Read and deserialize events
        5. Continue to next segments if needed
        ")
    }

    fn create_new_segment(&mut self) -> std::io::Result<()> {
        todo!("
        1. Close current segment
        2. Create file: {base_offset:020}.log
        3. Open BufWriter
        ")
    }
}

pub struct OffsetTracker {
    offsets: HashMap<String, u64>,
    checkpoint_file: PathBuf,
}

impl OffsetTracker {
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        todo!("Load offsets from JSON file")
    }

    pub fn commit(&mut self, consumer_group: &str, offset: u64) -> std::io::Result<()> {
        todo!("
        1. Update in-memory offset
        2. Write to checkpoint file (atomic rename)
        ")
    }

    pub fn get_offset(&self, consumer_group: &str) -> u64 {
        self.offsets.get(consumer_group).copied().unwrap_or(0)
    }
}

pub struct PersistentBus {
    topics: Arc<RwLock<HashMap<String, PersistentTopic>>>,
    data_dir: PathBuf,
}

impl PersistentBus {
    pub fn new(data_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        todo!("
        1. Create data directory
        2. Load existing topics from disk
        ")
    }

    pub fn create_topic(&self, name: &str, num_partitions: usize) -> std::io::Result<()> {
        todo!("
        1. Create topic directory
        2. Create partition directories
        3. Initialize commit logs
        4. Initialize offset trackers
        ")
    }

    pub fn publish(&self, topic: &str, key: &str, event: Event) -> std::io::Result<u64> {
        todo!("
        1. Determine partition from key
        2. Append to partition's commit log
        3. Notify consumers
        4. Return offset
        ")
    }

    pub fn subscribe<F>(
        &self,
        topic: &str,
        consumer_group: &str,
        handler: F,
    ) -> std::io::Result<ConsumerHandle>
    where
        F: Fn(PersistedEvent) + Send + 'static,
    {
        todo!("
        1. Load consumer's last offset
        2. Read from offset to end (catch up)
        3. Continue consuming new messages
        4. Periodically commit offset
        ")
    }
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_append_and_read() {
        let dir = TempDir::new().unwrap();
        let mut log = CommitLog::open(dir.path().join("partition-0")).unwrap();

        let offset1 = log.append("key1", Event::new("test", "data1")).unwrap();
        let offset2 = log.append("key2", Event::new("test", "data2")).unwrap();

        assert_eq!(offset1, 0);
        assert_eq!(offset2, 1);

        let events = log.read_from(0).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].offset, 0);
        assert_eq!(events[1].offset, 1);
    }

    #[test]
    fn test_read_from_middle() {
        let dir = TempDir::new().unwrap();
        let mut log = CommitLog::open(dir.path().join("partition-0")).unwrap();

        for i in 0..10 {
            log.append(&format!("key{}", i), Event::new("test", format!("{}", i))).unwrap();
        }

        let events = log.read_from(5).unwrap();
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].offset, 5);
        assert_eq!(events[4].offset, 9);
    }

    #[test]
    fn test_persistence_across_restarts() {
        let dir = TempDir::new().unwrap();
        let log_dir = dir.path().join("partition-0");

        {
            let mut log = CommitLog::open(&log_dir).unwrap();
            log.append("key1", Event::new("test", "data1")).unwrap();
            log.append("key2", Event::new("test", "data2")).unwrap();
        } // Close log

        // Reopen
        let log = CommitLog::open(&log_dir).unwrap();
        let events = log.read_from(0).unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_offset_tracking() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("offsets.json");

        let mut tracker = OffsetTracker::load(&path).unwrap();
        tracker.commit("group1", 100).unwrap();
        tracker.commit("group2", 200).unwrap();

        // Reload
        let tracker2 = OffsetTracker::load(&path).unwrap();
        assert_eq!(tracker2.get_offset("group1"), 100);
        assert_eq!(tracker2.get_offset("group2"), 200);
    }

    #[test]
    fn test_consumer_resume_from_offset() {
        let dir = TempDir::new().unwrap();
        let bus = Arc::new(PersistentBus::new(dir.path()).unwrap());

        bus.create_topic("orders", 1).unwrap();

        // Publish 10 events
        for i in 0..10 {
            bus.publish("orders", &format!("key{}", i),
                       Event::new("orders", format!("{}", i))).unwrap();
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        // Consumer processes first 5
        {
            let _handle = bus.subscribe("orders", "group1", move |event| {
                c.fetch_add(1, Ordering::SeqCst);
                if event.offset == 4 {
                    // Simulate processing up to offset 4
                }
            }).unwrap();

            thread::sleep(Duration::from_millis(100));
        }

        // New consumer in same group should resume from offset 5
        let counter2 = Arc::new(AtomicUsize::new(0));
        let c2 = counter2.clone();

        let _handle = bus.subscribe("orders", "group1", move |event| {
            assert!(event.offset >= 5);
            c2.fetch_add(1, Ordering::SeqCst);
        }).unwrap();

        thread::sleep(Duration::from_millis(100));
        // Should receive remaining 5 events
        assert_eq!(counter2.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_replay_from_beginning() {
        let dir = TempDir::new().unwrap();
        let bus = Arc::new(PersistentBus::new(dir.path()).unwrap());

        bus.create_topic("orders", 1).unwrap();

        for i in 0..100 {
            bus.publish("orders", &format!("key{}", i),
                       Event::new("orders", format!("{}", i))).unwrap();
        }

        // Consumer starts from beginning (offset 0)
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let _handle = bus.subscribe("orders", "replay-group", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        }).unwrap();

        thread::sleep(Duration::from_millis(200));
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_at_least_once_delivery() {
        let dir = TempDir::new().unwrap();
        let bus = Arc::new(PersistentBus::new(dir.path()).unwrap());

        bus.create_topic("orders", 1).unwrap();

        let processed = Arc::new(Mutex::new(Vec::new()));
        let p = processed.clone();

        let _handle = bus.subscribe("orders", "group1", move |event| {
            p.lock().unwrap().push(event.offset);
            // Simulate crash before committing offset
            if event.offset == 5 {
                panic!("Simulated crash");
            }
        }).unwrap();

        for i in 0..10 {
            bus.publish("orders", &format!("key{}", i),
                       Event::new("orders", format!("{}", i))).unwrap();
        }

        thread::sleep(Duration::from_millis(100));

        // After restart, should receive offset 5 again (at-least-once)
    }
}
```

**Performance Benchmark:**

```rust
#[test]
fn benchmark_persistent_vs_memory() {
    use std::time::Instant;

    let dir = TempDir::new().unwrap();
    let persistent_bus = Arc::new(PersistentBus::new(dir.path()).unwrap());
    persistent_bus.create_topic("orders", 4).unwrap();

    // Benchmark persistent
    let start = Instant::now();
    for i in 0..10_000 {
        persistent_bus.publish("orders", &format!("key{}", i),
                              Event::new("orders", format!("{}", i))).unwrap();
    }
    let persistent_duration = start.elapsed();

    // Compare with in-memory from Milestone 4
    let memory_bus = Arc::new(crate::milestone4::PartitionedBus::new());
    memory_bus.create_topic("orders", 4);

    let start = Instant::now();
    for i in 0..10_000 {
        memory_bus.publish("orders", &format!("key{}", i),
                          Event::new("orders", format!("{}", i)));
    }
    let memory_duration = start.elapsed();

    println!("Memory:     {:?} ({:.0} msgs/sec)",
             memory_duration,
             10_000.0 / memory_duration.as_secs_f64());
    println!("Persistent: {:?} ({:.0} msgs/sec)",
             persistent_duration,
             10_000.0 / persistent_duration.as_secs_f64());
    println!("Overhead:   {:.1}x",
             persistent_duration.as_secs_f64() / memory_duration.as_secs_f64());
}
```

**Expected Output:**
```
Memory:     12ms (833,333 msgs/sec)
Persistent: 85ms (117,647 msgs/sec)
Overhead:   7.1x

With batching (10 events/fsync):
Persistent: 25ms (400,000 msgs/sec)
Overhead:   2.1x
```

---

## Milestone 6: Distributed Kafka-Like System with Replication

**Goal:** Build a distributed messaging system with multiple brokers, replication, and leader election.

### Why Milestone 5 Isn't Enough

Single-node persistent storage has limitations:
1. **Single point of failure**: If node dies, system unavailable
2. **Limited throughput**: Bound by single machine's disk/CPU
3. **No fault tolerance**: Hardware failure loses data
4. **No scalability**: Can't add capacity

**Real-world scenario:** Production Kafka cluster:
- 3+ brokers for fault tolerance
- Replication factor 3 (tolerate 2 failures)
- Leader election on failure (sub-second failover)
- Horizontal scaling (add brokers for more throughput)

### Architecture

```rust
pub struct KafkaLikeBus {
    config: ClusterConfig,
    brokers: Arc<RwLock<HashMap<BrokerId, Broker>>>,
    metadata: Arc<RwLock<ClusterMetadata>>,
    coordinator: Arc<Coordinator>,
}

struct ClusterConfig {
    broker_id: BrokerId,
    broker_addresses: HashMap<BrokerId, String>,
    replication_factor: usize,
}

struct ClusterMetadata {
    topics: HashMap<String, TopicMetadata>,
}

struct TopicMetadata {
    partitions: Vec<PartitionMetadata>,
}

struct PartitionMetadata {
    id: usize,
    leader: BrokerId,
    replicas: Vec<BrokerId>,
    isr: Vec<BrokerId>, // In-Sync Replicas
}

struct Coordinator {
    // Leader election and health monitoring
}
```

**Key Concepts:**

1. **Broker**: A server in the cluster (identified by unique ID)
2. **Leader**: Broker responsible for reads/writes to partition
3. **Follower**: Broker that replicates leader's data
4. **ISR (In-Sync Replicas)**: Followers caught up with leader
5. **Replication**: Each partition replicated across N brokers
6. **Leader Election**: Automatic failover when leader dies

**Replication Protocol:**
1. Client sends write to leader
2. Leader appends to local log
3. Leader forwards to all followers
4. Followers append and ACK
5. Leader commits when majority ACK
6. Leader responds to client

**Fault Tolerance:**
- Replication factor 3: Tolerate 2 failures
- ISR tracking: Only count replicas that are caught up
- Automatic failover: Elect new leader from ISR within 1s

### Your Task

Implement distributed messaging with replication:

```rust
use std::collections::{HashMap, HashSet};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock, Mutex};
use std::io::{Read, Write};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

pub type BrokerId = u32;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Message {
    ProduceRequest {
        topic: String,
        partition: usize,
        key: String,
        event: Event,
    },
    ProduceResponse {
        offset: u64,
    },
    ReplicateRequest {
        topic: String,
        partition: usize,
        events: Vec<PersistedEvent>,
    },
    ReplicateAck {
        offset: u64,
    },
    FetchRequest {
        topic: String,
        partition: usize,
        offset: u64,
    },
    FetchResponse {
        events: Vec<PersistedEvent>,
    },
    MetadataRequest {
        topic: String,
    },
    MetadataResponse {
        metadata: TopicMetadata,
    },
    HeartbeatRequest {
        broker_id: BrokerId,
    },
    HeartbeatResponse,
}

pub struct KafkaLikeBus {
    config: ClusterConfig,
    storage: PersistentBus,
    metadata: Arc<RwLock<ClusterMetadata>>,
    connections: Arc<Mutex<HashMap<BrokerId, TcpStream>>>,
}

impl KafkaLikeBus {
    pub fn new(config: ClusterConfig, data_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        todo!("
        1. Initialize storage
        2. Load cluster metadata
        3. Start RPC server
        4. Start health monitor
        ")
    }

    pub fn create_topic(&self, name: &str, num_partitions: usize) -> std::io::Result<()> {
        todo!("
        1. Assign partitions to brokers (round-robin)
        2. Assign replicas (replication_factor copies)
        3. Elect leader for each partition (first replica)
        4. Broadcast metadata to all brokers
        5. Create local storage for partitions we own
        ")
    }

    pub fn publish(&self, topic: &str, key: &str, event: Event) -> std::io::Result<u64> {
        todo!("
        1. Get metadata for topic
        2. Determine partition from key
        3. Find leader broker for partition
        4. If we're leader:
           a. Append to local log
           b. Replicate to followers
           c. Wait for majority ACK
           d. Commit
        5. If not leader:
           a. Forward to leader
           b. Wait for response
        ")
    }

    fn replicate_to_followers(
        &self,
        topic: &str,
        partition: usize,
        events: &[PersistedEvent],
    ) -> std::io::Result<()> {
        todo!("
        1. Get follower broker IDs from metadata
        2. Send ReplicateRequest to each follower
        3. Wait for ACKs with timeout
        4. Update ISR based on responses
        ")
    }

    fn handle_replicate_request(
        &self,
        topic: &str,
        partition: usize,
        events: Vec<PersistedEvent>,
    ) -> std::io::Result<u64> {
        todo!("
        1. Verify we're a replica for this partition
        2. Append events to local log
        3. Return last offset
        ")
    }

    fn start_health_monitor(&self) {
        todo!("
        1. Periodically send heartbeats to all brokers
        2. Detect failed brokers (missed N heartbeats)
        3. Trigger leader election if leader fails
        ")
    }

    fn elect_new_leader(&self, topic: &str, partition: usize) {
        todo!("
        1. Check if we're coordinator (broker with lowest ID)
        2. Select new leader from ISR
        3. Update metadata
        4. Broadcast new metadata
        ")
    }

    pub fn subscribe<F>(
        &self,
        topic: &str,
        consumer_group: &str,
        handler: F,
    ) -> std::io::Result<ConsumerHandle>
    where
        F: Fn(PersistedEvent) + Send + 'static,
    {
        todo!("
        1. Get metadata to find leaders
        2. Connect to leader for each partition
        3. Load consumer offset
        4. Send FetchRequest starting from offset
        5. Process events and commit offsets
        ")
    }
}

struct ClusterMetadata {
    topics: HashMap<String, TopicMetadata>,
    brokers: HashMap<BrokerId, BrokerInfo>,
}

#[derive(Clone)]
struct TopicMetadata {
    partitions: Vec<PartitionMetadata>,
}

#[derive(Clone)]
struct PartitionMetadata {
    id: usize,
    leader: BrokerId,
    replicas: Vec<BrokerId>,
    isr: Vec<BrokerId>,
}

struct BrokerInfo {
    id: BrokerId,
    address: String,
    last_heartbeat: Instant,
}

struct ClusterConfig {
    broker_id: BrokerId,
    listen_address: String,
    broker_addresses: HashMap<BrokerId, String>,
    replication_factor: usize,
    data_dir: PathBuf,
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_three_broker_cluster() {
        let dir = TempDir::new().unwrap();

        let mut brokers = vec![];
        let mut configs = vec![];

        // Create 3 brokers
        for i in 0..3 {
            let config = ClusterConfig {
                broker_id: i,
                listen_address: format!("127.0.0.1:{}", 9092 + i),
                broker_addresses: (0..3)
                    .map(|j| (j, format!("127.0.0.1:{}", 9092 + j)))
                    .collect(),
                replication_factor: 2,
                data_dir: dir.path().join(format!("broker-{}", i)),
            };

            let broker = KafkaLikeBus::new(config.clone(), &config.data_dir).unwrap();
            brokers.push(Arc::new(broker));
            configs.push(config);
        }

        // Create topic on broker 0
        brokers[0].create_topic("orders", 4).unwrap();

        thread::sleep(Duration::from_millis(100)); // Wait for metadata propagation

        // Verify all brokers have metadata
        for broker in &brokers {
            let metadata = broker.metadata.read().unwrap();
            assert!(metadata.topics.contains_key("orders"));
        }
    }

    #[test]
    fn test_replication() {
        let dir = TempDir::new().unwrap();
        let (brokers, _configs) = create_test_cluster(3, &dir);

        brokers[0].create_topic("orders", 1).unwrap();
        thread::sleep(Duration::from_millis(100));

        // Publish to leader
        for i in 0..10 {
            brokers[0].publish("orders", "key", Event::new("orders", format!("{}", i))).unwrap();
        }

        thread::sleep(Duration::from_millis(200));

        // Verify followers have replicated data
        // Find which brokers are replicas
        let metadata = brokers[0].metadata.read().unwrap();
        let partition_meta = &metadata.topics.get("orders").unwrap().partitions[0];

        for &replica_id in &partition_meta.replicas {
            let broker = &brokers[replica_id as usize];
            // Read from local storage
            let events = broker.storage.read_partition("orders", 0, 0).unwrap();
            assert_eq!(events.len(), 10);
        }
    }

    #[test]
    fn test_leader_failover() {
        let dir = TempDir::new().unwrap();
        let (brokers, _configs) = create_test_cluster(3, &dir);

        brokers[0].create_topic("orders", 1).unwrap();
        thread::sleep(Duration::from_millis(100));

        // Find current leader
        let metadata = brokers[0].metadata.read().unwrap();
        let leader_id = metadata.topics.get("orders").unwrap().partitions[0].leader;
        drop(metadata);

        // Kill leader
        drop(brokers[leader_id as usize].clone());

        thread::sleep(Duration::from_millis(500)); // Wait for failure detection + election

        // Check that new leader was elected
        let metadata = brokers[0].metadata.read().unwrap();
        let new_leader = metadata.topics.get("orders").unwrap().partitions[0].leader;

        assert_ne!(new_leader, leader_id);
        assert!(metadata.topics.get("orders").unwrap().partitions[0]
                       .isr.contains(&new_leader));
    }

    #[test]
    fn test_write_after_failover() {
        let dir = TempDir::new().unwrap();
        let (brokers, _configs) = create_test_cluster(3, &dir);

        brokers[0].create_topic("orders", 1).unwrap();
        thread::sleep(Duration::from_millis(100));

        // Write before failover
        for i in 0..5 {
            brokers[0].publish("orders", "key", Event::new("orders", format!("{}", i))).unwrap();
        }

        // Kill leader
        let metadata = brokers[0].metadata.read().unwrap();
        let leader_id = metadata.topics.get("orders").unwrap().partitions[0].leader;
        drop(metadata);
        drop(brokers[leader_id as usize].clone());

        thread::sleep(Duration::from_millis(500));

        // Write after failover (should succeed with new leader)
        for i in 5..10 {
            brokers[0].publish("orders", "key", Event::new("orders", format!("{}", i))).unwrap();
        }

        // Verify all 10 events persisted
        let surviving_broker_id = if leader_id == 0 { 1 } else { 0 };
        let events = brokers[surviving_broker_id].storage
                    .read_partition("orders", 0, 0).unwrap();
        assert_eq!(events.len(), 10);
    }

    #[test]
    fn test_isr_tracking() {
        let dir = TempDir::new().unwrap();
        let (brokers, _configs) = create_test_cluster(3, &dir);

        brokers[0].create_topic("orders", 1).unwrap();
        thread::sleep(Duration::from_millis(100));

        // All replicas should be in ISR initially
        let metadata = brokers[0].metadata.read().unwrap();
        let partition_meta = &metadata.topics.get("orders").unwrap().partitions[0];
        assert_eq!(partition_meta.isr.len(), 2); // replication_factor
        drop(metadata);

        // Kill one follower
        let metadata = brokers[0].metadata.read().unwrap();
        let follower_id = partition_meta.replicas.iter()
            .find(|&&id| id != partition_meta.leader)
            .unwrap();
        drop(metadata);
        drop(brokers[*follower_id as usize].clone());

        thread::sleep(Duration::from_millis(500));

        // ISR should shrink
        let metadata = brokers[0].metadata.read().unwrap();
        let new_isr = &metadata.topics.get("orders").unwrap().partitions[0].isr;
        assert_eq!(new_isr.len(), 1);
        assert!(!new_isr.contains(follower_id));
    }

    fn create_test_cluster(num_brokers: usize, dir: &TempDir)
        -> (Vec<Arc<KafkaLikeBus>>, Vec<ClusterConfig>)
    {
        let mut brokers = vec![];
        let mut configs = vec![];

        for i in 0..num_brokers {
            let config = ClusterConfig {
                broker_id: i as u32,
                listen_address: format!("127.0.0.1:{}", 9092 + i),
                broker_addresses: (0..num_brokers)
                    .map(|j| (j as u32, format!("127.0.0.1:{}", 9092 + j)))
                    .collect(),
                replication_factor: 2,
                data_dir: dir.path().join(format!("broker-{}", i)),
            };

            let broker = KafkaLikeBus::new(config.clone(), &config.data_dir).unwrap();
            brokers.push(Arc::new(broker));
            configs.push(config);
        }

        (brokers, configs)
    }
}
```

**Performance Benchmark:**

```rust
#[test]
fn benchmark_distributed_throughput() {
    let dir = TempDir::new().unwrap();
    let (brokers, _) = create_test_cluster(5, &dir);

    // Create topic with 16 partitions across 5 brokers
    brokers[0].create_topic("orders", 16).unwrap();
    thread::sleep(Duration::from_millis(200));

    let counter = Arc::new(AtomicUsize::new(0));

    // 16 consumers (one per partition)
    let mut handles = vec![];
    for i in 0..16 {
        let broker = brokers[i % 5].clone();
        let c = counter.clone();

        let handle = thread::spawn(move || {
            broker.subscribe("orders", &format!("group-{}", i), move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            }).unwrap();
        });
        handles.push(handle);
    }

    thread::sleep(Duration::from_millis(100));

    // Publish from multiple threads
    let start = std::time::Instant::now();
    let mut publish_handles = vec![];

    for i in 0..5 {
        let broker = brokers[i].clone();
        let handle = thread::spawn(move || {
            for j in 0..10_000 {
                broker.publish("orders", &format!("key-{}", j),
                              Event::new("orders", format!("{}", j))).unwrap();
            }
        });
        publish_handles.push(handle);
    }

    for handle in publish_handles {
        handle.join().unwrap();
    }

    let publish_duration = start.elapsed();

    // Wait for consumption
    while counter.load(Ordering::SeqCst) < 50_000 {
        thread::sleep(Duration::from_millis(100));
    }

    let total_duration = start.elapsed();

    println!("Published 50,000 events in {:?}", publish_duration);
    println!("Throughput: {:.0} msgs/sec",
             50_000.0 / publish_duration.as_secs_f64());
    println!("End-to-end: {:?}", total_duration);
}
```

**Expected Output:**
```
Cluster Performance (5 brokers, 16 partitions, replication=2):
Published 50,000 events in 1.2s
Throughput: 41,667 msgs/sec
End-to-end: 1.5s

With replication overhead vs single-node:
Single-node: 400,000 msgs/sec
Distributed (r=1): 250,000 msgs/sec (0.6x)
Distributed (r=2): 150,000 msgs/sec (0.4x)
Distributed (r=3): 100,000 msgs/sec (0.25x)

But: Fault tolerance + horizontal scaling
```

---

## Complete Working Example

Here's a production-quality implementation demonstrating the full Kafka-like system:

```rust
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// ============================================================================
// Core Event Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: String,
    pub data: String,
    pub timestamp: u64,
}

impl Event {
    pub fn new(event_type: impl Into<String>, data: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Event {
            event_type: event_type.into(),
            data: data.into(),
            timestamp,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistedEvent {
    pub offset: u64,
    pub timestamp: u64,
    pub key: String,
    pub event: Event,
}

// ============================================================================
// Commit Log (Milestone 5)
// ============================================================================

pub struct CommitLog {
    dir: PathBuf,
    writer: BufWriter<File>,
    next_offset: u64,
}

impl CommitLog {
    pub fn open(dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let dir = dir.as_ref();
        create_dir_all(dir)?;

        let log_path = dir.join("events.log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        // Count existing events to determine next_offset
        let reader_file = OpenOptions::new().read(true).open(&log_path)?;
        let reader = BufReader::new(reader_file);
        let next_offset = reader.lines().count() as u64;

        Ok(CommitLog {
            dir: dir.to_path_buf(),
            writer: BufWriter::new(file),
            next_offset,
        })
    }

    pub fn append(&mut self, key: &str, event: Event) -> std::io::Result<u64> {
        let persisted = PersistedEvent {
            offset: self.next_offset,
            timestamp: event.timestamp,
            key: key.to_string(),
            event,
        };

        let json = serde_json::to_string(&persisted)?;
        writeln!(self.writer, "{}", json)?;
        self.writer.flush()?;

        let offset = self.next_offset;
        self.next_offset += 1;
        Ok(offset)
    }

    pub fn read_from(&self, start_offset: u64) -> std::io::Result<Vec<PersistedEvent>> {
        let log_path = self.dir.join("events.log");
        let file = OpenOptions::new().read(true).open(log_path)?;
        let reader = BufReader::new(file);

        let events: Vec<PersistedEvent> = reader
            .lines()
            .skip(start_offset as usize)
            .filter_map(|line| {
                line.ok()
                    .and_then(|l| serde_json::from_str(&l).ok())
            })
            .collect();

        Ok(events)
    }
}

// ============================================================================
// Offset Tracking
// ============================================================================

pub struct OffsetTracker {
    offsets: HashMap<String, u64>,
    path: PathBuf,
}

impl OffsetTracker {
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();

        let offsets = if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(OffsetTracker { offsets, path })
    }

    pub fn commit(&mut self, consumer_group: &str, offset: u64) -> std::io::Result<()> {
        self.offsets.insert(consumer_group.to_string(), offset);
        let json = serde_json::to_string(&self.offsets)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }

    pub fn get_offset(&self, consumer_group: &str) -> u64 {
        self.offsets.get(consumer_group).copied().unwrap_or(0)
    }
}

// ============================================================================
// Topic with Partitions
// ============================================================================

struct Partition {
    id: usize,
    log: Mutex<CommitLog>,
    offset_tracker: Mutex<OffsetTracker>,
    sender: Sender<PersistedEvent>,
}

pub struct Topic {
    name: String,
    partitions: Vec<Arc<Partition>>,
}

impl Topic {
    pub fn new(name: String, num_partitions: usize, data_dir: &Path) -> std::io::Result<Self> {
        let mut partitions = Vec::new();

        for i in 0..num_partitions {
            let partition_dir = data_dir.join(&name).join(format!("partition-{}", i));
            create_dir_all(&partition_dir)?;

            let log = CommitLog::open(&partition_dir)?;
            let offset_tracker = OffsetTracker::load(partition_dir.join("offsets.json"))?;

            let (sender, _receiver) = channel();

            partitions.push(Arc::new(Partition {
                id: i,
                log: Mutex::new(log),
                offset_tracker: Mutex::new(offset_tracker),
                sender,
            }));
        }

        Ok(Topic {
            name,
            partitions,
        })
    }

    pub fn publish(&self, key: &str, event: Event) -> std::io::Result<u64> {
        let partition_id = self.partition_for_key(key);
        let partition = &self.partitions[partition_id];

        let offset = partition.log.lock().unwrap().append(key, event.clone())?;

        // Notify consumers
        let _ = partition.sender.send(PersistedEvent {
            offset,
            timestamp: event.timestamp,
            key: key.to_string(),
            event,
        });

        Ok(offset)
    }

    pub fn subscribe<F>(
        &self,
        partition_id: usize,
        consumer_group: &str,
        handler: F,
    ) -> std::io::Result<()>
    where
        F: Fn(PersistedEvent) + Send + 'static,
    {
        let partition = self.partitions[partition_id].clone();
        let consumer_group = consumer_group.to_string();

        thread::spawn(move || {
            // Catch up on old events
            let start_offset = partition.offset_tracker.lock().unwrap()
                .get_offset(&consumer_group);

            if let Ok(events) = partition.log.lock().unwrap().read_from(start_offset) {
                for event in events {
                    handler(event.clone());
                    let _ = partition.offset_tracker.lock().unwrap()
                        .commit(&consumer_group, event.offset + 1);
                }
            }
        });

        Ok(())
    }

    fn partition_for_key(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.partitions.len()
    }
}

// ============================================================================
// Full Kafka-Like Bus
// ============================================================================

pub struct MessageBus {
    data_dir: PathBuf,
    topics: Arc<RwLock<HashMap<String, Arc<Topic>>>>,
}

impl MessageBus {
    pub fn new(data_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        create_dir_all(&data_dir)?;

        Ok(MessageBus {
            data_dir,
            topics: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn create_topic(&self, name: &str, num_partitions: usize) -> std::io::Result<()> {
        let topic = Topic::new(name.to_string(), num_partitions, &self.data_dir)?;
        self.topics.write().unwrap()
            .insert(name.to_string(), Arc::new(topic));
        Ok(())
    }

    pub fn publish(&self, topic: &str, key: &str, event: Event) -> std::io::Result<u64> {
        let topics = self.topics.read().unwrap();
        let topic_ref = topics.get(topic)
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Topic not found"
            ))?;

        topic_ref.publish(key, event)
    }

    pub fn subscribe<F>(
        &self,
        topic: &str,
        consumer_group: &str,
        handler: F,
    ) -> std::io::Result<()>
    where
        F: Fn(PersistedEvent) + Send + 'static + Clone,
    {
        let topics = self.topics.read().unwrap();
        let topic_ref = topics.get(topic)
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Topic not found"
            ))?
            .clone();

        // Subscribe to all partitions
        for i in 0..topic_ref.partitions.len() {
            let h = handler.clone();
            topic_ref.subscribe(i, consumer_group, h)?;
        }

        Ok(())
    }
}

// ============================================================================
// Example Usage
// ============================================================================

fn main() -> std::io::Result<()> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let bus = Arc::new(MessageBus::new("./kafka-data")?);

    // Create topic with 4 partitions
    bus.create_topic("orders", 4)?;

    // Consumer 1: Count all events
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    bus.subscribe("orders", "analytics", move |event| {
        c.fetch_add(1, Ordering::SeqCst);
        println!("Analytics: offset={} key={} data={}",
                 event.offset, event.key, event.event.data);
    })?;

    // Consumer 2: Process specific keys
    bus.subscribe("orders", "processor", move |event| {
        if event.key.starts_with("priority") {
            println!("Priority order: {}", event.event.data);
        }
    })?;

    // Publish events
    for i in 0..20 {
        let key = if i % 5 == 0 {
            format!("priority-{}", i)
        } else {
            format!("regular-{}", i)
        };

        let offset = bus.publish("orders", &key,
            Event::new("order.created", format!("Order #{}", i)))?;

        println!("Published: key={} offset={}", key, offset);
    }

    thread::sleep(Duration::from_secs(2));

    println!("\nTotal events processed: {}", counter.load(Ordering::SeqCst));

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_full_system() {
        let dir = TempDir::new().unwrap();
        let bus = Arc::new(MessageBus::new(dir.path()).unwrap());

        bus.create_topic("test", 2).unwrap();

        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        bus.subscribe("test", "group1", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        }).unwrap();

        for i in 0..100 {
            bus.publish("test", &format!("key{}", i),
                       Event::new("test", format!("{}", i))).unwrap();
        }

        thread::sleep(Duration::from_millis(200));
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }
}
```

---

## Summary

Congratulations! You've built a complete Kafka-like distributed messaging system from scratch.

### What You Built

1. **Observer Pattern**: Foundation of event-driven systems
2. **Thread-Safe Pub-Sub**: Non-blocking concurrent event delivery
3. **Topic-Based Routing**: Wildcard pattern matching for selective consumption
4. **Partitioned Topics**: Horizontal scaling with consumer group load balancing
5. **Persistent Log**: Durable storage with offset tracking and replay
6. **Distributed System**: Multi-broker cluster with replication and failover

### Key Concepts Mastered

- **Event-driven architecture**: Decoupling producers from consumers
- **Partitioning**: Distributing load across multiple consumers
- **Replication**: Fault tolerance through data redundancy
- **Leader election**: Automatic failover for high availability
- **Offset management**: Exactly-once and at-least-once semantics
- **Consumer groups**: Load balancing and parallel processing

### Performance Characteristics

| Milestone | Throughput | Latency | Durability | Fault Tolerance |
|-----------|-----------|---------|------------|-----------------|
| 1. Observer | 100K msg/s | <1ms | None | None |
| 2. Threaded | 500K msg/s | <2ms | None | None |
| 3. Topics | 400K msg/s | <2ms | None | None |
| 4. Partitions | 1M+ msg/s | <5ms | None | None |
| 5. Persistent | 150K msg/s | <10ms | Full | None |
| 6. Distributed | 300K msg/s | <20ms | Full | RF-1 failures |

### Real-World Applications

Your implementation mirrors production systems:
- **Apache Kafka**: Distributed log for LinkedIn, Uber, Netflix
- **AWS Kinesis**: Real-time data streaming for AWS services
- **Google Pub/Sub**: Global messaging for Cloud Platform
- **RabbitMQ/NATS**: Lightweight messaging for microservices

### Next Steps

1. **Exactly-once semantics**: Idempotent producers + transactional consumers
2. **Log compaction**: Keep only latest value per key
3. **Stream processing**: Aggregations, joins, windowing
4. **Schema registry**: Versioned message formats
5. **Monitoring**: Metrics, tracing, alerting
6. **Multi-datacenter**: Geo-replication and disaster recovery

You now understand the core principles behind every modern messaging system!
