// Pattern 2: The Deref Hierarchy
use std::ops::{Deref, DerefMut};

// Deref defines: fn deref(&self) -> &Self::Target
// DerefMut defines: fn deref_mut(&mut self) -> &mut Self::Target

// The relationship between * and deref:
// *x where x: T is equivalent to *Deref::deref(&x)
// This means *x: Self::Target, not &Self::Target

struct Wrapper<T>(T);

impl<T> Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

impl<T> DerefMut for Wrapper<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}

fn deref_typing() {
    let w: Wrapper<String> = Wrapper(String::from("hello"));

    // Type of expressions:
    let _: &String = &*w;           // explicit deref then ref
    let _: &String = w.deref();     // method call
    let _: &str = &*w;              // deref coercion: &String -> &str

    // The * operator dereferences the return value of deref()
    // *w is sugar for *(w.deref()), which is *(&self.0), which is self.0
}

fn main() {
    deref_typing();
    println!("Deref hierarchy example completed");
}
