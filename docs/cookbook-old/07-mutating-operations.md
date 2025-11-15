# Mutating Operations in Rust

> A comprehensive guide to in-place data manipulation with Vec, slices, and iterators

## Table of Contents

1. [Vec Mutating Methods](#vec-mutating-methods)
2. [Slice Mutating Methods](#slice-mutating-methods)
3. [Range-Based Mutations](#range-based-mutations)
4. [iter_mut() Patterns](#iter_mut-patterns)
5. [Advanced Mutation Patterns](#advanced-mutation-patterns)
6. [Performance Considerations](#performance-considerations)
7. [Common Pitfalls](#common-pitfalls)
8. [Quick Reference](#quick-reference)

---

## Vec Mutating Methods

### Adding Elements

#### push()

Add an element to the end of the vector.

```rust
let mut vec = vec![1, 2, 3];
vec.push(4);
// vec is now [1, 2, 3, 4]
```

**Time Complexity**: O(1) amortized

---

#### insert()

Insert an element at a specific position.

```rust
let mut vec = vec![1, 2, 4];
vec.insert(2, 3);
// vec is now [1, 2, 3, 4]
```

**Time Complexity**: O(n) - shifts all elements after the insertion point

---

#### append()

Move all elements from another vector.

```rust
let mut vec1 = vec![1, 2];
let mut vec2 = vec![3, 4];
vec1.append(&mut vec2);
// vec1 is now [1, 2, 3, 4]
// vec2 is now []
```

**Time Complexity**: O(m) where m is the length of the appended vector

---

#### extend()

Add elements from an iterator.

```rust
let mut vec = vec![1, 2];
vec.extend([3, 4, 5]);
// vec is now [1, 2, 3, 4, 5]

// Works with any iterator
vec.extend(vec![6, 7]);
// vec is now [1, 2, 3, 4, 5, 6, 7]
```

**Time Complexity**: O(m) where m is the number of elements being added

---

### Removing Elements

#### pop()

Remove and return the last element.

```rust
let mut vec = vec![1, 2, 3];
let last = vec.pop();
// last is Some(3)
// vec is now [1, 2]

let empty_vec: Vec<i32> = vec![];
let nothing = empty_vec.pop();
// nothing is None
```

**Time Complexity**: O(1)

---

#### remove()

Remove and return an element at a specific index.

```rust
let mut vec = vec![1, 2, 3, 4];
let removed = vec.remove(1);
// removed is 2
// vec is now [1, 3, 4]
```

**Time Complexity**: O(n) - shifts all elements after the removed element

---

#### swap_remove()

Remove an element by swapping it with the last element.

```rust
let mut vec = vec![1, 2, 3, 4];
let removed = vec.swap_remove(1);
// removed is 2
// vec is now [1, 4, 3] (order not preserved!)
```

**Time Complexity**: O(1) - much faster than `remove()` but doesn't preserve order

**Use when**: Order doesn't matter and you need performance

---

#### truncate()

Shorten the vector to a specified length.

```rust
let mut vec = vec![1, 2, 3, 4, 5];
vec.truncate(3);
// vec is now [1, 2, 3]

vec.truncate(10);
// vec is still [1, 2, 3] (no effect if length > current length)
```

**Time Complexity**: O(n) where n is the number of elements dropped

---

#### clear()

Remove all elements from the vector.

```rust
let mut vec = vec![1, 2, 3];
vec.clear();
// vec is now []
// Capacity is preserved
```

**Time Complexity**: O(n)

---

#### retain()

Keep only elements that satisfy a predicate.

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6];
vec.retain(|x| x % 2 == 0);
// vec is now [2, 4, 6]
```

**Time Complexity**: O(n)

---

#### retain_mut()

Like `retain()` but with mutable access to elements.

```rust
let mut vec = vec![1, 2, 3, 4, 5];
vec.retain_mut(|x| {
    if *x % 2 == 0 {
        *x *= 2;  // Can modify elements
        true
    } else {
        false
    }
});
// vec is now [4, 8]
```

**Time Complexity**: O(n)

---

#### dedup()

Remove consecutive duplicate elements.

```rust
let mut vec = vec![1, 2, 2, 3, 3, 3, 4];
vec.dedup();
// vec is now [1, 2, 3, 4]
```

**Time Complexity**: O(n)

**Note**: Only removes consecutive duplicates. For removing all duplicates, sort first:

```rust
let mut vec = vec![1, 2, 1, 3, 2];
vec.sort();
vec.dedup();
// vec is now [1, 2, 3]
```

---

#### dedup_by()

Remove consecutive duplicates using a custom comparison.

```rust
let mut vec = vec!["foo", "bar", "Bar", "baz", "BAZ"];
vec.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
// vec is now ["foo", "bar", "baz"]
```

---

#### dedup_by_key()

Remove consecutive duplicates based on a key function.

```rust
let mut vec = vec![10, 20, 21, 30, 31, 32];
vec.dedup_by_key(|x| *x / 10);
// vec is now [10, 20, 30]
```

---

### Resizing

#### resize()

Resize the vector to a new length with a clone value.

```rust
let mut vec = vec![1, 2, 3];
vec.resize(5, 0);
// vec is now [1, 2, 3, 0, 0]

vec.resize(2, 0);
// vec is now [1, 2]
```

**Time Complexity**: O(n) where n is the difference in size

---

#### resize_with()

Resize the vector using a closure to generate new elements.

```rust
let mut vec = vec![1, 2, 3];
vec.resize_with(5, Default::default);
// vec is now [1, 2, 3, 0, 0]

let mut vec2 = vec![];
let mut counter = 0;
vec2.resize_with(3, || { counter += 1; counter });
// vec2 is now [1, 2, 3]
```

---

### Reordering Elements

#### reverse()

Reverse the order of elements.

```rust
let mut vec = vec![1, 2, 3, 4];
vec.reverse();
// vec is now [4, 3, 2, 1]
```

**Time Complexity**: O(n)

---

#### sort()

Sort the vector in ascending order.

```rust
let mut vec = vec![3, 1, 4, 1, 5, 9];
vec.sort();
// vec is now [1, 1, 3, 4, 5, 9]
```

**Time Complexity**: O(n log n)

**Requirements**: Elements must implement `Ord`

---

#### sort_unstable()

Sort the vector (potentially faster but not stable).

```rust
let mut vec = vec![3, 1, 4, 1, 5, 9];
vec.sort_unstable();
// vec is now [1, 1, 3, 4, 5, 9]
```

**Time Complexity**: O(n log n)

**Note**: Faster than `sort()` but doesn't preserve the order of equal elements

---

#### sort_by()

Sort using a custom comparison function.

```rust
let mut vec = vec![3, 1, 4, 1, 5];
vec.sort_by(|a, b| b.cmp(a));  // Descending order
// vec is now [5, 4, 3, 1, 1]
```

---

#### sort_by_key()

Sort by a key extraction function.

```rust
let mut vec = vec!["aaa", "bb", "c", "dddd"];
vec.sort_by_key(|s| s.len());
// vec is now ["c", "bb", "aaa", "dddd"]
```

---

#### sort_by_cached_key()

Like `sort_by_key()` but caches key values (useful for expensive computations).

```rust
let mut vec = vec!["aaa", "bb", "c", "dddd"];
vec.sort_by_cached_key(|s| s.len());
// vec is now ["c", "bb", "aaa", "dddd"]
```

**Use when**: Key computation is expensive and will be called multiple times

---

#### rotate_left()

Rotate elements to the left.

```rust
let mut vec = vec![1, 2, 3, 4, 5];
vec.rotate_left(2);
// vec is now [3, 4, 5, 1, 2]
```

**Time Complexity**: O(n)

---

#### rotate_right()

Rotate elements to the right.

```rust
let mut vec = vec![1, 2, 3, 4, 5];
vec.rotate_right(2);
// vec is now [4, 5, 1, 2, 3]
```

**Time Complexity**: O(n)

---

### Swapping Elements

#### swap()

Swap two elements by index.

```rust
let mut vec = vec![1, 2, 3, 4];
vec.swap(0, 3);
// vec is now [4, 2, 3, 1]
```

**Time Complexity**: O(1)

---

#### swap_with_slice()

Swap elements with another slice of the same length.

```rust
let mut vec1 = vec![1, 2, 3];
let mut vec2 = vec![4, 5, 6];
vec1.swap_with_slice(&mut vec2);
// vec1 is now [4, 5, 6]
// vec2 is now [1, 2, 3]
```

**Time Complexity**: O(n)

---

### Filling

#### fill()

Fill the vector with a cloned value.

```rust
let mut vec = vec![1, 2, 3, 4];
vec.fill(0);
// vec is now [0, 0, 0, 0]
```

**Time Complexity**: O(n)

---

#### fill_with()

Fill the vector using a closure.

```rust
let mut vec = vec![0; 5];
let mut counter = 0;
vec.fill_with(|| { counter += 1; counter });
// vec is now [1, 2, 3, 4, 5]
```

**Time Complexity**: O(n)

---

### Capacity Management

#### reserve()

Reserve capacity for at least the specified number of additional elements.

```rust
let mut vec = Vec::new();
vec.reserve(10);
// vec can now hold at least 10 elements without reallocation
```

**Time Complexity**: O(n) if reallocation occurs, O(1) otherwise

---

#### reserve_exact()

Reserve capacity for exactly the specified number of additional elements.

```rust
let mut vec = vec![1];
vec.reserve_exact(10);
// vec has capacity for exactly 11 elements
```

---

#### shrink_to_fit()

Shrink capacity to match length.

```rust
let mut vec = Vec::with_capacity(10);
vec.extend([1, 2, 3]);
vec.shrink_to_fit();
// vec now has capacity of 3
```

**Time Complexity**: O(n)

---

#### shrink_to()

Shrink capacity to at least the specified value.

```rust
let mut vec = Vec::with_capacity(10);
vec.extend([1, 2, 3]);
vec.shrink_to(5);
// vec now has capacity between 3 and 5
```

---

## Slice Mutating Methods

Slices are views into contiguous sequences. These methods work on both `Vec<T>` (which derefs to `&mut [T]`) and array slices.

### Sorting

#### sort()

```rust
let mut slice = [3, 1, 4, 1, 5];
slice.sort();
// slice is now [1, 1, 3, 4, 5]
```

---

#### sort_unstable()

```rust
let mut slice = [3, 1, 4, 1, 5];
slice.sort_unstable();
// slice is now [1, 1, 3, 4, 5]
```

Faster than `sort()` but not stable (doesn't preserve order of equal elements).

---

#### sort_by()

```rust
let mut slice = [3, 1, 4, 1, 5];
slice.sort_by(|a, b| b.cmp(a));
// slice is now [5, 4, 3, 1, 1]
```

---

#### sort_by_key()

```rust
let mut slice = ["aaa", "bb", "c"];
slice.sort_by_key(|s| s.len());
// slice is now ["c", "bb", "aaa"]
```

---

#### sort_by_cached_key()

```rust
let mut slice = ["aaa", "bb", "c"];
slice.sort_by_cached_key(|s| s.len());
// slice is now ["c", "bb", "aaa"]
```

---

### Reversing

#### reverse()

```rust
let mut slice = [1, 2, 3, 4];
slice.reverse();
// slice is now [4, 3, 2, 1]
```

---

### Rotating

#### rotate_left()

```rust
let mut slice = [1, 2, 3, 4, 5];
slice.rotate_left(2);
// slice is now [3, 4, 5, 1, 2]
```

---

#### rotate_right()

```rust
let mut slice = [1, 2, 3, 4, 5];
slice.rotate_right(2);
// slice is now [4, 5, 1, 2, 3]
```

---

### Filling

#### fill()

```rust
let mut slice = [1, 2, 3, 4];
slice.fill(0);
// slice is now [0, 0, 0, 0]
```

---

#### fill_with()

```rust
let mut slice = [0; 5];
let mut n = 1;
slice.fill_with(|| { let val = n; n *= 2; val });
// slice is now [1, 2, 4, 8, 16]
```

---

### Swapping

#### swap()

```rust
let mut slice = [1, 2, 3, 4];
slice.swap(0, 3);
// slice is now [4, 2, 3, 1]
```

---

#### swap_with_slice()

```rust
let mut a = [1, 2, 3];
let mut b = [4, 5, 6];
a.swap_with_slice(&mut b);
// a is now [4, 5, 6]
// b is now [1, 2, 3]
```

---

### Copying

#### copy_from_slice()

Copy all elements from another slice (must be same length).

```rust
let mut dest = [0; 3];
let src = [1, 2, 3];
dest.copy_from_slice(&src);
// dest is now [1, 2, 3]
```

**Panics**: If slices have different lengths

---

#### copy_within()

Copy elements within the same slice.

```rust
let mut slice = [1, 2, 3, 4, 5];
slice.copy_within(1..4, 0);
// slice is now [2, 3, 4, 4, 5]
```

---

### Splitting

#### split_at_mut()

Split a slice into two mutable slices.

```rust
let mut slice = [1, 2, 3, 4, 5];
let (left, right) = slice.split_at_mut(2);
// left is [1, 2]
// right is [3, 4, 5]

left[0] = 10;
right[0] = 30;
// slice is now [10, 2, 30, 4, 5]
```

---

#### split_first_mut()

Get mutable references to the first element and the rest.

```rust
let mut slice = [1, 2, 3, 4];
if let Some((first, rest)) = slice.split_first_mut() {
    *first = 10;
    rest[0] = 20;
}
// slice is now [10, 20, 3, 4]
```

---

#### split_last_mut()

Get mutable references to the last element and the rest.

```rust
let mut slice = [1, 2, 3, 4];
if let Some((last, rest)) = slice.split_last_mut() {
    *last = 40;
    rest[0] = 10;
}
// slice is now [10, 2, 3, 40]
```

---

#### split_mut()

Split a slice into multiple mutable sub-slices.

```rust
let mut slice = [1, 2, 3, 0, 4, 5];
for sub in slice.split_mut(|x| *x == 0) {
    sub.fill(99);
}
// slice is now [99, 99, 99, 0, 99, 99]
```

---

## Range-Based Mutations

These methods operate on ranges within a vector, allowing efficient insertion, removal, and replacement.

### drain()

Remove and iterate over a range of elements.

```rust
let mut vec = vec![1, 2, 3, 4, 5];
let drained: Vec<_> = vec.drain(1..4).collect();
// drained is [2, 3, 4]
// vec is now [1, 5]
```

**Time Complexity**: O(n)

**Use cases**:
- Remove a range and process the removed elements
- Extract elements while maintaining the rest

```rust
// Remove all elements greater than 3
let mut vec = vec![1, 5, 2, 6, 3, 7, 4];
vec.retain(|&x| x <= 3);
// vec is now [1, 2, 3, 3, 4]

// Alternative with drain:
let mut vec = vec![1, 5, 2, 6, 3, 7, 4];
let mut i = 0;
while i < vec.len() {
    if vec[i] > 3 {
        vec.remove(i);
    } else {
        i += 1;
    }
}
```

---

### splice()

Replace a range with elements from an iterator.

```rust
let mut vec = vec![1, 2, 3, 4, 5];
let removed: Vec<_> = vec.splice(1..3, vec![10, 20, 30]).collect();
// removed is [2, 3]
// vec is now [1, 10, 20, 30, 4, 5]
```

**Time Complexity**: O(n + m) where m is the number of new elements

**Use cases**:
- Replace a section of a vector
- Insert multiple elements at once

```rust
// Insert without removing
let mut vec = vec![1, 2, 5, 6];
vec.splice(2..2, vec![3, 4]);
// vec is now [1, 2, 3, 4, 5, 6]

// Remove without inserting
let mut vec = vec![1, 2, 3, 4, 5];
vec.splice(1..4, std::iter::empty());
// vec is now [1, 5]
```

---

### split_off()

Split the vector at an index, returning the tail.

```rust
let mut vec = vec![1, 2, 3, 4, 5];
let tail = vec.split_off(2);
// vec is now [1, 2]
// tail is [3, 4, 5]
```

**Time Complexity**: O(n) where n is the length of the tail

**Use cases**:
- Split data into two parts
- Extract a suffix

```rust
// Split a path into directory and filename
let mut path = vec!["home", "user", "documents", "file.txt"];
let filename = path.split_off(path.len() - 1);
// path is ["home", "user", "documents"]
// filename is ["file.txt"]
```

---

## iter_mut() Patterns

The `iter_mut()` method creates an iterator that yields mutable references, allowing you to modify elements in place.

### Basic Mutation

```rust
let mut vec = vec![1, 2, 3, 4, 5];
for x in vec.iter_mut() {
    *x *= 2;
}
// vec is now [2, 4, 6, 8, 10]
```

---

### Conditional Mutation

```rust
let mut vec = vec![1, 2, 3, 4, 5];
for x in vec.iter_mut() {
    if *x % 2 == 0 {
        *x = 0;
    }
}
// vec is now [1, 0, 3, 0, 5]
```

---

### Using filter() with iter_mut()

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6];
vec.iter_mut()
    .filter(|x| **x % 2 == 0)
    .for_each(|x| *x *= 10);
// vec is now [1, 20, 3, 40, 5, 60]
```

---

### Enumerate with iter_mut()

```rust
let mut vec = vec!["a", "b", "c", "d"];
for (i, x) in vec.iter_mut().enumerate() {
    *x = if i % 2 == 0 { "even" } else { "odd" };
}
// vec is now ["even", "odd", "even", "odd"]
```

---

### Zipping with iter_mut()

```rust
let mut vec1 = vec![1, 2, 3];
let vec2 = vec![10, 20, 30];

for (x, y) in vec1.iter_mut().zip(&vec2) {
    *x += y;
}
// vec1 is now [11, 22, 33]
```

---

### Chaining Operations

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6, 7, 8];
vec.iter_mut()
    .filter(|x| **x % 2 == 0)  // Get even numbers
    .take(2)                    // Take first 2
    .for_each(|x| *x = 0);      // Set to 0
// vec is now [1, 0, 3, 0, 5, 6, 7, 8]
```

---

### Mutation with Side Effects

```rust
let mut vec = vec![1, 2, 3, 4];
let mut sum = 0;

for x in vec.iter_mut() {
    sum += *x;
    *x = sum;
}
// vec is now [1, 3, 6, 10] (running sum)
```

---

### Complex Transformations

```rust
struct Point { x: i32, y: i32 }

let mut points = vec![
    Point { x: 1, y: 2 },
    Point { x: 3, y: 4 },
    Point { x: 5, y: 6 },
];

for point in points.iter_mut() {
    point.x *= 2;
    point.y *= 2;
}
// All points are now doubled
```

---

### Using map() After iter_mut()

Note: `map()` on an iterator doesn't mutate. To mutate, use `for_each()`:

```rust
let mut vec = vec![1, 2, 3];

// This does NOT mutate:
// vec.iter_mut().map(|x| *x *= 2);  // No effect!

// Use for_each instead:
vec.iter_mut().for_each(|x| *x *= 2);
// vec is now [2, 4, 6]
```

---

### Combining with Chunks

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6];
for chunk in vec.chunks_mut(2) {
    chunk.reverse();
}
// vec is now [2, 1, 4, 3, 6, 5]
```

---

### Mutable Windows

```rust
let mut vec = vec![1, 2, 3, 4, 5];
for window in vec.windows(2) {
    // Can't mutate through windows() - use chunks_mut() instead
    println!("{:?}", window);
}

// For mutation, use chunks_mut:
for chunk in vec.chunks_mut(2) {
    if chunk.len() == 2 {
        chunk.swap(0, 1);
    }
}
// vec is now [2, 1, 4, 3, 5]
```

---

## Advanced Mutation Patterns

### Pattern 1: In-Place Transformation

Transform elements without allocating a new vector.

```rust
let mut vec = vec![1, 2, 3, 4, 5];

// Instead of:
// let new_vec: Vec<_> = vec.iter().map(|x| x * 2).collect();

// Do:
for x in vec.iter_mut() {
    *x *= 2;
}
// vec is now [2, 4, 6, 8, 10]
```

**Benefits**: No allocation, better cache locality

---

### Pattern 2: Conditional Removal with Partition

Split a vector based on a predicate.

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6];
let pos = {
    let mut pos = 0;
    for i in 0..vec.len() {
        if vec[i] % 2 == 0 {
            vec.swap(pos, i);
            pos += 1;
        }
    }
    pos
};
let odds = vec.split_off(pos);
// vec is now [2, 4, 6] (evens)
// odds is [1, 3, 5]
```

Better approach using `retain`:

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6];
let mut odds = Vec::new();
vec.retain(|&x| {
    if x % 2 == 0 {
        true
    } else {
        odds.push(x);
        false
    }
});
// vec is [2, 4, 6]
// odds is [1, 3, 5]
```

---

### Pattern 3: Batch Updates

Update multiple elements based on their relationships.

```rust
// Smooth data (each element becomes average of neighbors)
let mut data = vec![10, 5, 8, 12, 6];
let copy = data.clone();

for i in 1..data.len() - 1 {
    data[i] = (copy[i - 1] + copy[i] + copy[i + 1]) / 3;
}
// data is now [10, 7, 8, 8, 6]
```

---

### Pattern 4: Swap-Based Algorithms

Efficiently rearrange elements using swaps.

```rust
// Dutch National Flag algorithm (partition into three groups)
fn dutch_flag(vec: &mut Vec<i32>, pivot: i32) {
    let (mut low, mut mid, mut high) = (0, 0, vec.len());

    while mid < high {
        if vec[mid] < pivot {
            vec.swap(low, mid);
            low += 1;
            mid += 1;
        } else if vec[mid] > pivot {
            high -= 1;
            vec.swap(mid, high);
        } else {
            mid += 1;
        }
    }
}

let mut vec = vec![3, 5, 2, 7, 3, 8, 3, 1];
dutch_flag(&mut vec, 3);
// vec is now partitioned: [2, 1, 3, 3, 3, 5, 7, 8]
// All elements < 3, then all == 3, then all > 3
```

---

### Pattern 5: Circular Buffer Operations

Use rotate to implement circular buffer behavior.

```rust
struct CircularBuffer<T> {
    buffer: Vec<T>,
    start: usize,
}

impl<T> CircularBuffer<T> {
    fn push(&mut self, item: T) {
        self.buffer.push(item);
        if self.buffer.len() > 10 {  // Max capacity
            self.buffer.remove(0);
        }
    }

    fn rotate(&mut self, n: usize) {
        self.buffer.rotate_left(n);
    }
}
```

---

### Pattern 6: Two-Pointer Technique

Process elements from both ends.

```rust
// Remove duplicates from sorted array
fn remove_duplicates(vec: &mut Vec<i32>) {
    if vec.is_empty() {
        return;
    }

    let mut write = 1;
    for read in 1..vec.len() {
        if vec[read] != vec[read - 1] {
            vec[write] = vec[read];
            write += 1;
        }
    }
    vec.truncate(write);
}

let mut vec = vec![1, 1, 2, 2, 2, 3, 4, 4];
remove_duplicates(&mut vec);
// vec is now [1, 2, 3, 4]
```

---

### Pattern 7: Chunk Processing

Process elements in fixed-size groups.

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6, 7, 8];

for chunk in vec.chunks_mut(3) {
    // Normalize each chunk
    let sum: i32 = chunk.iter().sum();
    let avg = sum / chunk.len() as i32;
    for x in chunk {
        *x -= avg;
    }
}
// Each chunk is now normalized around its mean
```

---

### Pattern 8: State Machine with Mutations

Use mutations to implement state transitions.

```rust
#[derive(Debug, PartialEq)]
enum State { Pending, Processing, Complete, Failed }

struct Task {
    id: u32,
    state: State,
    retry_count: u32,
}

fn process_tasks(tasks: &mut Vec<Task>) {
    for task in tasks.iter_mut() {
        match task.state {
            State::Pending => {
                task.state = State::Processing;
            }
            State::Processing => {
                // Simulate processing
                if task.id % 2 == 0 {
                    task.state = State::Complete;
                } else {
                    task.retry_count += 1;
                    if task.retry_count > 3 {
                        task.state = State::Failed;
                    }
                }
            }
            _ => {}
        }
    }
}
```

---

## Performance Considerations

### Memory Allocation

**Avoid**: Creating new vectors unnecessarily

```rust
// Bad: Allocates new vector
let doubled: Vec<_> = vec.iter().map(|x| x * 2).collect();
```

**Prefer**: Mutating in place

```rust
// Good: No allocation
vec.iter_mut().for_each(|x| *x *= 2);
```

---

### Complexity Awareness

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| `push()` | O(1) amortized | May trigger reallocation |
| `pop()` | O(1) | Always fast |
| `insert(i, x)` | O(n) | Shifts all elements after i |
| `remove(i)` | O(n) | Shifts all elements after i |
| `swap_remove(i)` | O(1) | Doesn't preserve order |
| `drain(range)` | O(n) | Where n = range length |
| `append()` | O(m) | Where m = appended vec length |
| `sort()` | O(n log n) | Stable sort |
| `sort_unstable()` | O(n log n) | Faster but unstable |

---

### Choosing the Right Method

**For removing elements**:
- Use `swap_remove()` if order doesn't matter (O(1))
- Use `remove()` if order matters (O(n))
- Use `retain()` for removing many elements (O(n) with one pass)
- Use `drain()` if you need the removed elements

**For sorting**:
- Use `sort_unstable()` for primitive types (faster)
- Use `sort()` when stability matters
- Use `sort_by_cached_key()` for expensive key functions

**For iteration**:
- Use `iter_mut()` to modify in place
- Use `chunks_mut()` for batch processing
- Avoid allocating intermediate collections

---

### Pre-allocating Capacity

When you know the final size, pre-allocate:

```rust
// Bad: Multiple reallocations
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}

// Good: One allocation
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}
```

---

## Common Pitfalls

### Pitfall 1: Modifying While Iterating

```rust
// WRONG: Can't modify collection while iterating
let mut vec = vec![1, 2, 3, 4];
for x in &vec {
    vec.push(*x);  // Compile error!
}
```

**Solution**: Use indices or drain

```rust
let mut vec = vec![1, 2, 3, 4];
let len = vec.len();
for i in 0..len {
    vec.push(vec[i]);
}
// vec is now [1, 2, 3, 4, 1, 2, 3, 4]
```

---

### Pitfall 2: Inefficient Removal

```rust
// WRONG: O(n²) - calling remove() in a loop
let mut vec = vec![1, 2, 3, 4, 5, 6];
let mut i = 0;
while i < vec.len() {
    if vec[i] % 2 == 0 {
        vec.remove(i);  // O(n) each time!
    } else {
        i += 1;
    }
}
```

**Solution**: Use `retain()`

```rust
// CORRECT: O(n) - single pass
let mut vec = vec![1, 2, 3, 4, 5, 6];
vec.retain(|x| x % 2 != 0);
```

---

### Pitfall 3: Unnecessary Cloning

```rust
// WRONG: Clones entire vector
let mut vec = vec![1, 2, 3, 4];
let new_vec: Vec<_> = vec.iter().map(|x| x * 2).collect();
vec = new_vec;
```

**Solution**: Mutate in place

```rust
// CORRECT: No cloning
let mut vec = vec![1, 2, 3, 4];
vec.iter_mut().for_each(|x| *x *= 2);
```

---

### Pitfall 4: Index Out of Bounds

```rust
// WRONG: May panic
let mut vec = vec![1, 2, 3];
for i in 0..10 {
    vec[i] = 0;  // Panics when i >= 3
}
```

**Solution**: Use proper iteration or resize

```rust
// CORRECT: Resize first
let mut vec = vec![1, 2, 3];
vec.resize(10, 0);
for i in 0..10 {
    vec[i] = i as i32;
}
```

---

### Pitfall 5: Forgetting drain() is Lazy

```rust
// WRONG: Nothing happens
let mut vec = vec![1, 2, 3, 4, 5];
vec.drain(1..4);  // Iterator created but not consumed!
// vec is still [1, 2, 3, 4, 5]
```

**Solution**: Consume the iterator

```rust
// CORRECT
let mut vec = vec![1, 2, 3, 4, 5];
vec.drain(1..4).for_each(drop);
// Or simply:
let _: Vec<_> = vec.drain(1..4).collect();
// vec is now [1, 5]
```

---

### Pitfall 6: Not Using Capacity Efficiently

```rust
// WRONG: Many reallocations
let mut vec = Vec::new();
for i in 0..1_000_000 {
    vec.push(i);
}
```

**Solution**: Reserve capacity upfront

```rust
// CORRECT: One allocation
let mut vec = Vec::with_capacity(1_000_000);
for i in 0..1_000_000 {
    vec.push(i);
}
```

---

### Pitfall 7: Using sort() on Partially Sorted Data

```rust
// WRONG: Slow for nearly sorted data
let mut vec = vec![1, 2, 3, 5, 4, 6, 7, 8];
vec.sort();
```

**Consider**: Insertion sort or sorting only the unsorted portion

```rust
// For small unsorted sections, manual fixing can be faster
let mut vec = vec![1, 2, 3, 5, 4, 6, 7, 8];
// Manually swap if you know the problem area
vec.swap(3, 4);
// vec is now [1, 2, 3, 4, 5, 6, 7, 8]
```

---

### Pitfall 8: Mutating During Pattern Matching

```rust
// WRONG: Can't mutate during match
let mut vec = vec![Some(1), None, Some(3)];
for x in &vec {
    match x {
        Some(val) => vec.push(Some(*val)),  // Compile error!
        None => {}
    }
}
```

**Solution**: Collect changes first, then apply

```rust
// CORRECT
let mut vec = vec![Some(1), None, Some(3)];
let to_add: Vec<_> = vec.iter()
    .filter_map(|x| *x)
    .collect();
vec.extend(to_add.into_iter().map(Some));
```

---

## Quick Reference

### Adding Elements

| Method | Signature | Complexity | Notes |
|--------|-----------|------------|-------|
| `push(x)` | `Vec<T>` → `()` | O(1) | Add to end |
| `insert(i, x)` | `(usize, T)` → `()` | O(n) | Insert at index |
| `append(&mut other)` | `&mut Vec<T>` → `()` | O(m) | Move all from other |
| `extend(iter)` | `impl Iterator` → `()` | O(m) | Add from iterator |

### Removing Elements

| Method | Signature | Complexity | Notes |
|--------|-----------|------------|-------|
| `pop()` | `()` → `Option<T>` | O(1) | Remove from end |
| `remove(i)` | `usize` → `T` | O(n) | Remove at index |
| `swap_remove(i)` | `usize` → `T` | O(1) | Remove, don't preserve order |
| `truncate(len)` | `usize` → `()` | O(n) | Keep first len elements |
| `clear()` | `()` → `()` | O(n) | Remove all |
| `retain(pred)` | `impl Fn(&T) → bool` | O(n) | Keep matching |
| `dedup()` | `()` → `()` | O(n) | Remove consecutive duplicates |

### Modifying Elements

| Method | Signature | Complexity | Notes |
|--------|-----------|------------|-------|
| `iter_mut()` | `()` → `IterMut<T>` | - | Mutable iterator |
| `fill(value)` | `T` → `()` | O(n) | Set all to value |
| `fill_with(f)` | `impl Fn() → T` | O(n) | Set all from closure |
| `swap(i, j)` | `(usize, usize)` → `()` | O(1) | Swap two elements |

### Reordering

| Method | Signature | Complexity | Notes |
|--------|-----------|------------|-------|
| `reverse()` | `()` → `()` | O(n) | Reverse order |
| `sort()` | `()` → `()` | O(n log n) | Stable sort |
| `sort_unstable()` | `()` → `()` | O(n log n) | Faster, unstable |
| `sort_by(f)` | `impl Fn(&T, &T)` | O(n log n) | Custom comparison |
| `sort_by_key(f)` | `impl Fn(&T) → K` | O(n log n) | Sort by key |
| `rotate_left(n)` | `usize` → `()` | O(n) | Rotate left |
| `rotate_right(n)` | `usize` → `()` | O(n) | Rotate right |

### Range Operations

| Method | Signature | Complexity | Notes |
|--------|-----------|------------|-------|
| `drain(range)` | `Range` → `Drain<T>` | O(n) | Remove and iterate |
| `splice(range, iter)` | `(Range, impl Iterator)` | O(n + m) | Replace range |
| `split_off(i)` | `usize` → `Vec<T>` | O(n) | Split into two |

### Capacity

| Method | Signature | Complexity | Notes |
|--------|-----------|------------|-------|
| `reserve(n)` | `usize` → `()` | O(n)* | Reserve capacity |
| `reserve_exact(n)` | `usize` → `()` | O(n)* | Reserve exactly |
| `shrink_to_fit()` | `()` → `()` | O(n)* | Minimize capacity |
| `shrink_to(n)` | `usize` → `()` | O(n)* | Shrink to at least |

*O(n) only if reallocation occurs

---

### Decision Tree: Choosing the Right Method

**Need to add elements?**
- At the end → `push()`
- At specific position → `insert()`
- From another vec → `append()`
- From iterator → `extend()`

**Need to remove elements?**
- From end → `pop()`
- At specific position (preserve order) → `remove()`
- At specific position (order doesn't matter) → `swap_remove()`
- Many elements (with condition) → `retain()`
- Range of elements → `drain()`
- All elements → `clear()`

**Need to modify elements?**
- All elements → `iter_mut()` + `for_each()`
- Set to same value → `fill()`
- Set with function → `fill_with()`
- Conditional modification → `iter_mut()` + `filter()` + `for_each()`

**Need to reorder?**
- Reverse → `reverse()`
- Sort → `sort()` or `sort_unstable()`
- Custom order → `sort_by()` or `sort_by_key()`
- Rotate → `rotate_left()` or `rotate_right()`

---

### Example Combinations

#### Remove and collect removed elements

```rust
let mut vec = vec![1, 2, 3, 4, 5];
let removed: Vec<_> = vec.drain(1..4).collect();
// vec is [1, 5], removed is [2, 3, 4]
```

#### Sort and deduplicate

```rust
let mut vec = vec![3, 1, 4, 1, 5, 9, 2, 6, 5];
vec.sort_unstable();
vec.dedup();
// vec is [1, 2, 3, 4, 5, 6, 9]
```

#### Conditional transformation

```rust
let mut vec = vec![1, 2, 3, 4, 5];
vec.iter_mut()
    .filter(|x| **x % 2 == 0)
    .for_each(|x| *x *= 10);
// vec is [1, 20, 3, 40, 5]
```

#### Batch processing

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6];
for chunk in vec.chunks_mut(2) {
    chunk.reverse();
}
// vec is [2, 1, 4, 3, 6, 5]
```

---

## Summary

Mutating operations in Rust are powerful tools for efficient data manipulation:

1. **Vec methods** provide comprehensive control over dynamic arrays
2. **Slice methods** work on any contiguous sequence
3. **Range operations** enable efficient bulk modifications
4. **iter_mut()** allows safe in-place transformations
5. **Advanced patterns** solve complex algorithmic problems

**Key principles**:
- Mutate in place when possible to avoid allocations
- Choose the right method based on time complexity
- Pre-allocate capacity when the final size is known
- Use `retain()` instead of repeated `remove()`
- Prefer `swap_remove()` when order doesn't matter
- Be aware of the difference between stable and unstable sorts

By mastering these operations, you can write efficient, idiomatic Rust code that manipulates data with minimal overhead.
