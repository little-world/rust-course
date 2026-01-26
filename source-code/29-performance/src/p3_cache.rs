// Pattern 3: Cache-Friendly Data Structures
// Demonstrates cache-aware programming techniques.

use std::collections::HashMap;
use std::time::Instant;
use rustc_hash::FxHashMap;

// ============================================================================
// Example: Array-of-Structs vs Struct-of-Arrays
// ============================================================================

// Array of Structs (AoS) - potentially worse for cache
#[derive(Clone)]
struct ParticleAoS {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
}

fn update_positions_aos(particles: &mut [ParticleAoS], dt: f32) {
    for p in particles {
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.z += p.vz * dt;
    }
}

// Struct of Arrays (SoA) - cache-friendly
struct ParticlesSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    vz: Vec<f32>,
}

impl ParticlesSoA {
    fn new(count: usize) -> Self {
        ParticlesSoA {
            x: vec![0.0; count],
            y: vec![0.0; count],
            z: vec![0.0; count],
            vx: vec![1.0; count],
            vy: vec![1.0; count],
            vz: vec![1.0; count],
        }
    }

    fn len(&self) -> usize {
        self.x.len()
    }
}

fn update_positions_soa(particles: &mut ParticlesSoA, dt: f32) {
    for i in 0..particles.len() {
        particles.x[i] += particles.vx[i] * dt;
        particles.y[i] += particles.vy[i] * dt;
        particles.z[i] += particles.vz[i] * dt;
    }
}

// ============================================================================
// Example: Cache Line Awareness
// ============================================================================

// Bad: False sharing - both counters in same cache line
struct CounterBad {
    thread1_counter: i64,
    thread2_counter: i64,
}

// Good: Each counter on its own cache line
#[repr(C, align(64))]
struct CounterGood1 {
    thread1_counter: i64,
    _padding: [u8; 56],
}

#[repr(C, align(64))]
struct CounterGood2 {
    thread2_counter: i64,
    _padding: [u8; 56],
}

// ============================================================================
// Example: Prefetching and Sequential Access
// ============================================================================

fn sequential_access(data: &[i32]) -> i64 {
    let mut sum = 0i64;
    for &x in data {
        sum += x as i64;
    }
    sum
}

fn random_access(data: &[i32], indices: &[usize]) -> i64 {
    let mut sum = 0i64;
    for &idx in indices {
        sum += data[idx] as i64;
    }
    sum
}

// ============================================================================
// Example: Linked Lists vs Vectors
// ============================================================================

// Linked list - poor cache behavior
struct LinkedNode<T> {
    value: T,
    next: Option<Box<LinkedNode<T>>>,
}

struct LinkedList<T> {
    head: Option<Box<LinkedNode<T>>>,
}

impl<T> LinkedList<T> {
    fn new() -> Self {
        LinkedList { head: None }
    }

    fn push_front(&mut self, value: T) {
        let new_node = Box::new(LinkedNode {
            value,
            next: self.head.take(),
        });
        self.head = Some(new_node);
    }
}

fn sum_linked_list(list: &LinkedList<i32>) -> i64 {
    let mut sum = 0i64;
    let mut current = &list.head;
    while let Some(node) = current {
        sum += node.value as i64;
        current = &node.next;
    }
    sum
}

fn sum_vector(vec: &[i32]) -> i64 {
    vec.iter().map(|&x| x as i64).sum()
}

// ============================================================================
// Example: HashMap Optimization
// ============================================================================

fn compare_hashmaps() {
    let count = 10000;

    // Standard HashMap
    let start = Instant::now();
    let mut std_map = HashMap::new();
    for i in 0..count {
        std_map.insert(i, i * 2);
    }
    let std_insert = start.elapsed();

    let start = Instant::now();
    let mut sum = 0i64;
    for i in 0..count {
        sum += *std_map.get(&i).unwrap_or(&0) as i64;
    }
    let std_lookup = start.elapsed();

    // FxHashMap (faster for integer keys)
    let start = Instant::now();
    let mut fx_map = FxHashMap::default();
    for i in 0..count {
        fx_map.insert(i, i * 2);
    }
    let fx_insert = start.elapsed();

    let start = Instant::now();
    let mut sum2 = 0i64;
    for i in 0..count {
        sum2 += *fx_map.get(&i).unwrap_or(&0) as i64;
    }
    let fx_lookup = start.elapsed();

    println!("  std HashMap insert: {:?}, lookup: {:?}", std_insert, std_lookup);
    println!("  FxHashMap   insert: {:?}, lookup: {:?}", fx_insert, fx_lookup);
    assert_eq!(sum, sum2);
}

fn presized_hashmap_demo() {
    let items = vec![(1, "a"), (2, "b"), (3, "c")];

    // Bad: Allocates multiple times as it grows
    let mut map1 = HashMap::new();
    for (k, v) in &items {
        map1.insert(*k, *v);
    }

    // Good: Allocates once
    let mut map2 = HashMap::with_capacity(items.len());
    for (k, v) in &items {
        map2.insert(*k, *v);
    }

    // Better: Use FromIterator
    let map3: HashMap<_, _> = items.iter().copied().collect();

    assert_eq!(map1, map2);
    assert_eq!(map2, map3);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aos_update() {
        let mut particles = vec![
            ParticleAoS { x: 0.0, y: 0.0, z: 0.0, vx: 1.0, vy: 2.0, vz: 3.0 },
        ];
        update_positions_aos(&mut particles, 1.0);
        assert_eq!(particles[0].x, 1.0);
        assert_eq!(particles[0].y, 2.0);
        assert_eq!(particles[0].z, 3.0);
    }

    #[test]
    fn test_soa_update() {
        let mut particles = ParticlesSoA::new(1);
        particles.vx[0] = 1.0;
        particles.vy[0] = 2.0;
        particles.vz[0] = 3.0;
        update_positions_soa(&mut particles, 1.0);
        assert_eq!(particles.x[0], 1.0);
        assert_eq!(particles.y[0], 2.0);
        assert_eq!(particles.z[0], 3.0);
    }

    #[test]
    fn test_sequential_access() {
        let data: Vec<i32> = (0..100).collect();
        let sum = sequential_access(&data);
        assert_eq!(sum, (0..100).sum::<i32>() as i64);
    }

    #[test]
    fn test_linked_list_vs_vector() {
        let mut list = LinkedList::new();
        for i in (0..100).rev() {
            list.push_front(i);
        }
        let vec: Vec<i32> = (0..100).collect();

        assert_eq!(sum_linked_list(&list), sum_vector(&vec));
    }

    #[test]
    fn test_cache_line_alignment() {
        assert_eq!(std::mem::align_of::<CounterGood1>(), 64);
        assert_eq!(std::mem::align_of::<CounterGood2>(), 64);
    }
}

fn main() {
    println!("Pattern 3: Cache-Friendly Data Structures");
    println!("==========================================\n");

    let particle_count = 100_000;

    // AoS benchmark
    let mut aos_particles: Vec<ParticleAoS> = (0..particle_count)
        .map(|_| ParticleAoS { x: 0.0, y: 0.0, z: 0.0, vx: 1.0, vy: 1.0, vz: 1.0 })
        .collect();

    let start = Instant::now();
    for _ in 0..100 {
        update_positions_aos(&mut aos_particles, 0.016);
    }
    println!("AoS update (100 iterations): {:?}", start.elapsed());

    // SoA benchmark
    let mut soa_particles = ParticlesSoA::new(particle_count);

    let start = Instant::now();
    for _ in 0..100 {
        update_positions_soa(&mut soa_particles, 0.016);
    }
    println!("SoA update (100 iterations): {:?}", start.elapsed());

    // Sequential vs random access
    println!("\nSequential vs Random access:");
    let data: Vec<i32> = (0..1_000_000).collect();
    let indices: Vec<usize> = {
        use rand::seq::SliceRandom;
        let mut idx: Vec<usize> = (0..data.len()).collect();
        idx.shuffle(&mut rand::thread_rng());
        idx
    };

    let start = Instant::now();
    let _ = sequential_access(&data);
    println!("  Sequential: {:?}", start.elapsed());

    let start = Instant::now();
    let _ = random_access(&data, &indices);
    println!("  Random:     {:?}", start.elapsed());

    // HashMap comparison
    println!("\nHashMap comparison:");
    compare_hashmaps();

    presized_hashmap_demo();
    println!("  Pre-sized HashMap: OK");
}
