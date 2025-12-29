// Complete Generic Data Structures with Const Generics
// Implements all 5 milestones from the project specification

use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

// ============================================================================
// Milestone 1: Generic Fixed-Size Stack with Const Generics
// ============================================================================

pub struct Stack<T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        Self {
            storage: unsafe {
                // MaybeUninit doesn't require initialization
                MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init()
            },
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }
        self.storage[self.len].write(value);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(unsafe { self.storage[self.len].assume_init_read() })
    }

    pub fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        Some(unsafe { self.storage[self.len - 1].assume_init_ref() })
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            return None;
        }
        Some(unsafe { self.storage[self.len - 1].assume_init_mut() })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }

    pub fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Drop for Stack<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                self.storage[i].assume_init_drop();
            }
        }
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for Stack<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries((0..self.len).map(|i| unsafe { self.storage[i].assume_init_ref() }))
            .finish()
    }
}

// ============================================================================
// Milestone 2: Generic Ring Buffer with Circular Queuing
// ============================================================================

pub struct RingBuffer<T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            storage: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            // Overwrite oldest element (circular behavior)
            unsafe {
                self.storage[self.tail].assume_init_drop();
            }
            self.storage[self.tail].write(value);
            self.tail = (self.tail + 1) % N;
            self.head = (self.head + 1) % N;
            Ok(())
        } else {
            self.storage[self.tail].write(value);
            self.tail = (self.tail + 1) % N;
            self.len += 1;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = unsafe { self.storage[self.head].assume_init_read() };
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(value)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }

    pub fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

// Iterator support
pub struct RingBufferIter<T, const N: usize> {
    buffer: RingBuffer<T, N>,
}

impl<T, const N: usize> Iterator for RingBufferIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.buffer.len();
        (len, Some(len))
    }
}

impl<T, const N: usize> IntoIterator for RingBuffer<T, N> {
    type Item = T;
    type IntoIter = RingBufferIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIter { buffer: self }
    }
}

// ============================================================================
// Milestone 3: Generic Binary Heap with Ordering
// ============================================================================

pub struct BinaryHeap<T: Ord, const N: usize> {
    storage: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Ord, const N: usize> BinaryHeap<T, N> {
    pub fn new() -> Self {
        Self {
            storage: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }

        self.storage[self.len].write(value);
        self.heapify_up(self.len);
        self.len += 1;
        Ok(())
    }

    fn heapify_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;

            let child_ref = unsafe { self.storage[index].assume_init_ref() };
            let parent_ref = unsafe { self.storage[parent].assume_init_ref() };

            // Max-heap: child > parent
            if child_ref > parent_ref {
                self.storage.swap(index, parent);
                index = parent;
            } else {
                break;
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        self.storage.swap(0, self.len);
        let value = unsafe { self.storage[self.len].assume_init_read() };

        if self.len > 0 {
            self.heapify_down(0);
        }

        Some(value)
    }

    fn heapify_down(&mut self, mut index: usize) {
        loop {
            let left = 2 * index + 1;
            let right = 2 * index + 2;
            let mut largest = index;

            if left < self.len {
                let left_ref = unsafe { self.storage[left].assume_init_ref() };
                let largest_ref = unsafe { self.storage[largest].assume_init_ref() };
                if left_ref > largest_ref {
                    largest = left;
                }
            }

            if right < self.len {
                let right_ref = unsafe { self.storage[right].assume_init_ref() };
                let largest_ref = unsafe { self.storage[largest].assume_init_ref() };
                if right_ref > largest_ref {
                    largest = right;
                }
            }

            if largest != index {
                self.storage.swap(index, largest);
                index = largest;
            } else {
                break;
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.len > 0 {
            Some(unsafe { self.storage[0].assume_init_ref() })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }
}

impl<T: Ord, const N: usize> Drop for BinaryHeap<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                self.storage[i].assume_init_drop();
            }
        }
    }
}

// ============================================================================
// Milestone 4: Generic Container Trait with Associated Types
// ============================================================================

pub trait Container<T> {
    type Iter<'a>: Iterator<Item = &'a T>
    where
        T: 'a,
        Self: 'a;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn iter(&self) -> Self::Iter<'_>;
    fn clear(&mut self);
}

// Implement for Stack
impl<T, const N: usize> Container<T> for Stack<T, N> {
    type Iter<'a>
        = StackIter<'a, T, N>
    where
        T: 'a;

    fn len(&self) -> usize {
        self.len
    }

    fn iter(&self) -> Self::Iter<'_> {
        StackIter {
            stack: self,
            index: 0,
        }
    }

    fn clear(&mut self) {
        while self.pop().is_some() {}
    }
}

pub struct StackIter<'a, T, const N: usize> {
    stack: &'a Stack<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for StackIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.stack.len {
            let item = unsafe { self.stack.storage[self.index].assume_init_ref() };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// Generic function using Container trait
pub fn print_all<T, C>(container: &C)
where
    T: Display,
    C: Container<T>,
{
    for item in container.iter() {
        println!("{}", item);
    }
}

// Generic function to count elements matching predicate
pub fn count_matching<T, C, F>(container: &C, predicate: F) -> usize
where
    C: Container<T>,
    F: Fn(&T) -> bool,
{
    container.iter().filter(|item| predicate(item)).count()
}

// ============================================================================
// Milestone 5: Builder Pattern with Generic Constraints
// ============================================================================

// State marker types (ZSTs)
pub struct Empty;
pub struct Configured;
pub struct Ready;

pub struct ContainerBuilder<T, const N: usize, State> {
    config: Option<T>,
    _state: PhantomData<State>,
}

impl<T, const N: usize> ContainerBuilder<T, N, Empty> {
    pub fn new() -> Self {
        Self {
            config: None,
            _state: PhantomData,
        }
    }

    pub fn with_default(self, value: T) -> ContainerBuilder<T, N, Configured>
    where
        T: Clone,
    {
        ContainerBuilder {
            config: Some(value),
            _state: PhantomData,
        }
    }

    pub fn with_defaults(self) -> ContainerBuilder<T, N, Ready>
    where
        T: Default,
    {
        ContainerBuilder {
            config: None,
            _state: PhantomData,
        }
    }
}

impl<T, const N: usize> ContainerBuilder<T, N, Configured> {
    pub fn ready(self) -> ContainerBuilder<T, N, Ready> {
        ContainerBuilder {
            config: self.config,
            _state: PhantomData,
        }
    }
}

impl<T, const N: usize> ContainerBuilder<T, N, Ready> {
    pub fn build(self) -> Stack<T, N>
    where
        T: Clone,
    {
        let mut stack = Stack::new();

        if let Some(default) = self.config {
            for _ in 0..N {
                if stack.push(default.clone()).is_err() {
                    break;
                }
            }
        }

        stack
    }

    pub fn build_with<F>(self, mut init: F) -> Stack<T, N>
    where
        F: FnMut(usize) -> T,
    {
        let mut stack = Stack::new();

        for i in 0..N {
            let value = init(i);
            if stack.push(value).is_err() {
                break;
            }
        }

        stack
    }
}

// ============================================================================
// Main Function - Demonstrates All Milestones
// ============================================================================

fn main() {
    println!("=== Generic Data Structures with Const Generics ===\n");

    // Milestone 1: Stack
    println!("--- Milestone 1: Generic Fixed-Size Stack ---");
    let mut stack: Stack<i32, 5> = Stack::new();
    stack.push(1).unwrap();
    stack.push(2).unwrap();
    stack.push(3).unwrap();
    println!("Stack: {:?}", stack);
    println!("Peek: {:?}", stack.peek());
    println!("Pop: {:?}", stack.pop());
    println!("After pop: {:?}", stack);

    // Different types
    let mut str_stack: Stack<String, 3> = Stack::new();
    str_stack.push("hello".to_string()).unwrap();
    str_stack.push("world".to_string()).unwrap();
    println!("String stack: {:?}", str_stack);

    // Milestone 2: Ring Buffer
    println!("\n--- Milestone 2: Generic Ring Buffer ---");
    let mut ring: RingBuffer<i32, 4> = RingBuffer::new();
    ring.push(1).unwrap();
    ring.push(2).unwrap();
    ring.push(3).unwrap();
    println!("Ring buffer length: {}", ring.len());
    println!("Pop: {:?}", ring.pop());
    ring.push(4).unwrap();
    ring.push(5).unwrap();
    println!("After wraparound:");
    for val in ring.into_iter() {
        println!("  {}", val);
    }

    // Milestone 3: Binary Heap
    println!("\n--- Milestone 3: Generic Binary Heap ---");
    let mut heap: BinaryHeap<i32, 10> = BinaryHeap::new();
    heap.push(5).unwrap();
    heap.push(3).unwrap();
    heap.push(7).unwrap();
    heap.push(1).unwrap();
    heap.push(9).unwrap();
    println!("Heap peek (max): {:?}", heap.peek());
    println!("Popping in descending order:");
    while let Some(val) = heap.pop() {
        println!("  {}", val);
    }

    // Milestone 4: Container Trait
    println!("\n--- Milestone 4: Generic Container Trait ---");
    let mut stack2: Stack<i32, 10> = Stack::new();
    for i in 1..=5 {
        stack2.push(i).unwrap();
    }
    println!("Stack via Container trait:");
    println!("  Length: {}", stack2.len());
    println!("  Is empty: {}", stack2.is_empty());
    println!("  Elements:");
    for val in stack2.iter() {
        println!("    {}", val);
    }

    let even_count = count_matching(&stack2, |&x| x % 2 == 0);
    println!("  Even numbers count: {}", even_count);

    // Milestone 5: Builder Pattern
    println!("\n--- Milestone 5: Builder Pattern ---");
    let built_stack: Stack<i32, 5> = ContainerBuilder::new().with_default(42).ready().build();
    println!("Built stack with default (42): {:?}", built_stack);

    let custom_stack: Stack<i32, 5> = ContainerBuilder::new()
        .with_defaults()
        .build_with(|i| (i * 2) as i32);
    println!("Built stack with custom init: {:?}", custom_stack);

    println!("\n=== All Milestones Complete! ===");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Milestone 1 Tests
    #[test]
    fn test_new_stack_is_empty() {
        let stack: Stack<i32, 10> = Stack::new();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
        assert_eq!(stack.capacity(), 10);
    }

    #[test]
    fn test_push_and_pop() {
        let mut stack: Stack<i32, 5> = Stack::new();

        assert_eq!(stack.push(1), Ok(()));
        assert_eq!(stack.push(2), Ok(()));
        assert_eq!(stack.push(3), Ok(()));
        assert_eq!(stack.len(), 3);

        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_push_when_full() {
        let mut stack: Stack<i32, 3> = Stack::new();

        assert_eq!(stack.push(1), Ok(()));
        assert_eq!(stack.push(2), Ok(()));
        assert_eq!(stack.push(3), Ok(()));
        assert!(stack.is_full());

        assert_eq!(stack.push(4), Err(4));
    }

    #[test]
    fn test_peek() {
        let mut stack: Stack<String, 5> = Stack::new();

        assert_eq!(stack.peek(), None);

        stack.push("hello".to_string()).unwrap();
        stack.push("world".to_string()).unwrap();

        assert_eq!(stack.peek(), Some(&"world".to_string()));
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn test_drop_calls_destructor() {
        use std::sync::Arc;

        let value = Arc::new(42);
        assert_eq!(Arc::strong_count(&value), 1);

        {
            let mut stack: Stack<Arc<i32>, 5> = Stack::new();
            stack.push(Arc::clone(&value)).unwrap();
            stack.push(Arc::clone(&value)).unwrap();
            assert_eq!(Arc::strong_count(&value), 3);
        }

        assert_eq!(Arc::strong_count(&value), 1);
    }

    #[test]
    fn test_different_types() {
        let mut stack_i32: Stack<i32, 10> = Stack::new();
        stack_i32.push(42).unwrap();
        assert_eq!(stack_i32.pop(), Some(42));

        let mut stack_str: Stack<String, 10> = Stack::new();
        stack_str.push("test".to_string()).unwrap();
        assert_eq!(stack_str.pop(), Some("test".to_string()));

        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        let mut stack_point: Stack<Point, 10> = Stack::new();
        stack_point.push(Point { x: 1, y: 2 }).unwrap();
        assert_eq!(stack_point.pop(), Some(Point { x: 1, y: 2 }));
    }

    // Milestone 2 Tests
    #[test]
    fn test_ring_buffer_push_pop() {
        let mut buf: RingBuffer<i32, 4> = RingBuffer::new();

        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.push(3).unwrap();

        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), Some(2));

        buf.push(4).unwrap();
        buf.push(5).unwrap();

        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
        assert_eq!(buf.pop(), Some(5));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let mut buf: RingBuffer<i32, 3> = RingBuffer::new();

        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.push(3).unwrap();

        buf.push(4).unwrap();
        buf.push(5).unwrap();

        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
        assert_eq!(buf.pop(), Some(5));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_ring_buffer_iterator() {
        let mut buf: RingBuffer<i32, 5> = RingBuffer::new();

        buf.push(10).unwrap();
        buf.push(20).unwrap();
        buf.push(30).unwrap();

        let collected: Vec<i32> = buf.into_iter().collect();
        assert_eq!(collected, vec![10, 20, 30]);
    }

    #[test]
    fn test_ring_buffer_fifo_order() {
        let mut buf: RingBuffer<char, 4> = RingBuffer::new();

        buf.push('a').unwrap();
        buf.push('b').unwrap();
        buf.push('c').unwrap();

        assert_eq!(buf.pop(), Some('a'));

        buf.push('d').unwrap();
        buf.push('e').unwrap();

        assert_eq!(buf.pop(), Some('b'));
        assert_eq!(buf.pop(), Some('c'));
        assert_eq!(buf.pop(), Some('d'));
    }

    #[test]
    fn test_ring_buffer_size_hint() {
        let mut buf: RingBuffer<i32, 5> = RingBuffer::new();

        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.push(3).unwrap();

        let iter = buf.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    // Milestone 3 Tests
    #[test]
    fn test_heap_push_pop_order() {
        let mut heap: BinaryHeap<i32, 10> = BinaryHeap::new();

        heap.push(5).unwrap();
        heap.push(3).unwrap();
        heap.push(7).unwrap();
        heap.push(1).unwrap();
        heap.push(9).unwrap();

        assert_eq!(heap.pop(), Some(9));
        assert_eq!(heap.pop(), Some(7));
        assert_eq!(heap.pop(), Some(5));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn test_heap_peek() {
        let mut heap: BinaryHeap<i32, 5> = BinaryHeap::new();

        assert_eq!(heap.peek(), None);

        heap.push(10).unwrap();
        heap.push(20).unwrap();
        heap.push(5).unwrap();

        assert_eq!(heap.peek(), Some(&20));
        assert_eq!(heap.len(), 3);
    }

    #[test]
    fn test_heap_with_strings() {
        let mut heap: BinaryHeap<String, 5> = BinaryHeap::new();

        heap.push("apple".to_string()).unwrap();
        heap.push("zebra".to_string()).unwrap();
        heap.push("banana".to_string()).unwrap();

        assert_eq!(heap.pop(), Some("zebra".to_string()));
        assert_eq!(heap.pop(), Some("banana".to_string()));
        assert_eq!(heap.pop(), Some("apple".to_string()));
    }

    #[test]
    fn test_heap_capacity() {
        let mut heap: BinaryHeap<i32, 3> = BinaryHeap::new();

        assert_eq!(heap.push(1), Ok(()));
        assert_eq!(heap.push(2), Ok(()));
        assert_eq!(heap.push(3), Ok(()));

        assert_eq!(heap.push(4), Err(4));
    }

    #[test]
    fn test_heap_property_maintained() {
        let mut heap: BinaryHeap<i32, 10> = BinaryHeap::new();

        for i in 0..10 {
            heap.push(i).unwrap();
        }

        let mut prev = heap.pop().unwrap();
        while let Some(current) = heap.pop() {
            assert!(prev >= current, "Heap property violated");
            prev = current;
        }
    }

    // Milestone 4 Tests
    #[test]
    fn test_stack_container_trait() {
        let mut stack: Stack<i32, 10> = Stack::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();
        stack.push(3).unwrap();

        assert_eq!(stack.len(), 3);
        assert!(!stack.is_empty());

        let collected: Vec<&i32> = stack.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }

    #[test]
    fn test_container_clear() {
        let mut stack: Stack<i32, 5> = Stack::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();

        stack.clear();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_count_matching() {
        let mut stack: Stack<i32, 10> = Stack::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();
        stack.push(3).unwrap();
        stack.push(4).unwrap();
        stack.push(5).unwrap();

        let even_count = count_matching(&stack, |&x| x % 2 == 0);
        assert_eq!(even_count, 2);

        let greater_than_3 = count_matching(&stack, |&x| x > 3);
        assert_eq!(greater_than_3, 2);
    }

    #[test]
    fn test_container_polymorphism() {
        fn sum_all<T, C>(container: &C) -> T
        where
            T: std::ops::Add<Output = T> + Default + Copy,
            C: Container<T>,
        {
            container
                .iter()
                .copied()
                .fold(T::default(), |acc, x| acc + x)
        }

        let mut stack: Stack<i32, 5> = Stack::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();
        stack.push(3).unwrap();

        assert_eq!(sum_all(&stack), 6);
    }

    // Milestone 5 Tests
    #[test]
    fn test_builder_with_default() {
        let stack: Stack<i32, 5> = ContainerBuilder::new().with_default(42).ready().build();

        assert_eq!(stack.len(), 5);
        assert_eq!(stack.peek(), Some(&42));
    }

    #[test]
    fn test_builder_with_defaults() {
        let stack: Stack<i32, 3> = ContainerBuilder::new().with_defaults().build();

        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_builder_with_initializer() {
        let mut stack: Stack<i32, 5> = ContainerBuilder::new()
            .with_defaults()
            .build_with(|i| (i * 2) as i32);

        assert_eq!(stack.len(), 5);
        assert_eq!(stack.pop(), Some(8));
        assert_eq!(stack.pop(), Some(6));
    }

    #[test]
    fn test_builder_zero_size() {
        use std::mem::size_of;

        assert_eq!(
            size_of::<ContainerBuilder<i32, 100, Empty>>(),
            size_of::<Option<i32>>()
        );
        assert_eq!(
            size_of::<ContainerBuilder<i32, 100, Configured>>(),
            size_of::<Option<i32>>()
        );
        assert_eq!(
            size_of::<ContainerBuilder<i32, 100, Ready>>(),
            size_of::<Option<i32>>()
        );

        assert_eq!(size_of::<Empty>(), 0);
        assert_eq!(size_of::<Configured>(), 0);
        assert_eq!(size_of::<Ready>(), 0);
    }

    #[test]
    fn test_builder_method_chaining() {
        let stack: Stack<String, 5> = ContainerBuilder::new()
            .with_default(String::from("test"))
            .ready()
            .build();

        assert_eq!(stack.len(), 5);
    }
}
