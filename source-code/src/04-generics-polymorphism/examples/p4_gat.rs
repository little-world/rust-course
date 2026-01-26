//! Pattern 4: Associated Types vs Generic Parameters
//! Example: Generic Associated Types (GAT)
//!
//! Run with: cargo run --example p4_gat

// GAT: Associated type with its own generic parameter
trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

// A simple lending iterator that yields references to its internal buffer
struct RefIterator<'data> {
    data: &'data [i32],
    pos: usize,
}

impl<'data> LendingIterator for RefIterator<'data> {
    type Item<'a> = &'a i32 where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.pos < self.data.len() {
            let item = &self.data[self.pos];
            self.pos += 1;
            Some(item)
        } else {
            None
        }
    }
}

// Another GAT example: Borrowing container
trait BorrowingContainer {
    type Borrowed<'a>
    where
        Self: 'a;

    fn borrow(&self) -> Self::Borrowed<'_>;
}

struct MyData {
    values: Vec<i32>,
}

impl BorrowingContainer for MyData {
    type Borrowed<'a> = &'a [i32] where Self: 'a;

    fn borrow(&self) -> Self::Borrowed<'_> {
        &self.values
    }
}

// GAT for async-like patterns (simplified, synchronous version)
trait AsyncIterator {
    type Item<'a>
    where
        Self: 'a;

    fn poll_next(&mut self) -> Option<Self::Item<'_>>;
}

struct AsyncNumbers {
    current: i32,
    max: i32,
}

impl AsyncIterator for AsyncNumbers {
    type Item<'a> = i32 where Self: 'a;

    fn poll_next(&mut self) -> Option<Self::Item<'_>> {
        if self.current < self.max {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
        }
    }
}

fn main() {
    println!("=== Basic GAT: LendingIterator ===");
    // Usage: GAT enables returning references tied to the borrow
    let data = vec![10, 20, 30, 40, 50];
    let mut iter = RefIterator {
        data: &data,
        pos: 0,
    };

    println!("Iterating with LendingIterator:");
    while let Some(item) = iter.next() {
        println!("  Got: {}", item);
    }

    println!("\n=== BorrowingContainer GAT ===");
    let container = MyData {
        values: vec![1, 2, 3, 4, 5],
    };

    let borrowed = container.borrow();
    println!("Borrowed slice: {:?}", borrowed);

    // Can borrow again
    let borrowed2 = container.borrow();
    println!("Borrowed again: {:?}", borrowed2);

    println!("\n=== AsyncIterator GAT (Simplified) ===");
    let mut async_nums = AsyncNumbers { current: 1, max: 5 };

    println!("Polling AsyncIterator:");
    while let Some(n) = async_nums.poll_next() {
        println!("  Polled: {}", n);
    }

    println!("\n=== Why GATs Matter ===");
    println!("Without GATs, you couldn't have:");
    println!("  - Lending iterators that yield borrows from themselves");
    println!("  - Async traits with borrowed data");
    println!("  - Containers that return references with flexible lifetimes");
    println!("\nGATs add generic parameters to associated types:");
    println!("  type Item<'a> where Self: 'a;");
    println!("\nThis allows the lifetime to vary per method call,");
    println!("rather than being fixed when implementing the trait.");
}
