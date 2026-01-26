// Pattern 1: Stack vs Heap Allocation
fn stack_vs_heap() {
    // Stack allocated: size known at compile time
    let x: i32 = 42;                    // 4 bytes on stack
    let arr: [i32; 100] = [0; 100];     // 400 bytes on stack

    // Heap allocated: size can be dynamic
    let vec: Vec<i32> = vec![1, 2, 3];  // 24 bytes on stack (ptr, len, cap)
                                         // + 12 bytes on heap (actual data)

    let boxed: Box<i32> = Box::new(42); // 8 bytes on stack (pointer)
                                         // + 4 bytes on heap (the i32)

    let string: String = String::from("hello"); // 24 bytes on stack
                                                 // + 5 bytes on heap

    println!("x = {}", x);
    println!("arr[0] = {}", arr[0]);
    println!("vec = {:?}", vec);
    println!("boxed = {}", boxed);
    println!("string = {}", string);
}
// All memory (stack and heap) freed here automatically

fn main() {
    stack_vs_heap();
    println!("Stack vs Heap example completed");
}
