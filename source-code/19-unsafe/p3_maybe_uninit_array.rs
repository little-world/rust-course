// Pattern 3: Using MaybeUninit for Arrays
use std::mem::MaybeUninit;

/// Create a large array efficiently without stack overflow.
fn create_array_uninit() -> [i32; 1000] {
    let mut arr: [MaybeUninit<i32>; 1000] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    for (i, elem) in arr.iter_mut().enumerate() {
        *elem = MaybeUninit::new(i as i32);
    }

    unsafe {
        std::mem::transmute(arr)
    }
}

fn main() {
    let arr = create_array_uninit();
    println!("First 5 elements: {:?}", &arr[0..5]);
    println!("Last 5 elements: {:?}", &arr[995..1000]);

    // Usage: Create large array without stack overflow
    let mut uninit: MaybeUninit<[u8; 10000]> = MaybeUninit::uninit();
    unsafe { (*uninit.as_mut_ptr()).fill(0); } // Initialize all bytes
    let arr2 = unsafe { uninit.assume_init() };
    println!("Large array first byte: {}", arr2[0]);

    println!("MaybeUninit array example completed");
}
