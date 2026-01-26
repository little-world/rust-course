//! Pattern 1: Type-Safe Generic Functions
//! Example: Returning Owned vs Borrowed
//!
//! Run with: cargo run --example p1_return_types

// Returns owned value
fn create_default<T: Default>() -> T {
    T::default()
}

// Returns borrowed reference
fn first<T>(slice: &[T]) -> Option<&T> {
    slice.first()
}

// Returns reference with explicit lifetime
fn longest<'a, T>(x: &'a [T], y: &'a [T]) -> &'a [T] {
    if x.len() > y.len() { x } else { y }
}

// Returns the last element
fn last<T>(slice: &[T]) -> Option<&T> {
    slice.last()
}

// Returns middle element(s)
fn middle<T>(slice: &[T]) -> &[T] {
    let len = slice.len();
    if len <= 2 {
        slice
    } else {
        &slice[1..len - 1]
    }
}

fn main() {
    println!("=== Returning Owned Values ===");
    // Usage: Generic return types inferred from variable annotation.
    let s: String = create_default(); // Empty string via Default
    let v: Vec<i32> = create_default(); // Empty vec via Default
    let n: i32 = create_default(); // 0 via Default

    println!("create_default::<String>() = \"{}\"", s);
    println!("create_default::<Vec<i32>>() = {:?}", v);
    println!("create_default::<i32>() = {}", n);

    println!("\n=== Returning Borrowed References ===");
    let nums = [1, 2, 3, 4, 5];
    let f = first(&nums);
    let l = last(&nums);
    let m = middle(&nums);

    println!("Array: {:?}", nums);
    println!("first(&nums) = {:?}", f);
    println!("last(&nums) = {:?}", l);
    println!("middle(&nums) = {:?}", m);

    println!("\n=== Lifetime-Connected Returns ===");
    let arr1 = [1, 2];
    let arr2 = [3, 4, 5];
    let longer = longest(&arr1, &arr2);
    println!("longest(&[1, 2], &[3, 4, 5]) = {:?}", longer);

    let words1 = ["hello", "world"];
    let words2 = ["a"];
    let longer_words = longest(&words1, &words2);
    println!("longest(&[\"hello\", \"world\"], &[\"a\"]) = {:?}", longer_words);
}
