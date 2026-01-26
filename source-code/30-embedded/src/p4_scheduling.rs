// Pattern 4: Deterministic Scheduling with RTIC/Embassy
// Demonstrates real-time scheduling patterns (simulation on desktop).
//
// Note: Actual RTIC and Embassy require embedded targets.
// This file simulates the patterns for understanding and testing.

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering as AtomicOrdering};
use std::cell::RefCell;

// ============================================================================
// Priority-Based Task Scheduling (RTIC-style)
// ============================================================================

/// Task priority (higher number = higher priority)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Priority(pub u8);

/// Scheduled task representation
#[derive(Clone, Debug)]
pub struct Task {
    pub name: &'static str,
    pub priority: Priority,
    pub deadline_us: u64,
    pub handler: fn(&mut TaskContext),
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.deadline_us == other.deadline_us
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier deadline
        match self.priority.0.cmp(&other.priority.0) {
            Ordering::Equal => other.deadline_us.cmp(&self.deadline_us),
            ord => ord,
        }
    }
}

/// Context passed to task handlers
pub struct TaskContext {
    pub current_time_us: u64,
    pub shared: SharedResources,
}

/// Shared resources between tasks
#[derive(Default)]
pub struct SharedResources {
    pub rpm: AtomicU32,
    pub temperature: AtomicU32,
    pub duty_cycle: AtomicU32,
}

impl SharedResources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_rpm(&self) -> u32 {
        self.rpm.load(AtomicOrdering::Acquire)
    }

    pub fn set_rpm(&self, value: u32) {
        self.rpm.store(value, AtomicOrdering::Release);
    }

    pub fn get_temperature(&self) -> u32 {
        self.temperature.load(AtomicOrdering::Acquire)
    }

    pub fn set_temperature(&self, value: u32) {
        self.temperature.store(value, AtomicOrdering::Release);
    }

    pub fn get_duty_cycle(&self) -> u32 {
        self.duty_cycle.load(AtomicOrdering::Acquire)
    }

    pub fn set_duty_cycle(&self, value: u32) {
        self.duty_cycle.store(value, AtomicOrdering::Release);
    }
}

/// Simple priority-based scheduler
pub struct Scheduler {
    ready_queue: BinaryHeap<Task>,
    current_time_us: u64,
    shared: SharedResources,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
            current_time_us: 0,
            shared: SharedResources::new(),
        }
    }

    /// Schedule a task
    pub fn schedule(&mut self, task: Task) {
        self.ready_queue.push(task);
    }

    /// Advance time and run ready tasks
    pub fn tick(&mut self, elapsed_us: u64) {
        self.current_time_us += elapsed_us;

        // Run highest priority ready task
        while let Some(task) = self.ready_queue.pop() {
            if task.deadline_us <= self.current_time_us {
                let mut ctx = TaskContext {
                    current_time_us: self.current_time_us,
                    shared: SharedResources::new(),
                };
                // Copy shared state
                ctx.shared.rpm.store(self.shared.get_rpm(), AtomicOrdering::Relaxed);
                ctx.shared.temperature.store(self.shared.get_temperature(), AtomicOrdering::Relaxed);
                ctx.shared.duty_cycle.store(self.shared.get_duty_cycle(), AtomicOrdering::Relaxed);

                (task.handler)(&mut ctx);

                // Copy back
                self.shared.set_rpm(ctx.shared.get_rpm());
                self.shared.set_temperature(ctx.shared.get_temperature());
                self.shared.set_duty_cycle(ctx.shared.get_duty_cycle());
            } else {
                // Put it back, deadline not reached
                self.ready_queue.push(task);
                break;
            }
        }
    }

    pub fn shared(&self) -> &SharedResources {
        &self.shared
    }
}

// ============================================================================
// Async/Await Style (Embassy-like)
// ============================================================================

/// Simulated async channel
pub struct Channel<T, const N: usize> {
    buffer: RefCell<Vec<T>>,
}

impl<T, const N: usize> Channel<T, N> {
    pub const fn new() -> Self {
        Self {
            buffer: RefCell::new(Vec::new()),
        }
    }

    /// Send a value (blocks if full in real Embassy)
    pub fn send(&self, value: T) -> Result<(), T> {
        let mut buf = self.buffer.borrow_mut();
        if buf.len() >= N {
            Err(value)
        } else {
            buf.push(value);
            Ok(())
        }
    }

    /// Receive a value (blocks if empty in real Embassy)
    pub fn recv(&self) -> Option<T> {
        let mut buf = self.buffer.borrow_mut();
        if buf.is_empty() {
            None
        } else {
            Some(buf.remove(0))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.borrow().is_empty()
    }

    pub fn len(&self) -> usize {
        self.buffer.borrow().len()
    }
}

/// Simulated timer for async operations
pub struct Timer {
    deadline_us: u64,
    current_time: AtomicU64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            deadline_us: 0,
            current_time: AtomicU64::new(0),
        }
    }

    pub fn after_micros(&mut self, us: u64) {
        let now = self.current_time.load(AtomicOrdering::Relaxed);
        self.deadline_us = now + us;
    }

    pub fn after_millis(&mut self, ms: u64) {
        self.after_micros(ms * 1000);
    }

    pub fn after_secs(&mut self, secs: u64) {
        self.after_micros(secs * 1_000_000);
    }

    pub fn is_expired(&self) -> bool {
        let now = self.current_time.load(AtomicOrdering::Relaxed);
        now >= self.deadline_us
    }

    pub fn advance(&self, us: u64) {
        self.current_time.fetch_add(us, AtomicOrdering::Relaxed);
    }
}

// ============================================================================
// Example: Motor Control Tasks
// ============================================================================

/// Sample encoder task (low priority, periodic)
pub fn encoder_sample_task(ctx: &mut TaskContext) {
    // Simulate reading encoder
    let simulated_rpm = 1500 + (ctx.current_time_us / 1000 % 100) as u32;
    ctx.shared.set_rpm(simulated_rpm);
}

/// PID control task (high priority)
pub fn pid_control_task(ctx: &mut TaskContext) {
    let rpm = ctx.shared.get_rpm();
    let target_rpm = 1600;

    // Simple P controller
    let error = target_rpm as i32 - rpm as i32;
    let duty = (50 + error / 10).clamp(0, 100) as u32;

    ctx.shared.set_duty_cycle(duty);
}

/// Temperature monitoring task (low priority)
pub fn temperature_task(ctx: &mut TaskContext) {
    // Simulate temperature based on duty cycle
    let duty = ctx.shared.get_duty_cycle();
    let temp = 25 + duty / 5;
    ctx.shared.set_temperature(temp);
}

// ============================================================================
// Example: Producer-Consumer Pattern
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct AdcSample {
    pub channel: u8,
    pub value: u16,
    pub timestamp_us: u64,
}

/// ADC sampler (producer)
pub struct AdcSampler {
    channel: u8,
    sample_count: u32,
}

impl AdcSampler {
    pub fn new(channel: u8) -> Self {
        Self {
            channel,
            sample_count: 0,
        }
    }

    pub fn sample(&mut self, timestamp_us: u64) -> AdcSample {
        self.sample_count += 1;
        AdcSample {
            channel: self.channel,
            value: ((timestamp_us / 100) % 4096) as u16, // Simulated ADC value
            timestamp_us,
        }
    }
}

/// Signal processor (consumer)
pub struct SignalProcessor {
    filter_state: i32,
    alpha: i32, // Fixed-point filter coefficient (0-256)
}

impl SignalProcessor {
    pub fn new(alpha: i32) -> Self {
        Self {
            filter_state: 0,
            alpha,
        }
    }

    /// Low-pass filter
    pub fn process(&mut self, sample: &AdcSample) -> i32 {
        let input = sample.value as i32;
        // Fixed-point IIR filter: y = alpha*x + (1-alpha)*y_prev
        self.filter_state = (self.alpha * input + (256 - self.alpha) * self.filter_state) / 256;
        self.filter_state
    }

    pub fn current_output(&self) -> i32 {
        self.filter_state
    }
}

// ============================================================================
// Example: Periodic Task Manager
// ============================================================================

pub struct PeriodicTask {
    pub name: &'static str,
    pub period_us: u64,
    pub last_run_us: u64,
    pub handler: fn(u64) -> (),
}

pub struct PeriodicTaskManager {
    tasks: Vec<PeriodicTask>,
    current_time_us: u64,
}

impl PeriodicTaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_time_us: 0,
        }
    }

    pub fn add_task(&mut self, name: &'static str, period_us: u64, handler: fn(u64)) {
        self.tasks.push(PeriodicTask {
            name,
            period_us,
            last_run_us: 0,
            handler,
        });
    }

    pub fn tick(&mut self, elapsed_us: u64) -> Vec<&'static str> {
        self.current_time_us += elapsed_us;
        let mut executed = Vec::new();

        for task in &mut self.tasks {
            if self.current_time_us - task.last_run_us >= task.period_us {
                (task.handler)(self.current_time_us);
                task.last_run_us = self.current_time_us;
                executed.push(task.name);
            }
        }

        executed
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_priority_ordering() {
        let high = Task {
            name: "high",
            priority: Priority(2),
            deadline_us: 100,
            handler: |_| {},
        };
        let low = Task {
            name: "low",
            priority: Priority(1),
            deadline_us: 100,
            handler: |_| {},
        };

        assert!(high > low);
    }

    #[test]
    fn test_scheduler() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(Task {
            name: "encoder",
            priority: Priority(1),
            deadline_us: 0,
            handler: encoder_sample_task,
        });

        scheduler.tick(1000);

        assert!(scheduler.shared().get_rpm() > 0);
    }

    #[test]
    fn test_motor_control_loop() {
        let mut scheduler = Scheduler::new();

        // Run encoder first
        scheduler.schedule(Task {
            name: "encoder",
            priority: Priority(1),
            deadline_us: 0,
            handler: encoder_sample_task,
        });
        scheduler.tick(1000);

        // Then PID control
        scheduler.schedule(Task {
            name: "pid",
            priority: Priority(2),
            deadline_us: 1000,
            handler: pid_control_task,
        });
        scheduler.tick(1000);

        assert!(scheduler.shared().get_duty_cycle() > 0);
    }

    #[test]
    fn test_channel() {
        let channel: Channel<u32, 4> = Channel::new();

        assert!(channel.is_empty());

        channel.send(1).unwrap();
        channel.send(2).unwrap();
        channel.send(3).unwrap();

        assert_eq!(channel.len(), 3);

        assert_eq!(channel.recv(), Some(1));
        assert_eq!(channel.recv(), Some(2));
    }

    #[test]
    fn test_channel_overflow() {
        let channel: Channel<u32, 2> = Channel::new();

        channel.send(1).unwrap();
        channel.send(2).unwrap();
        assert!(channel.send(3).is_err());
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::new();
        timer.after_millis(10);

        assert!(!timer.is_expired());
        timer.advance(5000);
        assert!(!timer.is_expired());
        timer.advance(6000);
        assert!(timer.is_expired());
    }

    #[test]
    fn test_adc_sampler() {
        let mut sampler = AdcSampler::new(0);

        let s1 = sampler.sample(1000);
        let s2 = sampler.sample(2000);

        assert_eq!(s1.channel, 0);
        assert!(s2.timestamp_us > s1.timestamp_us);
    }

    #[test]
    fn test_signal_processor() {
        let mut processor = SignalProcessor::new(64); // alpha = 0.25

        let sample = AdcSample {
            channel: 0,
            value: 1000,
            timestamp_us: 0,
        };

        // Filter should converge towards input
        for _ in 0..20 {
            processor.process(&sample);
        }

        // Should be close to 1000 after convergence
        let output = processor.current_output();
        assert!(output > 900 && output < 1100);
    }

    #[test]
    fn test_periodic_task_manager() {
        let mut manager = PeriodicTaskManager::new();

        static mut FAST_COUNT: u32 = 0;
        static mut SLOW_COUNT: u32 = 0;

        manager.add_task("fast", 100, |_| unsafe { FAST_COUNT += 1 });
        manager.add_task("slow", 500, |_| unsafe { SLOW_COUNT += 1 });

        // Run for 1000us
        for _ in 0..10 {
            manager.tick(100);
        }

        unsafe {
            assert_eq!(FAST_COUNT, 10);
            assert_eq!(SLOW_COUNT, 2);
        }
    }
}

fn main() {
    println!("Pattern 4: Deterministic Scheduling");
    println!("====================================\n");

    // Priority-based scheduling (RTIC-style)
    println!("Priority-Based Scheduler (RTIC-style):");
    let mut scheduler = Scheduler::new();

    // Schedule initial tasks
    scheduler.schedule(Task {
        name: "encoder_sample",
        priority: Priority(1),
        deadline_us: 0,
        handler: encoder_sample_task,
    });

    scheduler.schedule(Task {
        name: "pid_control",
        priority: Priority(2),
        deadline_us: 100,
        handler: pid_control_task,
    });

    scheduler.schedule(Task {
        name: "temperature",
        priority: Priority(0),
        deadline_us: 200,
        handler: temperature_task,
    });

    // Simulate time progression
    for t in [100, 100, 100, 100, 100] {
        scheduler.tick(t);
        println!(
            "  t={}us: RPM={}, Duty={}%, Temp={}°C",
            scheduler.shared.rpm.load(AtomicOrdering::Relaxed),
            scheduler.shared().get_rpm(),
            scheduler.shared().get_duty_cycle(),
            scheduler.shared().get_temperature()
        );

        // Re-schedule periodic tasks
        let current = scheduler.current_time_us;
        scheduler.schedule(Task {
            name: "encoder_sample",
            priority: Priority(1),
            deadline_us: current + 100,
            handler: encoder_sample_task,
        });
        scheduler.schedule(Task {
            name: "pid_control",
            priority: Priority(2),
            deadline_us: current + 100,
            handler: pid_control_task,
        });
    }

    // Channel-based communication (Embassy-style)
    println!("\nChannel Communication (Embassy-style):");
    let channel: Channel<AdcSample, 8> = Channel::new();
    let mut sampler = AdcSampler::new(0);
    let mut processor = SignalProcessor::new(128);

    // Producer: sample ADC
    for t in (0..5).map(|i| i * 500) {
        let sample = sampler.sample(t);
        channel.send(sample).unwrap();
        println!("  Sampled: ch={}, val={}, t={}us", sample.channel, sample.value, sample.timestamp_us);
    }

    // Consumer: process samples
    println!("\n  Processing samples:");
    while let Some(sample) = channel.recv() {
        let filtered = processor.process(&sample);
        println!("    Raw={}, Filtered={}", sample.value, filtered);
    }

    // Timer demonstration
    println!("\nAsync Timer (Embassy-style):");
    let mut timer = Timer::new();
    timer.after_millis(100);
    println!("  Timer set for 100ms");
    println!("  At 50ms: expired = {}", { timer.advance(50_000); timer.is_expired() });
    println!("  At 100ms: expired = {}", { timer.advance(50_000); timer.is_expired() });

    // Periodic task manager
    println!("\nPeriodic Task Manager:");
    let mut manager = PeriodicTaskManager::new();

    manager.add_task("sensor_read", 1000, |t| println!("    [{}us] Sensor read", t));
    manager.add_task("control_loop", 500, |t| println!("    [{}us] Control loop", t));
    manager.add_task("logging", 2000, |t| println!("    [{}us] Logging", t));

    println!("  Running for 3000us:");
    for _ in 0..6 {
        manager.tick(500);
    }

    println!("\nKey scheduling patterns:");
    println!("  - RTIC: Priority-based preemption, automatic resource locking");
    println!("  - Embassy: Async/await with channels, timer-based delays");
    println!("  - Both: No heap allocation, deterministic timing");
}
