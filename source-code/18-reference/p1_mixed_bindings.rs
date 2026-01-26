// Pattern 1: Mixed Binding Modes
struct Record {
    id: u64,           // Copy
    data: Vec<u8>,     // !Copy
    name: String,      // !Copy
}

fn mixed_bindings(record: &Record) {
    // id copies, data and name are references
    let Record { id, data, name } = record;
    // id: u64 (copied), data: &Vec<u8>, name: &String

    // This works because u64: Copy, so it's copied rather than moved
    let _ = (id, data, name);
}

fn force_ref_for_copy(record: &Record) {
    // Force reference even for Copy types
    let Record { ref id, .. } = *record;
    // id: &u64
    let _ = id;
}

fn main() {
    let r = Record {
        id: 42,
        data: vec![1, 2, 3],
        name: String::from("test"),
    };
    mixed_bindings(&r);
    force_ref_for_copy(&r);
    println!("Mixed bindings example completed");
}
