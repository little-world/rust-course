// Pattern 1: Pointer Arithmetic
fn pointer_arithmetic() {
    let arr = [1, 2, 3, 4, 5];
    let ptr: *const i32 = arr.as_ptr();

    unsafe {
        for i in 0..arr.len() {
            let element_ptr = ptr.add(i);  // Equivalent to ptr + i * sizeof(i32)
            println!("Element {}: {}", i, *element_ptr);
        }

        let third = ptr.add(2);  // Points to third element
        println!("Third element: {}", *third);

        // ptr.add(10) would be UB - out of bounds!
    }
}

fn main() {
    pointer_arithmetic();

    // Usage: Iterate array via pointer offset
    let data = [10, 20, 30];
    unsafe { println!("Second: {}", *data.as_ptr().add(1)); }

    println!("Pointer arithmetic example completed");
}
