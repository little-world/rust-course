// Pattern 5: Generational Indices

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Handle {
    index: usize,
    generation: u64,
}

struct Slot<T> {
    value: Option<T>,
    generation: u64,
}

struct GenArena<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<usize>,
}

impl<T> GenArena<T> {
    fn new() -> Self {
        GenArena { slots: Vec::new(), free_list: Vec::new() }
    }

    fn insert(&mut self, value: T) -> Handle {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index];
            slot.generation += 1;
            slot.value = Some(value);
            Handle { index, generation: slot.generation }
        } else {
            let index = self.slots.len();
            self.slots.push(Slot { value: Some(value), generation: 0 });
            Handle { index, generation: 0 }
        }
    }

    fn get(&self, handle: Handle) -> Option<&T> {
        self.slots.get(handle.index)
            .filter(|slot| slot.generation == handle.generation)
            .and_then(|slot| slot.value.as_ref())
    }

    fn remove(&mut self, handle: Handle) -> Option<T> {
        let slot = self.slots.get_mut(handle.index)?;
        if slot.generation != handle.generation {
            return None;
        }
        self.free_list.push(handle.index);
        slot.value.take()
    }
}

fn main() {
    // Usage: Handles remain valid even after removals
    let mut arena = GenArena::new();
    let h1 = arena.insert("first");
    let h2 = arena.insert("second");

    println!("h1 points to: {:?}", arena.get(h1));
    println!("h2 points to: {:?}", arena.get(h2));

    arena.remove(h1);  // Slot 0 freed, generation incremented

    let h3 = arena.insert("third");  // Reuses slot 0 with new generation

    // Old handle is safely rejected
    assert!(arena.get(h1).is_none());  // Stale handle!
    assert_eq!(arena.get(h3), Some(&"third"));

    println!("After removal, h1 (stale): {:?}", arena.get(h1));
    println!("h3 (new): {:?}", arena.get(h3));
    println!("Generational arena example completed");
}
