
# Comprehensive Guide to Immutable Iterator Methods in Rust

> ✅ All examples below use **immutable iterator methods** that do not mutate the original collection and are **side-effect free**.

## Table of Contents
1. [Basic Iterator Creation](#basic-iterator-creation)
2. [Transformation Methods](#transformation-methods)
3. [Filtering Methods](#filtering-methods)
4. [Reduction Methods](#reduction-methods)
5. [Searching and Matching](#searching-and-matching)
6. [Combining Iterators](#combining-iterators)
7. [Limiting and Skipping](#limiting-and-skipping)
8. [Stateful Iteration](#stateful-iteration)
9. [Advanced Iterator Methods](#advanced-iterator-methods)
10. [Error Handling with Iterators](#error-handling-with-iterators)
11. [Advanced Patterns](#advanced-patterns)
12. [Performance Considerations](#performance-considerations)
13. [Common Pitfalls](#common-pitfalls)
14. [Quick Reference](#quick-reference-table)

---

## Basic Iterator Creation

### iter() Create an Immutable Iterator

**✅ Problem**: Iterate over a collection without taking ownership or mutating it.

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];
    
    for &x in v.iter() {
        println!("{}", x);
    }
    
    // v is still usable here
    println!("Original: {:?}", v);
}
```

**📘 Explanation**: `iter()` returns an iterator over `&T`. The collection remains unchanged and owned.

---

## Transformation Methods

### map() Transform Each Element

**✅ Problem**: Convert a collection of numbers to their squares.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4];
    let squared: Vec<i32> = numbers.iter()
        .map(|&x| x * x)
        .collect();
    
    println!("Original: {:?}", numbers); // [1, 2, 3, 4]
    println!("Squared: {:?}", squared);   // [1, 4, 9, 16]
}
```

**📘 Explanation**: `map` transforms each element through a closure without modifying the original collection.

---

## Filtering Methods

### filter() Select Elements by Predicate

**✅ Problem**: Keep only even numbers from a collection.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let evens: Vec<i32> = numbers.iter()
        .filter(|&&x| x % 2 == 0)
        .copied()
        .collect();
    
    println!("Evens: {:?}", evens); // [2, 4, 6]
}
```

**📘 Explanation**: `filter` creates an iterator containing only elements that satisfy the predicate.

---

### filter_map() Filter and Map in One Step

**✅ Problem**: Parse strings to numbers, ignoring invalid ones.

```rust
fn main() {
    let strings = vec!["1", "two", "3", "four", "5"];
    let numbers: Vec<i32> = strings.iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    println!("{:?}", numbers); // [1, 3, 5]
}
```

**📘 Explanation**: `filter_map` applies a function that returns `Option<T>`, keeping only `Some` values. More efficient than separate `filter` and `map`.

---

## Reduction Methods

### fold() Reduce to Single Value

**✅ Problem**: Calculate the sum of all elements.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let sum = numbers.iter().fold(0, |acc, &x| acc + x);
    
    println!("Sum: {}", sum); // 15
}
```

**📘 Explanation**: `fold` accumulates a single value by applying a function to each element with an accumulator.

---

### reduce() Reduce Without Initial Value

**✅ Problem**: Find the maximum element.

```rust
fn main() {
    let numbers = vec![3, 7, 2, 9, 1];
    let max = numbers.iter().reduce(|a, b| if a > b { a } else { b });
    
    println!("Max: {:?}", max); // Some(9)
}
```

**📘 Explanation**: `reduce` is like `fold` but uses the first element as the initial accumulator. Returns `Option<T>`.

---

### scan() Stateful Mapping

**✅ Problem**: Generate running totals.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4];
    let running_sum: Vec<i32> = numbers.iter()
        .scan(0, |state, &x| {
            *state += x;
            Some(*state)
        })
        .collect();
    
    println!("{:?}", running_sum); // [1, 3, 6, 10]
}
```

**📘 Explanation**: `scan` is like `fold` but yields intermediate states, allowing stateful transformations.

---

## Searching and Matching

### find() Get First Matching Element

**✅ Problem**: Find the first number divisible by 3.

```rust
fn main() {
    let numbers = vec![1, 2, 5, 6, 8, 9];
    let found = numbers.iter().find(|&&x| x % 3 == 0);

    println!("{:?}", found); // Some(6)
}
```

**📘 Explanation**: `find` returns the first element satisfying the predicate as `Option<&T>`.

---

### position() Find Index of First Match

**✅ Problem**: Find the index of the first even number.

```rust
fn main() {
    let numbers = vec![1, 3, 4, 5, 6];
    let pos = numbers.iter().position(|&x| x % 2 == 0);

    println!("{:?}", pos); // Some(2)
}
```

**📘 Explanation**: `position` returns the index of the first matching element as `Option<usize>`.

---

### rposition() Find Index from Right

**✅ Problem**: Find the last occurrence of an even number.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let pos = numbers.iter().rposition(|&x| x % 2 == 0);

    println!("{:?}", pos); // Some(5) - index of 6
}
```

**📘 Explanation**: `rposition` searches from the right (end) and returns the index. Requires `DoubleEndedIterator`.

---

### any() Check if Any Element Matches

**✅ Problem**: Check if there's any negative number.

```rust
fn main() {
    let numbers = vec![1, 2, -3, 4];
    let has_negative = numbers.iter().any(|&x| x < 0);

    println!("Has negative: {}", has_negative); // true
}
```

**📘 Explanation**: `any` returns `true` if at least one element satisfies the predicate. Short-circuits on first match.

---

### all() Check if All Elements Match

**✅ Problem**: Check if all numbers are positive.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4];
    let all_positive = numbers.iter().all(|&x| x > 0);

    println!("All positive: {}", all_positive); // true
}
```

**📘 Explanation**: `all` returns `true` only if every element satisfies the predicate. Short-circuits on first failure.

---

### find_map() Find and Transform

**✅ Problem**: Find the first valid parsed number.

```rust
fn main() {
    let strings = vec!["a", "1", "b", "2"];
    let first_num = strings.iter().find_map(|s| s.parse::<i32>().ok());

    println!("{:?}", first_num); // Some(1)
}
```

**📘 Explanation**: Combines `find` and `map`, returning the first `Some` result.

---

## Combining Iterators

### chain() Concatenate Iterators

**✅ Problem**: Combine two collections into one iterator.

```rust
fn main() {
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let combined: Vec<i32> = a.iter()
        .chain(b.iter())
        .copied()
        .collect();

    println!("{:?}", combined); // [1, 2, 3, 4, 5, 6]
}
```

**📘 Explanation**: `chain` concatenates two iterators sequentially without allocating intermediate storage.

---

### zip() Combine Two Iterators Pairwise

**✅ Problem**: Pair elements from two collections.

```rust
fn main() {
    let names = vec!["Alice", "Bob", "Charlie"];
    let ages = vec![30, 25, 35];
    let pairs: Vec<_> = names.iter()
        .zip(ages.iter())
        .collect();

    println!("{:?}", pairs);
    // [("Alice", 30), ("Bob", 25), ("Charlie", 35)]
}
```

**📘 Explanation**: `zip` pairs elements from two iterators. Stops when the shorter iterator is exhausted.

---

### intersperse() Insert Element Between Items

**✅ Problem**: Join words with commas (requires nightly or itertools crate).

```rust
// Using stable Rust with manual implementation
fn main() {
    let words = vec!["apple", "banana", "cherry"];
    let mut result = String::new();

    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        result.push_str(word);
    }

    println!("{}", result); // "apple, banana, cherry"
}
```

**📘 Explanation**: Intersperse adds a separator between elements. Available via itertools crate or as experimental feature.

---

## Limiting and Skipping

### take() Limit Number of Elements

**✅ Problem**: Get first 3 elements.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let first_three: Vec<i32> = numbers.iter()
        .copied()
        .take(3)
        .collect();
    
    println!("{:?}", first_three); // [1, 2, 3]
}
```

**📘 Explanation**: `take(n)` creates an iterator that yields at most `n` elements.

---

### skip() Skip First N Elements

**✅ Problem**: Skip the first 2 elements.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let after_skip: Vec<i32> = numbers.iter()
        .copied()
        .skip(2)
        .collect();
    
    println!("{:?}", after_skip); // [3, 4, 5]
}
```

**📘 Explanation**: `skip(n)` creates an iterator that skips the first `n` elements.

---

### take_while() Take Until Condition Fails

**✅ Problem**: Take elements while they are less than 5.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 6, 4, 5];
    let result: Vec<i32> = numbers.iter()
        .copied()
        .take_while(|&x| x < 5)
        .collect();
    
    println!("{:?}", result); // [1, 2, 3]
}
```

**📘 Explanation**: `take_while` yields elements until the predicate returns false, then stops.

---

### skip_while() Skip Until Condition Fails

**✅ Problem**: Skip elements while they are less than 4.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let result: Vec<i32> = numbers.iter()
        .copied()
        .skip_while(|&x| x < 4)
        .collect();
    
    println!("{:?}", result); // [4, 5]
}
```

**📘 Explanation**: `skip_while` skips elements until the predicate returns false, then yields the rest.

---

### chain() Concatenate Iterators

**✅ Problem**: Combine two collections into one iterator.

```rust
fn main() {
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let combined: Vec<i32> = a.iter()
        .chain(b.iter())
        .copied()
        .collect();
    
    println!("{:?}", combined); // [1, 2, 3, 4, 5, 6]
}
```

**📘 Explanation**: `chain` concatenates two iterators sequentially without allocating intermediate storage.

---

### zip() Combine Two Iterators Pairwise

**✅ Problem**: Pair elements from two collections.

```rust
fn main() {
    let names = vec!["Alice", "Bob", "Charlie"];
    let ages = vec![30, 25, 35];
    let pairs: Vec<_> = names.iter()
        .zip(ages.iter())
        .collect();
    
    println!("{:?}", pairs); 
    // [("Alice", 30), ("Bob", 25), ("Charlie", 35)]
}
```

**📘 Explanation**: `zip` pairs elements from two iterators. Stops when the shorter iterator is exhausted.

---

### enumerate() Get Index and Value

**✅ Problem**: Print each element with its index.

```rust
fn main() {
    let fruits = vec!["apple", "banana", "cherry"];
    
    for (i, fruit) in fruits.iter().enumerate() {
        println!("{}: {}", i, fruit);
    }
    // 0: apple
    // 1: banana
    // 2: cherry
}
```

**📘 Explanation**: `enumerate` wraps each element with its index as a tuple `(usize, T)`.

---

### flat_map() Map and Flatten

**✅ Problem**: Split strings into characters and flatten.

```rust
fn main() {
    let words = vec!["hi", "bye"];
    let chars: Vec<char> = words.iter()
        .flat_map(|s| s.chars())
        .collect();
    
    println!("{:?}", chars); // ['h', 'i', 'b', 'y', 'e']
}
```

**📘 Explanation**: `flat_map` maps each element to an iterator, then flattens all iterators into one.

---

### flatten() Flatten Nested Iterators

**✅ Problem**: Flatten a nested structure.

```rust
fn main() {
    let nested = vec![vec![1, 2], vec![3, 4], vec![5]];
    let flat: Vec<i32> = nested.iter()
        .flatten()
        .copied()
        .collect();
    
    println!("{:?}", flat); // [1, 2, 3, 4, 5]
}
```

**📘 Explanation**: `flatten` removes one level of nesting from nested iterators.

---

### filter_map() Filter and Map in One Step

**✅ Problem**: Parse strings to numbers, ignoring invalid ones.

```rust
fn main() {
    let strings = vec!["1", "two", "3", "four", "5"];
    let numbers: Vec<i32> = strings.iter()
        .filter_map(|s| s.parse().ok())
        .collect();
    
    println!("{:?}", numbers); // [1, 3, 5]
}
```

**📘 Explanation**: `filter_map` applies a function that returns `Option<T>`, keeping only `Some` values.

---

### find() Get First Matching Element

**✅ Problem**: Find the first number divisible by 3.

```rust
fn main() {
    let numbers = vec![1, 2, 5, 6, 8, 9];
    let found = numbers.iter().find(|&&x| x % 3 == 0);
    
    println!("{:?}", found); // Some(6)
}
```

**📘 Explanation**: `find` returns the first element satisfying the predicate as `Option<&T>`.

---

### position() Find Index of First Match

**✅ Problem**: Find the index of the first even number.

```rust
fn main() {
    let numbers = vec![1, 3, 4, 5, 6];
    let pos = numbers.iter().position(|&x| x % 2 == 0);
    
    println!("{:?}", pos); // Some(2)
}
```

**📘 Explanation**: `position` returns the index of the first matching element as `Option<usize>`.

---

### any() Check if Any Element Matches

**✅ Problem**: Check if there's any negative number.

```rust
fn main() {
    let numbers = vec![1, 2, -3, 4];
    let has_negative = numbers.iter().any(|&x| x < 0);
    
    println!("Has negative: {}", has_negative); // true
}
```

**📘 Explanation**: `any` returns `true` if at least one element satisfies the predicate.

---

### all() Check if All Elements Match

**✅ Problem**: Check if all numbers are positive.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4];
    let all_positive = numbers.iter().all(|&x| x > 0);
    
    println!("All positive: {}", all_positive); // true
}
```

**📘 Explanation**: `all` returns `true` only if every element satisfies the predicate.

---

### count() Count Elements

**✅ Problem**: Count how many elements satisfy a condition.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let even_count = numbers.iter().filter(|&&x| x % 2 == 0).count();
    
    println!("Even count: {}", even_count); // 3
}
```

**📘 Explanation**: `count` consumes the iterator and returns the number of elements.

---

### sum() Calculate Sum

**✅ Problem**: Sum all numbers in a collection.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let total: i32 = numbers.iter().sum();
    
    println!("Sum: {}", total); // 15
}
```

**📘 Explanation**: `sum` adds all elements together. Requires the element type to implement `Sum`.

---

### product() Calculate Product

**✅ Problem**: Calculate factorial of 5.

```rust
fn main() {
    let factorial: i32 = (1..=5).product();
    
    println!("5! = {}", factorial); // 120
}
```

**📘 Explanation**: `product` multiplies all elements together. Requires the element type to implement `Product`.

---

### min() / max() Find Minimum or Maximum

**✅ Problem**: Find the smallest and largest numbers.

```rust
fn main() {
    let numbers = vec![3, 7, 2, 9, 1, 5];
    let min = numbers.iter().min();
    let max = numbers.iter().max();
    
    println!("Min: {:?}, Max: {:?}", min, max); // Some(1), Some(9)
}
```

**📘 Explanation**: `min` and `max` return the smallest and largest elements as `Option<&T>`.

---

### min_by() / max_by() Custom Comparison

**✅ Problem**: Find longest string.

```rust
fn main() {
    let words = vec!["a", "abc", "ab"];
    let longest = words.iter().max_by(|a, b| a.len().cmp(&b.len()));
    
    println!("Longest: {:?}", longest); // Some("abc")
}
```

**📘 Explanation**: `max_by` uses a custom comparison function instead of the default `Ord` implementation.

---

### min_by_key() / max_by_key() Extract Key for Comparison

**✅ Problem**: Find person with highest age.

```rust
fn main() {
    let people = vec![("Alice", 30), ("Bob", 25), ("Charlie", 35)];
    let oldest = people.iter().max_by_key(|&&(_, age)| age);
    
    println!("Oldest: {:?}", oldest); // Some(("Charlie", 35))
}
```

**📘 Explanation**: `max_by_key` extracts a key from each element for comparison.

---

### partition() Split into Two Collections

**✅ Problem**: Separate even and odd numbers.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let (even, odd): (Vec<_>, Vec<_>) = numbers.iter()
        .partition(|&&x| x % 2 == 0);
    
    println!("Even: {:?}", even); // [2, 4, 6]
    println!("Odd: {:?}", odd);   // [1, 3, 5]
}
```

**📘 Explanation**: `partition` splits elements into two collections based on a predicate.

---

### cycle() Repeat Iterator Infinitely

**✅ Problem**: Cycle through colors repeatedly.

```rust
fn main() {
    let colors = vec!["red", "green", "blue"];
    let repeated: Vec<_> = colors.iter()
        .cycle()
        .take(7)
        .collect();
    
    println!("{:?}", repeated); 
    // ["red", "green", "blue", "red", "green", "blue", "red"]
}
```

**📘 Explanation**: `cycle` creates an infinite iterator that repeats the source forever. Use `take` to limit it.

---

### rev() Reverse Iterator

**✅ Problem**: Iterate in reverse order.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let reversed: Vec<i32> = numbers.iter()
        .rev()
        .copied()
        .collect();
    
    println!("{:?}", reversed); // [5, 4, 3, 2, 1]
}
```

**📘 Explanation**: `rev` reverses the direction of iteration. Only works on double-ended iterators.

---

### step_by() Take Every Nth Element

**✅ Problem**: Get every second element.

```rust
fn main() {
    let numbers = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    let every_second: Vec<i32> = numbers.iter()
        .copied()
        .step_by(2)
        .collect();
    
    println!("{:?}", every_second); // [0, 2, 4, 6, 8]
}
```

**📘 Explanation**: `step_by(n)` creates an iterator that yields every nth element.

---

### inspect() Debug Without Consuming

**✅ Problem**: Print elements during iteration for debugging.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4];
    let doubled: Vec<i32> = numbers.iter()
        .inspect(|&x| println!("Before: {}", x))
        .map(|&x| x * 2)
        .inspect(|&x| println!("After: {}", x))
        .collect();
    
    println!("Result: {:?}", doubled);
}
```

**📘 Explanation**: `inspect` allows side effects (like printing) without consuming the iterator.

---

### collect() Convert Iterator to Collection

**✅ Problem**: Convert an iterator to various collection types.

```rust
use std::collections::{HashSet, HashMap};

fn main() {
    let numbers = vec![1, 2, 3, 2, 1];
    
    // Collect to Vec
    let vec: Vec<i32> = numbers.iter().copied().collect();
    
    // Collect to HashSet (removes duplicates)
    let set: HashSet<i32> = numbers.iter().copied().collect();
    
    // Collect to HashMap
    let map: HashMap<_, _> = numbers.iter().enumerate().collect();
    
    println!("Vec: {:?}", vec);   // [1, 2, 3, 2, 1]
    println!("Set: {:?}", set);   // {1, 2, 3}
    println!("Map: {:?}", map);   // {0: 1, 1: 2, 2: 3, 3: 2, 4: 1}
}
```

**📘 Explanation**: `collect` is extremely versatile and can create any collection that implements `FromIterator`.

---

### copied() / cloned() Convert References to Owned Values

**✅ Problem**: Convert `&T` to `T`.

```rust
fn main() {
    let numbers = vec![1, 2, 3];
    
    // Using copied() for Copy types
    let owned: Vec<i32> = numbers.iter().copied().collect();
    
    let strings = vec![String::from("a"), String::from("b")];
    
    // Using cloned() for Clone types
    let cloned: Vec<String> = strings.iter().cloned().collect();
    
    println!("{:?}", owned);   // [1, 2, 3]
    println!("{:?}", cloned);  // ["a", "b"]
}
```

**📘 Explanation**: `copied` works for `Copy` types, `cloned` for `Clone` types. Both convert `&T` to `T`.

---

### peekable() Look Ahead Without Consuming

**✅ Problem**: Check the next element without consuming it.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4];
    let mut iter = numbers.iter().peekable();
    
    while let Some(&x) = iter.peek() {
        println!("Next is: {}", x);
        iter.next(); // Actually consume it
    }
}
```

**📘 Explanation**: `peekable` wraps an iterator allowing you to view the next element without advancing.

---

### fuse() Stop After First None

**✅ Problem**: Ensure iterator stops cleanly after exhaustion.

```rust
fn main() {
    let numbers = vec![1, 2, 3];
    let mut iter = numbers.iter().fuse();
    
    println!("{:?}", iter.next()); // Some(1)
    println!("{:?}", iter.next()); // Some(2)
    println!("{:?}", iter.next()); // Some(3)
    println!("{:?}", iter.next()); // None
    println!("{:?}", iter.next()); // None (guaranteed)
}
```

**📘 Explanation**: `fuse` guarantees that once the iterator returns `None`, it will always return `None`.

---

### nth() Get Element at Index

**✅ Problem**: Get the 3rd element.

```rust
fn main() {
    let numbers = vec![0, 1, 2, 3, 4, 5];
    let third = numbers.iter().nth(3);
    
    println!("{:?}", third); // Some(3)
}
```

**📘 Explanation**: `nth(n)` returns the element at index `n`. Consumes elements up to that index.

**⚠️ Warning**: Consumes all elements up to index n, so it's O(n) operation.

---

### nth_back() Get Element from End

**✅ Problem**: Get the 3rd element from the end.

```rust
fn main() {
    let numbers = vec![0, 1, 2, 3, 4, 5];
    let third_from_end = numbers.iter().nth_back(2);

    println!("{:?}", third_from_end); // Some(3)
}
```

**📘 Explanation**: `nth_back(n)` returns the nth element from the end. Requires `DoubleEndedIterator`.

---

### last() Get Last Element

**✅ Problem**: Find the last element.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let last = numbers.iter().last();
    
    println!("{:?}", last); // Some(5)
}
```

**📘 Explanation**: `last` consumes the entire iterator and returns the final element as `Option<T>`.

---

### for_each() Execute Closure for Each Element

**✅ Problem**: Print all elements.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4];
    numbers.iter().for_each(|&x| println!("{}", x));
}
```

**📘 Explanation**: `for_each` is like a `for` loop but in functional style. Useful for side effects.

---

### unzip() Split Tuples into Two Collections

**✅ Problem**: Separate names and ages.

```rust
fn main() {
    let people = vec![("Alice", 30), ("Bob", 25), ("Charlie", 35)];
    let (names, ages): (Vec<_>, Vec<_>) = people.iter().cloned().unzip();
    
    println!("Names: {:?}", names); // ["Alice", "Bob", "Charlie"]
    println!("Ages: {:?}", ages);   // [30, 25, 35]
}
```

**📘 Explanation**: `unzip` splits an iterator of tuples into two collections.

---

### try_fold() Fold with Early Exit

**✅ Problem**: Sum until encountering an error.

```rust
fn main() {
    let strings = vec!["1", "2", "3", "four", "5"];
    
    let result: Result<i32, _> = strings.iter().try_fold(0, |acc, s| {
        s.parse::<i32>().map(|n| acc + n)
    });
    
    println!("{:?}", result); // Err(ParseIntError)
}
```

**📘 Explanation**: `try_fold` is like `fold` but stops early if the closure returns an error.

---

### try_for_each() Execute with Early Exit

**✅ Problem**: Process items until an error occurs.

```rust
fn main() {
    let strings = vec!["1", "2", "three"];
    
    let result = strings.iter().try_for_each(|s| {
        s.parse::<i32>().map(|n| println!("Parsed: {}", n))
    });
    
    println!("Result: {:?}", result); // Err(ParseIntError)
}
```

**📘 Explanation**: `try_for_each` applies a fallible operation to each element, stopping on first error.

---

## Advanced Iterator Methods

### by_ref() Borrow Iterator

**✅ Problem**: Use an iterator multiple times while consuming it partially.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let mut iter = numbers.iter();

    // Take first 3 elements
    let first_three: Vec<_> = iter.by_ref().take(3).collect();

    // Use the same iterator for remaining elements
    let remaining: Vec<_> = iter.collect();

    println!("First three: {:?}", first_three); // [1, 2, 3]
    println!("Remaining: {:?}", remaining);     // [4, 5, 6]
}
```

**📘 Explanation**: `by_ref` creates a borrowing iterator adapter, allowing reuse of the original iterator.

---

### map_while() Map Until Condition Fails

**✅ Problem**: Transform elements while a condition holds.

```rust
fn main() {
    let numbers = vec![1, 2, 3, -1, 4, 5];

    // Map while positive (available since Rust 1.57)
    let positive_doubled: Vec<_> = numbers.iter()
        .map_while(|&x| if x > 0 { Some(x * 2) } else { None })
        .collect();

    println!("{:?}", positive_doubled); // [2, 4, 6]
}
```

**📘 Explanation**: Like `take_while` but allows transformation. Stops at first `None`.

---

### cmp() / partial_cmp() Compare Iterators

**✅ Problem**: Compare two sequences lexicographically.

```rust
use std::cmp::Ordering;

fn main() {
    let a = vec![1, 2, 3];
    let b = vec![1, 2, 4];

    match a.iter().cmp(b.iter()) {
        Ordering::Less => println!("a < b"),
        Ordering::Equal => println!("a == b"),
        Ordering::Greater => println!("a > b"),
    }
    // Output: a < b
}
```

**📘 Explanation**: Compares iterators element by element, lexicographically.

---

### eq() / ne() Test Equality

**✅ Problem**: Check if two iterators produce the same elements.

```rust
fn main() {
    let a = vec![1, 2, 3];
    let b = vec![1, 2, 3];
    let c = vec![1, 2, 4];

    println!("{}", a.iter().eq(b.iter())); // true
    println!("{}", a.iter().eq(c.iter())); // false
}
```

**📘 Explanation**: Tests if two iterators yield equal elements.

---

### lt() / le() / gt() / ge() Comparison Methods

**✅ Problem**: Lexicographic comparison.

```rust
fn main() {
    let a = vec![1, 2, 3];
    let b = vec![1, 2, 4];

    println!("{}", a.iter().lt(b.iter())); // true (a < b)
    println!("{}", a.iter().le(b.iter())); // true (a <= b)
}
```

**📘 Explanation**: Lexicographic comparison methods for iterators.

---

### is_sorted() Check if Sorted

**✅ Problem**: Verify if collection is already sorted.

```rust
fn main() {
    let sorted = vec![1, 2, 3, 4, 5];
    let unsorted = vec![3, 1, 4, 1, 5];

    println!("Sorted: {}", sorted.iter().is_sorted());     // true
    println!("Unsorted: {}", unsorted.iter().is_sorted()); // false
}
```

**📘 Explanation**: Returns `true` if iterator yields elements in sorted order.

---

### is_sorted_by() / is_sorted_by_key() Custom Sorted Check

**✅ Problem**: Check if sorted by custom criteria.

```rust
fn main() {
    let numbers = vec![5, 4, 3, 2, 1];

    // Check if sorted in descending order
    let descending = numbers.iter().is_sorted_by(|a, b| a >= b);
    println!("Descending: {}", descending); // true

    // Check if sorted by absolute value
    let values = vec![-1, 2, -3, 4];
    let by_abs = values.iter().is_sorted_by_key(|x| x.abs());
    println!("Sorted by absolute value: {}", by_abs); // false
}
```

---

### array_chunks() Chunk into Arrays

**✅ Problem**: Group elements into fixed-size arrays.

```rust
// Note: array_chunks is experimental (nightly)
// Using chunks as stable alternative
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];

    // Stable alternative using chunks
    for chunk in numbers.chunks(2) {
        println!("{:?}", chunk);
    }
    // [1, 2]
    // [3, 4]
    // [5, 6]
}
```

**📘 Explanation**: Groups elements into fixed-size arrays. Last chunk may be smaller.

---

### windows() Sliding Window View

**✅ Problem**: Get overlapping windows of elements.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // Get all windows of size 3
    for window in numbers.windows(3) {
        println!("{:?}", window);
    }
    // [1, 2, 3]
    // [2, 3, 4]
    // [3, 4, 5]
}
```

**📘 Explanation**: Creates overlapping slices of specified size. Not available on iterators, only slices.

---

### chunks() Non-Overlapping Chunks

**✅ Problem**: Divide into non-overlapping chunks.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7];

    for chunk in numbers.chunks(3) {
        println!("{:?}", chunk);
    }
    // [1, 2, 3]
    // [4, 5, 6]
    // [7]
}
```

**📘 Explanation**: Splits slice into non-overlapping chunks. Last chunk may be smaller.

---

### rchunks() Chunks from Right

**✅ Problem**: Chunk from the end.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7];

    for chunk in numbers.rchunks(3) {
        println!("{:?}", chunk);
    }
    // [5, 6, 7]
    // [2, 3, 4]
    // [1]
}
```

**📘 Explanation**: Like `chunks` but starts from the right end.

---

### split() / rsplit() Split by Separator

**✅ Problem**: Split on a separator element.

```rust
fn main() {
    let numbers = vec![1, 2, 0, 3, 4, 0, 5];

    // Split on zeros
    for segment in numbers.split(|&x| x == 0) {
        println!("{:?}", segment);
    }
    // [1, 2]
    // [3, 4]
    // [5]
}
```

**📘 Explanation**: Splits slice at each element matching the predicate.

---

### splitn() / rsplitn() Limited Splits

**✅ Problem**: Split into at most N parts.

```rust
fn main() {
    let text = "one,two,three,four,five";

    // Split into at most 3 parts
    let parts: Vec<_> = text.splitn(3, ',').collect();
    println!("{:?}", parts);
    // ["one", "two", "three,four,five"]
}
```

**📘 Explanation**: Like `split` but limits the number of splits.

---

### collect_into() Collect into Existing Collection

**✅ Problem**: Extend an existing vector without creating new one.

```rust
fn main() {
    let mut existing = vec![1, 2, 3];

    // Extend existing collection
    (4..=6).collect_into(&mut existing);

    println!("{:?}", existing); // [1, 2, 3, 4, 5, 6]
}
```

**📘 Explanation**: Collect iterator elements into an existing collection (experimental feature).

---

### sum() and product() with Different Types

**✅ Problem**: Sum into different numeric types.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    let sum_i32: i32 = numbers.iter().sum();
    let sum_f64: f64 = numbers.iter().map(|&x| x as f64).sum();

    println!("Sum as i32: {}", sum_i32);   // 15
    println!("Sum as f64: {}", sum_f64);   // 15.0

    // Product
    let factorial: i32 = (1..=5).product();
    println!("5! = {}", factorial);        // 120
}
```

---

### max_by_key() vs max() Performance

**✅ Problem**: Find person with maximum age efficiently.

```rust
#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

fn main() {
    let people = vec![
        Person { name: "Alice".to_string(), age: 30 },
        Person { name: "Bob".to_string(), age: 25 },
        Person { name: "Charlie".to_string(), age: 35 },
    ];

    // Using max_by_key (computes key multiple times)
    let oldest = people.iter().max_by_key(|p| p.age);
    println!("Oldest: {:?}", oldest.unwrap().name);

    // More efficient for expensive key functions: collect keys first
    let with_keys: Vec<_> = people.iter()
        .map(|p| (p.age, p))
        .collect();
    let oldest = with_keys.iter().max_by_key(|(age, _)| age);
}
```

---

## Error Handling with Iterators

### Collecting Results

**✅ Problem**: Parse multiple strings, fail on first error.

```rust
fn main() {
    let strings = vec!["1", "2", "3", "4"];

    // Collect into Result<Vec<_>, _>
    let result: Result<Vec<i32>, _> = strings.iter()
        .map(|s| s.parse::<i32>())
        .collect();

    match result {
        Ok(numbers) => println!("All parsed: {:?}", numbers),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

**📘 Explanation**: When collecting into `Result`, it short-circuits on first `Err`.

---

### Partition Results into Success/Failure

**✅ Problem**: Separate successful parses from failures.

```rust
fn main() {
    let strings = vec!["1", "two", "3", "four", "5"];

    let (successes, failures): (Vec<_>, Vec<_>) = strings.iter()
        .map(|s| s.parse::<i32>())
        .partition(Result::is_ok);

    let successes: Vec<i32> = successes.into_iter()
        .map(Result::unwrap)
        .collect();

    println!("Successes: {:?}", successes); // [1, 3, 5]
    println!("Failure count: {}", failures.len()); // 2
}
```

---

### Using filter_map for Error Handling

**✅ Problem**: Parse all valid numbers, skip invalid ones.

```rust
fn main() {
    let strings = vec!["1", "two", "3", "four", "5"];

    let numbers: Vec<i32> = strings.iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    println!("{:?}", numbers); // [1, 3, 5]
}
```

**📘 Explanation**: `filter_map` with `.ok()` converts `Result` to `Option`, keeping only `Some`.

---

### Collecting Options

**✅ Problem**: Collect only if all are Some.

```rust
fn main() {
    let opts1 = vec![Some(1), Some(2), Some(3)];
    let opts2 = vec![Some(1), None, Some(3)];

    let result1: Option<Vec<i32>> = opts1.into_iter().collect();
    let result2: Option<Vec<i32>> = opts2.into_iter().collect();

    println!("{:?}", result1); // Some([1, 2, 3])
    println!("{:?}", result2); // None
}
```

**📘 Explanation**: Collecting `Option` values yields `Some(Vec)` if all are `Some`, otherwise `None`.

---

### try_fold for Complex Error Handling

**✅ Problem**: Accumulate with custom error handling.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    let result = numbers.iter().try_fold(0, |acc, &x| {
        if x % 2 == 0 {
            Ok(acc + x)
        } else {
            Err(format!("Odd number encountered: {}", x))
        }
    });

    match result {
        Ok(sum) => println!("Sum: {}", sum),
        Err(e) => println!("Error: {}", e),
    }
    // Error: Odd number encountered: 1
}
```

---

## Advanced Patterns

### Chaining Multiple Operations

**✅ Problem**: Complex data transformation pipeline.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    let result: Vec<i32> = numbers.iter()
        .filter(|&&x| x % 2 == 0)     // Keep even numbers
        .map(|&x| x * x)               // Square them
        .skip(1)                       // Skip first result
        .take(2)                       // Take next 2
        .collect();
    
    println!("{:?}", result); // [16, 36]
}
```

**📘 Explanation**: Iterator methods can be chained together for complex transformations.

---

### Combining Multiple Collections

**✅ Problem**: Merge and transform multiple collections.

```rust
fn main() {
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    let c = vec![7, 8, 9];
    
    let combined: Vec<i32> = a.iter()
        .chain(b.iter())
        .chain(c.iter())
        .filter(|&&x| x % 2 != 0)
        .copied()
        .collect();
    
    println!("{:?}", combined); // [1, 3, 5, 7, 9]
}
```

---

### Nested Iteration Patterns

**✅ Problem**: Generate all pairs from two collections.

```rust
fn main() {
    let colors = vec!["red", "blue"];
    let sizes = vec!["small", "large"];
    
    let products: Vec<_> = colors.iter()
        .flat_map(|&color| {
            sizes.iter().map(move |&size| (color, size))
        })
        .collect();
    
    println!("{:?}", products);
    // [("red", "small"), ("red", "large"), ("blue", "small"), ("blue", "large")]
}
```

---

### Stateful Transformations

**✅ Problem**: Number each unique value encountered.

```rust
use std::collections::HashMap;

fn main() {
    let words = vec!["apple", "banana", "apple", "cherry", "banana"];
    
    let mut id_map = HashMap::new();
    let mut next_id = 0;
    
    let ids: Vec<usize> = words.iter()
        .map(|&word| {
            *id_map.entry(word).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            })
        })
        .collect();
    
    println!("{:?}", ids); // [0, 1, 0, 2, 1]
}
```

---

### Real-World Pattern: Data Processing Pipeline

**✅ Problem**: Process log entries - parse, filter, transform, and aggregate.

```rust
#[derive(Debug)]
struct LogEntry {
    level: String,
    message: String,
    timestamp: u64,
}

fn main() {
    let logs = vec![
        "ERROR:Database connection failed:1234567890",
        "INFO:User logged in:1234567891",
        "ERROR:Out of memory:1234567892",
        "WARN:Slow query detected:1234567893",
        "ERROR:Network timeout:1234567894",
    ];

    let error_count = logs.iter()
        .filter_map(|line| {
            let parts: Vec<_> = line.split(':').collect();
            if parts.len() == 3 {
                Some(LogEntry {
                    level: parts[0].to_string(),
                    message: parts[1].to_string(),
                    timestamp: parts[2].parse().unwrap_or(0),
                })
            } else {
                None
            }
        })
        .filter(|entry| entry.level == "ERROR")
        .count();

    println!("Total errors: {}", error_count); // 3
}
```

---

### Pattern: Building Lookup Tables

**✅ Problem**: Create index mappings from data.

```rust
use std::collections::HashMap;

fn main() {
    let words = vec!["apple", "banana", "cherry", "apple", "banana"];

    // Word to indices mapping
    let word_indices: HashMap<&str, Vec<usize>> = words.iter()
        .enumerate()
        .fold(HashMap::new(), |mut map, (i, &word)| {
            map.entry(word).or_insert_with(Vec::new).push(i);
            map
        });

    println!("{:?}", word_indices);
    // {"apple": [0, 3], "banana": [1, 4], "cherry": [2]}
}
```

---

### Pattern: Generating Combinations

**✅ Problem**: Generate all pairs from a single collection.

```rust
fn main() {
    let items = vec![1, 2, 3, 4];

    let pairs: Vec<_> = items.iter()
        .enumerate()
        .flat_map(|(i, &x)| {
            items[i+1..].iter().map(move |&y| (x, y))
        })
        .collect();

    println!("{:?}", pairs);
    // [(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]
}
```

---

### Pattern: State Machine with scan()

**✅ Problem**: Track running state through iterations.

```rust
fn main() {
    let commands = vec!["add 5", "sub 2", "mul 3", "add 1"];

    let results: Vec<i32> = commands.iter()
        .scan(0, |state, cmd| {
            let parts: Vec<_> = cmd.split_whitespace().collect();
            if parts.len() == 2 {
                let value: i32 = parts[1].parse().ok()?;
                match parts[0] {
                    "add" => *state += value,
                    "sub" => *state -= value,
                    "mul" => *state *= value,
                    _ => return None,
                }
                Some(*state)
            } else {
                None
            }
        })
        .collect();

    println!("{:?}", results); // [5, 3, 9, 10]
}
```

---

### Pattern: Lazy Computation with Iterators

**✅ Problem**: Build computation pipeline without executing until needed.

```rust
fn main() {
    let numbers = 1..=1_000_000;

    // This doesn't execute yet - it's lazy!
    let pipeline = numbers
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .filter(|x| x % 3 == 0);

    // Only compute first 5 results
    let first_five: Vec<_> = pipeline.take(5).collect();

    println!("{:?}", first_five); // [4, 36, 100, 196, 324]
}
```

---

### Pattern: Iterator Adapters for Infinite Sequences

**✅ Problem**: Work with potentially infinite data.

```rust
fn main() {
    // Fibonacci sequence
    let fibs: Vec<u64> = std::iter::successors(Some((0, 1)), |&(a, b)| {
        Some((b, a + b))
    })
    .map(|(a, _)| a)
    .take(10)
    .collect();

    println!("{:?}", fibs);
    // [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
}
```

---

### Pattern: Grouping and Aggregating

**✅ Problem**: Group by key and aggregate values.

```rust
use std::collections::HashMap;

fn main() {
    let sales = vec![
        ("apples", 5),
        ("oranges", 3),
        ("apples", 8),
        ("bananas", 2),
        ("oranges", 4),
    ];

    let totals: HashMap<&str, i32> = sales.iter()
        .fold(HashMap::new(), |mut acc, &(fruit, qty)| {
            *acc.entry(fruit).or_insert(0) += qty;
            acc
        });

    println!("{:?}", totals);
    // {"apples": 13, "oranges": 7, "bananas": 2}
}
```

---

### Pattern: Complex Filtering with Multiple Criteria

**✅ Problem**: Apply multiple filtering conditions efficiently.

```rust
#[derive(Debug)]
struct Product {
    name: String,
    price: f64,
    in_stock: bool,
    category: String,
}

fn main() {
    let products = vec![
        Product { name: "Laptop".to_string(), price: 999.99, in_stock: true, category: "Electronics".to_string() },
        Product { name: "Phone".to_string(), price: 699.99, in_stock: false, category: "Electronics".to_string() },
        Product { name: "Desk".to_string(), price: 299.99, in_stock: true, category: "Furniture".to_string() },
    ];

    let available_electronics: Vec<_> = products.iter()
        .filter(|p| p.in_stock)
        .filter(|p| p.category == "Electronics")
        .filter(|p| p.price < 1000.0)
        .collect();

    println!("{:?}", available_electronics);
}
```

---

## Performance Considerations

### Lazy vs Eager Evaluation

**Iterator chains are lazy** - no computation happens until a consuming operation like `collect()`, `fold()`, or `for_each()`.

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // This creates an iterator chain but doesn't execute anything yet
    let pipeline = numbers.iter()
        .inspect(|x| println!("Inspecting: {}", x))
        .map(|x| x * 2)
        .filter(|x| x % 3 == 0);

    // No output yet!

    // Now it executes
    let result: Vec<_> = pipeline.collect();
    // Prints: Inspecting: 1, Inspecting: 2, Inspecting: 3, ...
}
```

---

### Iterator Chain Optimization

The Rust compiler can optimize iterator chains into efficient loops, often matching or beating hand-written code.

```rust
// This iterator chain...
fn with_iterators(numbers: &[i32]) -> Vec<i32> {
    numbers.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * 2)
        .collect()
}

// ...compiles to roughly the same efficient code as this:
fn manual_loop(numbers: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for &x in numbers {
        if x % 2 == 0 {
            result.push(x * 2);
        }
    }
    result
}
```

**Key Point**: Iterator chains are zero-cost abstractions!

---

### When to Use collect() vs fold()

- **Use `collect()`** when you need the entire result as a collection
- **Use `fold()`/`reduce()`** when you need a single aggregated value
- **Use `for_each()`** for side effects only (no return value needed)

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // collect() - builds new collection
    let doubled: Vec<_> = numbers.iter().map(|x| x * 2).collect();

    // fold() - single value
    let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);

    // for_each() - side effects only
    numbers.iter().for_each(|x| println!("{}", x));
}
```

---

### Avoiding Unnecessary Allocations

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // ❌ Less efficient - multiple allocations
    let result: Vec<_> = numbers.iter()
        .map(|x| x * 2)
        .collect::<Vec<_>>() // First allocation
        .into_iter()
        .filter(|x| x % 3 == 0)
        .collect(); // Second allocation

    // ✅ More efficient - single allocation
    let result: Vec<_> = numbers.iter()
        .map(|x| x * 2)
        .filter(|x| x % 3 == 0)
        .collect(); // One allocation
}
```

---

### Short-Circuiting Operations

Operations like `any()`, `all()`, `find()` short-circuit - they stop as soon as the result is known.

```rust
fn main() {
    let numbers = 1..=1_000_000;

    // Stops at first match (very fast)
    let has_even = numbers.clone().any(|x| x % 2 == 0);

    // Must check all elements (slower)
    let all_positive = numbers.clone().all(|x| x > 0);
}
```

---

### Choosing Between filter().map() and filter_map()

```rust
fn main() {
    let strings = vec!["1", "two", "3", "four", "5"];

    // ✅ Efficient - single pass
    let nums: Vec<_> = strings.iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();

    // ❌ Less efficient - two passes (though compiler may optimize)
    let nums: Vec<_> = strings.iter()
        .map(|s| s.parse::<i32>())
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap())
        .collect();
}
```

---

## Common Pitfalls

### Pitfall 1: Forgetting to Consume the Iterator

```rust
fn main() {
    let numbers = vec![1, 2, 3];

    // ❌ This does nothing - iterator is created but never consumed!
    numbers.iter().map(|x| x * 2);

    // ✅ Correct - consume with collect, for_each, etc.
    let doubled: Vec<_> = numbers.iter().map(|x| x * 2).collect();
}
```

---

### Pitfall 2: Moving Out of Borrowed Iterator

```rust
fn main() {
    let strings = vec!["hello".to_string(), "world".to_string()];

    // ❌ Can't move out of borrowed content
    // let owned: Vec<String> = strings.iter().collect();

    // ✅ Use cloned() to clone elements
    let owned: Vec<String> = strings.iter().cloned().collect();

    // ✅ Or use into_iter() to consume the original
    let owned: Vec<String> = strings.into_iter().collect();
}
```

---

### Pitfall 3: zip() Stops at Shortest Iterator

```rust
fn main() {
    let a = vec![1, 2, 3];
    let b = vec![10, 20]; // Shorter!

    let pairs: Vec<_> = a.iter().zip(b.iter()).collect();

    println!("{:?}", pairs); // [(1, 10), (2, 20)] - lost element 3!
}
```

**Solution**: Check lengths or use `itertools::zip_longest()`.

---

### Pitfall 4: Closure Borrowing Issues

```rust
fn main() {
    let mut sum = 0;

    let numbers = vec![1, 2, 3, 4];

    // ❌ Can't borrow sum as mutable in map
    // let doubled: Vec<_> = numbers.iter()
    //     .map(|x| { sum += x; x * 2 })
    //     .collect();

    // ✅ Use for_each for side effects
    numbers.iter().for_each(|x| sum += x);
}
```

---

### Pitfall 5: Performance of nested flat_map

```rust
fn main() {
    // ❌ Can be slow for large datasets
    let large_range = 1..1000;
    let pairs: Vec<_> = large_range.clone()
        .flat_map(|x| large_range.clone().map(move |y| (x, y)))
        .collect();

    // ✅ Better: use explicit loops for better cache locality
    let mut pairs = Vec::new();
    for x in 1..1000 {
        for y in 1..1000 {
            pairs.push((x, y));
        }
    }
}
```

---

### Pitfall 6: Infinite Iterators Without take()

```rust
fn main() {
    // ❌ This will run forever!
    // let all_numbers: Vec<_> = (1..).collect();

    // ✅ Always limit infinite iterators
    let first_100: Vec<_> = (1..).take(100).collect();

    // ✅ Or use take_while
    let until_1000: Vec<_> = (1..).take_while(|&x| x < 1000).collect();
}
```

---

### Pitfall 7: Multiple Calls to Consuming Methods

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let iter = numbers.iter();

    // ❌ Can't use iterator twice
    // let sum: i32 = iter.sum();
    // let product: i32 = iter.product(); // Error: iter already consumed!

    // ✅ Clone the iterator or recreate it
    let sum: i32 = numbers.iter().sum();
    let product: i32 = numbers.iter().product();
}
```

---

### Pitfall 8: Expecting Mutation in Immutable Iterator

```rust
fn main() {
    let mut numbers = vec![1, 2, 3, 4];

    // ❌ This doesn't mutate the original vector!
    numbers.iter().map(|x| x * 2).collect::<Vec<_>>();

    println!("{:?}", numbers); // Still [1, 2, 3, 4]

    // ✅ Use iter_mut() or reassign
    numbers = numbers.iter().map(|x| x * 2).collect();
    // Or: numbers.iter_mut().for_each(|x| *x *= 2);
}
```

---

## Summary: Benefits of Immutable Iterator Methods

* ✅ **No Side Effects**: Original collections remain unchanged
* ✅ **Composability**: Methods chain naturally for complex operations
* ✅ **Lazy Evaluation**: Computations only happen when needed (e.g., on `collect`)
* ✅ **Type Safety**: Compile-time guarantees prevent common bugs
* ✅ **Zero-Cost Abstractions**: Iterator chains optimize to efficient machine code
* ✅ **Expressiveness**: Declarative style clearly expresses intent
* ✅ **Thread Safety**: Immutable operations are inherently safe for concurrent use

---

## Quick Reference Table

| Method | Purpose | Returns | Consumes Iterator? |
|--------|---------|---------|-------------------|
| `iter()` | Create iterator | `Iterator<&T>` | ❌ |
| `map()` | Transform elements | `Iterator` | ❌ |
| `filter()` | Select by predicate | `Iterator` | ❌ |
| `fold()` | Reduce to value | `T` | ✅ |
| `reduce()` | Reduce (no initial) | `Option<T>` | ✅ |
| `scan()` | Stateful mapping | `Iterator` | ❌ |
| `take()` | Limit count | `Iterator` | ❌ |
| `skip()` | Skip first N | `Iterator` | ❌ |
| `chain()` | Concatenate | `Iterator` | ❌ |
| `zip()` | Pair elements | `Iterator` | ❌ |
| `enumerate()` | Add indices | `Iterator` | ❌ |
| `flat_map()` | Map and flatten | `Iterator` | ❌ |
| `flatten()` | Remove nesting | `Iterator` | ❌ |
| `filter_map()` | Filter + map | `Iterator` | ❌ |
| `find()` | First match | `Option<&T>` | ✅ |
| `position()` | Index of match | `Option<usize>` | ✅ |
| `any()` | Test existence | `bool` | ✅ |
| `all()` | Test all | `bool` | ✅ |
| `count()` | Count elements | `usize` | ✅ |
| `sum()` | Add all | `T` | ✅ |
| `product()` | Multiply all | `T` | ✅ |
| `min()` / `max()` | Find extreme | `Option<&T>` | ✅ |
| `partition()` | Split by predicate | `(Collection, Collection)` | ✅ |
| `cycle()` | Repeat forever | `Iterator` | ❌ |
| `rev()` | Reverse | `Iterator` | ❌ |
| `step_by()` | Every Nth | `Iterator` | ❌ |
| `collect()` | To collection | `Collection` | ✅ |
| `copied()` | `&T` → `T` (Copy) | `Iterator` | ❌ |
| `cloned()` | `&T` → `T` (Clone) | `Iterator` | ❌ |
| `peekable()` | Look ahead | `Peekable` | ❌ |
| `for_each()` | Side effects | `()` | ✅ |

