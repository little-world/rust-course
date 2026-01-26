// Pattern 8: AsRef, AsMut, Borrow, BorrowMut
use std::borrow::Borrow;

// AsRef: cheap reference conversion
// Use when you want to accept anything that can be viewed as &T
fn print_path<P: AsRef<std::path::Path>>(path: P) {
    println!("{:?}", path.as_ref());
}

// Borrow: semantic equivalence (same Hash, Eq, Ord)
// Use for lookup keys in collections
fn lookup<'a, K, V, Q>(map: &'a std::collections::HashMap<K, V>, key: &Q) -> Option<&'a V>
where
    K: Borrow<Q> + std::hash::Hash + Eq,
    Q: std::hash::Hash + Eq + ?Sized,
{
    map.get(key)
}

// Key distinction:
// - AsRef is for type conversion (String -> Path is valid)
// - Borrow requires semantic equivalence (String::borrow() -> &str has same hash)

fn trait_differences() {
    let s = String::from("hello");

    // AsRef: many conversions
    let _: &str = s.as_ref();
    let _: &[u8] = s.as_ref();
    let _: &std::path::Path = s.as_ref();
    let _: &std::ffi::OsStr = s.as_ref();

    // Borrow: semantic equivalence only
    let _: &str = s.borrow();
    // String doesn't impl Borrow<[u8]> because hash would differ
}

fn main() {
    // AsRef example
    print_path("hello.txt");
    print_path(String::from("world.txt"));
    print_path(std::path::Path::new("foo/bar.txt"));

    // Borrow/lookup example
    let mut map = std::collections::HashMap::new();
    map.insert(String::from("key"), 42);

    // Can lookup with &str even though keys are String
    if let Some(val) = lookup(&map, "key") {
        println!("Found: {}", val);
    }

    trait_differences();

    println!("AsRef/Borrow example completed");
}
