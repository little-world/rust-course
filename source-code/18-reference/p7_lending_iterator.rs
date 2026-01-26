// Pattern 7: Lifetime Bounds in Iterators

// Lending iterator pattern (GAT-based)
trait LendingIterator {
    type Item<'a> where Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

// Windows iterator: yields overlapping slices
struct Windows<'a, T> {
    slice: &'a [T],
    size: usize,
    pos: usize,
}

impl<'a, T> Iterator for Windows<'a, T> {
    type Item = &'a [T];  // Item lifetime tied to original slice

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + self.size <= self.slice.len() {
            let window = &self.slice[self.pos..self.pos + self.size];
            self.pos += 1;
            Some(window)
        } else {
            None
        }
    }
}

fn main() {
    let data = vec![1, 2, 3, 4, 5];
    let windows = Windows {
        slice: &data,
        size: 3,
        pos: 0,
    };

    println!("Windows of size 3:");
    for window in windows {
        println!("  {:?}", window);
    }

    println!("Lending iterator example completed");
}
