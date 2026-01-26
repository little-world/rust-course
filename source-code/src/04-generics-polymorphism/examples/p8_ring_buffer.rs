//! Pattern 8: Const Generics
//! Example: Fixed-Size Ring Buffer
//!
//! Run with: cargo run --example p8_ring_buffer

#[derive(Debug)]
struct RingBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    head: usize, // Read position
    tail: usize, // Write position
    len: usize,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    fn new() -> Self {
        const { assert!(N > 0, "RingBuffer requires N > 0") }
        RingBuffer {
            buffer: [None; N],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    fn capacity(&self) -> usize {
        N
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn is_full(&self) -> bool {
        self.len == N
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            Err(value) // Buffer full
        } else {
            self.buffer[self.tail] = Some(value);
            self.tail = (self.tail + 1) % N;
            self.len += 1;
            Ok(())
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = self.buffer[self.head].take();
        self.head = (self.head + 1) % N;
        self.len -= 1;
        value
    }

    fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            self.buffer[self.head].as_ref()
        }
    }

    // Force push: overwrites oldest if full
    fn push_overwrite(&mut self, value: T) -> Option<T> {
        let evicted = if self.len == N {
            let old = self.buffer[self.head].take();
            self.head = (self.head + 1) % N;
            self.len -= 1;
            old
        } else {
            None
        };

        self.buffer[self.tail] = Some(value);
        self.tail = (self.tail + 1) % N;
        self.len += 1;

        evicted
    }

    fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = None;
        }
        self.head = 0;
        self.tail = 0;
        self.len = 0;
    }
}

// Iterator over the ring buffer
impl<T: Copy, const N: usize> RingBuffer<T, N> {
    fn iter(&self) -> impl Iterator<Item = T> + '_ {
        (0..self.len).map(move |i| {
            let index = (self.head + i) % N;
            self.buffer[index].unwrap()
        })
    }
}

fn main() {
    println!("=== Basic Ring Buffer Operations ===");
    let mut buf: RingBuffer<i32, 3> = RingBuffer::new();

    println!("Created RingBuffer<i32, 3>");
    println!("  capacity: {}", buf.capacity());
    println!("  len: {}", buf.len());
    println!("  is_empty: {}", buf.is_empty());

    println!("\n=== Push Operations ===");
    println!("push(1): {:?}", buf.push(1));
    println!("push(2): {:?}", buf.push(2));
    println!("push(3): {:?}", buf.push(3));
    println!("push(4): {:?}", buf.push(4)); // Full!

    println!("\nBuffer state:");
    println!("  len: {}", buf.len());
    println!("  is_full: {}", buf.is_full());
    println!("  contents: {:?}", buf.iter().collect::<Vec<_>>());

    println!("\n=== Pop Operations ===");
    println!("pop(): {:?}", buf.pop());
    println!("pop(): {:?}", buf.pop());
    println!("  len: {}", buf.len());
    println!("  contents: {:?}", buf.iter().collect::<Vec<_>>());

    println!("\n=== Push After Pop (Wrap Around) ===");
    buf.push(10).unwrap();
    buf.push(20).unwrap();
    println!("After push(10), push(20):");
    println!("  contents: {:?}", buf.iter().collect::<Vec<_>>());

    println!("\n=== Push Overwrite ===");
    let mut buf2: RingBuffer<char, 3> = RingBuffer::new();
    buf2.push('a').unwrap();
    buf2.push('b').unwrap();
    buf2.push('c').unwrap();
    println!("Buffer: {:?}", buf2.iter().collect::<Vec<_>>());

    let evicted = buf2.push_overwrite('d');
    println!("push_overwrite('d') evicted: {:?}", evicted);
    println!("Buffer: {:?}", buf2.iter().collect::<Vec<_>>());

    let evicted = buf2.push_overwrite('e');
    println!("push_overwrite('e') evicted: {:?}", evicted);
    println!("Buffer: {:?}", buf2.iter().collect::<Vec<_>>());

    println!("\n=== Peek ===");
    println!("peek(): {:?}", buf2.peek());
    println!("(doesn't remove the item)");
    println!("peek(): {:?}", buf2.peek());

    println!("\n=== Clear ===");
    buf2.clear();
    println!("After clear:");
    println!("  len: {}", buf2.len());
    println!("  is_empty: {}", buf2.is_empty());

    println!("\n=== Const Generic Benefits ===");
    println!("Capacity is part of the type:");
    println!("  - RingBuffer<i32, 3> and RingBuffer<i32, 10> are different types");
    println!("  - No heap allocation needed");
    println!("  - Capacity known at compile time");
    println!("  - Size validated at compile time (N > 0)");
}
