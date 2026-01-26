// Pattern 4: Lifetime Bounds and Higher-Ranked Trait Bounds

// Regular lifetime parameter
fn regular<'a, F>(_f: F)
where
    F: Fn(&'a str) -> &'a str
{
    // F works with one specific lifetime 'a
}

// Higher-ranked: F works for ANY lifetime
fn higher_ranked<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str
{
    // F must work for any lifetime the caller provides
    let s = String::from("hello");
    let result = f(&s);  // Works with the local lifetime of s
    println!("Result: {}", result);
}

// Practical example: comparison functions
fn sort_by<T, F>(slice: &mut [T], compare: F)
where
    F: for<'a> Fn(&'a T, &'a T) -> std::cmp::Ordering
{
    // compare must work with any borrowed elements
    slice.sort_by(|a, b| compare(a, b));
}

fn main() {
    // Regular: works with a specific lifetime
    let static_str: &'static str = "hello";
    regular(|s: &str| s);
    let _ = static_str;

    // Higher-ranked: works with any lifetime
    higher_ranked(|s| s);

    // Sort example
    let mut nums = vec![3, 1, 4, 1, 5, 9];
    sort_by(&mut nums, |a, b| a.cmp(b));
    println!("Sorted: {:?}", nums);

    println!("HRTB example completed");
}
