// Pattern 5: Compiler Optimizations
// Demonstrates const fn, const generics, and compile-time computation.

use std::marker::PhantomData;

// ============================================================================
// Example: Const Functions
// ============================================================================

const fn factorial(n: u32) -> u32 {
    match n {
        0 => 1,
        _ => n * factorial(n - 1),
    }
}

const FACTORIAL_10: u32 = factorial(10);  // Computed at compile time!

// ============================================================================
// Example: Const Generics for Compile-Time Values
// ============================================================================

#[derive(Debug)]
struct Matrix<const N: usize> {
    data: [[f64; N]; N],
}

impl<const N: usize> Matrix<N> {
    const fn zeros() -> Self {
        Matrix {
            data: [[0.0; N]; N],
        }
    }

    const fn identity() -> Self {
        let mut data = [[0.0; N]; N];
        let mut i = 0;
        while i < N {
            data[i][i] = 1.0;
            i += 1;
        }
        Matrix { data }
    }

    fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row][col]
    }

    fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row][col] = value;
    }
}

const IDENTITY_4X4: Matrix<4> = Matrix::identity();

// ============================================================================
// Example: Lookup Tables
// ============================================================================

const fn generate_squares_table() -> [u32; 100] {
    let mut table = [0u32; 100];
    let mut i = 0;
    while i < 100 {
        table[i] = (i * i) as u32;
        i += 1;
    }
    table
}

const SQUARES_TABLE: [u32; 100] = generate_squares_table();

fn fast_square(n: usize) -> u32 {
    if n < 100 {
        SQUARES_TABLE[n]  // O(1) lookup
    } else {
        (n * n) as u32
    }
}

// ============================================================================
// Example: Static Assertions
// ============================================================================

const fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

struct RingBuffer<T, const N: usize> {
    data: [Option<T>; N],
    head: usize,
    len: usize,
}

impl<T: Clone, const N: usize> RingBuffer<T, N> {
    const VALID_SIZE: () = assert!(is_power_of_two(N), "Size must be power of two");

    fn new() -> Self {
        let _ = Self::VALID_SIZE;  // Force compile-time check
        RingBuffer {
            data: std::array::from_fn(|_| None),
            head: 0,
            len: 0,
        }
    }

    fn push(&mut self, value: T) {
        let idx = (self.head + self.len) & (N - 1);  // Fast modulo for power of 2
        self.data[idx] = Some(value);
        if self.len < N {
            self.len += 1;
        } else {
            self.head = (self.head + 1) & (N - 1);
        }
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            let idx = (self.head + index) & (N - 1);
            self.data[idx].as_ref()
        } else {
            None
        }
    }
}

// ============================================================================
// Example: Compile-Time String Processing
// ============================================================================

const fn const_strlen(s: &str) -> usize {
    s.len()
}

const HELLO_LEN: usize = const_strlen("Hello, World!");

// ============================================================================
// Example: Type-Level Computation
// ============================================================================

struct Succ<N>(PhantomData<N>);
struct Zero;

type One = Succ<Zero>;
type Two = Succ<One>;
type Three = Succ<Two>;

trait NatNum {
    const VALUE: usize;
}

impl NatNum for Zero {
    const VALUE: usize = 0;
}

impl<N: NatNum> NatNum for Succ<N> {
    const VALUE: usize = N::VALUE + 1;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(FACTORIAL_10, 3628800);
    }

    #[test]
    fn test_matrix() {
        let m = Matrix::<3>::zeros();
        assert_eq!(m.get(0, 0), 0.0);
        assert_eq!(m.get(1, 2), 0.0);

        let id = Matrix::<3>::identity();
        assert_eq!(id.get(0, 0), 1.0);
        assert_eq!(id.get(1, 1), 1.0);
        assert_eq!(id.get(0, 1), 0.0);
    }

    #[test]
    fn test_identity_4x4() {
        assert_eq!(IDENTITY_4X4.get(0, 0), 1.0);
        assert_eq!(IDENTITY_4X4.get(3, 3), 1.0);
        assert_eq!(IDENTITY_4X4.get(0, 3), 0.0);
    }

    #[test]
    fn test_lookup_table() {
        assert_eq!(SQUARES_TABLE[0], 0);
        assert_eq!(SQUARES_TABLE[1], 1);
        assert_eq!(SQUARES_TABLE[10], 100);
        assert_eq!(SQUARES_TABLE[99], 9801);
    }

    #[test]
    fn test_fast_square() {
        assert_eq!(fast_square(5), 25);
        assert_eq!(fast_square(99), 9801);
        assert_eq!(fast_square(100), 10000);  // Falls back to calculation
    }

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(8));
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(7));
    }

    #[test]
    fn test_ring_buffer() {
        let mut buf: RingBuffer<i32, 4> = RingBuffer::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);

        assert_eq!(buf.get(0), Some(&1));
        assert_eq!(buf.get(1), Some(&2));
        assert_eq!(buf.get(2), Some(&3));
        assert_eq!(buf.get(3), None);

        // Overflow wraps around
        buf.push(4);
        buf.push(5);

        assert_eq!(buf.get(0), Some(&2));  // 1 was overwritten
    }

    #[test]
    fn test_const_strlen() {
        assert_eq!(HELLO_LEN, 13);
    }

    #[test]
    fn test_type_level_computation() {
        assert_eq!(Zero::VALUE, 0);
        assert_eq!(One::VALUE, 1);
        assert_eq!(Two::VALUE, 2);
        assert_eq!(Three::VALUE, 3);
    }
}

fn main() {
    println!("Pattern 5: Compiler Optimizations");
    println!("==================================\n");

    println!("Const function:");
    println!("  factorial(10) = {} (computed at compile time)", FACTORIAL_10);

    println!("\nConst generics:");
    let m = Matrix::<3>::identity();
    println!("  3x3 identity matrix diagonal: [{}, {}, {}]",
        m.get(0, 0), m.get(1, 1), m.get(2, 2));

    println!("\nLookup tables:");
    println!("  SQUARES_TABLE[7] = {} (O(1) lookup)", SQUARES_TABLE[7]);
    println!("  fast_square(12) = {}", fast_square(12));

    println!("\nRing buffer with power-of-two size:");
    let mut buf: RingBuffer<&str, 8> = RingBuffer::new();
    buf.push("a");
    buf.push("b");
    buf.push("c");
    println!("  Buffer contents: {:?}, {:?}, {:?}",
        buf.get(0), buf.get(1), buf.get(2));

    println!("\nCompile-time string length:");
    println!("  HELLO_LEN = {} (for \"Hello, World!\")", HELLO_LEN);

    println!("\nType-level computation:");
    println!("  Zero::VALUE = {}", Zero::VALUE);
    println!("  Three::VALUE = {}", Three::VALUE);

    println!("\n4x4 Identity matrix (compile-time constant):");
    for i in 0..4 {
        print!("  [");
        for j in 0..4 {
            print!("{:.0}", IDENTITY_4X4.get(i, j));
            if j < 3 { print!(", "); }
        }
        println!("]");
    }
}
