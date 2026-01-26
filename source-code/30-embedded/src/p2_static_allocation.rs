// Pattern 2: Static Allocation & Zero-Copy Buffers
// Demonstrates heap-free data structures for embedded systems.

use std::sync::atomic::{AtomicU32, Ordering};
use heapless::Vec as HeaplessVec;
use heapless::spsc::Queue;

// ============================================================================
// Example: Lock-Free SPSC Queue
// ============================================================================

/// Packet structure for telemetry
#[derive(Clone, Copy, Debug, Default)]
pub struct TelemetryPacket {
    pub id: u32,
    pub data: [u8; 28],
}

impl TelemetryPacket {
    pub fn new(id: u32) -> Self {
        let mut pkt = Self::default();
        pkt.id = id;
        pkt.data[..4].copy_from_slice(&id.to_le_bytes());
        pkt
    }
}

/// Telemetry queue with static capacity
pub struct TelemetryQueue {
    queue: Queue<TelemetryPacket, 8>,
    next_id: AtomicU32,
}

impl TelemetryQueue {
    pub const fn new() -> Self {
        Self {
            queue: Queue::new(),
            next_id: AtomicU32::new(0),
        }
    }

    /// Enqueue a new packet (producer side)
    pub fn enqueue(&mut self) -> Result<u32, TelemetryPacket> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let packet = TelemetryPacket::new(id);
        self.queue.enqueue(packet).map(|_| id)
    }

    /// Dequeue a packet (consumer side)
    pub fn dequeue(&mut self) -> Option<TelemetryPacket> {
        self.queue.dequeue()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get current queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }
}

// ============================================================================
// Example: Fixed-Capacity Command Log
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Command {
    pub opcode: u8,
    pub payload: [u8; 4],
}

impl Command {
    pub fn new(opcode: u8, payload: [u8; 4]) -> Self {
        Self { opcode, payload }
    }
}

/// Command log with compile-time capacity
pub struct CommandLog<const N: usize> {
    log: HeaplessVec<Command, N>,
}

impl<const N: usize> CommandLog<N> {
    pub const fn new() -> Self {
        Self {
            log: HeaplessVec::new(),
        }
    }

    /// Append a command, silently drop if full
    pub fn append(&mut self, cmd: Command) -> bool {
        self.log.push(cmd).is_ok()
    }

    /// Get the latest command
    pub fn latest(&self) -> Option<&Command> {
        self.log.last()
    }

    /// Get all commands
    pub fn all(&self) -> &[Command] {
        &self.log
    }

    /// Clear the log
    pub fn clear(&mut self) {
        self.log.clear();
    }

    /// Check if log is full
    pub fn is_full(&self) -> bool {
        self.log.is_full()
    }

    /// Get current count
    pub fn len(&self) -> usize {
        self.log.len()
    }
}

// ============================================================================
// Example: Static Ring Buffer
// ============================================================================

/// Ring buffer with static capacity
pub struct RingBuffer<T: Copy + Default, const N: usize> {
    data: [T; N],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T: Copy + Default, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// Push an item, overwriting oldest if full
    pub fn push(&mut self, item: T) {
        self.data[self.tail] = item;
        self.tail = (self.tail + 1) % N;

        if self.count == N {
            // Overwrite oldest
            self.head = (self.head + 1) % N;
        } else {
            self.count += 1;
        }
    }

    /// Pop the oldest item
    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let item = self.data[self.head];
            self.head = (self.head + 1) % N;
            self.count -= 1;
            Some(item)
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count == N
    }

    pub fn capacity(&self) -> usize {
        N
    }
}

// ============================================================================
// Example: DMA Buffer Simulation
// ============================================================================

/// Aligned buffer for DMA operations
#[repr(C, align(4))]
pub struct DmaBuffer<const N: usize> {
    data: [u16; N],
}

impl<const N: usize> DmaBuffer<N> {
    pub const fn new() -> Self {
        Self { data: [0; N] }
    }

    pub fn as_slice(&self) -> &[u16] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        &mut self.data
    }

    pub fn fill(&mut self, value: u16) {
        self.data.fill(value);
    }

    /// Simulate DMA filling the buffer
    pub fn simulate_dma_fill(&mut self, pattern: impl Fn(usize) -> u16) {
        for (i, val) in self.data.iter_mut().enumerate() {
            *val = pattern(i);
        }
    }
}

// ============================================================================
// Example: Double Buffer for Streaming
// ============================================================================

pub struct DoubleBuffer<const N: usize> {
    buffers: [[i16; N]; 2],
    active: usize,
}

impl<const N: usize> DoubleBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buffers: [[0; N]; 2],
            active: 0,
        }
    }

    /// Get the active buffer for reading
    pub fn active_buffer(&self) -> &[i16; N] {
        &self.buffers[self.active]
    }

    /// Get the inactive buffer for writing
    pub fn inactive_buffer_mut(&mut self) -> &mut [i16; N] {
        &mut self.buffers[1 - self.active]
    }

    /// Swap buffers (called when DMA completes)
    pub fn swap(&mut self) {
        self.active = 1 - self.active;
    }

    /// Fill inactive buffer with data
    pub fn fill_inactive(&mut self, data: &[i16]) {
        let buf = self.inactive_buffer_mut();
        let len = data.len().min(N);
        buf[..len].copy_from_slice(&data[..len]);
    }
}

// ============================================================================
// Example: Heapless String Buffer
// ============================================================================

pub struct StringBuffer<const N: usize> {
    buffer: HeaplessVec<u8, N>,
}

impl<const N: usize> StringBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buffer: HeaplessVec::new(),
        }
    }

    pub fn push_str(&mut self, s: &str) -> bool {
        for byte in s.bytes() {
            if self.buffer.push(byte).is_err() {
                return false;
            }
        }
        true
    }

    pub fn as_str(&self) -> &str {
        // Safety: we only push valid UTF-8 bytes
        std::str::from_utf8(&self.buffer).unwrap_or("")
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn capacity(&self) -> usize {
        N
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_queue() {
        let mut queue = TelemetryQueue::new();

        assert!(queue.is_empty());
        // heapless spsc::Queue<T, N> has capacity N-1 (one slot reserved)
        assert!(queue.capacity() >= 7);

        // Enqueue packets
        for i in 0..5 {
            let id = queue.enqueue().unwrap();
            assert_eq!(id, i);
        }

        assert_eq!(queue.len(), 5);

        // Dequeue and verify
        let pkt = queue.dequeue().unwrap();
        assert_eq!(pkt.id, 0);
    }

    #[test]
    fn test_telemetry_queue_overflow() {
        let mut queue = TelemetryQueue::new();

        // Fill to capacity (heapless queue capacity is N-1)
        let mut count = 0;
        while queue.enqueue().is_ok() {
            count += 1;
            if count > 10 { break; } // Safety limit
        }

        // Should have filled 7 items (capacity is 8, but usable is 7)
        assert!(count >= 7);

        // Additional enqueue should fail
        let result = queue.enqueue();
        assert!(result.is_err());
    }

    #[test]
    fn test_command_log() {
        let mut log: CommandLog<4> = CommandLog::new();

        let cmd1 = Command::new(0x01, [1, 2, 3, 4]);
        let cmd2 = Command::new(0x02, [5, 6, 7, 8]);

        assert!(log.append(cmd1));
        assert!(log.append(cmd2));

        assert_eq!(log.len(), 2);
        assert_eq!(log.latest(), Some(&cmd2));
    }

    #[test]
    fn test_command_log_full() {
        let mut log: CommandLog<2> = CommandLog::new();

        log.append(Command::new(0x01, [0; 4]));
        log.append(Command::new(0x02, [0; 4]));

        assert!(log.is_full());
        assert!(!log.append(Command::new(0x03, [0; 4])));
    }

    #[test]
    fn test_ring_buffer() {
        let mut rb: RingBuffer<u32, 4> = RingBuffer::new();

        assert!(rb.is_empty());
        assert_eq!(rb.capacity(), 4);

        rb.push(1);
        rb.push(2);
        rb.push(3);

        assert_eq!(rb.len(), 3);
        assert_eq!(rb.pop(), Some(1));
        assert_eq!(rb.pop(), Some(2));
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let mut rb: RingBuffer<u32, 3> = RingBuffer::new();

        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4); // Overwrites 1

        assert!(rb.is_full());
        assert_eq!(rb.pop(), Some(2)); // 1 was overwritten
    }

    #[test]
    fn test_dma_buffer() {
        let mut buf: DmaBuffer<128> = DmaBuffer::new();

        buf.simulate_dma_fill(|i| i as u16 * 10);

        assert_eq!(buf.as_slice()[0], 0);
        assert_eq!(buf.as_slice()[10], 100);
        assert_eq!(buf.as_slice()[127], 1270);
    }

    #[test]
    fn test_double_buffer() {
        let mut db: DoubleBuffer<256> = DoubleBuffer::new();

        // Fill inactive buffer
        let test_data: Vec<i16> = (0..256).map(|i| i as i16).collect();
        db.fill_inactive(&test_data);

        // Active buffer should still be zeros
        assert_eq!(db.active_buffer()[0], 0);

        // Swap and check
        db.swap();
        assert_eq!(db.active_buffer()[0], 0);
        assert_eq!(db.active_buffer()[255], 255);
    }

    #[test]
    fn test_string_buffer() {
        let mut sb: StringBuffer<32> = StringBuffer::new();

        assert!(sb.push_str("Hello"));
        assert!(sb.push_str(", "));
        assert!(sb.push_str("World!"));

        assert_eq!(sb.as_str(), "Hello, World!");
        assert_eq!(sb.len(), 13);
    }

    #[test]
    fn test_string_buffer_overflow() {
        let mut sb: StringBuffer<8> = StringBuffer::new();

        assert!(sb.push_str("Hello"));
        assert!(!sb.push_str("World!")); // Too long

        assert_eq!(sb.as_str(), "HelloWor"); // Partial write
    }
}

fn main() {
    println!("Pattern 2: Static Allocation & Zero-Copy Buffers");
    println!("=================================================\n");

    // Telemetry Queue
    println!("SPSC Telemetry Queue:");
    let mut queue = TelemetryQueue::new();
    for _ in 0..5 {
        let id = queue.enqueue().unwrap();
        println!("  Enqueued packet {}", id);
    }
    while let Some(pkt) = queue.dequeue() {
        println!("  Dequeued packet {}", pkt.id);
    }

    // Command Log
    println!("\nFixed-Capacity Command Log:");
    let mut log: CommandLog<32> = CommandLog::new();
    log.append(Command::new(0x01, [0x10, 0x20, 0x30, 0x40]));
    log.append(Command::new(0x02, [0x11, 0x21, 0x31, 0x41]));
    println!("  Commands logged: {}", log.len());
    if let Some(cmd) = log.latest() {
        println!("  Latest command: opcode=0x{:02X}", cmd.opcode);
    }

    // Ring Buffer
    println!("\nRing Buffer (overwriting):");
    let mut rb: RingBuffer<u32, 4> = RingBuffer::new();
    for i in 1..=6 {
        rb.push(i);
        println!("  Pushed {}, len={}", i, rb.len());
    }
    println!("  Contents after overflow:");
    while let Some(val) = rb.pop() {
        println!("    Popped: {}", val);
    }

    // DMA Buffer
    println!("\nDMA Buffer Simulation:");
    let mut dma_buf: DmaBuffer<16> = DmaBuffer::new();
    dma_buf.simulate_dma_fill(|i| (i * 100) as u16);
    println!("  Buffer contents: {:?}", &dma_buf.as_slice()[..8]);

    // Double Buffer
    println!("\nDouble Buffer for Streaming:");
    let mut db: DoubleBuffer<8> = DoubleBuffer::new();
    db.fill_inactive(&[100, 200, 300, 400, 500, 600, 700, 800]);
    println!("  Active buffer before swap: {:?}", db.active_buffer());
    db.swap();
    println!("  Active buffer after swap:  {:?}", db.active_buffer());

    // String Buffer
    println!("\nHeapless String Buffer:");
    let mut sb: StringBuffer<64> = StringBuffer::new();
    sb.push_str("Sensor reading: ");
    sb.push_str("25.5°C");
    println!("  Buffer content: \"{}\"", sb.as_str());
    println!("  Length: {}/{}", sb.len(), sb.capacity());
}
