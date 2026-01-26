// Pattern 2: Returning Ownership

// Give ownership to caller
fn create_string() -> String {
    let s = String::from("created");
    s  // Ownership moves to caller
}

// Take and return ownership (transfer pattern)
fn process_and_return(mut s: String) -> String {
    s.push_str(" - processed");
    s  // Return ownership
}

fn ownership_transfer() {
    let s1 = create_string();
    let s2 = process_and_return(s1);
    // s1 is invalid, s2 is valid
    println!("{}", s2);
}

fn main() {
    ownership_transfer();
    println!("Ownership transfer example completed");
}
