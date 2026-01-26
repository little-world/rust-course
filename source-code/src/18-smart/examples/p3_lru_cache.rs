// Pattern 3: Intrusive Data Structures - LRU Cache
use std::collections::HashMap;
use std::ptr;

struct LruNode<K, V> {
    key: K,
    value: V,
    prev: *mut LruNode<K, V>,
    next: *mut LruNode<K, V>,
}

struct LruCache<K, V> {
    map: HashMap<K, *mut LruNode<K, V>>,
    head: *mut LruNode<K, V>,  // Most recent
    tail: *mut LruNode<K, V>,  // Least recent
    capacity: usize,
}

impl<K: Eq + std::hash::Hash + Clone, V> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        LruCache {
            map: HashMap::new(),
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            capacity,
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        let node_ptr = *self.map.get(key)?;
        unsafe {
            self.detach(node_ptr);
            self.attach_front(node_ptr);
            Some(&(*node_ptr).value)
        }
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(&node_ptr) = self.map.get(&key) {
            unsafe {
                (*node_ptr).value = value;
                self.detach(node_ptr);
                self.attach_front(node_ptr);
            }
            return;
        }

        // Evict if at capacity
        if self.map.len() >= self.capacity && !self.tail.is_null() {
            unsafe {
                let tail_key = (*self.tail).key.clone();
                let old_tail = self.tail;
                self.detach(old_tail);
                self.map.remove(&tail_key);
                drop(Box::from_raw(old_tail));
            }
        }

        let node = Box::into_raw(Box::new(LruNode {
            key: key.clone(),
            value,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }));

        self.map.insert(key, node);
        unsafe { self.attach_front(node); }
    }

    unsafe fn detach(&mut self, node: *mut LruNode<K, V>) {
        let prev = (*node).prev;
        let next = (*node).next;

        if !prev.is_null() { (*prev).next = next; }
        else { self.head = next; }

        if !next.is_null() { (*next).prev = prev; }
        else { self.tail = prev; }

        (*node).prev = ptr::null_mut();
        (*node).next = ptr::null_mut();
    }

    unsafe fn attach_front(&mut self, node: *mut LruNode<K, V>) {
        (*node).next = self.head;
        if !self.head.is_null() { (*self.head).prev = node; }
        self.head = node;
        if self.tail.is_null() { self.tail = node; }
    }
}

impl<K, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        unsafe {
            let mut current = self.head;
            while !current.is_null() {
                let next = (*current).next;
                drop(Box::from_raw(current));
                current = next;
            }
        }
    }
}

fn main() {
    // Usage
    let mut cache = LruCache::new(2);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.get(&"a");      // "a" becomes most recent
    cache.put("c", 3);    // Evicts "b" (least recent)
    assert!(cache.get(&"b").is_none());

    println!("LRU cache example completed");
}
