// Pattern 3: Ambiguity and Explicit Disambiguation
trait A { fn method(&self) -> i32; }
trait B { fn method(&self) -> i32; }

struct S;
impl A for S { fn method(&self) -> i32 { 1 } }
impl B for S { fn method(&self) -> i32 { 2 } }

fn disambiguation() {
    let s = S;

    // s.method(); // Error: ambiguous

    // Fully qualified syntax:
    let _: i32 = A::method(&s);        // calls A::method
    let _: i32 = B::method(&s);        // calls B::method
    let _: i32 = <S as A>::method(&s); // explicit trait
}

fn main() {
    disambiguation();
    println!("Disambiguation example completed");
}
