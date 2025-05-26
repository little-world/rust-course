

## If 

Rust uses familiar syntax but with a strict type rule: **conditions must be boolean** (no implicit casting like in C/C++).

```rust
fn main() {
    let number = 7;

    if number < 10 {
        println!("Less than 10");
    } else if number == 10 {
        println!("Exactly 10");
    } else {
        println!("Greater than 10");
    }

    // `if` is also an expression:
    let result = if number % 2 == 0 { "even" } else { "odd" };
    println!("Number is {}", result);
}
```



## match 

Rust’s `match` is like a `switch` on steroids. It's **exhaustive**: all possible cases must be covered.

```rust
fn main() {
    let number = 3;

    match number {
        1 => println!("One"),
        2 | 3 | 5 => println!("Prime"),
        4 => println!("Four"),
        _ => println!("Something else"), // _ is a catch-all
    }

    // Match is also an expression
    let grade = 'A';
    let result = match grade {
        'A' => "Excellent",
        'B' => "Good",
        'C' => "Average",
        _ => "Unknown",
    };
    println!("Grade is: {}", result);
}
```


## while loop

Repeats as long as the condition is `true`.

```rust
fn main() {
    let mut counter = 0;

    while counter < 5 {
        println!("Counter: {}", counter);
        counter += 1;
    }
}
```



## infinite loop 

Runs forever unless you explicitly break out.

```rust
fn main() {
    let mut count = 0;

    loop {
        count += 1;
        println!("Count: {}", count);

        if count == 3 {
            break; // exit the loop
        }
    }
}
```

You can also **return a value** from a loop:


```rust
fn main() {
    let result = loop {
        let x = 5;
        if x == 5 {
            break x * 2; // returns 10
        }
    };
    println!("Result from loop: {}", result);
}
```


## for loop 

Used to iterate over collections or ranges.

```rust
fn main() {
    for i in 1..4 {
        println!("i: {}", i); // Prints 1, 2, 3 (range is exclusive at the end)
    }

    let arr = [10, 20, 30];
    for item in arr.iter() {
        println!("Item: {}", item);
    }
}
```


## Summary

| Keyword       | Description                          |
| ------------- | ------------------------------------ |
| `if` / `else` | Conditional branching                |
| `match`       | Exhaustive pattern matching          |
| `while`       | Loop with condition                  |
| `loop`        | Infinite loop with optional `break`  |
| `for`         | Iteration over a range or collection |