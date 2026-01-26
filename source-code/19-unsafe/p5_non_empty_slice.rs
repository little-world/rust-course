// Pattern 5: Invariants and Documentation

/// A slice type that is guaranteed to be non-empty.
///
/// # Safety Invariants
/// - The inner slice must always have at least one element
/// - The pointer must be valid and properly aligned
/// - The data must be valid for lifetime 'a
pub struct NonEmptySlice<'a, T> {
    slice: &'a [T],
}

impl<'a, T> NonEmptySlice<'a, T> {
    // Creates a non-empty slice from a regular slice.
    // Returns None if the slice is empty.
    pub fn new(slice: &'a [T]) -> Option<Self> {
        if slice.is_empty() {
            None
        } else {
            Some(NonEmptySlice { slice })
        }
    }

    // Creates a non-empty slice without checking.
    // # Safety: The caller must ensure that the slice is not empty.
    pub unsafe fn new_unchecked(slice: &'a [T]) -> Self {
        debug_assert!(!slice.is_empty());
        NonEmptySlice { slice }
    }

    /// Returns the first element (always exists).
    pub fn first(&self) -> &T {
        &self.slice[0]
    }

    /// Returns the last element (always exists).
    pub fn last(&self) -> &T {
        &self.slice[self.slice.len() - 1]
    }

    pub fn as_slice(&self) -> &[T] {
        self.slice
    }

    pub fn len(&self) -> usize {
        self.slice.len()
    }
}

fn main() {
    // Usage: Create non-empty slice safely
    let data = [1, 2, 3, 4, 5];
    let s = NonEmptySlice::new(&data).unwrap();

    println!("First element: {}", s.first());
    println!("Last element: {}", s.last());
    println!("Length: {}", s.len());
    println!("As slice: {:?}", s.as_slice());

    // Trying with empty slice returns None
    let empty: [i32; 0] = [];
    let result = NonEmptySlice::new(&empty);
    println!("Empty slice result: {:?}", result.is_none());

    println!("NonEmptySlice example completed");
}
