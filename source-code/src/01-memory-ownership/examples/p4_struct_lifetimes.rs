// Pattern 4: Struct Lifetimes

// Struct that borrows data
struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.position..]
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
    }
}

fn use_parser() {
    let text = String::from("hello world");
    let mut parser = Parser::new(&text);

    println!("Remaining: {}", parser.remaining());
    parser.advance(6);
    println!("Remaining: {}", parser.remaining());
}
// parser must not outlive text

fn main() {
    use_parser();
    println!("Struct lifetimes example completed");
}
