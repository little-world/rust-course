//! Pattern 1: The Entry API
//! LRU Cache Implementation
//!
//! Run with: cargo run --example p1_lru_cache

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

fn main() {
    println!("=== LRU Cache with Entry API ===\n");

    let mut cache = LruCache::new(3);

    // Insert some items
    println!("Inserting items:");
    cache.put("a", 1);
    println!("  put('a', 1) -> cache: {:?}", cache.keys());

    cache.put("b", 2);
    println!("  put('b', 2) -> cache: {:?}", cache.keys());

    cache.put("c", 3);
    println!("  put('c', 3) -> cache: {:?}", cache.keys());

    // This should evict "a" (least recently used)
    cache.put("d", 4);
    println!("  put('d', 4) -> cache: {:?} ('a' evicted)", cache.keys());

    // Access "b" to make it recently used
    println!("\nAccessing 'b':");
    if let Some(v) = cache.get(&"b") {
        println!("  get('b') = {}", v);
    }
    println!("  Order after get: {:?}", cache.keys());

    // This should evict "c" (now the least recently used)
    cache.put("e", 5);
    println!("\nput('e', 5) -> cache: {:?} ('c' evicted)", cache.keys());

    println!("\n=== Key Points ===");
    println!("1. Entry::Occupied for updating existing entries");
    println!("2. Entry::Vacant for inserting new entries");
    println!("3. VecDeque tracks usage order");
    println!("4. O(n) removal from order queue (O(1) with linked list)");
}

struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>, // Tracks usage order, from least to most recent.
}

impl<K: Eq + Hash + Clone, V> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn put(&mut self, key: K, value: V) {
        // Check if key already exists
        if self.map.contains_key(&key) {
            // Key already exists, update the value.
            self.map.insert(key.clone(), value);
            // Move it to the back of the usage queue (most recent).
            self.order.retain(|k| k != &key);
            self.order.push_back(key);
        } else {
            // Key is new. First, check if we need to evict an old entry.
            if self.map.len() >= self.capacity {
                if let Some(lru_key) = self.order.pop_front() {
                    self.map.remove(&lru_key);
                }
            }
            // Insert the new value and add it to the back of the usage queue.
            self.map.insert(key.clone(), value);
            self.order.push_back(key);
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            // Move to back of order queue (most recently used)
            self.order.retain(|k| k != key);
            self.order.push_back(key.clone());
            self.map.get(key)
        } else {
            None
        }
    }

    fn keys(&self) -> Vec<&K> {
        self.order.iter().collect()
    }
}
