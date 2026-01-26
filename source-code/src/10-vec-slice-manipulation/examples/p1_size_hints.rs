//! Pattern 1: Capacity Management
//! Example: Use Iterator Size Hints
//!
//! Run with: cargo run --example p1_size_hints

fn main() {
    println!("=== Iterator Size Hints ===\n");

    // Basic size_hint demonstration
    let vec = vec![1, 2, 3, 4, 5];
    let iter = vec.iter();
    let (lower, upper) = iter.size_hint();
    println!("Vec iterator size_hint: ({}, {:?})", lower, upper);

    // Filter iterator (unknown upper bound)
    let filtered = vec.iter().filter(|&&x| x % 2 == 0);
    let (lower, upper) = filtered.size_hint();
    println!("Filtered iterator size_hint: ({}, {:?})", lower, upper);

    // Custom collection using size_hint
    fn collect_filtered(items: impl Iterator<Item = i32>) -> Vec<i32> {
        let (lower, upper) = items.size_hint();

        let mut result = if let Some(upper) = upper {
            Vec::with_capacity(upper)
        } else {
            Vec::with_capacity(lower)
        };

        result.extend(items);
        result
    }

    println!("\n=== Custom Collection with Size Hints ===\n");

    // Known size
    let data = 0..100;
    println!("Range 0..100:");
    println!("  size_hint: {:?}", data.clone().size_hint());

    let collected = collect_filtered(data);
    println!("  Collected: {} items, capacity: {}", collected.len(), collected.capacity());

    // Unknown size (filter)
    let data = (0..100).filter(|x| x % 2 == 0);
    println!("\nFiltered range (even numbers):");
    println!("  size_hint: {:?}", data.clone().size_hint());

    let collected = collect_filtered(data);
    println!("  Collected: {} items, capacity: {}", collected.len(), collected.capacity());

    // Demonstrate how collect() uses size_hint
    println!("\n=== Standard collect() Uses Size Hints ===\n");

    let range: Vec<i32> = (0..1000).collect();
    println!("Collected range 0..1000:");
    println!("  len: {}, capacity: {}", range.len(), range.capacity());
    println!("  (collect optimally pre-allocates for ExactSizeIterator)");

    // Chain iterators affect size_hint
    println!("\n=== Chained Iterators ===\n");

    let a = vec![1, 2, 3];
    let b = vec![4, 5];
    let chained = a.iter().chain(b.iter());
    println!("Chained [1,2,3] + [4,5]:");
    println!("  size_hint: {:?}", chained.clone().size_hint());

    // Flat map (unknown size)
    let nested = vec![vec![1, 2], vec![3, 4, 5]];
    let flat = nested.iter().flat_map(|v| v.iter());
    println!("\nFlat map over [[1,2], [3,4,5]]:");
    println!("  size_hint: {:?}", flat.size_hint());

    // Custom ExactSizeIterator
    println!("\n=== ExactSizeIterator ===\n");

    struct CountingIter {
        current: usize,
        max: usize,
    }

    impl Iterator for CountingIter {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current < self.max {
                let val = self.current;
                self.current += 1;
                Some(val)
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.max - self.current;
            (remaining, Some(remaining))
        }
    }

    impl ExactSizeIterator for CountingIter {}

    let counter = CountingIter { current: 0, max: 10 };
    println!("Custom ExactSizeIterator:");
    println!("  len(): {}", counter.len());
    println!("  size_hint: {:?}", counter.size_hint());

    let collected: Vec<_> = counter.collect();
    println!("  Collected: {} items", collected.len());

    println!("\n=== Key Points ===");
    println!("1. size_hint() returns (lower_bound, Option<upper_bound>)");
    println!("2. collect() uses size_hint for optimal pre-allocation");
    println!("3. Filter/flat_map lose upper bound information");
    println!("4. Implement ExactSizeIterator when exact size is known");
}
