// Pattern 5: Borrow Trait for HashMap Keys
use std::collections::HashMap;
use std::borrow::Borrow;

fn lookup<Q>(map: &HashMap<String, i32>, key: &Q) -> Option<i32>
where
    String: Borrow<Q>,
    Q: Eq + std::hash::Hash + ?Sized,
{
    map.get(key).copied()
}

fn use_borrow() {
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert("Alice".into(), 100);
    scores.insert("Bob".into(), 85);

    // Can lookup with &str even though keys are String
    let alice_score = scores.get("Alice");  // Works!
    println!("Alice's score: {:?}", alice_score);

    // Our generic function works with both
    assert_eq!(lookup(&scores, "Alice"), Some(100));
    assert_eq!(lookup(&scores, &String::from("Bob")), Some(85));
}

fn main() {
    use_borrow();
    println!("Borrow trait example completed");
}
