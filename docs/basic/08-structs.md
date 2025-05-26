

## Struct

A `struct` in Rust is a **custom data type** that lets you group multiple related values.


### Defining a Struct

```rust
struct Person {
    name: String,
    age: u8,
}

fn main() {
    let user = Person {
        name: String::from("Alice"),
        age: 30,
    };

    println!("{} is {} years old.", user.name, user.age);
}
```

## Mutable Structs

To modify fields, both the instance and fields must be mutable:

```rust
fn main() {
    let mut user = Person {
        name: String::from("Bob"),
        age: 25,
    };

    user.age += 1;
    println!("Happy Birthday, {}!", user.name);
}
```


### Update Struct

Copy remaining fields from another instance:

```rust
let user2 = Person {
    name: String::from("Carol"),
    ..user // copies `age` from `user`
};
```


## Tuple Structs

Structs without named fields:

```rust
struct Color(u8, u8, u8);

let red = Color(255, 0, 0);
println!("Red: {}, {}, {}", red.0, red.1, red.2);
```



### Unit-like Structs

No data — useful for traits or markers.

```rust
struct Marker;
```


## Methods on Structs

Use `impl` to define methods.

```rust
impl Person {
    fn greet(&self) {
        println!("Hi, my name is {}!", self.name);
    }

    fn is_adult(&self) -> bool {
        self.age >= 18
    }
}

fn main() {
    let user = Person {
        name: String::from("Dana"),
        age: 20,
    };

    user.greet();
    println!("Is adult? {}", user.is_adult());
}
```

* `&self` = borrow the struct instance (like `this` in other languages).
* `&mut self` = mutable borrow.
* `self` = takes ownership.

---

## Static 
### Asssociated Functions 
No `self` – used like static functions:

```rust
impl Person {
    fn new(name: String, age: u8) -> Self {
        Self { name, age }
    }
}

fn main() {
    let p = Person::new(String::from("Eve"), 22);
    println!("{} is {}.", p.name, p.age);
}
```


## Summary

| Feature             | Syntax                             |
| ------------------- | ---------------------------------- |
| Define struct       | `struct Name { field: Type }`      |
| Create instance     | `let x = Struct { field: value }`  |
| Access field        | `x.field`                          |
| Add methods         | `impl Struct { fn method(&self) }` |
| Associated function | `fn new(...) -> Self`              |
| Update from another | `Struct { field, ..other }`        |