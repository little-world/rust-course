// Pattern 1: Binding Mode Inference
#[derive(Clone)]
struct Pair<T>(T, T);

fn binding_modes<T: std::fmt::Debug>(pair: &Pair<T>) {
    // Matching on &Pair<T>: binding mode is "ref" by default
    let Pair(a, b) = pair;
    // a: &T, b: &T (inferred from matching on reference)

    // Explicit ref is redundant here but clarifies intent
    let Pair(ref x, ref y) = *pair;
    // x: &T, y: &T
    let _ = (a, b, x, y);
}

fn explicit_move<T: Clone>(pair: &Pair<T>) {
    // To override binding mode and clone:
    let Pair(a, b) = pair.clone();
    // a: T, b: T (moved from cloned Pair)
    let _ = (a, b);
}

fn main() {
    let p = Pair(1, 2);
    binding_modes(&p);
    explicit_move(&p);
    println!("Binding modes example completed");
}
