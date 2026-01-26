// Pattern 1: Size Must Be Known - Trait Objects

trait Animal {
    fn speak(&self);
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) { println!("Woof!"); }
}

// Can't store trait object directly on stack - size unknown
// let animal: dyn Animal = Dog; // Error!

// Solutions: use references or Box
fn with_reference(animal: &dyn Animal) {
    animal.speak();
}

fn with_box(animal: Box<dyn Animal>) {
    animal.speak();
}

fn main() {
    let dog = Dog;
    with_reference(&dog);
    with_box(Box::new(Dog));
    println!("Trait objects example completed");
}
