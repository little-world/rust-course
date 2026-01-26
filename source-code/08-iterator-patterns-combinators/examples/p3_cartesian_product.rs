//! Pattern 3: Advanced Iterator Composition
//! Example: Cartesian Product
//!
//! Run with: cargo run --example p3_cartesian_product

/// Generate all pairs from two slices using flat_map.
/// No intermediate vectors created - pairs generated lazily.
fn cartesian_product<'a, T: Clone>(
    a: &'a [T],
    b: &'a [T],
) -> impl Iterator<Item = (T, T)> + 'a {
    a.iter().flat_map(move |x| {
        b.iter().map(move |y| (x.clone(), y.clone()))
    })
}

/// Generate triple product lazily.
fn triple_product<'a, T: Clone>(
    a: &'a [T],
    b: &'a [T],
    c: &'a [T],
) -> impl Iterator<Item = (T, T, T)> + 'a {
    a.iter().flat_map(move |x| {
        b.iter().flat_map(move |y| {
            c.iter().map(move |z| (x.clone(), y.clone(), z.clone()))
        })
    })
}

/// Generate combinations with indices.
fn indexed_product<'a, T: Clone>(
    a: &'a [T],
    b: &'a [T],
) -> impl Iterator<Item = ((usize, T), (usize, T))> + 'a {
    a.iter().enumerate().flat_map(move |(i, x)| {
        b.iter().enumerate().map(move |(j, y)| {
            ((i, x.clone()), (j, y.clone()))
        })
    })
}

fn main() {
    println!("=== Cartesian Product with flat_map ===\n");

    // Usage: generate all pairs from two slices
    let pairs: Vec<_> = cartesian_product(&[1, 2], &[3, 4]).collect();
    println!("cartesian_product([1,2], [3,4]):");
    for (a, b) in &pairs {
        println!("  ({}, {})", a, b);
    }
    // (1,3), (1,4), (2,3), (2,4)

    println!("\n=== Larger Product ===");
    let product: Vec<_> = cartesian_product(&['a', 'b', 'c'], &['x', 'y', 'z']).collect();
    println!("cartesian_product(['a','b','c'], ['x','y','z']): {} pairs", product.len());
    for (c, n) in &product {
        print!("({},{}) ", c, n);
    }
    println!();

    println!("\n=== Triple Product ===");
    let triples: Vec<_> = triple_product(&[1, 2], &[3, 4], &[5, 6]).collect();
    println!("triple_product([1,2], [3,4], [5,6]):");
    for (x, y, z) in &triples {
        println!("  ({}, {}, {})", x, y, z);
    }

    println!("\n=== Indexed Product ===");
    let indexed: Vec<_> = indexed_product(&["foo", "bar"], &["x", "y"]).collect();
    println!("indexed_product(['foo','bar'], ['x','y']):");
    for ((i, a), (j, b)) in &indexed {
        println!("  [{},{}] = ({}, {})", i, j, a, b);
    }

    println!("\n=== Practical Example: Grid Coordinates ===");
    let rows = 3;
    let cols = 4;
    let coords: Vec<_> = (0..rows).flat_map(|r| {
        (0..cols).map(move |c| (r, c))
    }).collect();
    println!("3x4 grid coordinates:");
    for (r, c) in &coords {
        print!("({},{}) ", r, c);
    }
    println!();

    println!("\n=== How flat_map Works ===");
    println!("a.iter().flat_map(|x| b.iter().map(|y| (x, y)))");
    println!("");
    println!("For each x in a:");
    println!("  Create iterator over all (x, y) pairs");
    println!("flat_map flattens these into single stream");

    println!("\n=== Key Points ===");
    println!("1. flat_map composes nested iteration");
    println!("2. move captures variables for inner closure");
    println!("3. Clone needed because items used multiple times");
    println!("4. Pairs generated lazily on demand");
}
