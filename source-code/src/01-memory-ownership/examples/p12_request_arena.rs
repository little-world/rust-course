// Pattern 12: Per-Request Arena (Web Server Pattern)

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

    fn alloc_slice<T: Copy>(&mut self, slice: &[T]) -> &mut [T] {
        let size = std::mem::size_of::<T>() * slice.len();
        let align = std::mem::align_of::<T>();

        let padding = (align - (self.position % align)) % align;
        self.position += padding;

        if self.position + size > self.current.len() {
            let new_size = (size + 4095) & !4095; // Round up to 4KB
            let old = std::mem::replace(&mut self.current, vec![0; new_size.max(4096)]);
            self.chunks.push(old);
            self.position = 0;
        }

        let ptr = &mut self.current[self.position] as *mut u8 as *mut T;
        self.position += size;

        unsafe {
            std::ptr::copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());
            std::slice::from_raw_parts_mut(ptr, slice.len())
        }
    }
}

struct RequestArena {
    arena: Arena,
}

impl RequestArena {
    fn new() -> Self {
        RequestArena { arena: Arena::new() }
    }

    fn alloc_str(&mut self, s: &str) -> &str {
        let bytes = self.arena.alloc_slice(s.as_bytes());
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}

// Usage: Each request gets its own arena
fn handle_request(data: &str) {
    let mut arena = RequestArena::new();
    let parsed = arena.alloc_str(data);
    println!("Processing: {}", parsed);
    // ... process request using arena for all allocations ...
} // All request memory freed instantly here

fn main() {
    handle_request("GET /api/users HTTP/1.1");
    handle_request("POST /api/data HTTP/1.1");
    println!("Request arena example completed");
}
