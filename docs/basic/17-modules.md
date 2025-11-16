## Modules

**Modules** let you organize code into **namespaces** and control **visibility**. They help structure large programs and prevent naming conflicts.

### Basic Module Declaration

```rust
mod my_module {
    fn private_function() {
        println!("This is private");
    }
    
    pub fn public_function() {
        println!("This is public");
    }
}

fn main() {
    // my_module::private_function(); // Error: private!
    my_module::public_function(); // Works!
}
```

> Use `pub` to make items **public** (accessible from outside the module).

---

## Module Paths

### Absolute Paths with `crate::`

Use `crate::` to start from the **crate root**:

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {
            println!("Adding to waitlist");
        }
    }
}

mod back_of_house {
    fn cook() {
        // Absolute path from crate root
        crate::front_of_house::hosting::add_to_waitlist();
    }
}
```

### Relative Paths with `self::`

Use `self::` for the **current module**:

```rust
mod utils {
    pub fn helper() {
        println!("Helper function");
    }
    
    pub fn main_util() {
        // Relative path within current module
        self::helper();
        // Or just call directly:
        helper();
    }
}
```

### Parent Module with `super::`

Use `super::` to access the **parent module**:

```rust
fn serve_order() {
    println!("Serving order");
}

mod back_of_house {
    fn cook() {
        // Call function in parent module
        super::serve_order();
    }
    
    pub fn prepare_meal() {
        cook();
        super::serve_order();
    }
}
```

---

## File Organization

### Single File Modules

```rust
// In main.rs or lib.rs
mod network {
    pub mod client {
        pub fn connect() {
            println!("Connecting...");
        }
    }
    
    pub mod server {
        pub fn run() {
            println!("Server running...");
        }
    }
}

fn main() {
    network::client::connect();
    network::server::run();
}
```

### Separate Files

**File structure:**
```
src/
├── main.rs
├── network.rs
└── network/
    ├── client.rs
    └── server.rs
```

**main.rs:**
```rust
mod network;

fn main() {
    network::client::connect();
    network::server::run();
}
```

**network.rs:**
```rust
pub mod client;
pub mod server;
```

**network/client.rs:**
```rust
pub fn connect() {
    println!("Connecting...");
}
```

**network/server.rs:**
```rust
pub fn run() {
    println!("Server running...");
}
```

---

## Visibility Rules

### Private by Default

```rust
mod my_module {
    fn private_fn() {}        // Private
    pub fn public_fn() {}     // Public
    
    struct PrivateStruct {    // Private
        field: i32,
    }
    
    pub struct PublicStruct { // Public struct
        pub public_field: i32,    // Public field
        private_field: i32,       // Private field
    }
}
```

### Making Modules Public

```rust
pub mod my_public_module {
    pub fn accessible_function() {
        println!("Can be called from outside!");
    }
}

// In another file:
// my_public_module::accessible_function();
```

---

## Practical Examples

### Restaurant Example

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
        pub fn seat_at_table() {}
    }
    
    pub mod serving {
        pub fn take_order() {}
        pub fn serve_order() {}
        
        fn take_payment() {} // Private
    }
}

pub fn eat_at_restaurant() {
    // Absolute path
    crate::front_of_house::hosting::add_to_waitlist();
    
    // Relative path
    front_of_house::hosting::add_to_waitlist();
    
    front_of_house::serving::take_order();
    front_of_house::serving::serve_order();
    
    // front_of_house::serving::take_payment(); // Error: private!
}
```

### Use Declarations

Bring paths into scope with `use`:

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

// Bring the module into scope
use crate::front_of_house::hosting;

// Or bring the function directly
use crate::front_of_house::hosting::add_to_waitlist;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
    // Or:
    add_to_waitlist();
}
```

### Re-exports with `pub use`

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

// Re-export for external use
pub use crate::front_of_house::hosting;

// Now external crates can use:
// my_crate::hosting::add_to_waitlist();
```

---

## Module Patterns

### Nested Modules

```rust
mod outer {
    pub mod inner {
        pub fn function() {
            println!("Inner function");
        }
        
        pub mod deeply_nested {
            pub fn deep_function() {
                // Access outer modules
                super::super::outer_function();
                // Or use crate:: for absolute path
                crate::outer::outer_function();
            }
        }
    }
    
    fn outer_function() {
        println!("Outer function");
    }
}
```

### Tests Module

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from parent module
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
```

---

## Summary

| Concept              | Syntax                           | Usage                              |
| -------------------- | -------------------------------- | ---------------------------------- |
| Module declaration   | `mod name { }`                   | Define a module                    |
| Public item          | `pub fn name() { }`              | Make function/struct/mod public    |
| Absolute path        | `crate::module::function()`      | Path from crate root               |
| Relative path        | `self::function()`               | Path within current module         |
| Parent module        | `super::function()`              | Path to parent module              |
| Bring into scope     | `use crate::module::function`    | Import for easier access           |
| Re-export            | `pub use crate::module`          | Make internal module public        |
| External file        | `mod module_name;`               | Load module from file              |
| Nested access        | `module::submodule::function()`  | Access nested module items         |