// Pattern 4: Variance of Reference Types
// Variance rules for references:
// &'a T     is covariant in 'a and covariant in T
// &'a mut T is covariant in 'a and invariant in T

fn covariance_demo<'long, 'short>(
    long_ref: &'long str,
    _short_ref: &'short str,
) where 'long: 'short {
    // 'long: 'short means 'long outlives 'short

    // &'long T can be used where &'short T is expected
    let _: &'short str = long_ref;  // OK: covariant in lifetime

    // Cannot go the other way
    // let _: &'long str = short_ref;  // Error
}

fn invariance_demo<'a>(
    _mut_ref: &'a mut Vec<&'a str>,
) {
    // &mut T is invariant in T
    // This prevents:
    // 1. Inserting shorter-lived references
    // 2. Extracting longer-lived references

    // If T = Vec<&'a str>, we cannot treat it as Vec<&'static str>
    // even though &'static str: 'a
}

fn main() {
    let long_lived = "static string";
    let short_lived = String::from("short");
    covariance_demo(long_lived, &short_lived);

    let mut vec: Vec<&str> = vec!["hello"];
    invariance_demo(&mut vec);

    println!("Variance example completed");
}
