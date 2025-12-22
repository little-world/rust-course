# Multi-Peripheral Interrupt Coordinator

### Problem Statement

Build an interrupt coordinator that safely manages shared state across multiple interrupt sources (UART, timers, GPIO, DMA) with different priorities. Your coordinator must prevent data races, avoid priority inversion, ensure bounded interrupt latency, and provide safe abstractions for interrupt-safe communication between ISRs and main tasks.

Your coordinator should support:
- Safe shared state with critical sections
- Priority-based interrupt management
- Lock-free communication patterns (atomics, lock-free queues)
- Interrupt statistics and profiling
- Deadline monitoring and timeout detection
- Integration with RTIC or Embassy for deterministic scheduling

## Why Interrupt Coordination Matters

### The Shared State Problem

**The Problem**: Multiple interrupts need to access shared data, but traditional locks cause deadlocks or priority inversion in interrupt contexts.

```rust
// ❌ DANGEROUS: Mutex in interrupt context
static COUNTER: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn TIMER_IRQ() {
    COUNTER.lock().unwrap().replace_with(|c| *c + 1); // Deadlock!
}

#[interrupt]
fn UART_IRQ() {
    let count = *COUNTER.lock().unwrap().borrow(); // Can't wait for lock!
}

// Problem: If TIMER_IRQ is interrupted by UART_IRQ,
// UART_IRQ tries to lock already-locked mutex → DEADLOCK
```

**Real-world disaster:**
```
Industrial controller:
├─ UART receives command (low priority interrupt)
├─ Takes mutex to update state
├─ Higher priority TIMER interrupt fires
├─ TIMER needs same mutex → spins forever
└─ System hangs, production line stops

Cost: $50,000/hour downtime
```

### Critical Sections: Safe Interrupt-Main Communication

**The Solution**: Disable interrupts only around minimal critical sections:

```rust
use cortex_m::interrupt::{free, Mutex};
use core::cell::RefCell;

static SHARED: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn TIMER() {
    free(|cs| {
        let mut val = SHARED.borrow(cs).borrow_mut();
        *val += 1; // Interrupts disabled here
    }); // Interrupts re-enabled
}

fn main_task() {
    let value = free(|cs| *SHARED.borrow(cs).borrow());
    println!("Count: {}", value);
}
```

**Key principle**: Critical section duration must be bounded and minimal.

### Priority Inversion: The Hidden Danger

**Priority Inversion scenario:**
```
Interrupt priorities (higher number = higher priority):
├─ TIMER: Priority 3 (highest)
├─ DMA: Priority 2
└─ UART: Priority 1 (lowest)

Timeline:
1. UART (priority 1) holds lock
2. TIMER (priority 3) preempts, needs lock → BLOCKED
3. DMA (priority 2) preempts UART
4. TIMER waits for UART (priority 1) to finish!

Result: High-priority interrupt blocked by medium-priority work
```

**Impact:**
```
Motor control system:
├─ Control loop: 1ms deadline (priority 3)
├─ Logging: 100ms deadline (priority 2)
├─ Bluetooth: 1s deadline (priority 1)

Priority inversion:
├─ Bluetooth holds shared state
├─ Control loop needs state → misses deadline
└─ Motor spins out of control → physical damage

Prevention: Use lock-free patterns or priority ceiling protocol
```

### Lock-Free Patterns: Zero-Wait Communication

**Atomic operations:**
```rust
use core::sync::atomic::{AtomicU32, Ordering};

static FLAGS: AtomicU32 = AtomicU32::new(0);

#[interrupt]
fn UART_RX() {
    FLAGS.fetch_or(1 << 0, Ordering::Release); // Set bit 0
}

#[interrupt]
fn TIMER() {
    FLAGS.fetch_or(1 << 1, Ordering::Release); // Set bit 1
}

fn main_loop() {
    let flags = FLAGS.swap(0, Ordering::AcqRel); // Atomic read-clear
    if flags & (1 << 0) != 0 {
        handle_uart();
    }
    if flags & (1 << 1) != 0 {
        handle_timer();
    }
}
```

**Performance:**
```
Critical section approach:
├─ Interrupt latency: 50-200 cycles (disable/enable interrupts)
├─ Jitter: Variable (depends on critical section length)

Lock-free atomic approach:
├─ Interrupt latency: 2-5 cycles (single atomic instruction)
├─ Jitter: Minimal (constant time)

10-100x faster!
```

## Use Cases

### 1. Real-Time Control Systems
- **Motor controllers**: Position feedback (timer), command input (CAN), safety limits (GPIO)
- **Robotics**: Sensor fusion from multiple interrupt sources
- **Industrial PLCs**: Coordinating I/O modules with strict timing
- **Challenge**: Deterministic response within microseconds

### 2. Communication Gateways
- **Multi-protocol bridges**: UART ↔ SPI ↔ I2C ↔ CAN
- **IoT gateways**: WiFi + LoRa + Cellular with shared packet buffer
- **Protocol converters**: Real-time translation without data loss
- **Challenge**: Handle burst traffic without blocking

### 3. Safety-Critical Systems
- **Medical devices**: Monitor multiple vitals (ECG, SpO2, pressure)
- **Automotive**: ADAS sensor fusion with fallback paths
- **Aviation**: Flight control with redundant sensor validation
- **Challenge**: Guaranteed response time for all conditions

### 4. Battery-Powered Systems
- **Low-power logging**: Wake on any interrupt, process, sleep
- **Sensor networks**: Coordinate radio, sensors, timers for efficiency
- **Wearables**: Balance responsiveness with power consumption
- **Challenge**: Fast wake-up, minimal interrupt overhead

---

## Building the Project

### Milestone 1: Safe Shared State with Critical Sections

**Goal**: Implement safe shared state primitives that work correctly in both interrupt and non-interrupt contexts using critical sections.

**Why we start here**: Before coordinating interrupts, we need thread-safe primitives. This milestone teaches the foundation: how to safely share data between ISRs and main code.

#### Architecture

**Structs:**
- `InterruptSafeCell<T>` - Interior mutability for interrupt contexts
  - **Field**: `data: UnsafeCell<T>` - Inner data storage
  - **Uses**: `cortex_m::interrupt::Mutex` wrapper

- `SharedCounter` - Example shared state
  - **Field**: `count: Mutex<RefCell<u32>>` - Protected counter
  - **Field**: `overflows: AtomicU32` - Overflow count (lock-free)

**Functions:**
- `SharedCounter::new() -> Self` - Create counter
- `increment(&self)` - Increment from interrupt or main
- `get(&self) -> u32` - Read current value
- `reset(&self)` - Reset to zero
- `overflow_count(&self) -> u32` - Get overflow count

**Starter Code**:

```rust
use core::cell::{RefCell, UnsafeCell};
use cortex_m::interrupt::{free, Mutex};
use core::sync::atomic::{AtomicU32, Ordering};

/// Safe wrapper for data shared between interrupts and main code
pub struct SharedCounter {
    count: Mutex<RefCell<u32>>,
    overflows: AtomicU32,
}

impl SharedCounter {
    pub const fn new() -> Self {
        // TODO: Initialize with Mutex wrapping RefCell
        todo!("Implement SharedCounter::new")
    }

    /// Increment counter (safe to call from interrupt or main)
    pub fn increment(&self) {
        // TODO: Use free() to create critical section
        // TODO: Borrow mutable reference
        // TODO: Increment, checking for overflow
        // TODO: If overflow, increment overflows atomically
        todo!("Implement increment")
    }

    /// Read current value
    pub fn get(&self) -> u32 {
        // TODO: Use critical section to safely read
        todo!("Implement get")
    }

    /// Reset counter to zero
    pub fn reset(&self) {
        // TODO: Use critical section to reset
        todo!("Implement reset")
    }

    /// Get overflow count
    pub fn overflow_count(&self) -> u32 {
        self.overflows.load(Ordering::Relaxed)
    }
}

// Make it safe to share across threads/interrupts
unsafe impl Sync for SharedCounter {}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_basic() {
        let counter = SharedCounter::new();
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);

        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 3);

        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_overflow_detection() {
        let counter = SharedCounter::new();

        // Manually set to max - 1
        cortex_m::interrupt::free(|cs| {
            *counter.count.borrow(cs).borrow_mut() = u32::MAX - 1;
        });

        counter.increment(); // Should not overflow
        assert_eq!(counter.get(), u32::MAX);
        assert_eq!(counter.overflow_count(), 0);

        counter.increment(); // Should detect overflow
        assert_eq!(counter.get(), 0);
        assert_eq!(counter.overflow_count(), 1);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let counter = Arc::new(SharedCounter::new());
        let mut handles = vec![];

        // Spawn 10 threads, each incrementing 1000 times
        for _ in 0..10 {
            let counter_clone = counter.clone();
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    counter_clone.increment();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get(), 10_000);
    }
}
```

**Check Your Understanding**:
- Why use `Mutex<RefCell<T>>` instead of just `RefCell<T>`?
- What's the difference between `cortex_m::interrupt::free` and `critical_section::with`?
- Why use `AtomicU32` for overflows instead of including it in the Mutex?

---

#### Why Milestone 1 Isn't Enough

**Limitation**: Critical sections work but they disable ALL interrupts. This causes high-priority interrupts to be delayed unnecessarily.

**What we're adding**: Priority-aware locking that only disables lower-priority interrupts, allowing critical work to continue.

**Improvement**:
- **Latency**: High-priority interrupts not delayed by low-priority work
- **Throughput**: More concurrent interrupt handling
- **Predictability**: Each interrupt class has bounded latency
- **Safety**: Still prevent races, but with finer granularity

---

### Milestone 2: Priority-Based Interrupt Management

**Goal**: Implement priority-aware interrupt coordination where high-priority interrupts can preempt low-priority ones, with safe state access based on priority levels.

**Why this milestone**: Real systems have interrupt hierarchies. This teaches NVIC priority configuration and priority-ceiling protocols.

#### Architecture

**Structs:**
- `InterruptPriority` - Priority levels
  - **Variant**: `Critical = 0` - Highest priority (never masked)
  - **Variant**: `High = 4` - Important interrupts
  - **Variant**: `Medium = 8` - Normal interrupts
  - **Variant**: `Low = 12` - Background interrupts

- `PriorityGroup<T>` - Data with priority-based access
  - **Field**: `data: Mutex<RefCell<T>>` - Protected data
  - **Field**: `priority: InterruptPriority` - Minimum priority to access

- `InterruptStats` - Per-interrupt statistics
  - **Field**: `count: AtomicU32` - Invocation count
  - **Field**: `max_duration_us: AtomicU32` - Longest execution time
  - **Field**: `preemption_count: AtomicU32` - Times preempted

**Functions:**
- `configure_interrupt_priority(irq: Interrupt, priority: InterruptPriority)` - Set NVIC priority
- `PriorityGroup::new(data: T, priority: InterruptPriority) -> Self` - Create protected data
- `access<F, R>(&self, f: F) -> R` - Access data if caller priority is sufficient
- `InterruptStats::record_execution(duration_us: u32)` - Update statistics

**Starter Code**:

```rust
use cortex_m::peripheral::NVIC;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InterruptPriority {
    Critical = 0,  // Highest - never preempted
    High = 4,
    Medium = 8,
    Low = 12,      // Lowest - can be preempted by all others
}

impl InterruptPriority {
    /// Convert to NVIC priority value
    pub fn to_nvic_priority(self) -> u8 {
        self as u8
    }
}

pub struct PriorityGroup<T> {
    data: Mutex<RefCell<T>>,
    priority: InterruptPriority,
}

impl<T> PriorityGroup<T> {
    pub const fn new(data: T, priority: InterruptPriority) -> Self {
        // TODO: Initialize with Mutex and priority
        todo!("Implement PriorityGroup::new")
    }

    /// Access data if caller has sufficient priority
    pub fn access<F, R>(&self, current_priority: InterruptPriority, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&mut T) -> R,
    {
        // TODO: Check if current_priority >= self.priority
        // TODO: If yes, access data in critical section
        // TODO: If no, return Err(AccessError::InsufficientPriority)
        todo!("Implement access")
    }

    pub fn priority(&self) -> InterruptPriority {
        self.priority
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessError {
    InsufficientPriority,
}

#[derive(Debug)]
pub struct InterruptStats {
    pub count: AtomicU32,
    pub max_duration_us: AtomicU32,
    pub preemption_count: AtomicU32,
}

impl InterruptStats {
    pub const fn new() -> Self {
        Self {
            count: AtomicU32::new(0),
            max_duration_us: AtomicU32::new(0),
            preemption_count: AtomicU32::new(0),
        }
    }

    pub fn record_execution(&self, duration_us: u32) {
        // TODO: Increment count
        // TODO: Update max_duration if current duration is larger
        // HINT: Use compare_exchange in loop for max update
        todo!("Implement record_execution")
    }

    pub fn record_preemption(&self) {
        self.preemption_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> StatSnapshot {
        StatSnapshot {
            count: self.count.load(Ordering::Relaxed),
            max_duration_us: self.max_duration_us.load(Ordering::Relaxed),
            preemption_count: self.preemption_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatSnapshot {
    pub count: u32,
    pub max_duration_us: u32,
    pub preemption_count: u32,
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_levels() {
        assert!(InterruptPriority::Critical < InterruptPriority::High);
        assert!(InterruptPriority::High < InterruptPriority::Medium);
        assert!(InterruptPriority::Medium < InterruptPriority::Low);
    }

    #[test]
    fn test_priority_group_access() {
        let group = PriorityGroup::new(42u32, InterruptPriority::Medium);

        // High priority can access medium priority data
        let result = group.access(InterruptPriority::High, |data| *data);
        assert_eq!(result, Ok(42));

        // Low priority cannot access medium priority data
        let result = group.access(InterruptPriority::Low, |data| *data);
        assert_eq!(result, Err(AccessError::InsufficientPriority));

        // Same priority can access
        let result = group.access(InterruptPriority::Medium, |data| {
            *data += 1;
            *data
        });
        assert_eq!(result, Ok(43));
    }

    #[test]
    fn test_interrupt_stats() {
        let stats = InterruptStats::new();

        stats.record_execution(100);
        stats.record_execution(150);
        stats.record_execution(120);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.count, 3);
        assert_eq!(snapshot.max_duration_us, 150);

        stats.record_preemption();
        stats.record_preemption();
        let snapshot = stats.snapshot();
        assert_eq!(snapshot.preemption_count, 2);
    }

    #[test]
    fn test_max_duration_update() {
        let stats = InterruptStats::new();

        // Record multiple durations
        for duration in [50, 100, 75, 200, 120] {
            stats.record_execution(duration);
        }

        assert_eq!(stats.max_duration_us.load(Ordering::Relaxed), 200);
        assert_eq!(stats.count.load(Ordering::Relaxed), 5);
    }
}
```

**Check Your Understanding**:
- How does priority-based access prevent priority inversion?
- Why use atomic compare-exchange for max duration update?
- What's the trade-off between more priority levels vs. fewer?

---

#### Why Milestone 2 Isn't Enough

**Limitation**: We can protect individual pieces of data, but real applications need to coordinate multiple interrupts with complex event flows.

**What we're adding**: An interrupt event dispatcher that routes events from various interrupt sources to handlers, with queuing and filtering.

**Improvement**:
- **Architecture**: Clean separation of interrupt handling from business logic
- **Flexibility**: Dynamic event routing and filtering
- **Testability**: Can inject events without hardware
- **Debugging**: Central point to monitor all interrupt activity

---

### Milestone 3: Event Dispatcher with Lock-Free Queues

**Goal**: Build an event dispatcher that receives events from multiple interrupt sources and routes them to handlers using lock-free queues.

**Why this milestone**: Decoupling interrupt sources from handlers improves system architecture. This teaches lock-free SPSC queues and event-driven design.

#### Architecture

**Structs:**
- `InterruptEvent` - Tagged event from interrupt source
  - **Field**: `source: EventSource` - Which peripheral generated event
  - **Field**: `data: u32` - Event-specific data
  - **Field**: `timestamp: u64` - When event occurred (microseconds)

- `EventSource` - Event origin
  - **Variant**: `Timer1`, `Timer2`, `Uart`, `Gpio(u8)`, `Dma(u8)`

- `EventDispatcher<const N: usize>` - Central event router
  - **Field**: `queue: SpscQueue<InterruptEvent, N>` - Event queue
  - **Field**: `stats: [InterruptStats; NUM_SOURCES]` - Per-source stats

**Functions:**
- `push_event(&self, event: InterruptEvent) -> Result<(), InterruptEvent>` - Add event from ISR
- `pop_event(&self) -> Option<InterruptEvent>` - Get event for processing
- `dispatch_all<F>(&self, handler: F)` - Process all pending events
- `get_source_stats(&self, source: EventSource) -> StatSnapshot` - Get statistics

**Starter Code**:

```rust
use heapless::spsc::Queue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSource {
    Timer1,
    Timer2,
    Uart,
    Gpio(u8),
    Dma(u8),
}

impl EventSource {
    fn to_index(&self) -> usize {
        match self {
            EventSource::Timer1 => 0,
            EventSource::Timer2 => 1,
            EventSource::Uart => 2,
            EventSource::Gpio(pin) => 3 + (*pin as usize % 8),
            EventSource::Dma(ch) => 11 + (*ch as usize % 8),
        }
    }
}

const NUM_SOURCES: usize = 20; // Enough for all variants

#[derive(Debug, Clone, Copy)]
pub struct InterruptEvent {
    pub source: EventSource,
    pub data: u32,
    pub timestamp: u64,
}

pub struct EventDispatcher<const N: usize> {
    queue: Mutex<RefCell<Queue<InterruptEvent, N>>>,
    stats: [InterruptStats; NUM_SOURCES],
    dropped_events: AtomicU32,
}

impl<const N: usize> EventDispatcher<N> {
    pub const fn new() -> Self {
        // TODO: Initialize queue, stats array, and dropped counter
        // HINT: Use array initialization with const fn
        todo!("Implement EventDispatcher::new")
    }

    /// Push event from interrupt (lock-free, fast)
    pub fn push_event(&self, event: InterruptEvent) -> Result<(), InterruptEvent> {
        // TODO: Try to enqueue event
        // TODO: If queue full, increment dropped_events and return Err
        // TODO: If successful, update source stats
        todo!("Implement push_event")
    }

    /// Pop event for processing (main loop)
    pub fn pop_event(&self) -> Option<InterruptEvent> {
        // TODO: Dequeue event
        todo!("Implement pop_event")
    }

    /// Process all pending events
    pub fn dispatch_all<F>(&self, mut handler: F)
    where
        F: FnMut(InterruptEvent),
    {
        // TODO: Pop and handle all events in queue
        todo!("Implement dispatch_all")
    }

    pub fn get_source_stats(&self, source: EventSource) -> StatSnapshot {
        self.stats[source.to_index()].snapshot()
    }

    pub fn dropped_count(&self) -> u32 {
        self.dropped_events.load(Ordering::Relaxed)
    }

    pub fn pending_count(&self) -> usize {
        cortex_m::interrupt::free(|cs| {
            self.queue.borrow(cs).borrow().len()
        })
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(source: EventSource, data: u32) -> InterruptEvent {
        InterruptEvent {
            source,
            data,
            timestamp: 1000,
        }
    }

    #[test]
    fn test_dispatcher_basic() {
        let dispatcher: EventDispatcher<16> = EventDispatcher::new();

        let event = make_event(EventSource::Timer1, 42);
        dispatcher.push_event(event).unwrap();

        assert_eq!(dispatcher.pending_count(), 1);

        let popped = dispatcher.pop_event().unwrap();
        assert_eq!(popped.source, EventSource::Timer1);
        assert_eq!(popped.data, 42);

        assert_eq!(dispatcher.pending_count(), 0);
    }

    #[test]
    fn test_multiple_sources() {
        let dispatcher: EventDispatcher<32> = EventDispatcher::new();

        dispatcher.push_event(make_event(EventSource::Timer1, 1)).unwrap();
        dispatcher.push_event(make_event(EventSource::Uart, 2)).unwrap();
        dispatcher.push_event(make_event(EventSource::Gpio(5), 3)).unwrap();

        let events: Vec<_> = core::iter::from_fn(|| dispatcher.pop_event()).collect();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].data, 1);
        assert_eq!(events[1].data, 2);
        assert_eq!(events[2].data, 3);
    }

    #[test]
    fn test_queue_overflow() {
        let dispatcher: EventDispatcher<4> = EventDispatcher::new();

        // Fill queue
        for i in 0..4 {
            dispatcher.push_event(make_event(EventSource::Timer1, i)).unwrap();
        }

        // Should fail - queue full
        let result = dispatcher.push_event(make_event(EventSource::Timer1, 999));
        assert!(result.is_err());
        assert_eq!(dispatcher.dropped_count(), 1);
    }

    #[test]
    fn test_dispatch_all() {
        let dispatcher: EventDispatcher<16> = EventDispatcher::new();

        for i in 0..5 {
            dispatcher.push_event(make_event(EventSource::Uart, i)).unwrap();
        }

        let mut received = Vec::new();
        dispatcher.dispatch_all(|event| {
            received.push(event.data);
        });

        assert_eq!(received, vec![0, 1, 2, 3, 4]);
        assert_eq!(dispatcher.pending_count(), 0);
    }

    #[test]
    fn test_source_stats() {
        let dispatcher: EventDispatcher<32> = EventDispatcher::new();

        // Send events from different sources
        for _ in 0..10 {
            dispatcher.push_event(make_event(EventSource::Timer1, 0)).unwrap();
        }
        for _ in 0..5 {
            dispatcher.push_event(make_event(EventSource::Uart, 0)).unwrap();
        }

        let timer_stats = dispatcher.get_source_stats(EventSource::Timer1);
        let uart_stats = dispatcher.get_source_stats(EventSource::Uart);

        assert_eq!(timer_stats.count, 10);
        assert_eq!(uart_stats.count, 5);
    }
}
```

**Check Your Understanding**:
- Why use SPSC queue instead of MPSC queue?
- What happens if main loop can't keep up with interrupt rate?
- How would you prioritize certain event sources over others?

---

#### Why Milestone 3 Isn't Enough

**Limitation**: Events are dispatched but there's no way to monitor deadlines or detect when interrupts are taking too long.

**What we're adding**: Deadline monitoring and watchdog integration to detect and handle timing violations.

**Improvement**:
- **Safety**: Detect runaway interrupts before they cause system failure
- **Observability**: Real-time visibility into timing behavior
- **Recovery**: Graceful handling of deadline violations
- **Debugging**: Identifies performance bottlenecks

---

### Milestone 4: Deadline Monitoring and Watchdog Integration

**Goal**: Add deadline monitoring for interrupt handlers and integrate with hardware watchdog to detect and recover from timing violations.

**Why this milestone**: Real-time systems must guarantee timing. This teaches deadline enforcement and watchdog patterns.

#### Architecture

**Structs:**
- `DeadlineMonitor` - Tracks timing violations
  - **Field**: `deadlines: [(EventSource, u32); N]` - Source → deadline (μs) mapping
  - **Field**: `violations: [AtomicU32; NUM_SOURCES]` - Violation counts
  - **Field**: `watchdog_enabled: AtomicBool` - Watchdog state

- `TimedEvent` - Event with deadline information
  - **Field**: `event: InterruptEvent` - Base event
  - **Field**: `deadline_us: u32` - Processing deadline
  - **Field**: `start_time: u64` - When processing started

**Functions:**
- `DeadlineMonitor::new() -> Self` - Create monitor
- `set_deadline(&mut self, source: EventSource, deadline_us: u32)` - Configure deadline
- `start_processing(&self, event: InterruptEvent) -> TimedEvent` - Begin timing
- `finish_processing(&self, timed: TimedEvent) -> Result<u32, DeadlineViolation>` - Check deadline
- `get_violations(&self, source: EventSource) -> u32` - Get violation count
- `reset_watchdog(&self)` - Pet the watchdog

**Starter Code**:

```rust
use cortex_m::peripheral::DWT;

#[derive(Debug, Clone, Copy)]
pub struct DeadlineViolation {
    pub source: EventSource,
    pub deadline_us: u32,
    pub actual_us: u32,
    pub overrun_us: u32,
}

pub struct DeadlineMonitor {
    deadlines: [(EventSource, u32); NUM_SOURCES],
    violations: [AtomicU32; NUM_SOURCES],
    watchdog_fed: AtomicU32, // Last watchdog reset time
}

impl DeadlineMonitor {
    pub const fn new() -> Self {
        // TODO: Initialize deadlines array with default values
        // TODO: Initialize violations to zero
        todo!("Implement DeadlineMonitor::new")
    }

    /// Set deadline for a source
    pub fn set_deadline(&mut self, source: EventSource, deadline_us: u32) {
        // TODO: Store deadline in array
        todo!("Implement set_deadline")
    }

    /// Start timing an event
    pub fn start_processing(&self, event: InterruptEvent) -> TimedEvent {
        // TODO: Get current timestamp
        // TODO: Look up deadline for source
        // TODO: Return TimedEvent with start time
        todo!("Implement start_processing")
    }

    /// Finish processing and check deadline
    pub fn finish_processing(&self, timed: TimedEvent) -> Result<u32, DeadlineViolation> {
        // TODO: Get current timestamp
        // TODO: Calculate elapsed time
        // TODO: Check against deadline
        // TODO: If violated, increment violation counter and return Err
        // TODO: If OK, return Ok(elapsed_us)
        todo!("Implement finish_processing")
    }

    pub fn get_violations(&self, source: EventSource) -> u32 {
        self.violations[source.to_index()].load(Ordering::Relaxed)
    }

    /// Reset watchdog timer
    pub fn reset_watchdog(&self) {
        // TODO: Update watchdog_fed timestamp
        // TODO: In real hardware, would write to watchdog peripheral
        self.watchdog_fed.store(current_time_us(), Ordering::Relaxed);
    }

    /// Check if watchdog needs feeding
    pub fn needs_watchdog_reset(&self, timeout_us: u32) -> bool {
        let last_fed = self.watchdog_fed.load(Ordering::Relaxed);
        let now = current_time_us();
        (now - last_fed) > timeout_us
    }
}

pub struct TimedEvent {
    pub event: InterruptEvent,
    pub deadline_us: u32,
    pub start_time: u64,
}

/// Get current time in microseconds (using DWT cycle counter)
fn current_time_us() -> u32 {
    // TODO: Read DWT cycle counter
    // TODO: Convert cycles to microseconds based on CPU frequency
    // Simplified version:
    0 // Placeholder
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_deadline_configuration() {
        let mut monitor = DeadlineMonitor::new();

        monitor.set_deadline(EventSource::Timer1, 1000);
        monitor.set_deadline(EventSource::Uart, 5000);

        // Verify deadlines stored correctly
        assert_eq!(monitor.deadlines[EventSource::Timer1.to_index()].1, 1000);
        assert_eq!(monitor.deadlines[EventSource::Uart.to_index()].1, 5000);
    }

    #[test]
    fn test_deadline_met() {
        let mut monitor = DeadlineMonitor::new();
        monitor.set_deadline(EventSource::Timer1, 10000); // 10ms deadline

        let event = InterruptEvent {
            source: EventSource::Timer1,
            data: 42,
            timestamp: 0,
        };

        let timed = monitor.start_processing(event);

        // Simulate fast processing
        thread::sleep(Duration::from_micros(100));

        let result = monitor.finish_processing(timed);
        assert!(result.is_ok());

        let elapsed = result.unwrap();
        assert!(elapsed < 10000);
    }

    #[test]
    fn test_deadline_violation() {
        let mut monitor = DeadlineMonitor::new();
        monitor.set_deadline(EventSource::Timer1, 100); // 100μs deadline

        let event = InterruptEvent {
            source: EventSource::Timer1,
            data: 42,
            timestamp: 0,
        };

        let timed = monitor.start_processing(event);

        // Simulate slow processing (exceeds deadline)
        thread::sleep(Duration::from_micros(200));

        let result = monitor.finish_processing(timed);
        assert!(result.is_err());

        let violation = result.unwrap_err();
        assert_eq!(violation.source, EventSource::Timer1);
        assert!(violation.actual_us > violation.deadline_us);
        assert_eq!(monitor.get_violations(EventSource::Timer1), 1);
    }

    #[test]
    fn test_watchdog_feeding() {
        let monitor = DeadlineMonitor::new();

        assert!(monitor.needs_watchdog_reset(1000)); // Not fed yet

        monitor.reset_watchdog();
        assert!(!monitor.needs_watchdog_reset(1000)); // Just fed

        thread::sleep(Duration::from_millis(1100));
        assert!(monitor.needs_watchdog_reset(1000)); // Timeout expired
    }

    #[test]
    fn test_multiple_violations() {
        let mut monitor = DeadlineMonitor::new();
        monitor.set_deadline(EventSource::Uart, 50);

        for _ in 0..5 {
            let event = InterruptEvent {
                source: EventSource::Uart,
                data: 0,
                timestamp: 0,
            };

            let timed = monitor.start_processing(event);
            thread::sleep(Duration::from_micros(100));
            let _ = monitor.finish_processing(timed);
        }

        assert_eq!(monitor.get_violations(EventSource::Uart), 5);
    }
}
```

**Check Your Understanding**:
- Why measure execution time instead of just detecting hangs?
- How does watchdog prevent system lockup?
- What should happen when a deadline is violated?

---

#### Why Milestone 4 Isn't Enough

**Limitation**: We have all the pieces but they're not integrated into a cohesive system. Real applications need everything working together.

**What we're adding**: Complete interrupt coordinator that integrates event dispatch, priority management, deadline monitoring, and statistics.

**Improvement**:
- **Integration**: All features working together
- **Production-ready**: Complete error handling and monitoring
- **Scalable**: Can manage 20+ interrupt sources
- **Observable**: Rich debugging and profiling data

---

### Milestone 5: Integrated Interrupt Coordinator

**Goal**: Combine all previous milestones into a unified interrupt coordinator that manages event dispatch, priorities, deadlines, and statistics.

**Why this milestone**: Real systems need all components integrated. This teaches system-level architecture and integration patterns.

#### Architecture

**Structs:**
- `InterruptCoordinator<const N: usize>` - Complete coordinator
  - **Field**: `dispatcher: EventDispatcher<N>` - Event routing
  - **Field**: `monitor: DeadlineMonitor` - Timing enforcement
  - **Field**: `priority_groups: [PriorityGroup<()>; 4]` - Priority management

**Functions:**
- `new() -> Self` - Create coordinator
- `configure_source(&mut self, source, priority, deadline)` - Setup interrupt source
- `handle_interrupt(&self, event)` - Called from ISR
- `process_events<F>(&self, handler: F)` - Main loop processing
- `get_system_health(&self) -> SystemHealth` - Comprehensive status

**Starter Code**:

```rust
pub struct InterruptCoordinator<const N: usize> {
    dispatcher: EventDispatcher<N>,
    monitor: DeadlineMonitor,
    total_events: AtomicU32,
    total_violations: AtomicU32,
}

#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub total_events: u32,
    pub pending_events: usize,
    pub dropped_events: u32,
    pub total_violations: u32,
    pub source_stats: [(EventSource, StatSnapshot); NUM_SOURCES],
}

impl<const N: usize> InterruptCoordinator<N> {
    pub const fn new() -> Self {
        // TODO: Initialize all components
        todo!("Implement InterruptCoordinator::new")
    }

    /// Configure an interrupt source
    pub fn configure_source(
        &mut self,
        source: EventSource,
        priority: InterruptPriority,
        deadline_us: u32,
    ) {
        // TODO: Set deadline in monitor
        // TODO: Configure NVIC priority
        todo!("Implement configure_source")
    }

    /// Handle interrupt (call from ISR)
    pub fn handle_interrupt(&self, event: InterruptEvent) {
        // TODO: Push event to dispatcher
        // TODO: Increment total_events counter
        // TODO: Reset watchdog if enabled
        todo!("Implement handle_interrupt")
    }

    /// Process all pending events
    pub fn process_events<F>(&self, mut handler: F)
    where
        F: FnMut(InterruptEvent),
    {
        // TODO: Dispatch all events
        // TODO: For each event:
        //   - Start timing with monitor
        //   - Call handler
        //   - Check deadline
        //   - Update violation count if needed
        todo!("Implement process_events")
    }

    /// Get comprehensive system health
    pub fn get_system_health(&self) -> SystemHealth {
        // TODO: Collect statistics from all sources
        // TODO: Aggregate violation counts
        // TODO: Return SystemHealth struct
        todo!("Implement get_system_health")
    }

    pub fn reset_statistics(&self) {
        // TODO: Reset all counters
        todo!("Implement reset_statistics")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_basic_flow() {
        let mut coordinator: InterruptCoordinator<32> = InterruptCoordinator::new();

        coordinator.configure_source(
            EventSource::Timer1,
            InterruptPriority::High,
            1000,
        );

        // Simulate interrupt
        let event = InterruptEvent {
            source: EventSource::Timer1,
            data: 42,
            timestamp: 0,
        };
        coordinator.handle_interrupt(event);

        // Process events
        let mut received = Vec::new();
        coordinator.process_events(|e| {
            received.push(e.data);
        });

        assert_eq!(received, vec![42]);
    }

    #[test]
    fn test_system_health() {
        let mut coordinator: InterruptCoordinator<16> = InterruptCoordinator::new();

        coordinator.configure_source(EventSource::Timer1, InterruptPriority::High, 1000);
        coordinator.configure_source(EventSource::Uart, InterruptPriority::Medium, 5000);

        // Generate events
        for i in 0..10 {
            coordinator.handle_interrupt(InterruptEvent {
                source: EventSource::Timer1,
                data: i,
                timestamp: i as u64 * 1000,
            });
        }

        let health = coordinator.get_system_health();
        assert_eq!(health.total_events, 10);
        assert_eq!(health.pending_events, 10);
        assert_eq!(health.dropped_events, 0);
    }

    #[test]
    fn test_mixed_priority_events() {
        let mut coordinator: InterruptCoordinator<64> = InterruptCoordinator::new();

        coordinator.configure_source(EventSource::Timer1, InterruptPriority::Critical, 100);
        coordinator.configure_source(EventSource::Uart, InterruptPriority::Low, 10000);

        // Interleave high and low priority events
        for i in 0..20 {
            let source = if i % 2 == 0 {
                EventSource::Timer1
            } else {
                EventSource::Uart
            };

            coordinator.handle_interrupt(InterruptEvent {
                source,
                data: i,
                timestamp: i as u64 * 100,
            });
        }

        let mut processed = Vec::new();
        coordinator.process_events(|e| {
            processed.push((e.source, e.data));
        });

        assert_eq!(processed.len(), 20);
    }
}
```

**Check Your Understanding**:
- How does the coordinator prevent priority inversion?
- What's the benefit of centralizing interrupt handling?
- How would you extend this for multi-core systems?

---

#### Why Milestone 5 Isn't Enough

**Limitation**: The coordinator works well but doesn't leverage modern async patterns. Embassy/RTIC provide better abstractions for real-time scheduling.

**What we're adding**: Integration with Embassy for async interrupt handling and deterministic task scheduling.

**Improvement**:
- **Modern patterns**: Async/await for interrupt coordination
- **Framework integration**: Works with Embassy ecosystem
- **Efficiency**: Zero-cost async abstractions
- **Developer experience**: Easier to reason about concurrent interrupts

---

### Milestone 6: Embassy/RTIC Integration

**Goal**: Integrate the interrupt coordinator with Embassy's async runtime for modern, efficient interrupt handling with zero-cost abstractions.

**Why this milestone**: Embassy is the future of embedded Rust. This teaches how to build interrupt systems on top of async foundations.

#### Architecture

**Key Integration Points:**
- Use Embassy channels for event passing
- Interrupt signals for async task waking
- Priority-based task spawning
- Async deadline monitoring

**Starter Code**:

```rust
use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer, with_timeout};

type EventChannel = Channel<CriticalSectionRawMutex, InterruptEvent, 32>;
static EVENT_CHANNEL: EventChannel = Channel::new();

// Signals for each interrupt source
static TIMER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static UART_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// High-priority task for critical interrupts
#[embassy_executor::task]
async fn critical_interrupt_handler(coordinator: &'static InterruptCoordinator<32>) {
    let receiver = EVENT_CHANNEL.receiver();

    loop {
        let event = receiver.receive().await;

        if event.source == EventSource::Timer1 {
            // Critical processing with deadline
            let result = with_timeout(
                Duration::from_micros(100),
                process_critical_event(event),
            ).await;

            if result.is_err() {
                defmt::error!("Critical event deadline missed!");
            }
        }
    }
}

async fn process_critical_event(event: InterruptEvent) {
    // Critical processing logic
    defmt::info!("Processing critical event: {:?}", event);
}

/// Medium-priority task for normal interrupts
#[embassy_executor::task]
async fn normal_interrupt_handler() {
    let receiver = EVENT_CHANNEL.receiver();

    loop {
        let event = receiver.receive().await;

        match event.source {
            EventSource::Uart => {
                process_uart_event(event).await;
            }
            EventSource::Gpio(pin) => {
                process_gpio_event(pin, event).await;
            }
            _ => {}
        }
    }
}

async fn process_uart_event(event: InterruptEvent) {
    defmt::info!("UART data: {}", event.data);
}

async fn process_gpio_event(pin: u8, event: InterruptEvent) {
    defmt::info!("GPIO {} changed: {}", pin, event.data);
}

/// Watchdog task ensures system health
#[embassy_executor::task]
async fn watchdog_task(coordinator: &'static InterruptCoordinator<32>) {
    loop {
        Timer::after(Duration::from_millis(500)).await;

        let health = coordinator.get_system_health();

        if health.dropped_events > 0 {
            defmt::warn!("Dropped {} events!", health.dropped_events);
        }

        if health.total_violations > 0 {
            defmt::warn!("Deadline violations: {}", health.total_violations);
        }

        defmt::debug!(
            "Health: {} events, {} pending",
            health.total_events,
            health.pending_events
        );
    }
}

/// Main application
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Interrupt Coordinator with Embassy starting...");

    // Initialize hardware
    let p = embassy_stm32::init(Default::default());

    // Create static coordinator
    static mut COORDINATOR: Option<InterruptCoordinator<32>> = None;
    unsafe {
        COORDINATOR = Some(InterruptCoordinator::new());
    }
    let coordinator = unsafe { COORDINATOR.as_ref().unwrap() };

    // Configure interrupt sources
    unsafe {
        COORDINATOR.as_mut().unwrap().configure_source(
            EventSource::Timer1,
            InterruptPriority::Critical,
            100,
        );
        COORDINATOR.as_mut().unwrap().configure_source(
            EventSource::Uart,
            InterruptPriority::Medium,
            5000,
        );
    }

    // Spawn interrupt handling tasks
    spawner.spawn(critical_interrupt_handler(coordinator)).unwrap();
    spawner.spawn(normal_interrupt_handler()).unwrap();
    spawner.spawn(watchdog_task(coordinator)).unwrap();

    defmt::info!("All tasks spawned, system running");

    // Main loop can do other work or just monitor
    loop {
        Timer::after(Duration::from_secs(10)).await;
        defmt::info!("System heartbeat");
    }
}

// ===== Actual interrupt handlers (hardware) =====

#[interrupt]
fn TIM2() {
    // Timer interrupt - push event and signal task
    let event = InterruptEvent {
        source: EventSource::Timer1,
        data: read_timer_data(),
        timestamp: current_time_us() as u64,
    };

    unsafe {
        if let Some(coordinator) = COORDINATOR.as_ref() {
            coordinator.handle_interrupt(event);
        }
    }

    TIMER_SIGNAL.signal(());
}

#[interrupt]
fn USART2() {
    // UART interrupt
    let event = InterruptEvent {
        source: EventSource::Uart,
        data: read_uart_data(),
        timestamp: current_time_us() as u64,
    };

    unsafe {
        if let Some(coordinator) = COORDINATOR.as_ref() {
            coordinator.handle_interrupt(event);
        }
    }

    UART_SIGNAL.signal(());
}
```

**Checkpoint Tests**:

```rust
#[embassy_executor::test]
async fn test_embassy_integration() {
    let coordinator: &'static InterruptCoordinator<64> =
        Box::leak(Box::new(InterruptCoordinator::new()));

    // Simulate interrupts
    for i in 0..5 {
        coordinator.handle_interrupt(InterruptEvent {
            source: EventSource::Timer1,
            data: i,
            timestamp: i as u64 * 1000,
        });
    }

    // Process with timeout
    let mut count = 0;
    let result = with_timeout(Duration::from_millis(100), async {
        coordinator.process_events(|_| {
            count += 1;
        });
    }).await;

    assert!(result.is_ok());
    assert_eq!(count, 5);
}
```

**Check Your Understanding**:
- How do Embassy channels differ from the SPSC queue?
- Why use `with_timeout` for deadline enforcement?
- What's the advantage of task-based interrupt handling?

---

## Complete Working Example

Full implementation available in `examples/interrupt_coordinator.rs`

**Features:**
- ✅ Safe shared state with critical sections
- ✅ Priority-based interrupt management
- ✅ Lock-free event dispatcher
- ✅ Deadline monitoring with watchdog
- ✅ Comprehensive statistics and profiling
- ✅ Embassy async integration
- ✅ Multi-source coordination

**Expected Performance:**
- Interrupt latency: <1μs for critical priority
- Event processing: 100,000+ events/sec
- Zero deadlocks or priority inversions
- Deterministic deadline enforcement

---

## Testing Your Implementation

### Unit Tests
```bash
cargo test --lib
```

### Hardware Tests
```bash
# STM32 with multiple interrupt sources
cargo test --features stm32f4 --target thumbv7em-none-eabihf
```

### Stress Tests
```bash
# Generate high interrupt load
cargo run --example stress_test --release
```

## Extensions

1. **Multi-core support**: Distribute interrupts across cores
2. **Dynamic priorities**: Adjust priorities based on load
3. **Advanced scheduling**: Rate monotonic or EDF scheduling
4. **Tracing integration**: defmt or RTT for interrupt tracing
5. **Power management**: Sleep between interrupts, wake on event
6. **Safety certification**: MISRA compliance, formal verification

This project demonstrates production-grade interrupt management for safety-critical embedded systems.
