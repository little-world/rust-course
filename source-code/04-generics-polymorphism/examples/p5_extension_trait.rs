//! Pattern 5: Blanket Implementations
//! Example: Extension Trait with Blanket Impl
//!
//! Run with: cargo run --example p5_extension_trait

// Extension trait for all iterators
trait IteratorExt: Iterator {
    // Count items matching a predicate
    fn count_where<P: FnMut(&Self::Item) -> bool>(self, p: P) -> usize
    where
        Self: Sized;

    // Get first N items as a Vec
    fn take_vec(self, n: usize) -> Vec<Self::Item>
    where
        Self: Sized;

    // Check if all items match
    fn all_match<P: FnMut(&Self::Item) -> bool>(self, p: P) -> bool
    where
        Self: Sized;
}

// Blanket impl for all iterators
impl<I: Iterator> IteratorExt for I {
    fn count_where<P: FnMut(&Self::Item) -> bool>(self, mut p: P) -> usize
    where
        Self: Sized,
    {
        self.filter(|item| p(item)).count()
    }

    fn take_vec(self, n: usize) -> Vec<Self::Item>
    where
        Self: Sized,
    {
        self.take(n).collect()
    }

    fn all_match<P: FnMut(&Self::Item) -> bool>(self, mut p: P) -> bool
    where
        Self: Sized,
    {
        for item in self {
            if !p(&item) {
                return false;
            }
        }
        true
    }
}

// Extension trait for slices
trait SliceExt<T> {
    fn second(&self) -> Option<&T>;
    fn middle(&self) -> &[T];
}

impl<T> SliceExt<T> for [T] {
    fn second(&self) -> Option<&T> {
        self.get(1)
    }

    fn middle(&self) -> &[T] {
        if self.len() <= 2 {
            self
        } else {
            &self[1..self.len() - 1]
        }
    }
}

// Extension for Options
trait OptionExt<T> {
    fn or_log(self, msg: &str) -> Option<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_log(self, msg: &str) -> Option<T> {
        if self.is_none() {
            eprintln!("[LOG] {}", msg);
        }
        self
    }
}

fn main() {
    println!("=== IteratorExt ===");
    // Usage: Extension adds count_where() to all iterators.
    let evens = (0..10).count_where(|x| x % 2 == 0);
    println!("(0..10).count_where(|x| x % 2 == 0) = {}", evens);

    let positives = vec![-2, -1, 0, 1, 2]
        .into_iter()
        .count_where(|x| *x > 0);
    println!("[-2, -1, 0, 1, 2].count_where(|x| x > 0) = {}", positives);

    println!("\n=== take_vec ===");
    let first_5: Vec<i32> = (0..100).take_vec(5);
    println!("(0..100).take_vec(5) = {:?}", first_5);

    println!("\n=== all_match ===");
    let all_positive = vec![1, 2, 3, 4, 5].into_iter().all_match(|x| *x > 0);
    println!("[1, 2, 3, 4, 5].all_match(|x| x > 0) = {}", all_positive);

    let all_even = vec![2, 4, 5, 8].into_iter().all_match(|x| *x % 2 == 0);
    println!("[2, 4, 5, 8].all_match(|x| x % 2 == 0) = {}", all_even);

    println!("\n=== SliceExt ===");
    let arr = [10, 20, 30, 40, 50];
    println!("[10, 20, 30, 40, 50].second() = {:?}", arr.second());
    println!("[10, 20, 30, 40, 50].middle() = {:?}", arr.middle());

    let short = [1, 2];
    println!("[1, 2].middle() = {:?}", short.middle());

    println!("\n=== OptionExt ===");
    let some_val: Option<i32> = Some(42);
    let result = some_val.or_log("Value was None");
    println!("Some(42).or_log(...) = {:?}", result);

    let none_val: Option<i32> = None;
    let result = none_val.or_log("Value was None - this will log");
    println!("None.or_log(...) = {:?}", result);

    println!("\n=== Extension Trait Pattern ===");
    println!("1. Define trait with desired methods");
    println!("2. Add supertrait bound: trait IteratorExt: Iterator");
    println!("3. Blanket impl: impl<I: Iterator> IteratorExt for I");
    println!("4. All iterators now have the new methods!");
}
