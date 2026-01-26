// Pattern 3: Reading Uninitialized Memory (What NOT to Do)
use std::mem::MaybeUninit;

fn correct_usage() {
    let mut uninit: MaybeUninit<i32> = MaybeUninit::uninit();

    // SAFE: Writing to uninitialized memory
    uninit.write(42);
    let value = unsafe { uninit.assume_init() };
    println!("Value: {}", value);
}

#[allow(dead_code)]
fn what_not_to_do() {
    let _uninit: MaybeUninit<i32> = MaybeUninit::uninit();

    // UB: Reading uninitialized memory!
    // let value = unsafe { uninit.assume_init() };  // DON'T DO THIS
}

fn main() {
    correct_usage();

    // Usage: Correct pattern - write before assume_init
    let mut x = MaybeUninit::uninit();
    x.write(42);
    let val = unsafe { x.assume_init() }; // Safe: was initialized
    println!("Initialized value: {}", val);

    println!("Uninitialized memory example completed");
}
