// Pattern 1: Raw Pointer Usage
fn raw_pointer_basics() {
    let mut num = 42;
    let r1: *const i32 = &num;            // Immutable raw pointer
    let r2: *mut i32 = &mut num;          // Mutable raw pointer
    let address = 0x12345usize;
    let _r3 = address as *const i32;       // Might point to invalid memory!

    unsafe {
        println!("r1 points to: {}", *r1);
        *r2 = 100;
        println!("num is now: {}", num);
        // Dereferencing r3 would be UB - it points to random memory!
    }
}

fn main() {
    raw_pointer_basics();

    // Usage: Create pointer from reference, dereference in unsafe block
    let x = 42;
    let ptr: *const i32 = &x;
    unsafe { println!("Value: {}", *ptr); }

    println!("Raw pointer basics example completed");
}
