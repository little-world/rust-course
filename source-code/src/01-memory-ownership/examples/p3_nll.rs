// Pattern 3: Borrow Scope (Non-Lexical Lifetimes)
fn nll_example() {
    let mut data = vec![1, 2, 3];

    let first = &data[0];  // Immutable borrow starts
    println!("First: {}", first);  // Last use of `first`
    // Borrow ends here (NLL) - not at end of scope!

    data.push(4);  // OK! Mutable borrow is fine now
    println!("{:?}", data);
}

// Before NLL (Rust 2015), this wouldn't compile:
fn before_nll() {
    let mut data = vec![1, 2, 3];

    let first = &data[0];
    println!("First: {}", first);

    // In old Rust, `first` lived until }, blocking this:
    data.push(4);  // Now OK thanks to NLL
}

fn main() {
    nll_example();
    before_nll();
    println!("NLL example completed");
}
