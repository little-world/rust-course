// Pattern 3: Deref Chains in Method Resolution
use std::rc::Rc;

struct Inner;
impl Inner {
    fn inner_method(&self) -> &'static str { "inner" }
}

struct Outer(Inner);
impl std::ops::Deref for Outer {
    type Target = Inner;
    fn deref(&self) -> &Inner { &self.0 }
}

fn deref_chain_resolution() {
    let rc: Rc<Outer> = Rc::new(Outer(Inner));

    // rc.inner_method() resolution:
    // 1. Rc<Outer> doesn't have inner_method
    // 2. Deref Rc<Outer> -> Outer, try Outer::inner_method
    // 3. Outer doesn't have inner_method
    // 4. Deref Outer -> Inner, try Inner::inner_method
    // 5. Found: Inner::inner_method(&self)
    // 6. Apply auto-ref: (&*(*rc)).inner_method()

    let _: &str = rc.inner_method();
}

fn main() {
    deref_chain_resolution();
    println!("Deref chains example completed");
}
