// Pattern 3: Interrupt-Safe Shared State
// Demonstrates synchronization primitives for ISR communication.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::cell::RefCell;
use critical_section::Mutex;

// ============================================================================
// Example: Atomic Counter for Button Presses
// ============================================================================

/// Thread/interrupt-safe button counter using atomics
pub struct AtomicCounter {
    count: AtomicU32,
}

impl AtomicCounter {
    pub const fn new() -> Self {
        Self {
            count: AtomicU32::new(0),
        }
    }

    /// Increment counter (safe to call from ISR)
    pub fn increment(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Read current count (safe from any context)
    pub fn read(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }

    /// Reset counter to zero
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
    }
}

// ============================================================================
// Example: Atomic Flag for Wake-Ups
// ============================================================================

/// Wake-up flag for ISR-to-main communication
pub struct WakeupFlag {
    ready: AtomicBool,
}

impl WakeupFlag {
    pub const fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
        }
    }

    /// Signal from ISR (Release ordering for visibility)
    pub fn signal(&self) {
        self.ready.store(true, Ordering::Release);
    }

    /// Check and clear (AcqRel for synchronization)
    pub fn check_and_clear(&self) -> bool {
        self.ready.swap(false, Ordering::AcqRel)
    }

    /// Check without clearing
    pub fn is_set(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }
}

// ============================================================================
// Example: Critical Section Protected State
// ============================================================================

/// State protected by critical section
pub struct ProtectedState<T> {
    inner: Mutex<RefCell<T>>,
}

impl<T> ProtectedState<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(value)),
        }
    }

    /// Access state within critical section
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        critical_section::with(|cs| {
            let mut guard = self.inner.borrow(cs).borrow_mut();
            f(&mut guard)
        })
    }

    /// Read state (cloning)
    pub fn read(&self) -> T
    where
        T: Clone,
    {
        critical_section::with(|cs| self.inner.borrow(cs).borrow().clone())
    }
}

// ============================================================================
// Example: Sensor Reading with Timestamp
// ============================================================================

#[derive(Clone, Copy, Debug, Default)]
pub struct TimestampedReading {
    pub value: i32,
    pub timestamp: u64,
}

/// Atomic sensor reading (ISR writes, main reads)
pub struct AtomicSensorReading {
    value: AtomicU32,
    timestamp: AtomicU64,
    valid: AtomicBool,
}

impl AtomicSensorReading {
    pub const fn new() -> Self {
        Self {
            value: AtomicU32::new(0),
            timestamp: AtomicU64::new(0),
            valid: AtomicBool::new(false),
        }
    }

    /// Update from ISR
    pub fn update(&self, value: i32, timestamp: u64) {
        // Store value first, then timestamp, then valid flag
        self.value.store(value as u32, Ordering::Relaxed);
        self.timestamp.store(timestamp, Ordering::Relaxed);
        self.valid.store(true, Ordering::Release);
    }

    /// Read with validation
    pub fn read(&self) -> Option<TimestampedReading> {
        if self.valid.load(Ordering::Acquire) {
            Some(TimestampedReading {
                value: self.value.load(Ordering::Relaxed) as i32,
                timestamp: self.timestamp.load(Ordering::Relaxed),
            })
        } else {
            None
        }
    }

    /// Consume the reading (mark as invalid)
    pub fn consume(&self) -> Option<TimestampedReading> {
        if self.valid.swap(false, Ordering::AcqRel) {
            Some(TimestampedReading {
                value: self.value.load(Ordering::Relaxed) as i32,
                timestamp: self.timestamp.load(Ordering::Relaxed),
            })
        } else {
            None
        }
    }
}

// ============================================================================
// Example: Event Queue (ISR to Main)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    ButtonPress,
    TimerTick,
    AdcComplete(u16),
    UartReceive(u8),
}

/// Lock-free event queue using atomics
pub struct EventQueue<const N: usize> {
    events: [Mutex<RefCell<Option<Event>>>; N],
    write_idx: AtomicU32,
    read_idx: AtomicU32,
}

impl<const N: usize> EventQueue<N> {
    const INIT: Mutex<RefCell<Option<Event>>> = Mutex::new(RefCell::new(None));

    pub const fn new() -> Self {
        Self {
            events: [Self::INIT; N],
            write_idx: AtomicU32::new(0),
            read_idx: AtomicU32::new(0),
        }
    }

    /// Post event (from ISR)
    pub fn post(&self, event: Event) -> bool {
        let write = self.write_idx.load(Ordering::Relaxed) as usize;
        let read = self.read_idx.load(Ordering::Relaxed) as usize;

        // Check if full
        if (write + 1) % N == read {
            return false;
        }

        critical_section::with(|cs| {
            *self.events[write].borrow(cs).borrow_mut() = Some(event);
        });

        self.write_idx.store(((write + 1) % N) as u32, Ordering::Release);
        true
    }

    /// Get next event (from main loop)
    pub fn get(&self) -> Option<Event> {
        let write = self.write_idx.load(Ordering::Acquire) as usize;
        let read = self.read_idx.load(Ordering::Relaxed) as usize;

        if read == write {
            return None;
        }

        let event = critical_section::with(|cs| {
            self.events[read].borrow(cs).borrow_mut().take()
        });

        self.read_idx.store(((read + 1) % N) as u32, Ordering::Release);
        event
    }

    pub fn is_empty(&self) -> bool {
        let write = self.write_idx.load(Ordering::Acquire);
        let read = self.read_idx.load(Ordering::Relaxed);
        read == write
    }
}

// ============================================================================
// Example: Shared Bus Access
// ============================================================================

/// Mock I2C driver
pub struct MockI2c {
    last_address: u8,
    last_data: Vec<u8>,
}

impl MockI2c {
    pub fn new() -> Self {
        Self {
            last_address: 0,
            last_data: Vec::new(),
        }
    }

    pub fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), &'static str> {
        self.last_address = addr;
        self.last_data = data.to_vec();
        Ok(())
    }

    pub fn read(&mut self, addr: u8, buf: &mut [u8]) -> Result<(), &'static str> {
        self.last_address = addr;
        // Return mock data
        for (i, b) in buf.iter_mut().enumerate() {
            *b = (addr.wrapping_add(i as u8)) as u8;
        }
        Ok(())
    }
}

/// Shared bus wrapper with critical section protection
pub struct SharedBus<BUS> {
    bus: Mutex<RefCell<Option<BUS>>>,
}

impl<BUS> SharedBus<BUS> {
    pub const fn new() -> Self {
        Self {
            bus: Mutex::new(RefCell::new(None)),
        }
    }

    pub fn init(&self, bus: BUS) {
        critical_section::with(|cs| {
            *self.bus.borrow(cs).borrow_mut() = Some(bus);
        });
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut BUS) -> R) -> Option<R> {
        critical_section::with(|cs| {
            let mut guard = self.bus.borrow(cs).borrow_mut();
            guard.as_mut().map(f)
        })
    }
}

// ============================================================================
// Example: Debounced Button
// ============================================================================

pub struct DebouncedButton {
    raw_state: AtomicBool,
    debounced_state: AtomicBool,
    last_change: AtomicU64,
    debounce_ms: u64,
}

impl DebouncedButton {
    pub const fn new(debounce_ms: u64) -> Self {
        Self {
            raw_state: AtomicBool::new(false),
            debounced_state: AtomicBool::new(false),
            last_change: AtomicU64::new(0),
            debounce_ms,
        }
    }

    /// Update raw state (from GPIO ISR)
    pub fn update_raw(&self, pressed: bool, timestamp_ms: u64) {
        let current = self.raw_state.load(Ordering::Relaxed);
        if pressed != current {
            self.raw_state.store(pressed, Ordering::Relaxed);
            self.last_change.store(timestamp_ms, Ordering::Release);
        }
    }

    /// Process debouncing (from timer tick)
    pub fn process(&self, current_ms: u64) -> bool {
        let last = self.last_change.load(Ordering::Acquire);
        let raw = self.raw_state.load(Ordering::Relaxed);
        let debounced = self.debounced_state.load(Ordering::Relaxed);

        if raw != debounced && current_ms.saturating_sub(last) >= self.debounce_ms {
            self.debounced_state.store(raw, Ordering::Release);
            return raw; // Return new state on transition
        }
        debounced
    }

    /// Get debounced state
    pub fn is_pressed(&self) -> bool {
        self.debounced_state.load(Ordering::Acquire)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new();
        assert_eq!(counter.read(), 0);

        counter.increment();
        counter.increment();
        counter.increment();

        assert_eq!(counter.read(), 3);

        counter.reset();
        assert_eq!(counter.read(), 0);
    }

    #[test]
    fn test_wakeup_flag() {
        let flag = WakeupFlag::new();
        assert!(!flag.is_set());

        flag.signal();
        assert!(flag.is_set());

        assert!(flag.check_and_clear());
        assert!(!flag.is_set());
    }

    #[test]
    fn test_protected_state() {
        let state: ProtectedState<u32> = ProtectedState::new(0);

        state.with(|v| *v = 42);
        assert_eq!(state.read(), 42);

        state.with(|v| *v += 10);
        assert_eq!(state.read(), 52);
    }

    #[test]
    fn test_atomic_sensor_reading() {
        let sensor = AtomicSensorReading::new();

        assert!(sensor.read().is_none());

        sensor.update(1234, 1000);
        let reading = sensor.read().unwrap();
        assert_eq!(reading.value, 1234);
        assert_eq!(reading.timestamp, 1000);

        // Consume should clear
        let consumed = sensor.consume().unwrap();
        assert_eq!(consumed.value, 1234);
        assert!(sensor.read().is_none());
    }

    #[test]
    fn test_event_queue() {
        let queue: EventQueue<8> = EventQueue::new();

        assert!(queue.is_empty());

        queue.post(Event::ButtonPress);
        queue.post(Event::TimerTick);
        queue.post(Event::AdcComplete(1024));

        assert!(!queue.is_empty());

        assert_eq!(queue.get(), Some(Event::ButtonPress));
        assert_eq!(queue.get(), Some(Event::TimerTick));
        assert_eq!(queue.get(), Some(Event::AdcComplete(1024)));
        assert_eq!(queue.get(), None);
    }

    #[test]
    fn test_shared_bus() {
        let bus: SharedBus<MockI2c> = SharedBus::new();

        // Before init
        assert!(bus.with(|_| ()).is_none());

        // After init
        bus.init(MockI2c::new());

        let result = bus.with(|i2c| {
            i2c.write(0x50, &[0x01, 0x02]).unwrap();
            i2c.last_address
        });

        assert_eq!(result, Some(0x50));
    }

    #[test]
    fn test_debounced_button() {
        let button = DebouncedButton::new(50); // 50ms debounce

        // Initial state
        assert!(!button.is_pressed());

        // Press detected
        button.update_raw(true, 0);
        button.process(10); // Too soon
        assert!(!button.is_pressed());

        button.process(60); // After debounce
        assert!(button.is_pressed());

        // Release
        button.update_raw(false, 100);
        button.process(110); // Too soon
        assert!(button.is_pressed());

        button.process(160); // After debounce
        assert!(!button.is_pressed());
    }
}

fn main() {
    println!("Pattern 3: Interrupt-Safe Shared State");
    println!("======================================\n");

    // Atomic Counter
    println!("Atomic Button Counter:");
    let counter = AtomicCounter::new();
    for i in 1..=5 {
        counter.increment();
        println!("  Button press {}: count = {}", i, counter.read());
    }

    // Wakeup Flag
    println!("\nWakeup Flag (ISR signaling):");
    let flag = WakeupFlag::new();
    println!("  Flag set: {}", flag.is_set());
    flag.signal();
    println!("  After ISR signal: {}", flag.is_set());
    println!("  Check and clear: {}", flag.check_and_clear());
    println!("  After clear: {}", flag.is_set());

    // Protected State
    println!("\nCritical Section Protected State:");
    let rpm: ProtectedState<u16> = ProtectedState::new(0);
    rpm.with(|v| *v = 1500);
    println!("  Motor RPM: {}", rpm.read());
    rpm.with(|v| *v += 500);
    println!("  After adjustment: {}", rpm.read());

    // Sensor Reading
    println!("\nTimestamped Sensor Reading:");
    let sensor = AtomicSensorReading::new();
    sensor.update(2750, 1234567);
    if let Some(reading) = sensor.read() {
        println!("  Value: {}, Timestamp: {}", reading.value, reading.timestamp);
    }

    // Event Queue
    println!("\nEvent Queue (ISR to Main):");
    let events: EventQueue<16> = EventQueue::new();
    events.post(Event::ButtonPress);
    events.post(Event::TimerTick);
    events.post(Event::AdcComplete(2048));
    events.post(Event::UartReceive(b'A'));

    println!("  Processing events:");
    while let Some(event) = events.get() {
        println!("    {:?}", event);
    }

    // Debounced Button
    println!("\nDebounced Button:");
    let button = DebouncedButton::new(20);
    println!("  Initial: pressed = {}", button.is_pressed());

    button.update_raw(true, 0);
    button.process(5);
    println!("  After 5ms: pressed = {} (bouncing)", button.is_pressed());

    button.process(25);
    println!("  After 25ms: pressed = {} (debounced)", button.is_pressed());

    // Shared Bus
    println!("\nShared I2C Bus:");
    let i2c_bus: SharedBus<MockI2c> = SharedBus::new();
    i2c_bus.init(MockI2c::new());

    i2c_bus.with(|bus| {
        bus.write(0x48, &[0x00, 0x60]).unwrap();
        println!("  Wrote to I2C address 0x{:02X}", bus.last_address);
    });

    println!("\nKey patterns:");
    println!("  - Atomics for simple flags/counters (no critical section needed)");
    println!("  - Critical sections for complex state (minimal duration)");
    println!("  - Event queues decouple ISR from main processing");
}
