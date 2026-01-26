// Pattern 3: Reborrowing
fn reborrow() {
    let mut data = String::from("hello");
    let r1: &mut String = &mut data;

    // Reborrow: create a new borrow from existing one
    let r2: &mut String = &mut *r1;  // r1 is temporarily "frozen"
    r2.push_str(" world");
    // r1 is unfrozen when r2 goes out of scope

    r1.push_str("!");
    println!("{}", r1);
}

// Reborrowing happens automatically in function calls
fn takes_ref(s: &mut String) {
    s.push_str("!");
}

fn auto_reborrow() {
    let mut s = String::from("hello");
    let r = &mut s;

    takes_ref(r);  // r is reborrowed, not moved
    takes_ref(r);  // Can use r again!
    println!("{}", r);
}

fn main() {
    reborrow();
    auto_reborrow();
    println!("Reborrow example completed");
}
