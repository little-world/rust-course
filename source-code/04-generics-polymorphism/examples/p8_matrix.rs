//! Pattern 8: Const Generics
//! Example: Matrix with Dimension Checking
//!
//! Run with: cargo run --example p8_matrix

use std::ops::{Add, Mul};

#[derive(Debug, Clone)]
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

impl<T: Default + Copy, const R: usize, const C: usize> Matrix<T, R, C> {
    fn new() -> Self {
        Matrix {
            data: [[T::default(); C]; R],
        }
    }

    fn from_array(data: [[T; C]; R]) -> Self {
        Matrix { data }
    }
}

impl<T: Copy, const R: usize, const C: usize> Matrix<T, R, C> {
    fn get(&self, row: usize, col: usize) -> Option<T> {
        if row < R && col < C {
            Some(self.data[row][col])
        } else {
            None
        }
    }

    fn set(&mut self, row: usize, col: usize, value: T) {
        if row < R && col < C {
            self.data[row][col] = value;
        }
    }

    fn rows(&self) -> usize {
        R
    }

    fn cols(&self) -> usize {
        C
    }

    // Transpose: Matrix<R, C> -> Matrix<C, R>
    fn transpose(&self) -> Matrix<T, C, R>
    where
        T: Default,
    {
        let mut result = Matrix::<T, C, R>::new();
        for i in 0..R {
            for j in 0..C {
                result.data[j][i] = self.data[i][j];
            }
        }
        result
    }
}

// Matrix addition: only works with same dimensions
impl<T: Add<Output = T> + Copy + Default, const R: usize, const C: usize> Add
    for Matrix<T, R, C>
{
    type Output = Matrix<T, R, C>;

    fn add(self, other: Self) -> Self::Output {
        let mut result = Matrix::new();
        for i in 0..R {
            for j in 0..C {
                result.data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        result
    }
}

// Matrix multiplication: M×N * N×P = M×P
impl<T, const M: usize, const N: usize> Matrix<T, M, N>
where
    T: Default + Copy + Add<Output = T> + Mul<Output = T>,
{
    fn multiply<const P: usize>(&self, other: &Matrix<T, N, P>) -> Matrix<T, M, P> {
        let mut result = Matrix::<T, M, P>::new();
        for i in 0..M {
            for j in 0..P {
                let mut sum = T::default();
                for k in 0..N {
                    sum = sum + self.data[i][k] * other.data[k][j];
                }
                result.data[i][j] = sum;
            }
        }
        result
    }
}

fn main() {
    println!("=== Basic Matrix Operations ===");
    let mut m: Matrix<i32, 2, 3> = Matrix::new();
    m.set(0, 0, 1);
    m.set(0, 1, 2);
    m.set(0, 2, 3);
    m.set(1, 0, 4);
    m.set(1, 1, 5);
    m.set(1, 2, 6);

    println!("Matrix<i32, 2, 3>:");
    println!("  {:?}", m.data[0]);
    println!("  {:?}", m.data[1]);
    println!("  rows: {}, cols: {}", m.rows(), m.cols());

    println!("\n=== Transpose ===");
    let transposed: Matrix<i32, 3, 2> = m.transpose();
    println!("Transposed to Matrix<i32, 3, 2>:");
    for row in &transposed.data {
        println!("  {:?}", row);
    }

    println!("\n=== Matrix Addition ===");
    let a: Matrix<i32, 2, 2> = Matrix::from_array([[1, 2], [3, 4]]);
    let b: Matrix<i32, 2, 2> = Matrix::from_array([[5, 6], [7, 8]]);
    let sum = a.clone() + b.clone();

    println!("A = {:?}", a.data);
    println!("B = {:?}", b.data);
    println!("A + B = {:?}", sum.data);

    println!("\n=== Matrix Multiplication ===");
    // 2×3 matrix
    let m1: Matrix<i32, 2, 3> = Matrix::from_array([[1, 2, 3], [4, 5, 6]]);

    // 3×2 matrix
    let m2: Matrix<i32, 3, 2> = Matrix::from_array([[7, 8], [9, 10], [11, 12]]);

    // Result: 2×2 matrix
    let product: Matrix<i32, 2, 2> = m1.multiply(&m2);

    println!("M1 (2×3):");
    for row in &m1.data {
        println!("  {:?}", row);
    }
    println!("M2 (3×2):");
    for row in &m2.data {
        println!("  {:?}", row);
    }
    println!("M1 × M2 (2×2):");
    for row in &product.data {
        println!("  {:?}", row);
    }

    println!("\n=== Compile-Time Dimension Checking ===");
    println!("These operations are type-checked at compile time:");
    println!("  Matrix<2,3> + Matrix<2,3> ✓ (same dimensions)");
    println!("  Matrix<2,3> × Matrix<3,4> = Matrix<2,4> ✓ (inner dims match)");
    println!();
    println!("These would NOT compile:");
    println!("  Matrix<2,3> + Matrix<2,4> ✗ (different dimensions)");
    println!("  Matrix<2,3> × Matrix<4,2> ✗ (inner dims don't match)");

    // This would NOT compile:
    // let bad: Matrix<i32, 2, 3> = Matrix::new();
    // let also_bad: Matrix<i32, 2, 4> = Matrix::new();
    // let _ = bad + also_bad; // ERROR: mismatched types
}
