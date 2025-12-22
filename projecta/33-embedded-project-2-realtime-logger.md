# Real-Time Data Logger with Zero-Copy Buffers

### Problem Statement

Build a real-time data logger for embedded systems that captures high-frequency sensor data (ADC readings, serial data, or network packets) using DMA and zero-copy buffer techniques. Your logger must operate without heap allocation, handle buffer overflow gracefully, support multiple data sources, and provide both in-memory ring buffers and persistent storage options.

Your data logger should support:
- Zero-copy DMA transfers for ADC, UART, and SPI
- Lock-free ring buffers for producer-consumer patterns
- Double buffering for continuous data capture
- Static memory allocation (no heap)
- Timestamp synchronization across sources
- Export to storage (SD card, flash memory)

## Why Zero-Copy and Static Allocation Matter

### The Dynamic Allocation Problem

**The Problem**: Traditional data logging uses `Vec<T>` or dynamic buffers. On embedded systems, this creates problems:

```rust
// ❌ Problematic dynamic allocation
fn log_data(value: u16) {
    static mut LOG: Option<Vec<u16>> = None;
    unsafe {
        if LOG.is_none() {
            LOG = Some(Vec::new()); // Heap allocation!
        }
        LOG.as_mut().unwrap().push(value); // Can fail, fragments heap
    }
}

// Problems:
// 1. Heap allocation in interrupt context → crash
// 2. Vec::push can reallocate → unbounded latency
// 3. Fragmentation after hours of operation
// 4. Can't prove memory safety statically
```

**Real-world disaster**:
```
Medical device logs patient vitals:
├─ After 6 hours: heap fragmented
├─ Vec reallocation takes 50ms
├─ Misses critical heart rhythm event
└─ Patient data gap causes misdiagnosis
```

### Zero-Copy DMA: Direct Memory Access

**Traditional approach (CPU-intensive):**
```rust
// CPU reads ADC register in loop - wastes cycles
loop {
    while !adc.is_ready() {} // Busy wait
    let value = adc.read(); // CPU copies data
    buffer[i] = value;      // CPU writes to buffer
    i += 1;
}
// Problem: CPU does nothing but copy data!
```

**Zero-copy DMA approach:**
```rust
// DMA controller moves data while CPU does other work
static mut ADC_BUFFER: [u16; 1024] = [0; 1024];

// Configure DMA once
unsafe {
    adc_dma.set_memory(&mut ADC_BUFFER);
    adc_dma.start(); // DMA copies ADC → buffer automatically
}

// CPU is FREE to do other tasks!
// When DMA finishes → interrupt → process full buffer
```

**Performance comparison:**
```
Sampling 1000 ADC readings at 100kHz:

CPU-based:
├─ CPU cycles: ~50,000
├─ Power: High (CPU always active)
└─ Data processing: Blocked until sampling done

DMA-based:
├─ CPU cycles: ~500 (just setup/teardown)
├─ Power: Low (CPU sleeps during transfer)
└─ Data processing: Happens concurrently!

100x efficiency improvement!
```

### Static Allocation: Predictable Memory

**Why it matters:**
```rust
// ✓ Static allocation - known at compile time
static mut DATA_LOG: [u16; 4096] = [0; 4096];

// Benefits:
// 1. Zero runtime overhead
// 2. No allocation failures
// 3. Placed in specific memory regions (SRAM, DTCM, SRAM_D2)
// 4. Compiler verifies size at build time
// 5. Perfect for certification (DO-178C, IEC 62304)
```

**Memory placement for DMA:**
```rust
// STM32H7: DMA can only access certain SRAM regions
#[link_section = ".sram_d2"]  // ← Specific DMA-accessible RAM
static mut DMA_BUFFER: [u16; 2048] = [0; 2048];

// Without this: DMA fails silently or crashes
// With this: Guaranteed to work
```

## Use Cases

### 1. High-Speed Data Acquisition
- **Scientific instruments**: Oscilloscopes, logic analyzers (1 MSPS+)
- **Audio recording**: 48kHz stereo samples (192 KB/s)
- **Industrial sensors**: Multi-channel ADC logging
- **Challenge**: Can't miss samples, CPU can't keep up with direct I/O

### 2. Black Box / Flight Recorders
- **Aviation**: Record sensor data for post-incident analysis
- **Automotive**: Event Data Recorders (EDR) for crashes
- **Medical**: Continuous patient monitoring (ECG, SpO2)
- **Requirements**: Reliable even during system failures, no allocation

### 3. Network Packet Capture
- **Embedded firewalls**: Log packet headers at 1 Gbps
- **IDS systems**: Capture for forensics without dropping packets
- **IoT gateways**: Buffer telemetry during connectivity loss
- **Challenge**: Bursty traffic, need buffer flexibility

### 4. Real-Time Telemetry
- **Robotics**: Log motor controller data at 10kHz
- **Drones**: IMU fusion data (accelerometer, gyro, mag)
- **Racing**: CAN bus logging (engine, suspension, GPS)
- **Challenge**: Multiple concurrent data streams, precise timestamps

---

## Building the Project

### Milestone 1: Static Ring Buffer

**Goal**: Implement a fixed-capacity ring buffer with static allocation that supports concurrent single-producer single-consumer access without locks.

**Why we start here**: Ring buffers are the foundation of zero-copy logging. They provide bounded memory usage and constant-time operations—critical for real-time systems.

#### Architecture

**Structs:**
- `RingBuffer<T, const N: usize>` - Fixed-capacity ring buffer
  - **Field**: `buffer: [MaybeUninit<T>; N]` - Storage array (uninitialized for efficiency)
  - **Field**: `write_pos: AtomicUsize` - Write pointer (producer)
  - **Field**: `read_pos: AtomicUsize` - Read pointer (consumer)

**Functions:**
- `new() -> Self` - Create empty ring buffer
- `push(&self, value: T) -> Result<(), T>` - Add item (producer)
- `pop(&self) -> Option<T>` - Remove item (consumer)
- `len(&self) -> usize` - Current item count
- `capacity(&self) -> usize` - Maximum capacity
- `is_full(&self) -> bool` - Check if buffer is full
- `is_empty(&self) -> bool` - Check if buffer is empty

**Constants:**
- None (capacity is const generic `N`)

**Starter Code**:

```rust
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct RingBuffer<T, const N: usize> {
    buffer: UnsafeCell<[MaybeUninit<T>; N]>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
}

unsafe impl<T: Send, const N: usize> Sync for RingBuffer<T, N> {}
unsafe impl<T: Send, const N: usize> Send for RingBuffer<T, N> {}

impl<T, const N: usize> RingBuffer<T, N> {
    pub const fn new() -> Self {
        // TODO: Initialize buffer with MaybeUninit::uninit()
        // TODO: Set write_pos and read_pos to 0
        // HINT: Use array initialization: [MaybeUninit::uninit(); N]
        todo!("Implement RingBuffer::new")
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        // TODO: Load current write and read positions
        // TODO: Calculate next write position: (write_pos + 1) % N
        // TODO: Check if buffer is full: next_write == read_pos
        // TODO: If full, return Err(value)
        // TODO: Write value to buffer[write_pos] using ptr::write
        // TODO: Update write_pos atomically
        // TODO: Return Ok(())
        todo!("Implement push")
    }

    pub fn pop(&self) -> Option<T> {
        // TODO: Load current read and write positions
        // TODO: Check if empty: read_pos == write_pos
        // TODO: If empty, return None
        // TODO: Read value from buffer[read_pos] using ptr::read
        // TODO: Update read_pos atomically: (read_pos + 1) % N
        // TODO: Return Some(value)
        todo!("Implement pop")
    }

    pub fn len(&self) -> usize {
        // TODO: Calculate items in buffer
        // HINT: (write_pos - read_pos) % N, but handle wrapping correctly
        todo!("Implement len")
    }

    pub const fn capacity(&self) -> usize {
        N - 1 // Reserve one slot to distinguish full from empty
    }

    pub fn is_full(&self) -> bool {
        // TODO: Check if (write_pos + 1) % N == read_pos
        todo!("Implement is_full")
    }

    pub fn is_empty(&self) -> bool {
        // TODO: Check if write_pos == read_pos
        todo!("Implement is_empty")
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        // TODO: Drop all remaining items in buffer
        // HINT: Pop until empty
        while self.pop().is_some() {}
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let buffer: RingBuffer<u32, 4> = RingBuffer::new();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        assert_eq!(buffer.capacity(), 3); // N-1
    }

    #[test]
    fn test_push_pop() {
        let buffer: RingBuffer<u32, 8> = RingBuffer::new();

        buffer.push(10).unwrap();
        buffer.push(20).unwrap();
        buffer.push(30).unwrap();

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.pop(), Some(10));
        assert_eq!(buffer.pop(), Some(20));
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.pop(), Some(30));
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn test_full_buffer() {
        let buffer: RingBuffer<u32, 4> = RingBuffer::new();

        // Capacity is 3 (N-1)
        buffer.push(1).unwrap();
        buffer.push(2).unwrap();
        buffer.push(3).unwrap();

        assert!(buffer.is_full());
        assert_eq!(buffer.push(4), Err(4)); // Should fail
    }

    #[test]
    fn test_wrap_around() {
        let buffer: RingBuffer<u32, 4> = RingBuffer::new();

        // Fill buffer
        buffer.push(1).unwrap();
        buffer.push(2).unwrap();
        buffer.push(3).unwrap();

        // Pop two
        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.pop(), Some(2));

        // Push two more (wraps around)
        buffer.push(4).unwrap();
        buffer.push(5).unwrap();

        // Should read in order
        assert_eq!(buffer.pop(), Some(3));
        assert_eq!(buffer.pop(), Some(4));
        assert_eq!(buffer.pop(), Some(5));
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let buffer = Arc::new(RingBuffer::<u32, 256>::new());
        let producer = buffer.clone();
        let consumer = buffer.clone();

        let producer_thread = thread::spawn(move || {
            for i in 0..100 {
                while producer.push(i).is_err() {
                    thread::yield_now(); // Wait if full
                }
            }
        });

        let consumer_thread = thread::spawn(move || {
            let mut values = Vec::new();
            for _ in 0..100 {
                loop {
                    if let Some(val) = consumer.pop() {
                        values.push(val);
                        break;
                    }
                    thread::yield_now(); // Wait if empty
                }
            }
            values
        });

        producer_thread.join().unwrap();
        let values = consumer_thread.join().unwrap();

        assert_eq!(values.len(), 100);
        for (i, &val) in values.iter().enumerate() {
            assert_eq!(val, i as u32);
        }
    }
}
```

**Check Your Understanding**:
- Why use `MaybeUninit<T>` instead of `Option<T>`?
- Why is capacity `N-1` instead of `N`?
- How do atomics enable lock-free concurrent access?

---

#### Why Milestone 1 Isn't Enough

**Limitation**: The ring buffer works but is limited to single-producer single-consumer. Real data loggers need integration with DMA hardware for zero-copy transfers.

**What we're adding**: DMA integration with double buffering, where DMA fills one buffer while the CPU processes another.

**Improvement**:
- **Throughput**: DMA can sustain 10x higher data rates than CPU polling
- **Efficiency**: CPU freed for data processing instead of I/O
- **Determinism**: No missed samples due to CPU workload
- **Power**: CPU can sleep during DMA transfers

---

### Milestone 2: DMA Double Buffer for ADC

**Goal**: Implement double-buffered DMA for continuous ADC sampling, where DMA alternates between two buffers while the CPU processes the idle buffer.

**Why this milestone**: Double buffering is essential for continuous high-speed data capture. This teaches DMA configuration and interrupt-driven buffer swapping.

#### Architecture

**Structs:**
- `DmaAdcLogger<const N: usize>` - Double-buffered ADC logger
  - **Field**: `buffers: [[u16; N]; 2]` - Two static buffers
  - **Field**: `active_buffer: AtomicU8` - Which buffer DMA is writing (0 or 1)
  - **Field**: `processed_count: AtomicUsize` - Total samples processed

**Functions:**
- `new() -> Self` - Initialize logger
- `start_dma(&mut self, adc_dma: &mut AdcDma)` - Begin DMA transfers
- `on_dma_complete(&self) -> &[u16; N]` - Called from interrupt, returns full buffer
- `swap_buffers(&self)` - Switch active buffer
- `get_processing_buffer(&self) -> &[u16; N]` - Get buffer ready for processing
- `samples_logged(&self) -> usize` - Total samples captured

**Constants:**
- `DMA_BUFFER_SIZE: usize = 512` - Samples per buffer

**Starter Code**:

```rust
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use core::mem::MaybeUninit;

pub const DMA_BUFFER_SIZE: usize = 512;

// Place buffers in DMA-accessible memory
#[link_section = ".dma_data"]
static mut DMA_BUFFERS: [[u16; DMA_BUFFER_SIZE]; 2] = [[0; DMA_BUFFER_SIZE]; 2];

pub struct DmaAdcLogger {
    active_buffer: AtomicU8,
    processed_count: AtomicUsize,
}

impl DmaAdcLogger {
    pub const fn new() -> Self {
        // TODO: Initialize atomic fields
        todo!("Implement DmaAdcLogger::new")
    }

    /// Start DMA transfers (called once at initialization)
    pub unsafe fn start_dma(&self, adc_dma: &mut impl AdcDma) {
        // TODO: Get pointers to both buffers
        // TODO: Configure DMA to use buffer 0 initially
        // TODO: Enable DMA transfer complete interrupt
        // TODO: Start DMA
        todo!("Implement start_dma")
    }

    /// Called from DMA interrupt when buffer is full
    pub fn on_dma_complete(&self) -> &'static [u16] {
        // TODO: Get current active buffer index
        // TODO: Increment processed count
        // TODO: Swap to other buffer
        // TODO: Return slice to newly-filled buffer
        todo!("Implement on_dma_complete")
    }

    /// Switch DMA to other buffer
    fn swap_buffers(&self, adc_dma: &mut impl AdcDma) {
        // TODO: Get current buffer index
        // TODO: Calculate next buffer index (0 <-> 1)
        // TODO: Update active_buffer atomically
        // TODO: Reconfigure DMA to use new buffer
        todo!("Implement swap_buffers")
    }

    pub fn samples_logged(&self) -> usize {
        self.processed_count.load(Ordering::Relaxed)
    }

    pub fn active_buffer_index(&self) -> u8 {
        self.active_buffer.load(Ordering::Acquire)
    }
}

// Trait to abstract DMA hardware
pub trait AdcDma {
    fn set_memory_address(&mut self, addr: *mut u16, len: usize);
    fn enable_transfer_complete_interrupt(&mut self);
    fn start(&mut self);
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;

    struct MockAdcDma {
        memory_addr: RefCell<Option<*mut u16>>,
        memory_len: RefCell<usize>,
        started: RefCell<bool>,
        interrupt_enabled: RefCell<bool>,
    }

    impl MockAdcDma {
        fn new() -> Self {
            Self {
                memory_addr: RefCell::new(None),
                memory_len: RefCell::new(0),
                started: RefCell::new(false),
                interrupt_enabled: RefCell::new(false),
            }
        }

        fn simulate_transfer(&mut self, data: &[u16]) {
            // Simulate DMA writing to memory
            let addr = self.memory_addr.borrow().unwrap();
            let len = *self.memory_len.borrow();
            unsafe {
                core::ptr::copy_nonoverlapping(data.as_ptr(), addr, len.min(data.len()));
            }
        }
    }

    impl AdcDma for MockAdcDma {
        fn set_memory_address(&mut self, addr: *mut u16, len: usize) {
            *self.memory_addr.borrow_mut() = Some(addr);
            *self.memory_len.borrow_mut() = len;
        }

        fn enable_transfer_complete_interrupt(&mut self) {
            *self.interrupt_enabled.borrow_mut() = true;
        }

        fn start(&mut self) {
            *self.started.borrow_mut() = true;
        }
    }

    #[test]
    fn test_dma_logger_init() {
        let logger = DmaAdcLogger::new();
        assert_eq!(logger.active_buffer_index(), 0);
        assert_eq!(logger.samples_logged(), 0);
    }

    #[test]
    fn test_start_dma() {
        let logger = DmaAdcLogger::new();
        let mut mock_dma = MockAdcDma::new();

        unsafe {
            logger.start_dma(&mut mock_dma);
        }

        assert!(*mock_dma.started.borrow());
        assert!(*mock_dma.interrupt_enabled.borrow());
        assert!(mock_dma.memory_addr.borrow().is_some());
    }

    #[test]
    fn test_buffer_swap() {
        let logger = DmaAdcLogger::new();
        let mut mock_dma = MockAdcDma::new();

        unsafe {
            logger.start_dma(&mut mock_dma);
        }

        assert_eq!(logger.active_buffer_index(), 0);

        // Simulate DMA completion
        let buffer = logger.on_dma_complete();
        assert_eq!(buffer.len(), DMA_BUFFER_SIZE);
        assert_eq!(logger.samples_logged(), DMA_BUFFER_SIZE);

        // Buffer should have swapped
        assert_eq!(logger.active_buffer_index(), 1);
    }

    #[test]
    fn test_continuous_logging() {
        let logger = DmaAdcLogger::new();
        let mut mock_dma = MockAdcDma::new();

        unsafe {
            logger.start_dma(&mut mock_dma);
        }

        // Simulate 5 buffer completions
        for i in 0..5 {
            let test_data: Vec<u16> = (0..DMA_BUFFER_SIZE).map(|x| x as u16).collect();
            mock_dma.simulate_transfer(&test_data);

            let buffer = logger.on_dma_complete();
            assert_eq!(buffer.len(), DMA_BUFFER_SIZE);
            assert_eq!(logger.samples_logged(), (i + 1) * DMA_BUFFER_SIZE);
        }

        assert_eq!(logger.samples_logged(), 5 * DMA_BUFFER_SIZE);
    }
}
```

**Check Your Understanding**:
- Why use two buffers instead of one?
- What happens if CPU processing is slower than DMA filling?
- Why place buffers in a specific memory section?

---

#### Why Milestone 2 Isn't Enough

**Limitation**: DMA captures data efficiently, but we can only process one buffer at a time. If processing is slow, we lose data. We need a queue of buffers.

**What we're adding**: A buffer pool system where multiple buffers can be queued, processed asynchronously, and recycled.

**Improvement**:
- **Resilience**: Can handle processing delays without data loss
- **Throughput**: Multiple buffers in flight increases effective bandwidth
- **Flexibility**: Different processing rates for different data types
- **Architecture**: Producer-consumer with buffer reuse

---

### Milestone 3: Buffer Pool and Queue System

**Goal**: Create a pool of reusable buffers and a queue system to decouple data capture from processing, preventing data loss when processing is slower than capture.

**Why this milestone**: Real systems need elasticity. This milestone teaches object pooling and queue-based architectures for embedded systems.

#### Architecture

**Structs:**
- `BufferPool<T, const N: usize, const POOL_SIZE: usize>` - Pool of reusable buffers
  - **Field**: `buffers: [MaybeUninit<[T; N]>; POOL_SIZE]` - Buffer storage
  - **Field**: `free_list: RingBuffer<usize, POOL_SIZE>` - Indices of free buffers
  - **Field**: `initialized: AtomicBool` - Pool initialization state

- `BufferHandle<'a, T, const N: usize>` - RAII handle to borrowed buffer
  - **Field**: `buffer: &'a mut [T; N]` - Buffer reference
  - **Field**: `index: usize` - Buffer index for return
  - **Field**: `pool: &'a BufferPool<T, N, POOL_SIZE>` - Parent pool

**Functions:**
- `BufferPool::new() -> Self` - Create buffer pool
- `init(&mut self)` - Initialize all buffers (must call before use)
- `acquire(&self) -> Option<BufferHandle>` - Get free buffer
- `release(&self, index: usize)` - Return buffer to pool
- `available_count(&self) -> usize` - Free buffer count

**Starter Code**:

```rust
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};
use core::ops::{Deref, DerefMut};

pub struct BufferPool<T, const N: usize, const POOL_SIZE: usize> {
    buffers: [MaybeUninit<[T; N]>; POOL_SIZE],
    free_list: RingBuffer<usize, POOL_SIZE>,
    initialized: AtomicBool,
}

impl<T, const N: usize, const POOL_SIZE: usize> BufferPool<T, N, POOL_SIZE> {
    pub const fn new() -> Self {
        // TODO: Initialize buffers as MaybeUninit
        // TODO: Create free_list ring buffer
        // TODO: Set initialized to false
        todo!("Implement BufferPool::new")
    }

    /// Initialize the pool (must be called before use)
    pub fn init(&mut self) where T: Default + Copy {
        // TODO: Check if already initialized
        // TODO: Initialize each buffer with default values
        // TODO: Push all buffer indices (0..POOL_SIZE) to free_list
        // TODO: Set initialized to true
        todo!("Implement init")
    }

    /// Acquire a buffer from the pool
    pub fn acquire(&self) -> Option<BufferHandle<'_, T, N, POOL_SIZE>> {
        // TODO: Check if initialized
        // TODO: Pop an index from free_list
        // TODO: Get mutable reference to buffer at that index
        // TODO: Return BufferHandle wrapping the buffer
        todo!("Implement acquire")
    }

    /// Release a buffer back to the pool
    fn release(&self, index: usize) {
        // TODO: Push index back to free_list
        // TODO: Handle case where push fails (pool corrupted)
        todo!("Implement release")
    }

    pub fn available_count(&self) -> usize {
        self.free_list.len()
    }

    pub fn total_capacity(&self) -> usize {
        POOL_SIZE
    }
}

/// RAII handle that automatically returns buffer to pool on drop
pub struct BufferHandle<'a, T, const N: usize, const POOL_SIZE: usize> {
    buffer: &'a mut [T; N],
    index: usize,
    pool: &'a BufferPool<T, N, POOL_SIZE>,
}

impl<'a, T, const N: usize, const POOL_SIZE: usize> BufferHandle<'a, T, N, POOL_SIZE> {
    fn new(buffer: &'a mut [T; N], index: usize, pool: &'a BufferPool<T, N, POOL_SIZE>) -> Self {
        Self { buffer, index, pool }
    }
}

impl<T, const N: usize, const POOL_SIZE: usize> Deref for BufferHandle<'_, T, N, POOL_SIZE> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        self.buffer
    }
}

impl<T, const N: usize, const POOL_SIZE: usize> DerefMut for BufferHandle<'_, T, N, POOL_SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer
    }
}

impl<T, const N: usize, const POOL_SIZE: usize> Drop for BufferHandle<'_, T, N, POOL_SIZE> {
    fn drop(&mut self) {
        // TODO: Return buffer to pool
        self.pool.release(self.index);
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_init() {
        let mut pool: BufferPool<u16, 256, 4> = BufferPool::new();
        pool.init();

        assert_eq!(pool.available_count(), 4);
        assert_eq!(pool.total_capacity(), 4);
    }

    #[test]
    fn test_acquire_release() {
        let mut pool: BufferPool<u16, 128, 8> = BufferPool::new();
        pool.init();

        // Acquire buffer
        let handle = pool.acquire().unwrap();
        assert_eq!(pool.available_count(), 7);

        // Drop returns to pool
        drop(handle);
        assert_eq!(pool.available_count(), 8);
    }

    #[test]
    fn test_exhaust_pool() {
        let mut pool: BufferPool<u32, 64, 3> = BufferPool::new();
        pool.init();

        let _h1 = pool.acquire().unwrap();
        let _h2 = pool.acquire().unwrap();
        let _h3 = pool.acquire().unwrap();

        assert_eq!(pool.available_count(), 0);

        // Should fail - pool exhausted
        assert!(pool.acquire().is_none());

        // Drop one, should be able to acquire again
        drop(_h1);
        assert_eq!(pool.available_count(), 1);
        let _h4 = pool.acquire().unwrap();
        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn test_buffer_handle_write() {
        let mut pool: BufferPool<u16, 4, 2> = BufferPool::new();
        pool.init();

        {
            let mut handle = pool.acquire().unwrap();
            handle[0] = 100;
            handle[1] = 200;
            handle[2] = 300;
            handle[3] = 400;

            assert_eq!(handle[0], 100);
            assert_eq!(handle[3], 400);
        } // handle dropped, buffer returned

        assert_eq!(pool.available_count(), 2);
    }

    #[test]
    fn test_multiple_acquire_release_cycles() {
        let mut pool: BufferPool<u8, 256, 4> = BufferPool::new();
        pool.init();

        for cycle in 0..10 {
            let mut handles = Vec::new();

            // Acquire all buffers
            for _ in 0..4 {
                handles.push(pool.acquire().unwrap());
            }
            assert_eq!(pool.available_count(), 0);

            // Release all
            handles.clear();
            assert_eq!(pool.available_count(), 4);
        }
    }
}
```

**Check Your Understanding**:
- Why use RAII (Drop trait) for BufferHandle?
- What happens if release() is called with wrong index?
- How does this pattern prevent memory leaks?

---

#### Why Milestone 3 Isn't Enough

**Limitation**: Buffer pool works but we still don't have a complete logging pipeline: capture → queue → process → store.

**What we're adding**: A full pipeline integrating DMA capture, buffer queuing, background processing, and timestamping.

**Improvement**:
- **Integration**: Complete end-to-end data flow
- **Timestamps**: Precise timing for each sample
- **Processing**: Compression, filtering, or analysis
- **Architecture**: Production-ready logging system

---

### Milestone 4: Complete Logging Pipeline

**Goal**: Build a complete data logging pipeline that integrates DMA capture, buffer queue, timestamp synchronization, and data processing tasks.

**Why this milestone**: Real systems need all components working together. This milestone teaches system integration and data flow architecture.

#### Architecture

**Structs:**
- `DataLogger<const BUFFER_SIZE: usize, const POOL_SIZE: usize>` - Complete logger
  - **Field**: `buffer_pool: BufferPool<Sample, BUFFER_SIZE, POOL_SIZE>` - Buffer management
  - **Field**: `pending_queue: RingBuffer<usize, POOL_SIZE>` - Buffers awaiting processing
  - **Field**: `stats: LoggerStats` - Performance metrics

- `Sample` - Single data sample with metadata
  - **Field**: `value: u16` - Sample value
  - **Field**: `timestamp_us: u64` - Microsecond timestamp
  - **Field**: `channel: u8` - ADC channel number

- `LoggerStats` - Performance counters
  - **Field**: `samples_captured: AtomicUsize` - Total samples captured
  - **Field**: `samples_processed: AtomicUsize` - Total samples processed
  - **Field**: `buffer_overflows: AtomicUsize` - Lost buffers due to full queue

**Functions:**
- `new() -> Self` - Create logger
- `on_dma_interrupt(&self, data: &[u16], timestamp: u64)` - DMA completion handler
- `process_next_buffer(&self) -> Option<ProcessedData>` - Process one queued buffer
- `get_stats(&self) -> LoggerStats` - Get performance counters

**Starter Code**:

```rust
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Default)]
pub struct Sample {
    pub value: u16,
    pub timestamp_us: u64,
    pub channel: u8,
}

#[derive(Clone, Copy)]
pub struct LoggerStats {
    pub samples_captured: usize,
    pub samples_processed: usize,
    pub buffer_overflows: usize,
    pub buffers_pending: usize,
}

pub struct DataLogger<const BUFFER_SIZE: usize, const POOL_SIZE: usize> {
    buffer_pool: BufferPool<Sample, BUFFER_SIZE, POOL_SIZE>,
    pending_queue: RingBuffer<usize, POOL_SIZE>,
    samples_captured: AtomicUsize,
    samples_processed: AtomicUsize,
    buffer_overflows: AtomicUsize,
}

impl<const BUFFER_SIZE: usize, const POOL_SIZE: usize> DataLogger<BUFFER_SIZE, POOL_SIZE> {
    pub fn new() -> Self {
        // TODO: Initialize all fields
        todo!("Implement DataLogger::new")
    }

    /// Called from DMA interrupt with new data
    pub fn on_dma_interrupt(&self, raw_data: &[u16], base_timestamp_us: u64, channel: u8) {
        // TODO: Acquire buffer from pool
        // TODO: If no buffer available, increment overflow counter and return
        // TODO: Copy data into buffer with timestamps
        // TODO: Calculate timestamp for each sample based on sample rate
        // TODO: Queue buffer index for processing
        // TODO: Update samples_captured counter
        todo!("Implement on_dma_interrupt")
    }

    /// Process one queued buffer (call from background task)
    pub fn process_next_buffer(&self) -> Option<ProcessedData> {
        // TODO: Pop buffer index from pending_queue
        // TODO: Get buffer from pool using index
        // TODO: Process data (e.g., filter, compress, store)
        // TODO: Update samples_processed counter
        // TODO: Buffer automatically returned to pool when handle drops
        // TODO: Return processed results
        todo!("Implement process_next_buffer")
    }

    pub fn get_stats(&self) -> LoggerStats {
        LoggerStats {
            samples_captured: self.samples_captured.load(Ordering::Relaxed),
            samples_processed: self.samples_processed.load(Ordering::Relaxed),
            buffer_overflows: self.buffer_overflows.load(Ordering::Relaxed),
            buffers_pending: self.pending_queue.len(),
        }
    }

    pub fn has_pending_data(&self) -> bool {
        !self.pending_queue.is_empty()
    }
}

#[derive(Debug)]
pub struct ProcessedData {
    pub min: u16,
    pub max: u16,
    pub avg: u16,
    pub sample_count: usize,
    pub duration_us: u64,
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_init() {
        let logger: DataLogger<256, 8> = DataLogger::new();
        let stats = logger.get_stats();

        assert_eq!(stats.samples_captured, 0);
        assert_eq!(stats.samples_processed, 0);
        assert_eq!(stats.buffer_overflows, 0);
        assert!(!logger.has_pending_data());
    }

    #[test]
    fn test_dma_to_queue_flow() {
        let logger: DataLogger<128, 4> = DataLogger::new();

        // Simulate DMA interrupt with data
        let test_data: Vec<u16> = (0..128).collect();
        logger.on_dma_interrupt(&test_data, 1000, 0);

        let stats = logger.get_stats();
        assert_eq!(stats.samples_captured, 128);
        assert_eq!(stats.buffers_pending, 1);
        assert!(logger.has_pending_data());
    }

    #[test]
    fn test_process_buffer() {
        let logger: DataLogger<64, 4> = DataLogger::new();

        // Capture data
        let test_data: Vec<u16> = vec![100, 200, 300, 400];
        logger.on_dma_interrupt(&test_data, 5000, 0);

        // Process
        let result = logger.process_next_buffer().unwrap();

        assert_eq!(result.sample_count, 4);
        assert_eq!(result.min, 100);
        assert_eq!(result.max, 400);
        assert_eq!(result.avg, 250);

        let stats = logger.get_stats();
        assert_eq!(stats.samples_processed, 4);
        assert_eq!(stats.buffers_pending, 0);
    }

    #[test]
    fn test_buffer_overflow() {
        let logger: DataLogger<32, 2> = DataLogger::new();

        // Fill pool + queue
        let data: Vec<u16> = vec![1; 32];
        logger.on_dma_interrupt(&data, 1000, 0);
        logger.on_dma_interrupt(&data, 2000, 0);

        // Third interrupt should overflow (no buffers left)
        logger.on_dma_interrupt(&data, 3000, 0);

        let stats = logger.get_stats();
        assert_eq!(stats.buffer_overflows, 1);
    }

    #[test]
    fn test_continuous_logging() {
        let logger: DataLogger<128, 8> = DataLogger::new();

        // Simulate continuous data capture and processing
        for i in 0..20 {
            let data: Vec<u16> = (i * 128..(i + 1) * 128).map(|x| x as u16).collect();
            logger.on_dma_interrupt(&data, i * 1000, 0);

            // Process every other buffer
            if i % 2 == 0 {
                logger.process_next_buffer();
            }
        }

        let stats = logger.get_stats();
        assert_eq!(stats.samples_captured, 20 * 128);
        assert!(stats.samples_processed > 0);
        assert!(stats.buffers_pending > 0);
    }
}
```

**Check Your Understanding**:
- Why separate capture and processing into different operations?
- How do timestamps get assigned to samples?
- What causes buffer_overflow and how to prevent it?

---

#### Why Milestone 4 Isn't Enough

**Limitation**: The logger works but data stays in memory. Real systems need persistent storage (SD card, flash) and the ability to export logs.

**What we're adding**: Storage backend abstraction and export functionality for writing logs to persistent media.

**Improvement**:
- **Persistence**: Data survives power loss
- **Capacity**: Store gigabytes of logs
- **Export**: Transfer logs for analysis
- **Abstraction**: Same code works with SD, SPI flash, or even network storage

---

### Milestone 5: Storage Backend Integration

**Goal**: Add storage abstraction and implement backends for different media (SD card via SPI, SPI flash, in-memory mock).

**Why this milestone**: Embedded systems need reliable data persistence. This milestone teaches storage abstractions and error handling for I/O operations.

#### Architecture

**Traits:**
- `StorageBackend` - Storage abstraction
  - **Method**: `fn write(&mut self, offset: u64, data: &[u8]) -> Result<(), StorageError>` - Write data
  - **Method**: `fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), StorageError>` - Read data
  - **Method**: `fn flush(&mut self) -> Result<(), StorageError>` - Ensure data persisted
  - **Method**: `fn capacity(&self) -> u64` - Storage capacity in bytes

**Structs:**
- `StorageLogger<S: StorageBackend, const BUFFER_SIZE: usize>` - Logger with storage
  - **Field**: `logger: DataLogger<BUFFER_SIZE, 8>` - Core logger
  - **Field**: `storage: S` - Storage backend
  - **Field**: `write_position: AtomicU64` - Current write offset

- `StorageError` - Storage operation errors
  - **Variant**: `IoError` - Read/write failed
  - **Variant**: `Full` - No space remaining
  - **Variant**: `Corrupted` - Data integrity check failed

**Functions:**
- `StorageLogger::new(storage: S) -> Self` - Create logger with storage
- `flush_to_storage(&mut self) -> Result<usize, StorageError>` - Write pending buffers
- `export_log(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, StorageError>` - Read stored data

**Starter Code**:

```rust
#[derive(Debug, Clone, Copy)]
pub enum StorageError {
    IoError,
    Full,
    Corrupted,
}

pub trait StorageBackend {
    fn write(&mut self, offset: u64, data: &[u8]) -> Result<(), StorageError>;
    fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), StorageError>;
    fn flush(&mut self) -> Result<(), StorageError>;
    fn capacity(&self) -> u64;
}

pub struct StorageLogger<S: StorageBackend, const BUFFER_SIZE: usize> {
    logger: DataLogger<BUFFER_SIZE, 8>,
    storage: S,
    write_position: AtomicU64,
}

impl<S: StorageBackend, const BUFFER_SIZE: usize> StorageLogger<S, BUFFER_SIZE> {
    pub fn new(storage: S) -> Self {
        Self {
            logger: DataLogger::new(),
            storage,
            write_position: AtomicU64::new(0),
        }
    }

    /// Flush pending buffers to storage
    pub fn flush_to_storage(&mut self) -> Result<usize, StorageError> {
        // TODO: Process all pending buffers
        // TODO: Serialize ProcessedData to bytes
        // TODO: Write to storage at current write_position
        // TODO: Update write_position
        // TODO: Return number of bytes written
        todo!("Implement flush_to_storage")
    }

    /// Export stored log data
    pub fn export_log(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, StorageError> {
        // TODO: Allocate buffer
        // TODO: Read from storage
        // TODO: Return data
        todo!("Implement export_log")
    }

    pub fn storage_used(&self) -> u64 {
        self.write_position.load(Ordering::Relaxed)
    }

    pub fn storage_available(&self) -> u64 {
        self.storage.capacity() - self.storage_used()
    }
}

// ===== Mock Storage for Testing =====

pub struct MemoryStorage {
    data: Vec<u8>,
    capacity: u64,
}

impl MemoryStorage {
    pub fn new(capacity: u64) -> Self {
        Self {
            data: vec![0; capacity as usize],
            capacity,
        }
    }
}

impl StorageBackend for MemoryStorage {
    fn write(&mut self, offset: u64, data: &[u8]) -> Result<(), StorageError> {
        // TODO: Check bounds
        // TODO: Copy data to internal buffer
        todo!("Implement MemoryStorage::write")
    }

    fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), StorageError> {
        // TODO: Check bounds
        // TODO: Copy from internal buffer to buf
        todo!("Implement MemoryStorage::read")
    }

    fn flush(&mut self) -> Result<(), StorageError> {
        Ok(()) // No-op for memory
    }

    fn capacity(&self) -> u64 {
        self.capacity
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage() {
        let mut storage = MemoryStorage::new(1024);

        let data = b"Hello, embedded storage!";
        storage.write(0, data).unwrap();

        let mut read_buf = vec![0u8; data.len()];
        storage.read(0, &mut read_buf).unwrap();

        assert_eq!(&read_buf, data);
    }

    #[test]
    fn test_storage_logger_init() {
        let storage = MemoryStorage::new(4096);
        let logger: StorageLogger<_, 256> = StorageLogger::new(storage);

        assert_eq!(logger.storage_used(), 0);
        assert_eq!(logger.storage_available(), 4096);
    }

    #[test]
    fn test_flush_to_storage() {
        let storage = MemoryStorage::new(8192);
        let mut logger: StorageLogger<_, 128> = StorageLogger::new(storage);

        // Simulate data capture
        let data: Vec<u16> = (0..128).collect();
        logger.logger.on_dma_interrupt(&data, 1000, 0);

        // Flush to storage
        let bytes_written = logger.flush_to_storage().unwrap();
        assert!(bytes_written > 0);
        assert_eq!(logger.storage_used(), bytes_written as u64);
    }

    #[test]
    fn test_export_log() {
        let storage = MemoryStorage::new(4096);
        let mut logger: StorageLogger<_, 64> = StorageLogger::new(storage);

        // Capture and flush
        let data = vec![100u16; 64];
        logger.logger.on_dma_interrupt(&data, 5000, 0);
        let written = logger.flush_to_storage().unwrap();

        // Export what we wrote
        let exported = logger.export_log(0, written).unwrap();
        assert_eq!(exported.len(), written);
    }

    #[test]
    fn test_storage_full() {
        let storage = MemoryStorage::new(256); // Small capacity
        let mut logger: StorageLogger<_, 128> = StorageLogger::new(storage);

        // Fill storage
        let data = vec![1u16; 128];
        logger.logger.on_dma_interrupt(&data, 1000, 0);
        logger.flush_to_storage().unwrap();

        // Try to write more - should fail
        logger.logger.on_dma_interrupt(&data, 2000, 0);
        let result = logger.flush_to_storage();
        assert!(result.is_err());
    }
}
```

**Check Your Understanding**:
- Why abstract storage behind a trait?
- How would you implement wear leveling for flash storage?
- What happens if storage write fails mid-flush?

---

#### Why Milestone 5 Isn't Enough

**Limitation**: The logger is still blocking - processing happens synchronously. Modern embedded systems use async/await for efficient concurrency.

**What we're adding**: Embassy async integration for concurrent logging, processing, and storage operations without blocking.

**Improvement**:
- **Efficiency**: Process multiple buffers concurrently
- **Responsiveness**: Storage writes don't block data capture
- **Power**: CPU sleeps during I/O operations
- **Scalability**: Handle multiple concurrent data sources

---

### Milestone 6: Async Embassy Integration

**Goal**: Refactor the logger to use Embassy's async runtime, enabling concurrent data capture, processing, and storage without blocking.

**Why this milestone**: Async/await is the modern approach to embedded concurrency. This milestone demonstrates zero-cost async patterns.

#### Architecture

**Key Changes:**
- Replace synchronous processing with async tasks
- Use Embassy channels for buffer passing
- Add async storage backend trait
- Implement concurrent capture/process/store pipeline

**Structs:**
- Same as Milestone 5, but with async methods

**Starter Code**:

```rust
use embassy_executor::Spawner;
use embassy_sync::channel::{Channel, Sender, Receiver};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};

// Channel for passing buffer indices from DMA to processor
static BUFFER_QUEUE: Channel<NoopRawMutex, usize, 8> = Channel::new();

#[embassy_executor::task]
async fn capture_task(logger: &'static DataLogger<512, 8>) {
    // TODO: Simulate DMA interrupts (in real system, this would be actual interrupts)
    // TODO: Capture data into buffers
    // TODO: Send buffer indices to BUFFER_QUEUE
    loop {
        // Simulate data capture
        Timer::after(Duration::from_millis(10)).await;

        // In real system: DMA interrupt would call logger.on_dma_interrupt()
        // and send buffer index to channel
    }
}

#[embassy_executor::task]
async fn process_task(
    logger: &'static DataLogger<512, 8>,
    sender: Sender<'static, NoopRawMutex, ProcessedData, 4>,
) {
    let receiver = BUFFER_QUEUE.receiver();

    loop {
        // Wait for buffer from capture task
        let buffer_idx = receiver.receive().await;

        // Process buffer (this is where your DSP, filtering, etc. happens)
        if let Some(processed) = logger.process_next_buffer() {
            sender.send(processed).await;
        }
    }
}

#[embassy_executor::task]
async fn storage_task(
    mut storage_logger: StorageLogger<MemoryStorage, 512>,
    receiver: Receiver<'static, NoopRawMutex, ProcessedData, 4>,
) {
    loop {
        // Wait for processed data
        let processed = receiver.receive().await;

        // Write to storage (async I/O)
        match storage_logger.flush_to_storage() {
            Ok(bytes) => {
                defmt::info!("Wrote {} bytes to storage", bytes);
            }
            Err(e) => {
                defmt::error!("Storage error: {:?}", e);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Real-Time Data Logger starting...");

    // Initialize logger and storage
    static mut LOGGER: Option<DataLogger<512, 8>> = None;
    static mut STORAGE_LOGGER: Option<StorageLogger<MemoryStorage, 512>> = None;

    unsafe {
        LOGGER = Some(DataLogger::new());
        STORAGE_LOGGER = Some(StorageLogger::new(MemoryStorage::new(1024 * 1024)));
    }

    static PROCESSED_CHANNEL: Channel<NoopRawMutex, ProcessedData, 4> = Channel::new();

    // Spawn tasks
    spawner.spawn(capture_task(unsafe { LOGGER.as_ref().unwrap() })).unwrap();
    spawner.spawn(process_task(
        unsafe { LOGGER.as_ref().unwrap() },
        PROCESSED_CHANNEL.sender(),
    )).unwrap();
    spawner.spawn(storage_task(
        unsafe { STORAGE_LOGGER.take().unwrap() },
        PROCESSED_CHANNEL.receiver(),
    )).unwrap();

    defmt::info!("All tasks spawned");

    // Main loop: monitor system
    loop {
        Timer::after(Duration::from_secs(5)).await;

        let stats = unsafe { LOGGER.as_ref().unwrap().get_stats() };
        defmt::info!(
            "Stats: captured={} processed={} overflows={} pending={}",
            stats.samples_captured,
            stats.samples_processed,
            stats.buffer_overflows,
            stats.buffers_pending
        );
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod async_tests {
    use super::*;

    #[embassy_executor::test]
    async fn test_async_capture_process() {
        static LOGGER: DataLogger<256, 4> = DataLogger::new();
        static CHANNEL: Channel<NoopRawMutex, usize, 4> = Channel::new();

        // Spawn processor task
        spawner.spawn(async {
            let receiver = CHANNEL.receiver();
            let mut processed_count = 0;

            for _ in 0..3 {
                receiver.receive().await;
                LOGGER.process_next_buffer();
                processed_count += 1;
            }

            assert_eq!(processed_count, 3);
        }).unwrap();

        // Simulate captures
        let sender = CHANNEL.sender();
        for i in 0..3 {
            let data = vec![i as u16; 256];
            LOGGER.on_dma_interrupt(&data, i * 1000, 0);
            sender.send(i).await;
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
```

**Check Your Understanding**:
- How does async improve efficiency compared to threads?
- Why use channels instead of shared state?
- What's the trade-off between channel capacity and memory usage?

---

## Complete Working Example

See full implementation in the repository: `examples/realtime_logger.rs`

Key features demonstrated:
- Zero-copy DMA transfers
- Lock-free ring buffers
- Buffer pool management
- Async concurrent pipeline
- Storage abstraction
- Performance monitoring

**Expected Performance:**
- 1 MSPS ADC sampling with 0 overflows
- <5% CPU usage during continuous logging
- <50μs latency from capture to storage queue

---

## Testing Your Implementation

### Unit Tests
```bash
cargo test --lib
```

### Hardware Integration Tests
```bash
# With STM32 Discovery board
cargo test --features stm32f4 --target thumbv7em-none-eabihf

# With Raspberry Pi
cargo test --features rpi --target aarch64-unknown-linux-gnu
```

### Performance Benchmarks
```bash
cargo bench --bench logger_throughput
```

## Extensions

1. **Compression**: Add real-time compression (LZ4, Delta encoding)
2. **Multiple channels**: Log from multiple ADC channels simultaneously
3. **Triggers**: Implement pre-trigger buffering for event capture
4. **Circular storage**: Overwrite oldest data when storage full
5. **Network export**: Stream logs over TCP/UDP
6. **Power management**: Sleep between samples, wake on interrupt

This project showcases the core patterns of efficient embedded data logging: zero-copy, static allocation, lock-free concurrency, and async I/O.
