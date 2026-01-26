// Pattern 4: Transmuting References (Dangerous!)
use std::mem;

// DANGEROUS: Transmuting references
fn transmute_reference_unsafe() {
    let x: &i32 = &42;
    let y: &u32 = unsafe { mem::transmute(x) };
    println!("Transmuted: {}", y);
}

// BETTER: Using pointer casting
fn transmute_reference_safer() {
    let x: i32 = 42;
    let ptr = &x as *const i32 as *const u32;
    let y = unsafe { &*ptr };
    println!("Casted: {}", y);
}

// SAFEST: Just use from_ne_bytes or as cast
fn safe_conversion() {
    let x: i32 = 42;
    let y = x as u32;  // Sign-extends negative values
    println!("Converted: {}", y);
}

fn main() {
    transmute_reference_unsafe();
    transmute_reference_safer();
    safe_conversion();

    // Usage: Safe integer reinterpretation via as cast
    let signed: i32 = -1;
    let unsigned: u32 = signed as u32; // 4294967295 (0xFFFFFFFF)
    println!("Signed {} as unsigned: {}", signed, unsigned);

    println!("Transmute references example completed");
}
