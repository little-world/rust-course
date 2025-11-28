# Project 3: Copy-on-Write Data Structures for Efficient Sharing

## Problem Statement

Build a library of Copy-on-Write (CoW) data structures that enable efficient sharing of data until modification is needed. When data is shared, cloning is O(1) (just increment reference count). When data is modified, make a private copy only if other references exist.

The library must support:
- CoW String with cheap cloning
- CoW Vec with structural sharing
- CoW HashMap with lazy copying
- Automatic copy detection (only copy if shared)
- Configurable sharing strategies
- Performance tracking and optimization

## Why It Matters

**Performance Impact:**
- **Cloning large strings**: Normal clone takes ~1μs per KB, CoW clone takes ~10ns (100x faster!)
- **Configuration systems**: Share config across threads, copy only on write
- **Immutable data structures**: Functional programming patterns in Rust
- **Version control**: Git uses CoW for file storage

**Memory Savings:**
```
Normal clones:  5 copies × 1MB = 5MB memory
CoW clones:     5 references × 8 bytes = 40 bytes (until write)
Savings:        99.9% memory reduction
```

## Use Cases

1. **Configuration Management**: Share config across threads, clone on modification
2. **Caching Systems**: Cache entries share backing data until mutated
3. **Immutable Collections**: Functional-style data structures
4. **Version Control**: Store file versions efficiently (like Git)
5. **String Interning**: Share common strings (like "http", "200 OK")
6. **Game State**: Share game state snapshots for replay/undo

---

## Milestone 1: Basic CoW String with Arc

**Goal:** Create a copy-on-write string that shares data until modification.

### Introduction

We start with the simplest CoW implementation: a string that:
- Uses `Arc<String>` for shared data
- Cloning is O(1) (just increment refcount)
- First write makes a private copy
- Supports transparent read access

**Limitations we'll address later:**
- Always copies entire string on first write
- No slice sharing
- Only works for String, not Vec or HashMap
- No performance tracking

### Architecture

```rust
use std::sync::Arc;

pub struct CowString {
    data: Arc<String>,
}
```

**Key Concepts:**
- `Arc::strong_count()`: Check if data is shared
- Clone makes copy only if `strong_count() > 1`
- `Arc::make_mut()`: Gets mutable reference, copying if needed

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let s = CowString::new("hello");
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_cheap_clone() {
        let s1 = CowString::new("hello world");
        let s2 = s1.clone();

        // Both point to same data
        assert_eq!(s1.as_str(), "hello world");
        assert_eq!(s2.as_str(), "hello world");
        assert_eq!(Arc::strong_count(&s1.data), 2);
    }

    #[test]
    fn test_copy_on_write() {
        let s1 = CowString::new("hello");
        let mut s2 = s1.clone();

        // s2 shares data with s1
        assert_eq!(Arc::strong_count(&s1.data), 2);

        // Modify s2 - should copy
        s2.push_str(" world");

        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s2.as_str(), "hello world");

        // s2 now has independent copy
        assert_eq!(Arc::strong_count(&s1.data), 1);
        assert_eq!(Arc::strong_count(&s2.data), 1);
    }

    #[test]
    fn test_exclusive_modification() {
        let mut s = CowString::new("hello");

        // Not shared - no copy needed
        s.push_str(" world");

        assert_eq!(s.as_str(), "hello world");
        assert_eq!(Arc::strong_count(&s.data), 1);
    }

    #[test]
    fn test_multiple_clones() {
        let s1 = CowString::new("shared");
        let s2 = s1.clone();
        let s3 = s1.clone();

        assert_eq!(Arc::strong_count(&s1.data), 3);

        drop(s2);
        assert_eq!(Arc::strong_count(&s1.data), 2);

        drop(s3);
        assert_eq!(Arc::strong_count(&s1.data), 1);
    }

    #[test]
    fn test_from_string() {
        let s = String::from("hello");
        let cow = CowString::from(s);
        assert_eq!(cow.as_str(), "hello");
    }

    #[test]
    fn test_deref() {
        let s = CowString::new("hello world");

        // Can use String methods through Deref
        assert_eq!(s.len(), 11);
        assert!(s.starts_with("hello"));
        assert_eq!(&s[0..5], "hello");
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::ops::Deref;

#[derive(Clone)]
pub struct CowString {
    data: Arc<String>,
}

impl CowString {
    pub fn new(s: impl Into<String>) -> Self {
        todo!("
        Wrap string in Arc:
        CowString {
            data: Arc::new(s.into()),
        }
        ")
    }

    pub fn as_str(&self) -> &str {
        todo!("Return &str from inner Arc<String>")
    }

    pub fn push_str(&mut self, s: &str) {
        todo!("
        Get mutable reference with Arc::make_mut:
        1. Arc::make_mut(&mut self.data) - copies if shared
        2. Call push_str on the String

        Arc::make_mut automatically:
        - Returns &mut String if strong_count == 1
        - Clones and returns &mut String if shared
        ")
    }

    pub fn is_shared(&self) -> bool {
        todo!("Return Arc::strong_count(&self.data) > 1")
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}

impl From<String> for CowString {
    fn from(s: String) -> Self {
        CowString::new(s)
    }
}

impl From<&str> for CowString {
    fn from(s: &str) -> Self {
        CowString::new(s)
    }
}

impl Deref for CowString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        todo!("Return &str from Arc<String>")
    }
}
```

---

## Milestone 2: CoW Vec with Structural Sharing

**Goal:** Extend CoW pattern to `Vec<T>` with element-level sharing.

### Introduction

**Why Milestone 1 Isn't Enough:**

Strings are simple, but Vecs have more complex operations:
1. **Push/pop**: Modify size
2. **Indexing**: Access/modify individual elements
3. **Slicing**: View subsets
4. **Generic types**: Must work with any `T: Clone`

**Real-world scenario:** Configuration system with arrays:
```rust
let config = CowVec::from(vec![1, 2, 3, 4, 5]);

// 10 threads read config (cheap clones)
let threads: Vec<_> = (0..10)
    .map(|_| {
        let cfg = config.clone(); // O(1) clone
        thread::spawn(move || process(cfg))
    })
    .collect();

// One thread modifies (triggers copy)
let mut modified = config.clone();
modified.push(6); // Copy happens here
```

**Challenge:** Implement Index, IndexMut, push, pop, etc.

### Architecture

```rust
use std::sync::Arc;

pub struct CowVec<T> {
    data: Arc<Vec<T>>,
}
```

**Key Methods:**
- `push(&mut self, value: T)`: Append element
- `pop(&mut self) -> Option<T>`: Remove last
- `get(&self, index: usize) -> Option<&T>`: Read element
- `get_mut(&mut self, index: usize) -> Option<&mut T>`: Write element

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vec() {
        let v = CowVec::from(vec![1, 2, 3]);
        assert_eq!(v.len(), 3);
        assert_eq!(v.get(0), Some(&1));
    }

    #[test]
    fn test_clone_vec() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let v2 = v1.clone();

        assert_eq!(v1.strong_count(), 2);
        assert_eq!(v1[0], 1);
        assert_eq!(v2[0], 1);
    }

    #[test]
    fn test_copy_on_push() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let mut v2 = v1.clone();

        assert_eq!(v1.strong_count(), 2);

        v2.push(4);

        // v2 copied
        assert_eq!(v1.len(), 3);
        assert_eq!(v2.len(), 4);
        assert_eq!(v1.strong_count(), 1);
        assert_eq!(v2.strong_count(), 1);
    }

    #[test]
    fn test_copy_on_modify() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let mut v2 = v1.clone();

        // Modify via index
        v2[0] = 99;

        assert_eq!(v1[0], 1);
        assert_eq!(v2[0], 99);
    }

    #[test]
    fn test_pop() {
        let mut v = CowVec::from(vec![1, 2, 3]);

        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_iter() {
        let v = CowVec::from(vec![1, 2, 3, 4, 5]);

        let sum: i32 = v.iter().sum();
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_shared_iter() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let v2 = v1.clone();

        // Both can iterate
        assert_eq!(v1.iter().sum::<i32>(), 6);
        assert_eq!(v2.iter().sum::<i32>(), 6);

        // Still shared
        assert_eq!(v1.strong_count(), 2);
    }

    #[test]
    fn test_into_vec() {
        let cow = CowVec::from(vec![1, 2, 3]);

        let vec = cow.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_into_vec_shared() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let v2 = v1.clone();

        // Must clone because shared
        let vec = v1.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);

        // v2 still valid
        assert_eq!(v2[0], 1);
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::ops::{Deref, Index, IndexMut};

#[derive(Clone)]
pub struct CowVec<T> {
    data: Arc<Vec<T>>,
}

impl<T: Clone> CowVec<T> {
    pub fn new() -> Self {
        CowVec {
            data: Arc::new(Vec::new()),
        }
    }

    pub fn push(&mut self, value: T) {
        todo!("
        Use Arc::make_mut to get mutable Vec:
        Arc::make_mut(&mut self.data).push(value);
        ")
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!("Get mut ref and pop")
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        todo!("Return self.data.get(index)")
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        todo!("
        Get mutable reference:
        Arc::make_mut(&mut self.data).get_mut(index)
        ")
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        todo!("Return self.data.iter()")
    }

    pub fn into_vec(self) -> Vec<T> {
        todo!("
        Try to unwrap Arc:
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())

        If successful (not shared), returns Vec without clone.
        If shared, clones the Vec.
        ")
    }
}

impl<T: Clone> From<Vec<T>> for CowVec<T> {
    fn from(vec: Vec<T>) -> Self {
        CowVec {
            data: Arc::new(vec),
        }
    }
}

impl<T: Clone> Index<usize> for CowVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Clone> IndexMut<usize> for CowVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        todo!("
        Get mutable reference via Arc::make_mut:
        &mut Arc::make_mut(&mut self.data)[index]
        ")
    }
}
```

---

## Milestone 3: CoW HashMap with Lazy Copying

**Goal:** Implement CoW for HashMap to enable efficient configuration sharing.

### Introduction

**Why Milestone 2 Isn't Enough:**

HashMaps are more complex than Vecs:
1. **Key-value pairs**: Must handle both
2. **No indexing**: Use `get()` and `insert()`
3. **Iteration**: Over keys, values, or pairs
4. **Entry API**: Complex mutable access pattern

**Real-world scenario:** Web server configuration:
```rust
// Load config once
let config: CowHashMap<String, String> = load_config();

// Each request handler gets cheap clone
for request in requests {
    let cfg = config.clone(); // O(1)
    handle_request(request, cfg);
}

// Admin updates config (triggers copy)
let mut new_config = config.clone();
new_config.insert("feature_flag".into(), "enabled".into());
```

**Performance Benefit:**
- 1000 concurrent requests × 1KB config = 1MB with CoW
- 1000 concurrent requests × 1KB config = 1GB without CoW
- **1000x memory savings!**

### Architecture

```rust
use std::sync::Arc;
use std::collections::HashMap;

pub struct CowHashMap<K, V> {
    data: Arc<HashMap<K, V>>,
}
```

**Challenges:**
- Entry API (`entry().or_insert()`) needs mutable access
- Iteration should not trigger copy
- `insert()` returns old value

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_map() {
        let mut map = CowHashMap::new();
        map.insert("key".to_string(), 42);

        assert_eq!(map.get("key"), Some(&42));
    }

    #[test]
    fn test_clone_map() {
        let mut m1 = CowHashMap::new();
        m1.insert("a".to_string(), 1);
        m1.insert("b".to_string(), 2);

        let m2 = m1.clone();

        assert_eq!(m1.strong_count(), 2);
        assert_eq!(m1.get("a"), Some(&1));
        assert_eq!(m2.get("a"), Some(&1));
    }

    #[test]
    fn test_copy_on_insert() {
        let mut m1 = CowHashMap::new();
        m1.insert("shared".to_string(), 100);

        let mut m2 = m1.clone();
        assert_eq!(m1.strong_count(), 2);

        m2.insert("new".to_string(), 200);

        // m2 copied
        assert!(m1.get("new").is_none());
        assert_eq!(m2.get("new"), Some(&200));
        assert_eq!(m1.strong_count(), 1);
    }

    #[test]
    fn test_copy_on_remove() {
        let mut m1 = CowHashMap::new();
        m1.insert("key".to_string(), 42);

        let mut m2 = m1.clone();

        m2.remove("key");

        assert_eq!(m1.get("key"), Some(&42));
        assert!(m2.get("key").is_none());
    }

    #[test]
    fn test_iter_no_copy() {
        let mut m1 = CowHashMap::new();
        m1.insert("a".to_string(), 1);
        m1.insert("b".to_string(), 2);

        let m2 = m1.clone();

        // Iteration doesn't copy
        let sum: i32 = m1.values().sum();
        assert_eq!(sum, 3);

        assert_eq!(m1.strong_count(), 2);
    }

    #[test]
    fn test_contains_key() {
        let mut map = CowHashMap::new();
        map.insert("exists".to_string(), 1);

        assert!(map.contains_key("exists"));
        assert!(!map.contains_key("missing"));
    }

    #[test]
    fn test_from_hashmap() {
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), 1);
        hm.insert("b".to_string(), 2);

        let cow = CowHashMap::from(hm);

        assert_eq!(cow.len(), 2);
        assert_eq!(cow.get("a"), Some(&1));
    }

    #[test]
    fn test_into_hashmap() {
        let mut cow = CowHashMap::new();
        cow.insert("a".to_string(), 1);

        let hm = cow.into_hashmap();
        assert_eq!(hm.get("a"), Some(&1));
    }

    #[test]
    fn test_keys_values() {
        let mut map = CowHashMap::new();
        map.insert("x".to_string(), 10);
        map.insert("y".to_string(), 20);

        let keys: Vec<_> = map.keys().collect();
        let values: Vec<_> = map.values().collect();

        assert_eq!(keys.len(), 2);
        assert_eq!(values.len(), 2);
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
pub struct CowHashMap<K, V> {
    data: Arc<HashMap<K, V>>,
}

impl<K, V> CowHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        CowHashMap {
            data: Arc::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        todo!("Return self.data.get(key)")
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        todo!("
        Get mutable HashMap and insert:
        Arc::make_mut(&mut self.data).insert(key, value)
        ")
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        todo!("Get mut HashMap and remove")
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter()
    }

    pub fn into_hashmap(self) -> HashMap<K, V> {
        todo!("
        Try to unwrap Arc:
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())
        ")
    }
}

impl<K, V> From<HashMap<K, V>> for CowHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn from(map: HashMap<K, V>) -> Self {
        CowHashMap {
            data: Arc::new(map),
        }
    }
}

impl<K, V> Default for CowHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Milestone 4: Generic CoW Wrapper

**Goal:** Create a generic `Cow<T>` wrapper that works with any cloneable type.

### Introduction

**Why Milestone 3 Isn't Enough:**

We've implemented CoW for String, Vec, and HashMap separately:
- Lots of code duplication
- Hard to add new types
- Inconsistent API

**Solution:** Generic wrapper `Cow<T>` that works for any `T: Clone`.

**Benefits:**
- Works with any type (String, Vec, HashMap, custom structs)
- Consistent API
- Less code to maintain

**Challenge:** How to provide mutable access? We can't implement `DerefMut` for all types.

### Architecture

```rust
use std::sync::Arc;

pub struct Cow<T: Clone> {
    data: Arc<T>,
}
```

**API Design:**
- `Cow::new(value)`: Create from value
- `clone()`: Cheap refcount increment
- `make_mut() -> &mut T`: Get mutable ref, copying if shared
- `is_shared() -> bool`: Check if data is shared
- `into_inner() -> T`: Consume and get inner value

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cow_string() {
        let s1 = Cow::new(String::from("hello"));
        let mut s2 = s1.clone();

        assert_eq!(s1.strong_count(), 2);

        s2.make_mut().push_str(" world");

        assert_eq!(&**s1, "hello");
        assert_eq!(&**s2, "hello world");
    }

    #[test]
    fn test_cow_vec() {
        let v1 = Cow::new(vec![1, 2, 3]);
        let mut v2 = v1.clone();

        v2.make_mut().push(4);

        assert_eq!(&**v1, &[1, 2, 3]);
        assert_eq!(&**v2, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_cow_hashmap() {
        let mut map = HashMap::new();
        map.insert("a", 1);

        let m1 = Cow::new(map);
        let mut m2 = m1.clone();

        m2.make_mut().insert("b", 2);

        assert_eq!(m1.get("b"), None);
        assert_eq!(m2.get("b"), Some(&2));
    }

    #[test]
    fn test_custom_struct() {
        #[derive(Clone, PartialEq, Debug)]
        struct Config {
            host: String,
            port: u16,
        }

        let c1 = Cow::new(Config {
            host: "localhost".into(),
            port: 8080,
        });

        let mut c2 = c1.clone();

        c2.make_mut().port = 9090;

        assert_eq!(c1.port, 8080);
        assert_eq!(c2.port, 9090);
    }

    #[test]
    fn test_make_mut_exclusive() {
        let mut cow = Cow::new(vec![1, 2, 3]);

        // Not shared - no copy
        let ptr1 = cow.data.as_ptr();
        cow.make_mut().push(4);
        let ptr2 = cow.data.as_ptr();

        assert_eq!(ptr1, ptr2); // Same allocation
    }

    #[test]
    fn test_into_inner() {
        let cow = Cow::new(String::from("test"));
        let s = cow.into_inner();

        assert_eq!(s, "test");
    }

    #[test]
    fn test_into_inner_shared() {
        let cow1 = Cow::new(vec![1, 2, 3]);
        let cow2 = cow1.clone();

        // Must clone because shared
        let vec = cow1.into_inner();

        assert_eq!(vec, vec![1, 2, 3]);
        assert_eq!(&**cow2, &[1, 2, 3]);
    }

    #[test]
    fn test_map() {
        let cow = Cow::new(5);
        let mapped = cow.map(|n| n * 2);

        assert_eq!(*mapped, 10);
    }

    #[test]
    fn test_map_shared() {
        let c1 = Cow::new(10);
        let c2 = c1.clone();

        let c3 = c1.map(|n| n + 1);

        assert_eq!(*c1, 10);
        assert_eq!(*c2, 10);
        assert_eq!(*c3, 11);
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::ops::Deref;

#[derive(Clone)]
pub struct Cow<T: Clone> {
    data: Arc<T>,
}

impl<T: Clone> Cow<T> {
    pub fn new(value: T) -> Self {
        Cow {
            data: Arc::new(value),
        }
    }

    pub fn make_mut(&mut self) -> &mut T {
        todo!("
        Use Arc::make_mut:
        Arc::make_mut(&mut self.data)

        This automatically:
        - Returns &mut T if strong_count == 1
        - Clones and returns &mut T if shared
        ")
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn into_inner(self) -> T {
        todo!("
        Try to unwrap Arc:
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())
        ")
    }

    pub fn map<F, U>(&self, f: F) -> Cow<U>
    where
        F: FnOnce(&T) -> U,
        U: Clone,
    {
        todo!("
        Apply function to inner value:
        let result = f(&*self.data);
        Cow::new(result)
        ")
    }

    pub fn get(&self) -> &T {
        &self.data
    }
}

impl<T: Clone> Deref for Cow<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Clone> From<T> for Cow<T> {
    fn from(value: T) -> Self {
        Cow::new(value)
    }
}
```

---

## Milestone 5: Thread-Safe CoW with Arc (Already Thread-Safe!)

**Goal:** Verify and optimize thread-safe sharing of CoW structures.

### Introduction

**Why Milestone 4 is Almost Enough:**

Good news: `Cow<T>` using `Arc` is already thread-safe if `T: Send + Sync`!

However, we need to:
1. **Verify safety**: Add Send/Sync bounds
2. **Add utilities**: Thread-safe modification helpers
3. **Optimize**: Reduce contention on writes
4. **Document**: Clear thread-safety guarantees

**Thread-safety properties:**
- Multiple threads can clone and read simultaneously
- Writes are safe (each thread gets private copy)
- No locks needed for reads (unlike Mutex)
- Lock-free for read-heavy workloads

### Architecture

```rust
use std::sync::Arc;
use std::marker::PhantomData;

pub struct Cow<T: Clone + Send + Sync> {
    data: Arc<T>,
    _marker: PhantomData<T>,
}
```

**Thread-safety guarantees:**
- `Clone`: Lock-free atomic refcount increment
- `Deref`: Lock-free read access
- `make_mut`: Clones if shared (no waiting)
- No deadlocks possible

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;

    #[test]
    fn test_concurrent_clone() {
        let cow = Cow::new(vec![1, 2, 3, 4, 5]);
        let mut handles = vec![];

        for _ in 0..10 {
            let c = cow.clone();
            let handle = thread::spawn(move || {
                let sum: i32 = c.iter().sum();
                sum
            });
            handles.push(handle);
        }

        for handle in handles {
            assert_eq!(handle.join().unwrap(), 15);
        }
    }

    #[test]
    fn test_concurrent_read() {
        let cow = Cow::new(String::from("shared data"));
        let mut handles = vec![];

        for i in 0..20 {
            let c = cow.clone();
            let handle = thread::spawn(move || {
                assert_eq!(&*c, "shared data");
                c.len()
            });
            handles.push(handle);
        }

        for handle in handles {
            assert_eq!(handle.join().unwrap(), 11);
        }
    }

    #[test]
    fn test_concurrent_write() {
        let cow = Cow::new(vec![1, 2, 3]);
        let mut handles = vec![];

        // 10 threads each make their own modification
        for i in 0..10 {
            let mut c = cow.clone();
            let handle = thread::spawn(move || {
                c.make_mut().push(i);
                c.clone()
            });
            handles.push(handle);
        }

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        // Each thread got its own copy
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.len(), 4);
            assert_eq!(result[3], i);
        }

        // Original unchanged
        assert_eq!(&*cow, &[1, 2, 3]);
    }

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Cow<Vec<i32>>>();
        assert_sync::<Cow<Vec<i32>>>();
    }

    #[test]
    fn test_shared_config() {
        use std::collections::HashMap;

        let mut config = HashMap::new();
        config.insert("workers", 4);
        config.insert("timeout", 30);

        let cow_config = Cow::new(config);
        let mut handles = vec![];

        // 100 worker threads using shared config
        for worker_id in 0..100 {
            let cfg = cow_config.clone();
            let handle = thread::spawn(move || {
                let workers = cfg.get("workers").unwrap();
                let timeout = cfg.get("timeout").unwrap();
                (*workers, *timeout, worker_id)
            });
            handles.push(handle);
        }

        for handle in handles {
            let (workers, timeout, _id) = handle.join().unwrap();
            assert_eq!(workers, 4);
            assert_eq!(timeout, 30);
        }

        // Still shared!
        assert_eq!(cow_config.strong_count(), 1);
    }

    #[test]
    fn test_memory_efficiency() {
        use std::mem::size_of;

        let vec = vec![0u8; 1_000_000]; // 1MB
        let cow1 = Cow::new(vec);

        // Clone 100 times
        let clones: Vec<_> = (0..100).map(|_| cow1.clone()).collect();

        // Memory used: ~1MB data + 100 * 8 bytes = ~1MB
        // vs 100MB if each clone copied

        assert_eq!(cow1.strong_count(), 101);

        // Size of Cow itself
        assert_eq!(size_of::<Cow<Vec<u8>>>(), size_of::<Arc<Vec<u8>>>());
    }

    #[test]
    fn test_update_check() {
        let counter = StdArc::new(AtomicUsize::new(0));
        let cow = Cow::new(vec![1, 2, 3]);

        let mut handles = vec![];

        for _ in 0..50 {
            let mut c = cow.clone();
            let cnt = counter.clone();

            let handle = thread::spawn(move || {
                // Modify triggers copy
                c.make_mut().push(4);
                cnt.fetch_add(1, Ordering::SeqCst);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 50);
        // Original still unchanged
        assert_eq!(&*cow, &[1, 2, 3]);
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::ops::Deref;

#[derive(Clone)]
pub struct Cow<T>
where
    T: Clone + Send + Sync,
{
    data: Arc<T>,
}

// Explicitly implement Send + Sync
unsafe impl<T: Clone + Send + Sync> Send for Cow<T> {}
unsafe impl<T: Clone + Send + Sync> Sync for Cow<T> {}

impl<T> Cow<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(value: T) -> Self {
        Cow {
            data: Arc::new(value),
        }
    }

    pub fn make_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.data)
    }

    pub fn try_update<F, E>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        todo!("
        Apply function to mutable reference:
        f(self.make_mut())
        ")
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        todo!("Apply function to make_mut()")
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn into_inner(self) -> T {
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        todo!("Use Arc::ptr_eq to check if both point to same data")
    }
}

impl<T> Deref for Cow<T>
where
    T: Clone + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> From<T> for Cow<T>
where
    T: Clone + Send + Sync,
{
    fn from(value: T) -> Self {
        Cow::new(value)
    }
}
```

---

## Milestone 6: Performance Tracking and Optimization

**Goal:** Add metrics to track copy frequency and optimize hot paths.

### Introduction

**Why Milestone 5 Isn't Enough:**

Production CoW structures need observability:
1. **Copy tracking**: How often does copy-on-write trigger?
2. **Sharing metrics**: What's the sharing ratio?
3. **Memory profiling**: Is CoW actually saving memory?
4. **Performance validation**: Is CoW faster than clone?

**Metrics to track:**
- Total clones (refcount increments)
- Actual copies (data duplicated)
- Copy rate = copies / clones
- Memory saved = (clones - copies) × size
- Strong count distribution

### Architecture

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Cow<T: Clone + Send + Sync> {
    data: Arc<T>,
    stats: Arc<CowStats>,
}

struct CowStats {
    clones: AtomicUsize,
    copies: AtomicUsize,
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_stats() {
        let cow = Cow::new(vec![1, 2, 3]);

        let _c1 = cow.clone();
        let _c2 = cow.clone();
        let _c3 = cow.clone();

        let stats = cow.stats();
        assert_eq!(stats.clones, 3);
        assert_eq!(stats.copies, 0);
    }

    #[test]
    fn test_copy_stats() {
        let cow = Cow::new(vec![1, 2, 3]);

        let mut c1 = cow.clone();
        let mut c2 = cow.clone();

        c1.make_mut().push(4);
        c2.make_mut().push(5);

        let stats = cow.stats();
        assert_eq!(stats.clones, 2);
        assert_eq!(stats.copies, 2);
        assert_eq!(stats.copy_rate(), 1.0);
    }

    #[test]
    fn test_copy_rate() {
        let cow = Cow::new(String::from("test"));

        // 10 clones
        let clones: Vec<_> = (0..10).map(|_| cow.clone()).collect();

        // 5 copies
        let mut mutated: Vec<_> = clones.into_iter().take(5).collect();
        for c in &mut mutated {
            c.make_mut().push_str("!");
        }

        let stats = cow.stats();
        assert_eq!(stats.clones, 10);
        assert_eq!(stats.copies, 5);
        assert_eq!(stats.copy_rate(), 0.5);
    }

    #[test]
    fn test_memory_savings() {
        use std::mem::size_of_val;

        let data = vec![0u8; 1_000_000]; // 1MB
        let size = size_of_val(&*data);

        let cow = Cow::new(data);

        // 100 clones
        let _clones: Vec<_> = (0..100).map(|_| cow.clone()).collect();

        let stats = cow.stats();
        let saved = stats.memory_saved(size);

        // Saved = (100 clones - 0 copies) * 1MB = 100MB
        assert_eq!(saved, 100 * 1_000_000);
    }

    #[test]
    fn test_stats_report() {
        let cow = Cow::new(vec![0u8; 1024]);

        let mut clones: Vec<_> = (0..10).map(|_| cow.clone()).collect();

        for i in 0..5 {
            clones[i].make_mut().push(1);
        }

        let report = cow.stats_report(1024);
        assert!(report.contains("Clones:"));
        assert!(report.contains("Copies:"));
        assert!(report.contains("Copy rate:"));
        assert!(report.contains("Memory saved:"));
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::Deref;

#[derive(Clone)]
pub struct Cow<T>
where
    T: Clone + Send + Sync,
{
    data: Arc<T>,
    stats: Arc<CowStats>,
}

struct CowStats {
    clones: AtomicUsize,
    copies: AtomicUsize,
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub clones: usize,
    pub copies: usize,
}

impl Stats {
    pub fn copy_rate(&self) -> f64 {
        todo!("Calculate copies / clones (handle division by zero)")
    }

    pub fn memory_saved(&self, item_size: usize) -> usize {
        todo!("Calculate (clones - copies) * item_size")
    }
}

impl<T> Cow<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(value: T) -> Self {
        Cow {
            data: Arc::new(value),
            stats: Arc::new(CowStats {
                clones: AtomicUsize::new(0),
                copies: AtomicUsize::new(0),
            }),
        }
    }

    pub fn make_mut(&mut self) -> &mut T {
        todo!("
        Check if shared before calling Arc::make_mut:
        let was_shared = self.is_shared();
        let result = Arc::make_mut(&mut self.data);

        if was_shared {
            self.stats.copies.fetch_add(1, Ordering::Relaxed);
        }

        result
        ")
    }

    pub fn stats(&self) -> Stats {
        Stats {
            clones: self.stats.clones.load(Ordering::Relaxed),
            copies: self.stats.copies.load(Ordering::Relaxed),
        }
    }

    pub fn stats_report(&self, item_size: usize) -> String {
        todo!("
        Format stats into string:
        - Clones: {}
        - Copies: {}
        - Copy rate: {:.1}%
        - Memory saved: {} bytes ({:.1} MB)
        ")
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }
}

impl<T> Clone for Cow<T>
where
    T: Clone + Send + Sync,
{
    fn clone(&self) -> Self {
        todo!("
        Increment clone counter:
        self.stats.clones.fetch_add(1, Ordering::Relaxed);

        Clone data Arc and stats Arc:
        Cow {
            data: self.data.clone(),
            stats: self.stats.clone(),
        }
        ")
    }
}

impl<T> Deref for Cow<T>
where
    T: Clone + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
```

---

## Complete Working Example

Here's a production-quality CoW implementation with full feature set:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::Deref;
use std::fmt;

// ============================================================================
// Statistics Tracking
// ============================================================================

struct CowStats {
    clones: AtomicUsize,
    copies: AtomicUsize,
}

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub clones: usize,
    pub copies: usize,
}

impl Stats {
    pub fn copy_rate(&self) -> f64 {
        if self.clones == 0 {
            0.0
        } else {
            self.copies as f64 / self.clones as f64
        }
    }

    pub fn memory_saved(&self, item_size: usize) -> usize {
        if self.copies >= self.clones {
            0
        } else {
            (self.clones - self.copies) * item_size
        }
    }
}

// ============================================================================
// Copy-on-Write Wrapper
// ============================================================================

pub struct Cow<T>
where
    T: Clone + Send + Sync,
{
    data: Arc<T>,
    stats: Arc<CowStats>,
}

impl<T> Cow<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(value: T) -> Self {
        Cow {
            data: Arc::new(value),
            stats: Arc::new(CowStats {
                clones: AtomicUsize::new(0),
                copies: AtomicUsize::new(0),
            }),
        }
    }

    pub fn make_mut(&mut self) -> &mut T {
        let was_shared = self.is_shared();
        let result = Arc::make_mut(&mut self.data);

        if was_shared {
            self.stats.copies.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(self.make_mut());
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn into_inner(self) -> T {
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    pub fn stats(&self) -> Stats {
        Stats {
            clones: self.stats.clones.load(Ordering::Relaxed),
            copies: self.stats.copies.load(Ordering::Relaxed),
        }
    }

    pub fn stats_report(&self, item_size: usize) -> String {
        let stats = self.stats();
        format!(
            "CoW Statistics:\n\
             - Clones: {}\n\
             - Copies: {}\n\
             - Copy rate: {:.1}%\n\
             - Memory saved: {} bytes ({:.2} MB)",
            stats.clones,
            stats.copies,
            stats.copy_rate() * 100.0,
            stats.memory_saved(item_size),
            stats.memory_saved(item_size) as f64 / 1_000_000.0
        )
    }
}

impl<T> Clone for Cow<T>
where
    T: Clone + Send + Sync,
{
    fn clone(&self) -> Self {
        self.stats.clones.fetch_add(1, Ordering::Relaxed);

        Cow {
            data: self.data.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl<T> Deref for Cow<T>
where
    T: Clone + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> From<T> for Cow<T>
where
    T: Clone + Send + Sync,
{
    fn from(value: T) -> Self {
        Cow::new(value)
    }
}

impl<T> fmt::Debug for Cow<T>
where
    T: Clone + Send + Sync + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cow")
            .field("data", &*self.data)
            .field("shared", &self.is_shared())
            .field("strong_count", &self.strong_count())
            .finish()
    }
}

unsafe impl<T: Clone + Send + Sync> Send for Cow<T> {}
unsafe impl<T: Clone + Send + Sync> Sync for Cow<T> {}

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    use std::collections::HashMap;
    use std::thread;

    println!("=== CoW String Example ===\n");

    let s1 = Cow::new(String::from("Hello, CoW!"));
    println!("Created: {:?}", s1);

    let s2 = s1.clone();
    let s3 = s1.clone();
    println!("Cloned 2 times, shared: {}", s1.is_shared());

    let mut s4 = s1.clone();
    s4.make_mut().push_str(" - Modified");
    println!("Modified clone: {}", s4);
    println!("Original: {}\n", s1);

    println!("{}\n", s1.stats_report(s1.len()));

    println!("=== CoW Vec Example ===\n");

    let v = Cow::new(vec![1, 2, 3, 4, 5]);

    // Share across 10 threads
    let mut handles = vec![];

    for i in 0..10 {
        let mut vc = v.clone();

        let handle = thread::spawn(move || {
            if i % 2 == 0 {
                // Even threads modify (triggers copy)
                vc.make_mut().push(i * 10);
            }
            vc.iter().sum::<i32>()
        });

        handles.push(handle);
    }

    for (i, handle) in handles.into_iter().enumerate() {
        let sum = handle.join().unwrap();
        println!("Thread {}: sum = {}", i, sum);
    }

    println!("\nOriginal vec: {:?}", &*v);
    println!("{}\n", v.stats_report(std::mem::size_of::<Vec<i32>>()));

    println!("=== CoW HashMap Config Example ===\n");

    let mut config = HashMap::new();
    config.insert("workers", 4);
    config.insert("timeout", 30);
    config.insert("max_connections", 1000);

    let cow_config = Cow::new(config);

    // Simulate 100 worker threads using config
    let mut handles = vec![];

    for worker_id in 0..100 {
        let cfg = cow_config.clone();

        let handle = thread::spawn(move || {
            let workers = *cfg.get("workers").unwrap();
            // Simulate work
            std::thread::sleep(std::time::Duration::from_micros(10));
            workers
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Config shared across 100 threads!");
    println!("{}", cow_config.stats_report(std::mem::size_of::<HashMap<&str, i32>>()));

    println!("\nDone!");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_workflow() {
        let v1 = Cow::new(vec![1, 2, 3]);
        let v2 = v1.clone();
        let mut v3 = v1.clone();

        assert_eq!(v1.strong_count(), 3);

        v3.make_mut().push(4);

        assert_eq!(&*v1, &[1, 2, 3]);
        assert_eq!(&*v2, &[1, 2, 3]);
        assert_eq!(&*v3, &[1, 2, 3, 4]);

        let stats = v1.stats();
        assert_eq!(stats.clones, 2);
        assert_eq!(stats.copies, 1);
    }
}
```

**Example Output:**
```
=== CoW String Example ===

Created: Cow { data: "Hello, CoW!", shared: false, strong_count: 1 }
Cloned 2 times, shared: true
Modified clone: Hello, CoW! - Modified
Original: Hello, CoW!

CoW Statistics:
- Clones: 3
- Copies: 1
- Copy rate: 33.3%
- Memory saved: 22 bytes (0.00 MB)

=== CoW Vec Example ===

Thread 0: sum = 15
Thread 1: sum = 15
Thread 2: sum = 35
Thread 3: sum = 15
Thread 4: sum = 55
Thread 5: sum = 15
Thread 6: sum = 75
Thread 7: sum = 15
Thread 8: sum = 95
Thread 9: sum = 15

Original vec: [1, 2, 3, 4, 5]
CoW Statistics:
- Clones: 10
- Copies: 5
- Copy rate: 50.0%
- Memory saved: 120 bytes (0.00 MB)

=== CoW HashMap Config Example ===

Config shared across 100 threads!
CoW Statistics:
- Clones: 100
- Copies: 0
- Copy rate: 0.0%
- Memory saved: 4800 bytes (0.00 MB)

Done!
```

---

## Summary

You've built a complete Copy-on-Write library with production-grade features!

### Features Implemented
1. ✅ CoW String (Milestone 1)
2. ✅ CoW Vec (Milestone 2)
3. ✅ CoW HashMap (Milestone 3)
4. ✅ Generic Cow<T> (Milestone 4)
5. ✅ Thread-safe sharing (Milestone 5)
6. ✅ Performance tracking (Milestone 6)

### Smart Pointer Patterns Used
- `Arc<T>`: Atomic reference counting for thread-safe sharing
- `Arc::make_mut()`: Copy-on-write primitive
- `Arc::try_unwrap()`: Extract value without copy if possible
- `Arc::strong_count()`: Check sharing status
- `Deref`: Transparent read access

### Performance Characteristics
| Operation | Normal Clone | CoW Clone | Speedup |
|-----------|-------------|-----------|---------|
| 1KB string | 1μs | 10ns | 100x |
| 1MB buffer | 500μs | 10ns | 50,000x |
| HashMap | 5μs | 10ns | 500x |
| Modify after clone | 0 | Copy cost | N/A |

### When to Use CoW
✅ **Use CoW when:**
- Read-heavy workloads (10:1 read:write ratio)
- Sharing data across threads
- Implementing immutable data structures
- Cloning large data structures
- Version control systems

❌ **Don't use CoW when:**
- Write-heavy workloads (copies negate benefits)
- Data is always modified after clone
- Small data (clone cost negligible)
- Need guaranteed O(1) writes

### Real-World Uses
- **Git**: Blob storage with content-addressable CoW
- **Immutable.js**: Persistent data structures
- **Rust std::borrow::Cow**: Standard library CoW
- **Arc<RwLock<T>>**: Common Rust pattern
- **im crate**: Immutable collections

### Key Lessons
1. **Arc::make_mut is magic**: Automatic copy detection
2. **Read-heavy wins**: CoW excels with rare writes
3. **Memory vs speed**: Trade write speed for memory efficiency
4. **Thread-safety**: Arc makes CoW naturally thread-safe
5. **Measure impact**: Use stats to validate performance

Congratulations! You understand the CoW patterns used in functional programming languages, version control systems, and immutable data structures!
