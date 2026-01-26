//! Pattern 4: Extension Traits
//! Example: Basic Extension Trait
//!
//! Run with: cargo run --example p4_basic_extension

// Define extension trait for Vec<i32>
trait SumExt {
    fn sum_ext(&self) -> i32;
    fn average(&self) -> f64;
}

impl SumExt for Vec<i32> {
    fn sum_ext(&self) -> i32 {
        self.iter().sum()
    }

    fn average(&self) -> f64 {
        if self.is_empty() {
            0.0
        } else {
            self.sum_ext() as f64 / self.len() as f64
        }
    }
}

// Extend Vec<f64> too
impl SumExt for Vec<f64> {
    fn sum_ext(&self) -> i32 {
        self.iter().sum::<f64>() as i32
    }

    fn average(&self) -> f64 {
        if self.is_empty() {
            0.0
        } else {
            self.iter().sum::<f64>() / self.len() as f64
        }
    }
}

// Extension for slices
trait SliceExt<T> {
    fn second(&self) -> Option<&T>;
    fn last_two(&self) -> Option<(&T, &T)>;
}

impl<T> SliceExt<T> for [T] {
    fn second(&self) -> Option<&T> {
        self.get(1)
    }

    fn last_two(&self) -> Option<(&T, &T)> {
        if self.len() >= 2 {
            Some((&self[self.len() - 2], &self[self.len() - 1]))
        } else {
            None
        }
    }
}

fn main() {
    // Usage: Extension trait adds sum_ext() method to Vec types.
    let numbers = vec![1, 2, 3, 4, 5];
    let sum = numbers.sum_ext();
    let avg = numbers.average();

    println!("=== Vec<i32> Extension ===");
    println!("Numbers: {:?}", numbers);
    println!("Sum: {}", sum);
    println!("Average: {:.2}", avg);

    let floats = vec![1.5, 2.5, 3.5];
    println!("\n=== Vec<f64> Extension ===");
    println!("Floats: {:?}", floats);
    println!("Sum (as i32): {}", floats.sum_ext());
    println!("Average: {:.2}", floats.average());

    println!("\n=== Slice Extension ===");
    let items = [10, 20, 30, 40, 50];
    println!("Items: {:?}", items);
    println!("Second: {:?}", items.second());
    println!("Last two: {:?}", items.last_two());

    // Works on Vec too (via Deref to slice)
    let vec_items = vec!["a", "b", "c"];
    println!("\nVec items: {:?}", vec_items);
    println!("Second: {:?}", vec_items.second());
}
