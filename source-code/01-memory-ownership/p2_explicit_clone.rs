// Pattern 2: Clone for Explicit Copies
fn explicit_clone() {
    let s1 = String::from("hello");
    let s2 = s1.clone();  // Explicit deep copy

    println!("s1 = {}, s2 = {}", s1, s2);  // Both valid!

    // Clone is explicit - makes cost visible
    let vec1 = vec![1, 2, 3, 4, 5];
    let _vec2 = vec1.clone();  // O(n) operation - you see it
}

// Derive Clone for custom types
#[derive(Clone)]
#[allow(dead_code)]
struct Document {
    title: String,
    content: String,
}

fn clone_custom() {
    let doc1 = Document {
        title: "Report".into(),
        content: "...".into(),
    };
    let _doc2 = doc1.clone();  // Deep copy of both Strings
    println!("Cloned document: {}", doc1.title);
}

fn main() {
    explicit_clone();
    clone_custom();
    println!("Explicit clone example completed");
}
