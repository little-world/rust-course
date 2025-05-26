

## Iterator 

An **iterator** lets you **loop over** items one by one. It's lazy — values are only computed as needed.

### Basic Example:

```rust
fn main() {
    let nums = vec![1, 2, 3];

    for n in nums.iter() {
        println!("{}", n);
    }
}
```

* `iter()` → immutable iterator
* `iter_mut()` → mutable iterator
* `into_iter()` → takes ownership



## Adapters 
### (Combinators)

These are methods you chain **on iterators** to transform data.

### map Transform Each Item

```rust
let nums = vec![1, 2, 3];
let squared: Vec<i32> = nums.iter().map(|x| x * x).collect();

println!("{:?}", squared); // [1, 4, 9]
```

### filter Keep Only Matching Items

```rust
let nums = vec![1, 2, 3, 4];
let even: Vec<_> = nums.iter().filter(|x| *x % 2 == 0).collect();

println!("{:?}", even); // [2, 4]
```

### sum, product Calculate Total

```rust
let total: i32 = (1..=5).sum(); // 15
let prod: i32 = (1..=4).product(); // 24
```

### chain Join Iterators

```rust
let a = vec![1, 2];
let b = vec![3, 4];
let all: Vec<_> = a.iter().chain(b.iter()).collect();
```

### find Return First Match

```rust
let found = (1..10).find(|&x| x % 7 == 0);
println!("{:?}", found); // Some(7)
```


## Collecting Results

Use `.collect()` to build collections like:

| Type      | Example Syntax                |
| --------- | ----------------------------- |
| `Vec<T>`  | `.collect::<Vec<_>>()`        |
| `HashSet` | `.collect::<HashSet<_>>()`    |
| `HashMap` | `.collect::<HashMap<K, V>>()` |

```rust
let doubled: Vec<_> = (1..5).map(|x| x * 2).collect();
```


## Chaining Iterators

```rust
let data = vec![Some(1), None, Some(3)];

let doubled: Vec<_> = data
    .into_iter()
    .filter_map(|x| x)       // remove None
    .map(|x| x * 2)          // double each
    .collect();

println!("{:?}", doubled); // [2, 6]
```


## Summary Table

| Adapter      | Description                   | Example                   | 
| ------------ | ----------------------------- |---------------------------| 
| `map`        | Transform each item           | `.map(\| x \| x + 1)`     |
| `filter`     | Keep matching items           | `.filter(\| x \| \*x > 5)` |
| `find`       | First item matching condition | `.find(\| x \| x == 3)`   |
| `sum`        | Add up all items              | `.sum()`                  | 
| `collect`    | Turn iterator into collection | `.collect()`              | 
| `filter_map` | Filter and unwrap in one step | `.filter\_map(\| x \| x)` |
| `enumerate`  | Get index and item            | `.enumerate()`            | 
| `zip`        | Combine two iterators         | `.zip(other_iter)`        | 