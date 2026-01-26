// Pattern 4: PhantomData for Variance Control
use std::marker::PhantomData;

// Covariant in T (default, like storing T)
struct Covariant<T>(PhantomData<T>);

// Contravariant in T (like taking T as input)
struct Contravariant<T>(PhantomData<fn(T)>);

// Invariant in T (like storing &mut T)
struct Invariant<T>(PhantomData<fn(T) -> T>);

// Practical example: a handle that conceptually "owns" T
struct Handle<T> {
    id: u64,
    _marker: PhantomData<T>,  // Covariant: Handle<Cat> can be Handle<Animal>
}

// A handle that can produce T values
struct Producer<T> {
    id: u64,
    _marker: PhantomData<fn() -> T>,  // Covariant in T
}

// A handle that consumes T values
struct Consumer<T> {
    id: u64,
    _marker: PhantomData<fn(T)>,  // Contravariant in T
}

fn main() {
    let _cov: Covariant<i32> = Covariant(PhantomData);
    let _contra: Contravariant<i32> = Contravariant(PhantomData);
    let _inv: Invariant<i32> = Invariant(PhantomData);

    let _handle: Handle<String> = Handle { id: 1, _marker: PhantomData };
    let _producer: Producer<i32> = Producer { id: 2, _marker: PhantomData };
    let _consumer: Consumer<i32> = Consumer { id: 3, _marker: PhantomData };

    println!("PhantomData variance example completed");
}
