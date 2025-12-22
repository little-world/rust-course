// Milestone 1: Basic Statistics Tracker
mod milestone_1 {
    use std::cell::Cell;

    struct StatsTracker {
        hits: Cell<usize>,
        misses: Cell<usize>,
    }

    impl StatsTracker {
        fn new() -> Self {
            StatsTracker {
                hits: Cell::new(0),
                misses: Cell::new(0),
            }
        }

        fn record_hit(&self) {
            self.hits.set(self.hits.get() + 1);
        }

        fn record_miss(&self) {
            self.misses.set(self.misses.get() + 1);
        }

        fn get_stats(&self) -> (usize, usize) {
            (self.hits.get(), self.misses.get())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_stats_tracker() {
            let tracker = StatsTracker::new();
            assert_eq!(tracker.get_stats(), (0, 0));

            tracker.record_hit();
            tracker.record_hit();
            assert_eq!(tracker.get_stats(), (2, 0));

            tracker.record_miss();
            assert_eq!(tracker.get_stats(), (2, 1));
        }

        #[test]
        fn test_multiple_references() {
            let tracker = StatsTracker::new();
            let ref1 = &tracker;
            let ref2 = &tracker;

            ref1.record_hit();
            ref2.record_miss();

            assert_eq!(tracker.get_stats(), (1, 1));
        }
    }
}

// Milestone 2: Simple HashMap Cache (No Eviction Yet)
mod milestone_2 {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::hash::Hash;

    struct SimpleCache<K, V> {
        data: RefCell<HashMap<K, V>>,
    }

    impl<K, V> SimpleCache<K, V>
    where
        K: Eq + Hash,
        V: Clone,
    {
        fn new() -> Self {
            SimpleCache {
                data: RefCell::new(HashMap::new()),
            }
        }

        fn get(&self, key: &K) -> Option<V> {
            self.data.borrow().get(key).cloned()
        }

        fn put(&self, key: K, value: V) {
            self.data.borrow_mut().insert(key, value);
        }

        fn len(&self) -> usize {
            self.data.borrow().len()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_simple_cache() {
            let cache = SimpleCache::new();
            assert_eq!(cache.len(), 0);

            cache.put("key1", "value1");
            assert_eq!(cache.len(), 1);

            assert_eq!(cache.get(&"key1"), Some("value1"));
            assert_eq!(cache.get(&"key2"), None);
        }

        #[test]
        fn test_update_existing() {
            let cache = SimpleCache::new();
            cache.put("key", "value1");
            cache.put("key", "value2");

            assert_eq!(cache.len(), 1);
            assert_eq!(cache.get(&"key"), Some("value2"));
        }

        #[test]
        #[should_panic]
        fn test_borrow_violation() {
            let cache: SimpleCache<i32, i32> = SimpleCache::new();
            cache.put(1, 100);

            // This should panic: holding borrow across another borrow_mut
            let _data = cache.data.borrow();
            cache.put(2, 200); // This will panic!
        }
    }
}

// Milestone 3: LRU Cache with Fixed Capacity
mod milestone_3 {
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::hash::Hash;

    struct LRUCache<K, V> {
        capacity: usize,
        data: RefCell<HashMap<K, V>>,
        order: RefCell<VecDeque<K>>, // Most recent at back
    }

    impl<K, V> LRUCache<K, V>
    where
        K: Eq + Hash + Clone,
        V: Clone,
    {
        fn new(capacity: usize) -> Self {
            assert!(capacity > 0, "Capacity must be greater than 0");
            LRUCache {
                capacity,
                data: RefCell::new(HashMap::with_capacity(capacity)),
                order: RefCell::new(VecDeque::with_capacity(capacity)),
            }
        }

        fn get(&self, key: &K) -> Option<V> {
            let data = self.data.borrow();
            if let Some(value) = data.get(key) {
                let value_clone = value.clone();
                drop(data); // Release borrow before mutating order

                let mut order = self.order.borrow_mut();
                if let Some(pos) = order.iter().position(|k| k == key) {
                    order.remove(pos);
                }
                order.push_back(key.clone());
                Some(value_clone)
            } else {
                None
            }
        }

        fn put(&self, key: K, value: V) {
            let mut data = self.data.borrow_mut();
            let mut order = self.order.borrow_mut();

            if let Some(_old_value) = data.insert(key.clone(), value) {
                // Key already existed, just update value and move to back
                if let Some(pos) = order.iter().position(|k| k == &key) {
                    order.remove(pos);
                }
            } else {
                // Key is new, check capacity
                if data.len() > self.capacity {
                    if let Some(lru_key) = order.pop_front() {
                        data.remove(&lru_key);
                    }
                }
            }
            order.push_back(key);
        }

        fn len(&self) -> usize {
            self.data.borrow().len()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_lru_basic() {
            let cache = LRUCache::new(2);
            cache.put("a", 1);
            cache.put("b", 2);

            assert_eq!(cache.get(&"a"), Some(1));
            assert_eq!(cache.get(&"b"), Some(2));
            assert_eq!(cache.len(), 2);
        }

        #[test]
        fn test_lru_eviction() {
            let cache = LRUCache::new(2);
            cache.put("a", 1);
            cache.put("b", 2);
            cache.put("c", 3); // Should evict "a"

            assert_eq!(cache.get(&"a"), None); // "a" was evicted
            assert_eq!(cache.get(&"b"), Some(2));
            assert_eq!(cache.get(&"c"), Some(3));
            assert_eq!(cache.len(), 2);
        }

        #[test]
        fn test_lru_access_order() {
            let cache = LRUCache::new(2);
            cache.put("a", 1);
            cache.put("b", 2);

            // Access "a" to make it more recent
            assert_eq!(cache.get(&"a"), Some(1));

            // Insert "c" - should evict "b" (now least recent)
            cache.put("c", 3);

            assert_eq!(cache.get(&"a"), Some(1));
            assert_eq!(cache.get(&"b"), None); // "b" was evicted
            assert_eq!(cache.get(&"c"), Some(3));
        }

        #[test]
        fn test_update_existing() {
            let cache = LRUCache::new(2);
            cache.put("a", 1);
            cache.put("a", 10); // Update

            assert_eq!(cache.get(&"a"), Some(10));
            assert_eq!(cache.len(), 1);
        }

        #[test]
        fn test_capacity_one() {
            let cache = LRUCache::new(1);
            cache.put("a", 1);
            cache.put("b", 2);

            assert_eq!(cache.get(&"a"), None);
            assert_eq!(cache.get(&"b"), Some(2));
        }
    }
}

// Milestone 4: Add Statistics Tracking
mod milestone_4 {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::hash::Hash;

    struct StatsTracker {
        hits: Cell<usize>,
        misses: Cell<usize>,
    }

    impl StatsTracker {
        fn new() -> Self {
            StatsTracker {
                hits: Cell::new(0),
                misses: Cell::new(0),
            }
        }
        fn record_hit(&self) { self.hits.set(self.hits.get() + 1); }
        fn record_miss(&self) { self.misses.set(self.misses.get() + 1); }
        fn get_stats(&self) -> (usize, usize) { (self.hits.get(), self.misses.get()) }
    }

    struct LRUCache<K, V> {
        capacity: usize,
        data: RefCell<HashMap<K, V>>,
        order: RefCell<VecDeque<K>>,
        stats: StatsTracker,
    }

    impl<K, V> LRUCache<K, V>
    where
        K: Eq + Hash + Clone,
        V: Clone,
    {
        fn new(capacity: usize) -> Self {
            assert!(capacity > 0, "Capacity must be > 0");
            LRUCache {
                capacity,
                data: RefCell::new(HashMap::with_capacity(capacity)),
                order: RefCell::new(VecDeque::with_capacity(capacity)),
                stats: StatsTracker::new(),
            }
        }

        fn get(&self, key: &K) -> Option<V> {
            let data = self.data.borrow();
            if let Some(value) = data.get(key) {
                self.stats.record_hit();
                let value_clone = value.clone();
                drop(data);

                let mut order = self.order.borrow_mut();
                if let Some(pos) = order.iter().position(|k| k == key) {
                    order.remove(pos);
                }
                order.push_back(key.clone());
                Some(value_clone)
            } else {
                self.stats.record_miss();
                None
            }
        }

        fn put(&self, key: K, value: V) {
            let mut data = self.data.borrow_mut();
            let mut order = self.order.borrow_mut();

            if data.insert(key.clone(), value).is_none() {
                // New key inserted
                if data.len() > self.capacity {
                    if let Some(lru_key) = order.pop_front() {
                        data.remove(&lru_key);
                    }
                }
            } else {
                // Key existed, remove old entry from order
                if let Some(pos) = order.iter().position(|k| k == &key) {
                    order.remove(pos);
                }
            }
            order.push_back(key);
        }

        fn len(&self) -> usize { self.data.borrow().len() }
        fn stats(&self) -> (usize, usize) { self.stats.get_stats() }
        fn clear(&self) {
            self.data.borrow_mut().clear();
            self.order.borrow_mut().clear();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_stats_tracking() {
            let cache = LRUCache::new(2);
            cache.put("a", 1);
            cache.put("b", 2);

            cache.get(&"a"); // Hit
            cache.get(&"b"); // Hit
            cache.get(&"c"); // Miss

            let (hits, misses) = cache.stats();
            assert_eq!(hits, 2);
            assert_eq!(misses, 1);
        }

        #[test]
        fn test_clear() {
            let cache = LRUCache::new(2);
            cache.put("a", 1);
            cache.get(&"a");
            cache.get(&"b"); // miss

            cache.clear();
            assert_eq!(cache.len(), 0);

            let (hits, misses) = cache.stats();
            assert_eq!(hits, 1); // Stats persist
            assert_eq!(misses, 1);
        }
    }
}

// Milestone 5: Thread-Safe Version with Mutex
mod milestone_5 {
    use std::collections::{HashMap, VecDeque};
    use std::hash::Hash;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    // StatsTracker now uses AtomicUsize for thread safety
    struct StatsTracker {
        hits: AtomicUsize,
        misses: AtomicUsize,
    }

    impl StatsTracker {
        fn new() -> Self {
            StatsTracker {
                hits: AtomicUsize::new(0),
                misses: AtomicUsize::new(0),
            }
        }
        fn record_hit(&self) { self.hits.fetch_add(1, Ordering::Relaxed); }
        fn record_miss(&self) { self.misses.fetch_add(1, Ordering::Relaxed); }
        fn get_stats(&self) -> (usize, usize) {
            (self.hits.load(Ordering::Relaxed), self.misses.load(Ordering::Relaxed))
        }
    }

    // Inner state of the cache, to be protected by a Mutex
    struct CacheInner<K, V> {
        data: HashMap<K, V>,
        order: VecDeque<K>,
    }

    struct ThreadSafeLRUCache<K, V> {
        capacity: usize,
        inner: Mutex<CacheInner<K, V>>,
        stats: StatsTracker,
    }

    impl<K, V> ThreadSafeLRUCache<K, V>
    where
        K: Eq + Hash + Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        fn new(capacity: usize) -> Self {
            assert!(capacity > 0, "Capacity must be > 0");
            ThreadSafeLRUCache {
                capacity,
                inner: Mutex::new(CacheInner {
                    data: HashMap::with_capacity(capacity),
                    order: VecDeque::with_capacity(capacity),
                }),
                stats: StatsTracker::new(),
            }
        }

        fn get(&self, key: &K) -> Option<V> {
            // Lock is held for a short duration
            let mut inner = self.inner.lock().unwrap();
            // Clone the value first to end the immutable borrow before mutating `order`
            if let Some(value) = inner.data.get(key).cloned() {
                self.stats.record_hit();
                // Move key to back
                if let Some(pos) = inner.order.iter().position(|k| k == key) {
                    inner.order.remove(pos);
                }
                inner.order.push_back(key.clone());
                Some(value)
            } else {
                self.stats.record_miss();
                None
            }
        }

        fn put(&self, key: K, value: V) {
            let mut inner = self.inner.lock().unwrap();
            if inner.data.insert(key.clone(), value).is_none() {
                // New key
                if inner.data.len() > self.capacity {
                    if let Some(lru_key) = inner.order.pop_front() {
                        inner.data.remove(&lru_key);
                    }
                }
            } else {
                // Key existed, remove old from order
                if let Some(pos) = inner.order.iter().position(|k| k == &key) {
                    inner.order.remove(pos);
                }
            }
            inner.order.push_back(key);
        }

        fn len(&self) -> usize {
            self.inner.lock().unwrap().data.len()
        }

        fn stats(&self) -> (usize, usize) {
            self.stats.get_stats()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_thread_safe_basic() {
            let cache = Arc::new(ThreadSafeLRUCache::new(10));

            let cache_clone = Arc::clone(&cache);
            let handle = std::thread::spawn(move || {
                cache_clone.put("thread_key".to_string(), 42);
            });

            handle.join().unwrap();
            assert_eq!(cache.get(&"thread_key".to_string()), Some(42));
            assert_eq!(cache.stats(), (0, 1)); // get was a miss before put
        }

        #[test]
        fn test_concurrent_access() {
            let cache = Arc::new(ThreadSafeLRUCache::new(100));
            let mut handles = vec![];

            for i in 0..10 {
                let cache_clone = Arc::clone(&cache);
                let handle = std::thread::spawn(move || {
                    for j in 0..10 {
                        cache_clone.put(i * 10 + j, i * 100 + j);
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            assert_eq!(cache.len(), 100);
        }
    }
}

// Provide a main entry point to satisfy the binary crate requirements
fn main() {}