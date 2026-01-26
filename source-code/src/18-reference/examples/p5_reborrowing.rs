// Pattern 5: Reborrowing Mechanics
fn takes_mut(s: &mut String) {
    s.push_str(" world");
}

fn reborrow_demo() {
    let mut s = String::from("hello");
    let r: &mut String = &mut s;

    // This works via reborrowing:
    takes_mut(r);  // Implicitly: takes_mut(&mut *r)
    takes_mut(r);  // r is NOT moved, it's reborrowed

    // Reborrowing creates a new &mut that borrows from r
    // Original r is temporarily "frozen" during the reborrow

    r.push_str("!");  // r is usable again after takes_mut returns
    println!("Result: {}", r);
}

fn explicit_reborrow() {
    let mut s = String::from("hello");
    let r = &mut s;

    // Explicit reborrow syntax - use block to scope reborrow
    {
        let r2: &mut String = &mut *r;
        r2.push_str(" world");
        // r is frozen while r2 exists
    }  // r2's lifetime ends here

    r.push_str("!");  // r usable again
    println!("Result: {}", r);
}

fn main() {
    reborrow_demo();
    explicit_reborrow();
    println!("Reborrowing example completed");
}
