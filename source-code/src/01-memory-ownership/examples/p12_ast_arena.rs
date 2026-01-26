// Pattern 12: AST Arena
// Note: This is a simplified example. For production, use typed-arena or bumpalo crates.

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

        let padding = (align - (self.position % align)) % align;
        self.position += padding;

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

#[derive(Debug)]
#[allow(dead_code)]
enum Expr {
    Number(i64),
    Add(usize, usize),  // Indices into arena (simplified)
    Mul(usize, usize),
}

struct AstArena {
    arena: Arena,
    exprs: Vec<*mut Expr>,
}

impl AstArena {
    fn new() -> Self {
        AstArena { arena: Arena::new(), exprs: Vec::new() }
    }

    fn number(&mut self, n: i64) -> usize {
        let expr = self.arena.alloc(Expr::Number(n));
        let idx = self.exprs.len();
        self.exprs.push(expr as *mut Expr);
        idx
    }

    fn add(&mut self, l: usize, r: usize) -> usize {
        let expr = self.arena.alloc(Expr::Add(l, r));
        let idx = self.exprs.len();
        self.exprs.push(expr as *mut Expr);
        idx
    }

    fn get(&self, idx: usize) -> &Expr {
        unsafe { &*self.exprs[idx] }
    }
}

fn main() {
    // Usage: Build AST with fast allocation
    let mut ast = AstArena::new();
    let one = ast.number(1);
    let two = ast.number(2);
    let sum = ast.add(one, two);

    println!("Expression at index {}: {:?}", sum, ast.get(sum));
    println!("AST Arena example completed");
}
