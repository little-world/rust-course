//! Pattern 6: Advanced Slice Patterns
//! Example: Circular Buffer Implementation
//!
//! Run with: cargo run --example p6_circular_buffer

fn main() {
    println!("=== Circular Buffer Implementation ===\n");

    struct CircularBuffer<T> {
        data: Vec<T>,
        head: usize,   // Read position
        tail: usize,   // Write position
        size: usize,   // Current number of elements
    }

    impl<T: Default + Clone + std::fmt::Debug> CircularBuffer<T> {
        fn new(capacity: usize) -> Self {
            CircularBuffer {
                data: vec![T::default(); capacity],
                head: 0,
                tail: 0,
                size: 0,
            }
        }

        fn capacity(&self) -> usize {
            self.data.len()
        }

        fn len(&self) -> usize {
            self.size
        }

        fn is_empty(&self) -> bool {
            self.size == 0
        }

        fn is_full(&self) -> bool {
            self.size == self.data.len()
        }

        fn push(&mut self, item: T) -> Option<T> {
            let evicted = if self.is_full() {
                let old = std::mem::take(&mut self.data[self.head]);
                self.head = (self.head + 1) % self.data.len();
                Some(old)
            } else {
                self.size += 1;
                None
            };

            self.data[self.tail] = item;
            self.tail = (self.tail + 1) % self.data.len();

            evicted
        }

        fn pop(&mut self) -> Option<T> {
            if self.is_empty() {
                None
            } else {
                let item = std::mem::take(&mut self.data[self.head]);
                self.head = (self.head + 1) % self.data.len();
                self.size -= 1;
                Some(item)
            }
        }

        fn peek(&self) -> Option<&T> {
            if self.is_empty() {
                None
            } else {
                Some(&self.data[self.head])
            }
        }

        fn as_slices(&self) -> (&[T], &[T]) {
            if self.is_empty() {
                return (&[], &[]);
            }

            if self.head < self.tail {
                (&self.data[self.head..self.tail], &[])
            } else {
                (&self.data[self.head..], &self.data[..self.tail])
            }
        }

        fn iter(&self) -> impl Iterator<Item = &T> {
            let (first, second) = self.as_slices();
            first.iter().chain(second.iter())
        }
    }

    // Basic operations
    println!("=== Basic Operations ===\n");

    let mut buffer: CircularBuffer<i32> = CircularBuffer::new(5);
    println!("Created buffer with capacity {}", buffer.capacity());

    for i in 1..=5 {
        buffer.push(i);
        println!("Pushed {}: len={}", i, buffer.len());
    }

    println!("\nBuffer is full: {}", buffer.is_full());
    println!("Contents: {:?}", buffer.iter().collect::<Vec<_>>());

    // Overwrite oldest
    println!("\n=== Overwriting Oldest Elements ===\n");

    for i in 6..=8 {
        let evicted = buffer.push(i);
        println!("Pushed {}, evicted: {:?}", i, evicted);
    }

    println!("Contents: {:?}", buffer.iter().collect::<Vec<_>>());

    // Pop elements
    println!("\n=== Popping Elements ===\n");

    while !buffer.is_empty() {
        let item = buffer.pop();
        println!("Popped: {:?}, len={}", item, buffer.len());
    }

    // as_slices demonstration
    println!("\n=== as_slices() Demonstration ===\n");

    let mut buffer: CircularBuffer<char> = CircularBuffer::new(5);

    // Push some elements
    for c in ['A', 'B', 'C'] {
        buffer.push(c);
    }

    // Pop one
    buffer.pop();

    // Push more (wraps around)
    for c in ['D', 'E', 'F'] {
        buffer.push(c);
    }

    let (first, second) = buffer.as_slices();
    println!("First slice:  {:?}", first);
    println!("Second slice: {:?}", second);
    println!("Full contents: {:?}", buffer.iter().collect::<Vec<_>>());

    // Use case: Rolling average
    println!("\n=== Use Case: Rolling Average ===\n");

    struct RollingAverage {
        buffer: CircularBuffer<f64>,
        sum: f64,
    }

    impl RollingAverage {
        fn new(window_size: usize) -> Self {
            RollingAverage {
                buffer: CircularBuffer::new(window_size),
                sum: 0.0,
            }
        }

        fn add(&mut self, value: f64) {
            if let Some(evicted) = self.buffer.push(value) {
                self.sum -= evicted;
            }
            self.sum += value;
        }

        fn average(&self) -> Option<f64> {
            if self.buffer.is_empty() {
                None
            } else {
                Some(self.sum / self.buffer.len() as f64)
            }
        }
    }

    let mut avg = RollingAverage::new(3);

    for value in [10.0, 20.0, 30.0, 40.0, 50.0] {
        avg.add(value);
        println!("Added {:.0}: average = {:?}", value, avg.average());
    }

    // Use case: Event log
    println!("\n=== Use Case: Bounded Event Log ===\n");

    #[derive(Default, Clone, Debug)]
    struct Event {
        timestamp: u64,
        message: String,
    }

    let mut log: CircularBuffer<Event> = CircularBuffer::new(3);

    for i in 1..=5 {
        let evicted = log.push(Event {
            timestamp: i * 100,
            message: format!("Event {}", i),
        });
        if let Some(e) = evicted {
            println!("Evicted: {:?}", e);
        }
    }

    println!("\nCurrent log:");
    for event in log.iter() {
        println!("  [{:4}] {}", event.timestamp, event.message);
    }

    println!("\n=== Key Points ===");
    println!("1. Fixed capacity, overwrites oldest on overflow");
    println!("2. O(1) push/pop using modular arithmetic");
    println!("3. as_slices returns two parts when wrapped");
    println!("4. Perfect for rolling windows, logs, streams");
    println!("5. No shifting elements - just move head/tail");
}
