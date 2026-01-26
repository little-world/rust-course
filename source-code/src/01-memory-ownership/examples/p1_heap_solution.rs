// Pattern 1: When Stack Fails - Heap Solution

// This would overflow the stack (don't do this!)
// fn stack_overflow() {
//     let huge: [u8; 10_000_000] = [0; 10_000_000]; // 10MB on stack!
// }

// Solution: Use heap allocation
fn heap_solution() {
    let huge: Vec<u8> = vec![0; 10_000_000]; // 10MB on heap, safe
    println!("Allocated {} bytes", huge.len());
}

fn main() {
    heap_solution();
    println!("Heap solution example completed");
}
