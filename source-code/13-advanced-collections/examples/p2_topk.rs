//! Pattern 2: BinaryHeap and Priority Queues
//! Top-K Frequent Elements
//!
//! Run with: cargo run --example p2_topk

use std::collections::{HashMap, BinaryHeap};
use std::cmp::{Ordering, Reverse};

#[derive(Eq, PartialEq)]
struct FreqItem<T> {
    item: T,
    count: usize,
}

impl<T: Eq> Ord for FreqItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)
    }
}

impl<T: Eq> PartialOrd for FreqItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct TopKFrequent<T> {
    counts: HashMap<T, usize>,
    k: usize,
}

impl<T> TopKFrequent<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn new(k: usize) -> Self {
        Self {
            counts: HashMap::new(),
            k,
        }
    }

    fn add(&mut self, item: T) {
        *self.counts.entry(item).or_insert(0) += 1;
    }

    fn add_batch(&mut self, items: Vec<T>) {
        for item in items {
            self.add(item);
        }
    }

    fn top_k(&self) -> Vec<(T, usize)> {
        // Use min-heap to keep only top k
        let mut heap: BinaryHeap<Reverse<FreqItem<&T>>> = BinaryHeap::new();

        for (item, &count) in &self.counts {
            heap.push(Reverse(FreqItem { item, count }));

            if heap.len() > self.k {
                heap.pop();
            }
        }

        heap.into_iter()
            .map(|Reverse(freq_item)| (freq_item.item.clone(), freq_item.count))
            .collect()
    }

    fn top_k_sorted(&self) -> Vec<(T, usize)> {
        let mut result = self.top_k();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }
}

//=========================
// Real-world: Log analysis
//=========================
struct LogAnalyzer {
    error_tracker: TopKFrequent<String>,
    ip_tracker: TopKFrequent<String>,
    endpoint_tracker: TopKFrequent<String>,
}

impl LogAnalyzer {
    fn new(k: usize) -> Self {
        Self {
            error_tracker: TopKFrequent::new(k),
            ip_tracker: TopKFrequent::new(k),
            endpoint_tracker: TopKFrequent::new(k),
        }
    }

    fn process_log(&mut self, log_entry: LogEntry) {
        if let Some(error) = log_entry.error {
            self.error_tracker.add(error);
        }
        self.ip_tracker.add(log_entry.ip);
        self.endpoint_tracker.add(log_entry.endpoint);
    }

    fn report(&self) {
        println!("Top Errors:");
        for (error, count) in self.error_tracker.top_k_sorted() {
            println!("  {}: {}", error, count);
        }

        println!("\nTop IP Addresses:");
        for (ip, count) in self.ip_tracker.top_k_sorted() {
            println!("  {}: {}", ip, count);
        }

        println!("\nTop Endpoints:");
        for (endpoint, count) in self.endpoint_tracker.top_k_sorted() {
            println!("  {}: {}", endpoint, count);
        }
    }
}

#[derive(Debug, Clone)]
struct LogEntry {
    ip: String,
    endpoint: String,
    error: Option<String>,
}

fn main() {
    println!("=== Top-K Frequent Elements ===\n");

    let mut tracker = TopKFrequent::new(3);

    let words = vec![
        "apple", "banana", "apple", "cherry", "banana", "apple",
        "date", "banana", "apple", "cherry",
    ];

    tracker.add_batch(words.iter().map(|&s| s.to_string()).collect());

    println!("Top 3 words:");
    for (word, count) in tracker.top_k_sorted() {
        println!("  {}: {}", word, count);
    }

    println!("\n=== Log Analysis ===\n");

    let mut analyzer = LogAnalyzer::new(3);

    // Simulate logs
    let logs = vec![
        LogEntry {
            ip: "192.168.1.1".to_string(),
            endpoint: "/api/users".to_string(),
            error: None,
        },
        LogEntry {
            ip: "192.168.1.2".to_string(),
            endpoint: "/api/posts".to_string(),
            error: Some("404 Not Found".to_string()),
        },
        LogEntry {
            ip: "192.168.1.1".to_string(),
            endpoint: "/api/users".to_string(),
            error: None,
        },
        LogEntry {
            ip: "192.168.1.3".to_string(),
            endpoint: "/api/posts".to_string(),
            error: Some("500 Internal Error".to_string()),
        },
        LogEntry {
            ip: "192.168.1.1".to_string(),
            endpoint: "/api/comments".to_string(),
            error: Some("404 Not Found".to_string()),
        },
    ];

    for log in logs {
        analyzer.process_log(log);
    }

    analyzer.report();

    println!("\n=== Key Points ===");
    println!("1. Min-heap of size k for top-k elements");
    println!("2. Time: O(n log k) vs O(n log n) for full sort");
    println!("3. Space: O(k) for heap vs O(n) for sorting all");
    println!("4. Efficient for streaming data analysis");
}
