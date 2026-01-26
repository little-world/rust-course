// Pattern 12: Bump Allocator (Arena)
struct Arena {
    chunks: Vec<Vec<u8>>,
    current: Vec<u8>,
    position: usize,
}

impl Arena {
    fn new() -> Self {
        Arena {
            chunks: Vec::new(),
            current: vec![0; 4096],
            position: 0,
        }
    }

    fn alloc<T>(&mut self, value: T) -> &mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Align position
        let padding = (align - (self.position % align)) % align;
        self.position += padding;

        // New chunk if needed
        if self.position + size > self.current.len() {
            let old = std::mem::replace(&mut self.current, vec![0; 4096]);
            self.chunks.push(old);
            self.position = 0;
        }

        let ptr = &mut self.current[self.position] as *mut u8 as *mut T;
        self.position += size;

        unsafe {
            std::ptr::write(ptr, value);
            &mut *ptr
        }
    }
}

fn main() {
    // Usage: Fast allocation for many small objects
    let mut arena = Arena::new();

    // Allocate and use each value separately to satisfy borrow checker
    let a = arena.alloc(42i32);
    println!("Allocated integer: {}", a);

    let b = arena.alloc(String::from("hello"));
    println!("Allocated string: {}", b);

    // Allocate more values
    let c = arena.alloc(3.14f64);
    println!("Allocated float: {}", c);

    // All memory freed when arena drops
    println!("Arena example completed");
}
