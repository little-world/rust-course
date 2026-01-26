//! Pattern 5: Variance and Subtyping
//! Example: PhantomData for Variance Control
//!
//! Run with: cargo run --example p5_phantomdata_variance

use std::cell::Cell;
use std::marker::PhantomData;

// PhantomData<T> is covariant over T
// Use it when you logically "own" T but don't store it
struct Covariant<T> {
    _marker: PhantomData<T>,
}

impl<T> Covariant<T> {
    fn new() -> Self {
        Covariant {
            _marker: PhantomData,
        }
    }
}

// PhantomData<Cell<T>> is invariant over T
// Use when you need invariance without storing T
struct Invariant<T> {
    _marker: PhantomData<Cell<T>>,
}

impl<T> Invariant<T> {
    fn new() -> Self {
        Invariant {
            _marker: PhantomData,
        }
    }
}

// PhantomData<fn(T)> is contravariant over T
// Rare, but useful for certain unsafe abstractions
struct Contravariant<T> {
    _marker: PhantomData<fn(T)>,
}

impl<T> Contravariant<T> {
    fn new() -> Self {
        Contravariant {
            _marker: PhantomData,
        }
    }
}

// Practical example: A raw pointer wrapper with correct variance
struct RawPtr<'a, T> {
    ptr: *const T,
    _lifetime: PhantomData<&'a T>, // Covariant over 'a
}

impl<'a, T> RawPtr<'a, T> {
    fn new(reference: &'a T) -> Self {
        RawPtr {
            ptr: reference as *const T,
            _lifetime: PhantomData,
        }
    }

    fn get(&self) -> Option<&'a T> {
        // Safe because we track the lifetime with PhantomData
        if self.ptr.is_null() {
            None
        } else {
            Some(unsafe { &*self.ptr })
        }
    }
}

// Invariant raw pointer wrapper (for mutable access)
struct RawMutPtr<'a, T> {
    ptr: *mut T,
    _lifetime: PhantomData<&'a mut T>, // Invariant over 'a
}

impl<'a, T> RawMutPtr<'a, T> {
    fn new(reference: &'a mut T) -> Self {
        RawMutPtr {
            ptr: reference as *mut T,
            _lifetime: PhantomData,
        }
    }

    fn get_mut(&mut self) -> Option<&'a mut T> {
        if self.ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.ptr })
        }
    }
}

// Type-level tag with phantom data
struct Tagged<T, Tag> {
    value: T,
    _tag: PhantomData<Tag>,
}

struct Meters;
struct Feet;

impl<T, Tag> Tagged<T, Tag> {
    fn new(value: T) -> Self {
        Tagged {
            value,
            _tag: PhantomData,
        }
    }

    fn value(&self) -> &T {
        &self.value
    }
}

// Lifetime-parameterized phantom type
struct Handle<'a, T> {
    id: usize,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> Handle<'a, T> {
    fn new(id: usize) -> Self {
        Handle {
            id,
            _lifetime: PhantomData,
        }
    }

    fn id(&self) -> usize {
        self.id
    }
}

fn main() {
    println!("=== PhantomData for Variance Control ===\n");

    println!("=== Covariant PhantomData<T> ===");
    // Usage: PhantomData controls variance without storing T.
    let cov: Covariant<&'static str> = Covariant::new();
    // Covariant allows subtyping
    let _cov2: Covariant<&str> = cov;
    println!("Covariant<&'static str> can be used as Covariant<&str>");

    println!("\n=== Invariant PhantomData<Cell<T>> ===");
    let _inv: Invariant<i32> = Invariant::new();
    // Cannot substitute different types due to invariance
    println!("Invariant<T> prevents type substitution");

    println!("\n=== Contravariant PhantomData<fn(T)> ===");
    let _contra: Contravariant<&str> = Contravariant::new();
    // Contravariance is rare but exists
    println!("Contravariant types reverse the subtyping direction");

    println!("\n=== RawPtr with Lifetime Tracking ===");
    let data = 42;
    let ptr = RawPtr::new(&data);
    println!("RawPtr value: {:?}", ptr.get());

    // The phantom lifetime ensures we can't use the pointer after data is dropped
    // This is covariant, so a pointer with longer lifetime can be used where shorter expected

    println!("\n=== Tagged Values with Phantom Types ===");
    let meters: Tagged<f64, Meters> = Tagged::new(100.0);
    let feet: Tagged<f64, Feet> = Tagged::new(328.084);

    println!("Distance in meters: {}", meters.value());
    println!("Distance in feet: {}", feet.value());

    // These are different types, can't be mixed!
    // let wrong: Tagged<f64, Meters> = feet; // ERROR: type mismatch
    println!("Tagged<f64, Meters> and Tagged<f64, Feet> are distinct types");

    println!("\n=== Handle with Lifetime ===");
    let handle: Handle<i32> = Handle::new(42);
    println!("Handle ID: {}", handle.id());

    println!("\n=== PhantomData Patterns ===");
    println!("PhantomData<T>           -> covariant over T");
    println!("PhantomData<&'a T>       -> covariant over 'a");
    println!("PhantomData<&'a mut T>   -> invariant over 'a");
    println!("PhantomData<Cell<T>>     -> invariant over T");
    println!("PhantomData<fn(T)>       -> contravariant over T");
    println!("PhantomData<fn() -> T>   -> covariant over T");

    println!("\n=== When to Use PhantomData ===");
    println!("1. Raw pointers that should track lifetimes");
    println!("2. Type-level tags (units, states, markers)");
    println!("3. FFI types that logically own data");
    println!("4. Controlling variance for unsafe abstractions");
    println!("5. Drop check hints (PhantomData<T> vs PhantomData<*const T>)");

    println!("\n=== Zero Runtime Cost ===");
    println!("PhantomData is zero-sized - no memory overhead!");
    println!("size_of::<PhantomData<String>>() = {}", std::mem::size_of::<PhantomData<String>>());
    println!("size_of::<Covariant<String>>() = {}", std::mem::size_of::<Covariant<String>>());
}
