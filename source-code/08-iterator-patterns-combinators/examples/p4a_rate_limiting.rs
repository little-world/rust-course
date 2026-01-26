//! Pattern 4a: Streaming Algorithms
//! Example: Rate Limiting Iterator
//!
//! Run with: cargo run --example p4a_rate_limiting

use std::time::{Duration, Instant};

/// Wraps any iterator and rate-limits items to the specified interval.
struct RateLimited<I> {
    iter: I,
    interval: Duration,
    last_yield: Option<Instant>,
}

impl<I: Iterator> RateLimited<I> {
    fn new(iter: I, interval: Duration) -> Self {
        RateLimited {
            iter,
            interval,
            last_yield: None,
        }
    }
}

impl<I: Iterator> Iterator for RateLimited<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(last) = self.last_yield {
            let elapsed = last.elapsed();
            if elapsed < self.interval {
                std::thread::sleep(self.interval - elapsed);
            }
        }

        let item = self.iter.next()?;
        self.last_yield = Some(Instant::now());
        Some(item)
    }
}

/// Helper function to create rate-limited iterator.
fn rate_limit<I: IntoIterator>(
    iter: I,
    interval: Duration,
) -> RateLimited<I::IntoIter> {
    RateLimited::new(iter.into_iter(), interval)
}

/// Token bucket rate limiter for bursty traffic.
struct TokenBucket<I> {
    iter: I,
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl<I: Iterator> TokenBucket<I> {
    fn new(iter: I, max_tokens: f64, refill_rate: f64) -> Self {
        TokenBucket {
            iter,
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

impl<I: Iterator> Iterator for TokenBucket<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.refill();

        // Wait for a token if needed
        while self.tokens < 1.0 {
            let wait_time = (1.0 - self.tokens) / self.refill_rate;
            std::thread::sleep(Duration::from_secs_f64(wait_time));
            self.refill();
        }

        self.tokens -= 1.0;
        self.iter.next()
    }
}

fn main() {
    println!("=== Rate Limiting Iterator ===\n");

    println!("Note: This example involves real delays.\n");

    // Demonstrate basic rate limiting
    println!("=== Fixed Rate (100ms interval) ===");
    let items = vec!["request_1", "request_2", "request_3", "request_4"];
    let start = Instant::now();

    for item in rate_limit(items, Duration::from_millis(100)) {
        println!("  [{:?}] Processing: {}", start.elapsed(), item);
    }

    println!("\n=== Rate Limiting Concepts ===");
    println!("Fixed interval:");
    println!("  - Guarantees minimum time between items");
    println!("  - Use case: API calls with rate limits");
    println!("");
    println!("Token bucket:");
    println!("  - Allows bursts up to max_tokens");
    println!("  - Refills at constant rate");
    println!("  - More flexible for bursty workloads");

    // Demonstrate token bucket (small example)
    println!("\n=== Token Bucket (3 tokens, 2/sec refill) ===");
    let items2 = vec!["A", "B", "C", "D", "E", "F"];
    let start2 = Instant::now();

    let bucket = TokenBucket::new(items2.into_iter(), 3.0, 2.0);
    for item in bucket {
        println!("  [{:?}] Processing: {}", start2.elapsed(), item);
    }

    println!("\n=== Typical Use Cases ===");
    println!("1. API rate limiting (e.g., 100 requests/minute)");
    println!("2. Network request pacing");
    println!("3. Database query throttling");
    println!("4. Event processing backpressure");
    println!("5. Resource consumption control");

    println!("\n=== Key Points ===");
    println!("1. Wrap any iterator to add rate limiting");
    println!("2. Use Instant for accurate timing");
    println!("3. thread::sleep for waiting");
    println!("4. Token bucket allows controlled bursting");
}
