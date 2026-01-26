// Pattern 3: Exclusive Borrows (&mut T)
fn exclusive_borrows() {
    let mut s = String::from("hello");

    // Only ONE mutable borrow at a time
    let r1 = &mut s;
    r1.push_str(" world");
    // let r2 = &mut s;  // Error! Can't have two &mut
    println!("{}", r1);

    // After r1 is done, we can borrow again
    let r2 = &mut s;
    r2.push_str("!");
    println!("{}", r2);
}

fn modify_string(s: &mut String) {
    s.push_str(" - modified");
}

fn mutable_borrow_function() {
    let mut s = String::from("data");
    modify_string(&mut s);
    println!("{}", s);  // "data - modified"
}

fn main() {
    exclusive_borrows();
    mutable_borrow_function();
    println!("Exclusive borrows example completed");
}
