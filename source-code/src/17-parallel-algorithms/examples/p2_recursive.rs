//! Pattern 2: Recursive Parallelism
//!
//! Run with: cargo run --bin p2_recursive

use rayon::prelude::*;
use std::time::Instant;

// Parallel quicksort
fn parallel_quicksort<T: Ord + Send>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }

    let pivot_idx = partition(arr);

    let (left, right) = arr.split_at_mut(pivot_idx);

    // Parallelize recursion
    rayon::join(
        || parallel_quicksort(left),
        || parallel_quicksort(&mut right[1..]),
    );
}

fn partition<T: Ord>(arr: &mut [T]) -> usize {
    let len = arr.len();
    let pivot_idx = len / 2;
    arr.swap(pivot_idx, len - 1);

    let mut i = 0;
    for j in 0..len - 1 {
        if arr[j] <= arr[len - 1] {
            arr.swap(i, j);
            i += 1;
        }
    }

    arr.swap(i, len - 1);
    i
}

// Parallel tree traversal
#[derive(Debug)]
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}

impl<T: Send + Sync> TreeNode<T> {
    fn parallel_map<F, U>(&self, f: &F) -> TreeNode<U>
    where
        F: Fn(&T) -> U + Sync,
        U: Send + Sync,
    {
        let value = f(&self.value);

        let (left, right) = rayon::join(
            || self.left.as_ref().map(|node| Box::new(node.parallel_map(f))),
            || self.right.as_ref().map(|node| Box::new(node.parallel_map(f))),
        );

        TreeNode { value, left, right }
    }

    fn parallel_sum(&self) -> T
    where
        T: std::ops::Add<Output = T> + Default + Copy + Send + Sync,
    {
        let mut sum = self.value;

        let (left_sum, right_sum) = rayon::join(
            || self.left.as_ref().map_or(T::default(), |node| node.parallel_sum()),
            || self.right.as_ref().map_or(T::default(), |node| node.parallel_sum()),
        );

        sum = sum + left_sum + right_sum;
        sum
    }
}

// Parallel Fibonacci (demonstrative)
fn parallel_fib(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }

    if n < 20 {
        // Sequential threshold to avoid overhead
        return fib_sequential(n);
    }

    let (a, b) = rayon::join(
        || parallel_fib(n - 1),
        || parallel_fib(n - 2),
    );

    a + b
}

fn fib_sequential(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    let mut a = 0;
    let mut b = 1;
    for _ in 2..=n {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}

// Parallel directory size
use std::fs;
use std::path::Path;

fn parallel_dir_size<P: AsRef<Path>>(path: P) -> u64 {
    let path = path.as_ref();

    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }

    if !path.is_dir() {
        return 0;
    }

    let entries: Vec<_> = fs::read_dir(path)
        .ok()
        .map(|entries| entries.filter_map(|e| e.ok()).collect())
        .unwrap_or_default();

    entries
        .par_iter()
        .map(|entry| parallel_dir_size(entry.path()))
        .sum()
}

// Parallel expression evaluation
#[derive(Debug, Clone)]
enum Expr {
    Num(i32),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn parallel_eval(&self) -> i32 {
        match self {
            Expr::Num(n) => *n,
            Expr::Add(left, right) => {
                let (l, r) = rayon::join(
                    || left.parallel_eval(),
                    || right.parallel_eval(),
                );
                l + r
            }
            Expr::Mul(left, right) => {
                let (l, r) = rayon::join(
                    || left.parallel_eval(),
                    || right.parallel_eval(),
                );
                l * r
            }
            Expr::Sub(left, right) => {
                let (l, r) = rayon::join(
                    || left.parallel_eval(),
                    || right.parallel_eval(),
                );
                l - r
            }
        }
    }
}

fn main() {
    println!("=== Parallel Quicksort ===\n");
    let mut data: Vec<i32> = (0..100_000).rev().collect();
    let start = Instant::now();
    parallel_quicksort(&mut data);
    println!("Sort time: {:?}", start.elapsed());
    println!("Sorted: {}", data.windows(2).all(|w| w[0] <= w[1]));

    println!("\n=== Parallel Fibonacci ===\n");
    let start = Instant::now();
    let result = parallel_fib(35);
    println!("fib(35) = {} in {:?}", result, start.elapsed());

    println!("\n=== Parallel Directory Size ===\n");
    let size = parallel_dir_size(".");
    println!("Current directory size: {} bytes", size);

    println!("\n=== Parallel Expression Evaluation ===\n");
    let expr = Expr::Add(
        Box::new(Expr::Mul(
            Box::new(Expr::Num(5)),
            Box::new(Expr::Num(10)),
        )),
        Box::new(Expr::Sub(
            Box::new(Expr::Num(20)),
            Box::new(Expr::Num(8)),
        )),
    );
    let result = expr.parallel_eval();
    println!("Expression result: {} (expected: 62)", result);

    println!("\n=== Parallel Tree Sum ===\n");
    let tree = TreeNode {
        value: 1,
        left: Some(Box::new(TreeNode {
            value: 2,
            left: Some(Box::new(TreeNode { value: 4, left: None, right: None })),
            right: Some(Box::new(TreeNode { value: 5, left: None, right: None })),
        })),
        right: Some(Box::new(TreeNode {
            value: 3,
            left: Some(Box::new(TreeNode { value: 6, left: None, right: None })),
            right: Some(Box::new(TreeNode { value: 7, left: None, right: None })),
        })),
    };
    let sum = tree.parallel_sum();
    println!("Tree sum: {} (expected: 28)", sum);
}
