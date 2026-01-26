// Pattern 6: Reference Counting Optimization - String Interning
use std::rc::Rc;
use std::collections::HashMap;

struct StringInterner {
    map: HashMap<String, Rc<str>>,
}

impl StringInterner {
    fn new() -> Self {
        StringInterner { map: HashMap::new() }
    }

    fn intern(&mut self, s: &str) -> Rc<str> {
        if let Some(interned) = self.map.get(s) {
            Rc::clone(interned)
        } else {
            let rc: Rc<str> = Rc::from(s);
            self.map.insert(s.to_string(), Rc::clone(&rc));
            rc
        }
    }
}

fn main() {
    // Usage: Repeated strings share allocation
    let mut interner = StringInterner::new();
    let s1 = interner.intern("hello");
    let s2 = interner.intern("hello");  // Returns same Rc
    assert!(Rc::ptr_eq(&s1, &s2));      // Same allocation!

    let s3 = interner.intern("world");
    assert!(!Rc::ptr_eq(&s1, &s3));     // Different strings

    println!("s1 and s2 same allocation: {}", Rc::ptr_eq(&s1, &s2));
    println!("String interning example completed");
}
