// Pattern 3: Method Resolution Order
#[derive(Clone)]
struct S;

impl S {
    fn by_value(self) { println!("by_value"); }
    fn by_ref(&self) { println!("by_ref"); }
    fn by_mut(&mut self) { println!("by_mut"); }
}

fn resolution_order() {
    let mut s = S;

    // For s.method(), Rust tries in order:
    // 1. S::method(s)           - inherent, by value
    // 2. S::method(&s)          - inherent, by ref
    // 3. S::method(&mut s)      - inherent, by mut ref
    // 4. <S as Trait>::method(s)    - trait, by value
    // 5. <S as Trait>::method(&s)   - trait, by ref
    // 6. <S as Trait>::method(&mut s) - trait, by mut ref
    // 7. Deref to U, repeat 1-6 with U
    // 8. Unsized coercion, repeat

    // This order means by_ref is NOT called via auto-ref when
    // by_value exists, unless by_value's receiver doesn't match.

    s.by_ref();
    s.by_mut();
    s.by_value(); // consumes s
}

fn main() {
    resolution_order();
    println!("Method resolution example completed");
}
