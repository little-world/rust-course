// Pattern 4: Lifetime Elision (The Common Case)
// The compiler infers lifetimes in these common cases:

// Rule 1: Each reference parameter gets its own lifetime
fn print(s: &str) { println!("{}", s); }
// Compiler sees: fn print<'a>(s: &'a str)

// Rule 2: If one input lifetime, output gets that lifetime
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
// Compiler sees: fn first_word<'a>(s: &'a str) -> &'a str

// Rule 3: If &self or &mut self, output gets self's lifetime
struct Player { name: String }
impl Player {
    fn name(&self) -> &str {
        &self.name
    }
    // Compiler sees: fn name<'a>(&'a self) -> &'a str
}

fn main() {
    print("hello");

    let sentence = "hello world";
    println!("First word: {}", first_word(sentence));

    let player = Player { name: "Hero".into() };
    println!("Player name: {}", player.name());

    println!("Lifetime elision example completed");
}
