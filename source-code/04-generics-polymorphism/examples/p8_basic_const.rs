//! Pattern 8: Const Generics
//! Example: Basic Const Generic Array
//!
//! Run with: cargo run --example p8_basic_const

// Basic const generic: size N is part of the type
struct Array<T, const N: usize> {
    data: [T; N],
}

impl<T: Default + Copy, const N: usize> Array<T, N> {
    fn new() -> Self {
        Array {
            data: [T::default(); N],
        }
    }

    fn filled(value: T) -> Self {
        Array { data: [value; N] }
    }
}

impl<T, const N: usize> Array<T, N> {
    fn len(&self) -> usize {
        N // Known at compile time
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index < N {
            Some(&self.data[index])
        } else {
            None
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < N {
            Some(&mut self.data[index])
        } else {
            None
        }
    }

    fn as_slice(&self) -> &[T] {
        &self.data
    }
}

impl<T: Copy, const N: usize> Array<T, N> {
    fn map<U: Default + Copy, F: Fn(T) -> U>(&self, f: F) -> Array<U, N> {
        let mut result = Array::<U, N>::new();
        for i in 0..N {
            result.data[i] = f(self.data[i]);
        }
        result
    }

    fn map_to_vec<U, F: Fn(T) -> U>(&self, f: F) -> Vec<U> {
        self.data.iter().map(|&x| f(x)).collect()
    }
}

// Compile-time size validation using const blocks
struct NonEmpty<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> NonEmpty<T, N> {
    fn new(data: [T; N]) -> Self {
        const { assert!(N > 0, "NonEmpty requires N > 0") }
        NonEmpty { data }
    }

    fn first(&self) -> &T {
        &self.data[0] // Always safe, N > 0
    }
}

fn main() {
    println!("=== Basic Const Generic Array ===");
    // Usage: Size N is part of type; Array<i32, 5> differs from Array<i32, 10>
    let arr5: Array<i32, 5> = Array::new();
    let arr10: Array<i32, 10> = Array::filled(42);

    println!("Array<i32, 5>::new():");
    println!("  len: {}", arr5.len());
    println!("  data: {:?}", arr5.as_slice());

    println!("\nArray<i32, 10>::filled(42):");
    println!("  len: {}", arr10.len());
    println!("  data: {:?}", arr10.as_slice());

    println!("\n=== Array Access ===");
    let mut arr: Array<i32, 3> = Array::new();
    if let Some(elem) = arr.get_mut(1) {
        *elem = 100;
    }
    println!("After setting index 1 to 100: {:?}", arr.as_slice());
    println!("get(1) = {:?}", arr.get(1));
    println!("get(10) = {:?}", arr.get(10)); // Out of bounds

    println!("\n=== Map Operation ===");
    let arr: Array<i32, 4> = Array::filled(5);
    let doubled: Array<i32, 4> = arr.map(|x| x * 2);
    println!("Array::filled(5).map(|x| x * 2) = {:?}", doubled.as_slice());

    // For non-Copy types like String, use map_to_vec
    let stringified: Vec<String> = arr.map_to_vec(|x| x.to_string());
    println!("Array::filled(5).map_to_vec(to_string) = {:?}", stringified);

    println!("\n=== NonEmpty with Compile-Time Validation ===");
    let non_empty: NonEmpty<i32, 3> = NonEmpty::new([1, 2, 3]);
    println!("NonEmpty::new([1, 2, 3]).first() = {}", non_empty.first());

    // This would cause a compile error:
    // let empty: NonEmpty<i32, 0> = NonEmpty::new([]);
    // error: assertion failed: N > 0

    println!("\n=== Type Distinction ===");
    println!("Array<i32, 5> and Array<i32, 10> are DIFFERENT types");
    println!("You cannot assign one to the other or mix them.");

    // This would NOT compile:
    // let arr5: Array<i32, 5> = Array::new();
    // let arr10: Array<i32, 10> = arr5; // ERROR: mismatched types
}
