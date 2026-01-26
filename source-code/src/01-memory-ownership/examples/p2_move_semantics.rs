// Pattern 2: Move Semantics
fn move_semantics() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 is MOVED to s2

    // println!("{}", s1);  // Error! s1 is no longer valid
    println!("{}", s2);     // OK: s2 owns the data

    // Same with function calls
    let s3 = String::from("world");
    take_ownership(s3);
    // println!("{}", s3);  // Error! s3 was moved into function
}

fn take_ownership(s: String) {
    println!("Got: {}", s);
} // s is dropped here

fn main() {
    move_semantics();
    println!("Move semantics example completed");
}
