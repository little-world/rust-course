// Pattern 4: Enum Discrimination
#[repr(u8)]
enum MyEnum {
    A = 0,
    B = 1,
    C = 2,
}

fn get_discriminant(e: &MyEnum) -> u8 {
    unsafe { *(e as *const MyEnum as *const u8) }
}

fn enum_discriminant_safe(e: &MyEnum) -> u8 {
    match e {
        MyEnum::A => 0,
        MyEnum::B => 1,
        MyEnum::C => 2,
    }
}

// Also safe: std::mem::discriminant
fn enum_discriminant_std(e: &MyEnum) -> std::mem::Discriminant<MyEnum> {
    std::mem::discriminant(e)
}

fn main() {
    let a = MyEnum::A;
    let b = MyEnum::B;
    let c = MyEnum::C;

    println!("Discriminant of A (unsafe): {}", get_discriminant(&a));
    println!("Discriminant of B (safe): {}", enum_discriminant_safe(&b));
    println!("Discriminant of C (std): {:?}", enum_discriminant_std(&c));

    // Usage: Safe enum discriminant
    let disc = enum_discriminant_safe(&MyEnum::B); // Returns 1
    println!("B discriminant: {}", disc);

    println!("Enum discriminant example completed");
}
