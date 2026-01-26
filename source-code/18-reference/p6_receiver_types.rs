// Pattern 6: Receiver Type Inference
use std::rc::Rc;
use std::sync::Arc;
use std::pin::Pin;

struct Widget { name: String }

impl Widget {
    // Each receiver type has specific semantics
    fn ref_method(&self) -> &str { &self.name }
    fn mut_method(&mut self) { self.name.push_str("!"); }
    fn owned_method(self) -> String { self.name }

    // Arbitrary self types (requires #![feature(arbitrary_self_types)]
    // for custom types, but these work in stable):
    fn box_method(self: Box<Self>) -> String { self.name }
    fn rc_method(self: Rc<Self>) -> Rc<Self> { self }
    fn arc_method(self: Arc<Self>) -> Arc<Self> { self }
    fn pin_method(self: Pin<&Self>) -> &str { &self.get_ref().name }
    fn pin_mut_method(self: Pin<&mut Self>) {
        // Must maintain pin invariants
        self.get_mut().name.push_str("!");
    }
}

fn receiver_types() {
    // Each receiver type determines how the method is called
    let w = Widget { name: "w".into() };
    let _ = w.ref_method();      // auto-ref to &Widget

    let boxed = Box::new(Widget { name: "boxed".into() });
    let _ = boxed.box_method();  // consumes Box<Widget>

    let rc = Rc::new(Widget { name: "rc".into() });
    let rc2 = rc.clone();
    let _ = rc.rc_method();      // consumes one Rc handle
    let _ = rc2.ref_method();    // Deref through Rc to call &self method
}

fn main() {
    receiver_types();

    // Additional demonstrations
    let mut w = Widget { name: "test".into() };
    println!("Initial: {}", w.ref_method());
    w.mut_method();
    println!("After mut: {}", w.ref_method());

    let arc = Arc::new(Widget { name: "arc_widget".into() });
    let arc2 = arc.clone();
    let _ = arc.arc_method();
    println!("Arc widget: {}", arc2.ref_method());

    println!("Receiver types example completed");
}
