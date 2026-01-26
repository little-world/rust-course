// Pattern 5: Deref Coercion
// Deref coercion automatically converts &T to &U if T: Deref<Target=U>

fn print_str(s: &str) {
    println!("{}", s);
}

fn deref_coercion() {
    let owned = String::from("hello");
    let boxed = Box::new(String::from("world"));

    // All automatically coerce to &str
    print_str(&owned);      // &String -> &str
    print_str(&boxed);      // &Box<String> -> &String -> &str
    print_str("literal");   // &str -> &str

    // Works with slices too
    fn sum(nums: &[i32]) -> i32 { nums.iter().sum() }

    let vec = vec![1, 2, 3];
    let arr = [4, 5, 6];

    println!("Vec sum: {}", sum(&vec));  // &Vec<i32> -> &[i32]
    println!("Arr sum: {}", sum(&arr));  // &[i32; 3] -> &[i32]
}

fn main() {
    deref_coercion();
    println!("Deref coercion example completed");
}
