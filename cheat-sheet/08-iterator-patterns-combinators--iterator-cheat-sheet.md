### Iterator Cheat Sheet

```rust
// Core iterator traits
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // 70+ provided methods built on next()
}

trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}

// Common iterator methods
iter.map(|x| x * 2)              // Transform each element
iter.filter(|x| *x > 0)          // Keep only matching elements
iter.fold(0, |acc, x| acc + x)   // Reduce to single value
iter.collect::<Vec<_>>()         // Consume into collection
iter.take(5)                     // Limit to first n elements
iter.skip(3)                     // Skip first n elements
iter.chain(other)                // Concatenate iterators
iter.zip(other)                  // Pair elements from two iterators
iter.enumerate()                 // Add indices
iter.flat_map(|x| vec![x, x])    // Map and flatten

// Iterator consumers (methods that consume the iterator)
iter.count()                     // Count elements
iter.sum()                       // Sum numeric elements
iter.any(|x| x > 5)             // Check if any match
iter.all(|x| x > 0)             // Check if all match
iter.find(|x| *x == target)     // Find first match
```

