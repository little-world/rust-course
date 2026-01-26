// Pattern 3: Partial Initialization
use std::mem::MaybeUninit;

struct ComplexStruct {
    field1: String,
    field2: Vec<i32>,
    field3: Box<i32>,
}

fn initialize_complex_struct() -> ComplexStruct {
    let mut uninit: MaybeUninit<ComplexStruct> = MaybeUninit::uninit();
    let ptr = uninit.as_mut_ptr();

    unsafe {
        // SAFETY: Using addr_of_mut! to get field pointers without creating references
        std::ptr::addr_of_mut!((*ptr).field1).write(String::from("hello"));
        std::ptr::addr_of_mut!((*ptr).field2).write(vec![1, 2, 3]);
        std::ptr::addr_of_mut!((*ptr).field3).write(Box::new(42));

        uninit.assume_init()
    }
}

fn main() {
    // Usage: Initialize struct field-by-field
    let s = initialize_complex_struct();
    println!("{}, {:?}, {}", s.field1, s.field2, s.field3);

    println!("Partial initialization example completed");
}
