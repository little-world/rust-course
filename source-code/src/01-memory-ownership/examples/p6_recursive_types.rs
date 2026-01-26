// Pattern 6: Recursive Types Require Box
#[derive(Debug)]
#[allow(dead_code)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T> List<T> {
    fn new() -> Self {
        List::Nil
    }

    fn prepend(self, elem: T) -> Self {
        List::Cons(elem, Box::new(self))
    }
}

fn main() {
    // Usage: Build a linked list
    let list = List::new().prepend(3).prepend(2).prepend(1);
    println!("{:?}", list); // Cons(1, Cons(2, Cons(3, Nil)))
    println!("Recursive types example completed");
}
