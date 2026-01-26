// Pattern 2: Deref Coercion Rules
fn takes_str(s: &str) {
    let _ = s;
}

fn coercion_contexts() {
    let s = String::from("hello");
    let boxed = Box::new(String::from("boxed"));

    // Coercion sites:
    // 1. Function/method arguments
    takes_str(&s);          // &String -> &str
    takes_str(&boxed);      // &Box<String> -> &String -> &str

    // 2. Let bindings with explicit type
    let _: &str = &s;       // coerced
    let _: &str = &boxed;   // coerced through two Derefs

    // 3. Struct field initialization (if field type is explicit)
    struct Holder<'a> { s: &'a str }
    let _ = Holder { s: &s };

    // 4. Return position (if return type is explicit)
    fn return_coercion(s: &String) -> &str { s }
    let _ = return_coercion(&s);
}

// Coercion rules:
// &T      -> &U       where T: Deref<Target=U>
// &mut T  -> &mut U   where T: DerefMut<Target=U>
// &mut T  -> &U       where T: Deref<Target=U>  (mut to shared OK)
// &T      -> &mut U   NEVER (shared to mut forbidden)

fn main() {
    coercion_contexts();
    println!("Deref coercion example completed");
}
