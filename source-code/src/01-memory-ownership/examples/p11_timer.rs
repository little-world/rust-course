// Pattern 11: Timing Guard
use std::time::Instant;

struct Timer<'a> {
    name: &'a str,
    start: Instant,
}

impl<'a> Timer<'a> {
    fn new(name: &'a str) -> Self {
        Timer { name, start: Instant::now() }
    }
}

impl Drop for Timer<'_> {
    fn drop(&mut self) {
        println!("{}: {:?}", self.name, self.start.elapsed());
    }
}

// Usage: Automatically prints elapsed time
fn do_work() {
    let _timer = Timer::new("do_work");
    // ... expensive operation ...
    let mut sum = 0u64;
    for i in 0..1_000_000 {
        sum += i;
    }
    println!("Sum: {}", sum);
} // Prints "do_work: Xms" when scope ends

fn main() {
    do_work();
    println!("Timer example completed");
}
