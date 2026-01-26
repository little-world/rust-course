// Pattern 3: Shared Borrows (&T)
fn shared_borrows() {
    let s = String::from("hello");

    // Multiple shared borrows are OK
    let r1 = &s;
    let r2 = &s;
    let r3 = &s;

    println!("{}, {}, {}", r1, r2, r3);  // All valid simultaneously

    // Shared borrow in function - doesn't take ownership
    print_length(&s);
    println!("Still have s: {}", s);  // s still valid!
}

fn print_length(s: &String) {
    println!("Length: {}", s.len());
}  // s goes out of scope but nothing is dropped (we don't own it)

fn main() {
    shared_borrows();
    println!("Shared borrows example completed");
}
