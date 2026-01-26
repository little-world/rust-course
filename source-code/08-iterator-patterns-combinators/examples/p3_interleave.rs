//! Pattern 3: Advanced Iterator Composition
//! Example: Interleaving Iterators
//!
//! Run with: cargo run --example p3_interleave

/// A custom iterator that alternates between two input iterators.
struct Interleave<I, J> {
    a: I,
    b: J,
    use_a: bool,
}

impl<I, J> Iterator for Interleave<I, J>
where
    I: Iterator,
    J: Iterator<Item = I::Item>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.use_a {
            self.use_a = false;
            self.a.next().or_else(|| self.b.next())
        } else {
            self.use_a = true;
            self.b.next().or_else(|| self.a.next())
        }
    }
}

/// Helper function to create an Interleave iterator.
fn interleave<I, J>(
    a: I,
    b: J,
) -> Interleave<I::IntoIter, J::IntoIter>
where
    I: IntoIterator,
    J: IntoIterator<Item = I::Item>,
{
    Interleave {
        a: a.into_iter(),
        b: b.into_iter(),
        use_a: true,
    }
}

/// Round-robin interleave of multiple iterators.
fn round_robin<I, T>(iters: Vec<I>) -> impl Iterator<Item = T>
where
    I: Iterator<Item = T>,
{
    let mut iters: Vec<_> = iters.into_iter().map(|i| i.peekable()).collect();
    let mut current = 0;

    std::iter::from_fn(move || {
        if iters.is_empty() {
            return None;
        }

        // Find next non-empty iterator
        let start = current;
        loop {
            if iters[current].peek().is_some() {
                let item = iters[current].next();
                current = (current + 1) % iters.len();
                return item;
            }
            current = (current + 1) % iters.len();
            if current == start {
                // All iterators exhausted
                return None;
            }
        }
    })
}

fn main() {
    println!("=== Interleaving Iterators ===\n");

    // Usage: alternate elements from two iterators
    let merged: Vec<_> = interleave([1, 3, 5], [2, 4, 6]).collect();
    println!("Interleave [1,3,5] and [2,4,6]: {:?}", merged);
    // [1, 2, 3, 4, 5, 6]

    // Unequal lengths
    let merged2: Vec<_> = interleave([1, 3, 5, 7, 9], [2, 4]).collect();
    println!("Interleave [1,3,5,7,9] and [2,4]: {:?}", merged2);
    // [1, 2, 3, 4, 5, 7, 9]

    // With strings
    let merged3: Vec<_> = interleave(["a", "c", "e"], ["b", "d", "f"]).collect();
    println!("Interleave ['a','c','e'] and ['b','d','f']: {:?}", merged3);

    println!("\n=== How It Works ===");
    println!("1. Start with use_a = true");
    println!("2. If use_a, try to get from a, fallback to b");
    println!("3. If !use_a, try to get from b, fallback to a");
    println!("4. Toggle use_a each iteration");

    println!("\n=== Round-Robin Multiple Iterators ===");
    let iters = vec![
        vec![1, 4, 7].into_iter(),
        vec![2, 5, 8].into_iter(),
        vec![3, 6, 9].into_iter(),
    ];
    let result: Vec<_> = round_robin(iters).collect();
    println!("Round-robin [[1,4,7], [2,5,8], [3,6,9]]: {:?}", result);
    // [1, 2, 3, 4, 5, 6, 7, 8, 9]

    // With unequal lengths
    println!("\n=== Unequal Length Round-Robin ===");
    let iters2 = vec![
        vec!['a', 'b'].into_iter(),
        vec!['1', '2', '3', '4'].into_iter(),
        vec!['X'].into_iter(),
    ];
    let result2: Vec<_> = round_robin(iters2).collect();
    println!("Round-robin [['a','b'], ['1','2','3','4'], ['X']]: {:?}", result2);

    println!("\n=== Key Points ===");
    println!("1. Custom iterators can combine multiple sources");
    println!("2. use_a flag tracks which iterator to pull from");
    println!("3. or_else provides fallback when one is exhausted");
    println!("4. Works with any iterator yielding the same Item type");
}
