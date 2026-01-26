// Pattern 4: Converting Between Slice Types
use std::slice;

fn slice_transmute() {
    let data: Vec<u32> = vec![0x12345678, 0x9abcdef0];

    let bytes: &[u8] = unsafe {
        slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * std::mem::size_of::<u32>(),
        )
    };

    println!("Bytes: {:?}", bytes);
    // Reverse: bytes to u32 (must ensure alignment!)
}

fn main() {
    slice_transmute();

    // Usage: View u32 slice as bytes for serialization
    let numbers: [u32; 2] = [1, 2];
    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(numbers.as_ptr() as *const u8, 8)
    };
    println!("Numbers as bytes: {:?}", bytes);

    println!("Slice transmute example completed");
}
