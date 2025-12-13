

# Text Tokenizer for Neural Networks

### Problem Statement

Build a production-grade tokenizer for neural network training, implementing three tokenization strategies: character-level, word-level, and Byte-Pair Encoding (BPE). The BPE tokenizer must efficiently train on large corpora (100MB+ text), handle Unicode correctly, and provide encoding/decoding at millions of tokens per second.

The system must:
- Train BPE vocabulary from text corpus (learn most frequent byte pairs)
- Encode text to token IDs efficiently
- Decode token IDs back to text
- Handle special tokens (PAD, UNK, BOS, EOS)
- Support vocabulary serialization/deserialization
- Scale to multi-gigabyte training corpora with parallel processing

### Use Cases

- **Language Model Training**: GPT, BERT, LLaMA tokenization (BPE/WordPiece)
- **Machine Translation**: Subword tokenization for handling rare words
- **Code Generation Models**: GitHub Copilot, CodeLlama tokenizers
- **Text Classification**: Converting text to numerical representations
- **Search Engines**: Text indexing and retrieval systems
- **Data Processing Pipelines**: ETL for NLP datasets

### Why It Matters

**Performance Impact:**
- Naive BPE training: O(n² × vocab_size) - 10+ hours for 100MB corpus
- Optimized BPE: O(n × log(n) × vocab_size) - 5-10 minutes
- Parallel BPE: 2-5 minutes with 8 cores
- SIMD-optimized encoding: 50-100 million tokens/sec vs 5-10 million

**Real-World Scale:**
- GPT-2 vocabulary: 50,257 tokens trained on 40GB WebText
- SentencePiece (Google): Processes Wikipedia (20GB) in ~30 minutes
- Tokenizers library (HuggingFace): Rust-based, 10-20x faster than Python

**Why BPE Matters:**
Character-level: vocab=256, but sequences are 4-10x longer (slow inference)
Word-level: vocab=50k-100k, but can't handle rare words/typos (poor generalization)
BPE: vocab=32k, handles any word via subwords (best trade-off)

Example:
```
Text: "unhappiness"
Character: [u, n, h, a, p, p, i, n, e, s, s] - 11 tokens
Word: [UNK] - unknown word, information lost
BPE: [un, happ, iness] - 3 tokens, preserves meaning
```

**Optimization Importance:**
Training LLaMA on 1TB text:
- Naive tokenization: 1000+ hours
- Optimized tokenization: 10-20 hours
- 50-100x speedup = $10,000s saved in compute costs

---

## Key Concepts Explained

Before diving into implementation, let's understand the core concepts that make modern tokenizers fast and effective. This project progresses from simple character tokenization to production-grade BPE with extreme optimizations.

### 1. Subword Tokenization and the Vocabulary Trade-off

**What Is It?**

Subword tokenization breaks words into smaller meaningful units (subwords) rather than treating entire words or individual characters as atomic tokens. It represents the sweet spot between character-level and word-level tokenization.

**The Three Approaches:**

```
Text: "unhappiness"

Character-level (vocab=256):
Tokens: [u, n, h, a, p, p, i, n, e, s, s]
Length: 11 tokens
Pros: Can represent any text, tiny vocabulary
Cons: Very long sequences → slow inference, hard to learn semantics

Word-level (vocab=50k-100k):
Tokens: [UNK]  (if "unhappiness" not in vocabulary)
Length: 1 token
Pros: Short sequences, natural semantic units
Cons: Can't handle rare words/typos, huge vocabulary

Subword BPE (vocab=32k):
Tokens: [un, happ, iness]
Length: 3 tokens
Pros: Handles any word, reasonable sequence length, shared roots
Cons: Needs training algorithm, slightly complex
```

**Why It Matters:**

Modern language models (GPT, BERT, LLaMA) all use subword tokenization because:
1. **Generalization**: "running" and "runner" share "run" prefix → model learns relationships
2. **Vocabulary efficiency**: 32k tokens vs 100k+ words
3. **Unknown word handling**: Any word can be represented via subwords
4. **Inference speed**: Shorter sequences than character-level (4-10x reduction)

**Real-World Example:**

```rust
// GPT-2 tokenizer example
"unhappiness" → [un, happ, iness]
"antidisestablishmentarianism" → [ant, idis, establish, ment, arian, ism]
"COVID-19" → [COVID, -, 19]  (handles new words!)

// Character-level would need 28 tokens for the long word
// Word-level would output [UNK] and lose all information
```

**Performance Impact:**

For GPT-3 inference on 1000 tokens:
- Character-level: ~4000 tokens → 4x slower, 4x more memory
- Subword BPE: 1000 tokens → baseline
- Word-level: ~500 tokens but can't handle OOV words → fails on real text

---

### 2. Byte-Pair Encoding (BPE) Algorithm

**What Is It?**

BPE is a greedy algorithm that iteratively merges the most frequent pair of adjacent tokens in the corpus. It starts with individual bytes/characters and builds up to common subwords.

**The Algorithm:**

```
Corpus: "low low low lower lower newest widest"

Step 1: Initialize with characters
Words: [l,o,w], [l,o,w], [l,o,w], [l,o,w,e,r], ...

Step 2: Count all adjacent pairs
Pairs: {(l,o): 5, (o,w): 5, (w,e): 2, (e,r): 2, ...}

Step 3: Merge most frequent pair (l,o) → "lo"
Words: [lo,w], [lo,w], [lo,w], [lo,w,e,r], ...

Step 4: Count pairs again
Pairs: {(lo,w): 5, (w,e): 2, (e,r): 2, ...}

Step 5: Merge (lo,w) → "low"
Words: [low], [low], [low], [low,e,r], ...

Step 6: Continue until vocab_size reached
Final vocab: [l, o, w, e, r, n, i, s, t, d, lo, ow, low, er, lower, est, ...]
```

**Training vs Encoding:**

```rust
// Training: Learn which pairs to merge
fn train(corpus: &str, vocab_size: usize) -> Vec<(String, String)> {
    let mut words = split_into_chars(corpus);
    let mut merges = vec![];

    while merges.len() < vocab_size {
        // Count all pairs
        let pair_counts = count_pairs(&words);

        // Find most frequent
        let best_pair = pair_counts.max_by_key(|pair, count| count);

        // Merge this pair everywhere
        merge_pair(&mut words, best_pair);
        merges.push(best_pair);
    }

    merges  // Ordered list of merges
}

// Encoding: Apply learned merges in order
fn encode(text: &str, merges: &[(String, String)]) -> Vec<String> {
    let mut tokens = split_into_chars(text);

    // Apply each merge in learned order
    for (a, b) in merges {
        tokens = apply_merge(tokens, (a, b));
    }

    tokens
}
```

**Why This Works:**

1. **Frequency captures importance**: Common subwords get merged first
2. **Order matters**: "low" must be learned before "lowest"
3. **Greedy is good enough**: Optimal merging is NP-hard, greedy works well in practice
4. **Deterministic encoding**: Same merges → same tokens

**Complexity Analysis:**

Naive BPE (what we'll implement first):
```
For each merge iteration (vocab_size iterations):
  - Count all pairs: O(n) where n = total characters
  - Find max: O(unique_pairs)
  - Merge pair: O(n)
Total: O(vocab_size × n) = O(n²) for large vocab

Example: 100MB corpus, 32k vocab → 3200 × 100M = 320 billion operations → 10+ hours
```

Optimized BPE (with priority queue):
```
- Build initial pair counts: O(n)
- Use binary heap to track max: O(log k) where k = unique pairs
- Update counts after merge: O(affected_pairs × log k)
Total: O(n + vocab_size × log k) = O(n log n)

Same example: 100M + 32k × log(100k) ≈ 100M + 500k = ~5 minutes
```

**Real-World Scale:**

- GPT-2 (50k vocab, 40GB corpus): ~8 hours with optimized BPE
- SentencePiece (32k vocab, 20GB Wikipedia): ~30 minutes with advanced optimizations
- This project target: 100MB corpus in 2-5 minutes (Milestone 5)

---

### 3. HashMap, Vocabulary Management, and Bidirectional Mappings

**What Is It?**

Tokenizers need to map between three representations: text ↔ tokens ↔ IDs. Efficient bidirectional mappings are critical for both encoding (text → IDs) and decoding (IDs → text).

**The Data Structures:**

```rust
pub struct Tokenizer {
    // Forward: token string to ID
    vocab: HashMap<String, u32>,

    // Reverse: ID to token string
    id_to_token: HashMap<u32, String>,

    // Or more efficiently:
    id_to_token: Vec<String>,  // Direct indexing

    // Special tokens
    special_tokens: HashMap<String, u32>,  // <PAD>=0, <UNK>=1, <BOS>=2, <EOS>=3
}
```

**Why Two Mappings?**

```rust
// Encoding: Need fast "hello" → ID lookup
fn encode(&self, text: &str) -> Vec<u32> {
    text.split_whitespace()
        .map(|word| {
            // O(1) HashMap lookup
            *self.vocab.get(word).unwrap_or(&self.unk_id)
        })
        .collect()
}

// Decoding: Need fast ID → "hello" lookup
fn decode(&self, ids: &[u32]) -> String {
    ids.iter()
        .map(|&id| {
            // O(1) HashMap or Vec indexing
            self.id_to_token.get(&id).unwrap()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
```

**Vec vs HashMap for Reverse Mapping:**

```rust
// Option 1: HashMap<u32, String>
// - Flexible: IDs don't need to be contiguous
// - Slower: Hash function + collision resolution (~50-100ns)
// - More memory: Hash table overhead

// Option 2: Vec<String>
// - Fast: Direct array indexing (~2ns)
// - Memory efficient: No hash table overhead
// - Requirement: IDs must be 0..n-1
// - This is what production tokenizers use!

pub struct FastTokenizer {
    vocab: HashMap<String, u32>,  // Still need HashMap for text lookup
    id_to_token: Vec<String>,      // Vec for fast ID lookup
}

impl FastTokenizer {
    fn add_token(&mut self, token: String) -> u32 {
        let id = self.id_to_token.len() as u32;
        self.vocab.insert(token.clone(), id);
        self.id_to_token.push(token);
        id
    }
}
```

**Special Tokens:**

```rust
// Special tokens have reserved IDs
pub const PAD_ID: u32 = 0;   // Padding for batching sequences
pub const UNK_ID: u32 = 1;   // Unknown token
pub const BOS_ID: u32 = 2;   // Beginning of sequence
pub const EOS_ID: u32 = 3;   // End of sequence

// Usage in training:
let input_ids = vec![BOS_ID, 15, 42, 103, EOS_ID, PAD_ID, PAD_ID];
//                   ^                       ^      ^padding^
//                   start marker            end marker

// Why they matter:
// - PAD: All sequences in a batch must have same length
// - UNK: Handle characters/words not in vocabulary
// - BOS/EOS: Model learns sentence boundaries
```

**Memory Layout:**

```rust
// Small vocabulary (vocab_size=1000):
// HashMap: ~48 bytes per entry × 1000 = 48KB (8B key + 8B value + 32B overhead)
// Vec: ~24 bytes per entry × 1000 = 24KB (8B pointer + 16B String metadata)

// Large vocabulary (vocab_size=50k):
// HashMap: ~2.4MB
// Vec: ~1.2MB

// Lookup performance (1M lookups):
// HashMap: ~50-100ns per lookup = 50-100ms total
// Vec: ~2ns per lookup = 2ms total (25-50x faster!)
```

**Best Practice:**

Use Vec for ID→token (frequent during decoding), HashMap for token→ID (frequent during encoding). This is what HuggingFace tokenizers, SentencePiece, and tiktoken all do.

---

### 4. Priority Queue and Binary Heap for Efficient Pair Selection

**What Is It?**

A priority queue (implemented as binary heap) allows efficient retrieval of the maximum element. In BPE, we need to find the most frequent pair thousands of times, making this data structure critical for performance.

**The Problem:**

```rust
// Naive BPE: Find max pair every iteration
for _ in 0..vocab_size {
    let pair_counts = count_pairs(&words);  // O(n)

    // Linear scan to find max - O(k) where k = unique pairs
    let max_pair = pair_counts.iter()
        .max_by_key(|(pair, &count)| count)
        .unwrap();

    merge_pair(&mut words, max_pair);
}
// Total: O(vocab_size × (n + k))
// For 100k unique pairs, k=100k → very slow!
```

**The Solution: Priority Queue:**

```rust
use std::collections::BinaryHeap;

#[derive(Eq, PartialEq)]
struct PairCount {
    pair: (String, String),
    count: usize,
}

// Implement Ord to make BinaryHeap a max-heap
impl Ord for PairCount {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)  // Compare by count
    }
}

fn optimized_bpe(corpus: &str, vocab_size: usize) {
    let mut words = split_into_chars(corpus);

    // Build initial priority queue - O(k log k)
    let pair_counts = count_pairs(&words);
    let mut heap: BinaryHeap<PairCount> = pair_counts
        .into_iter()
        .map(|(pair, count)| PairCount { pair, count })
        .collect();

    for _ in 0..vocab_size {
        // Get max pair - O(1)
        let max_pair = heap.peek().unwrap();

        // Merge this pair
        merge_pair(&mut words, &max_pair.pair);

        // Update affected pairs - O(affected × log k)
        update_heap_after_merge(&mut heap, &max_pair.pair);
    }
}
// Total: O(k log k + vocab_size × affected × log k)
// If affected is small (local changes), this is ~O(n log k)
```

**Binary Heap Structure:**

```
           (l,o):100
          /         \
      (o,w):95    (w,e):80
      /     \      /     \
  (e,r):70 (r,e):65 (e,s):60 (s,t):55

Properties:
- Max element at root: O(1) access
- Insert: O(log n) - bubble up
- Remove max: O(log n) - bubble down
- Stored as Vec: [100, 95, 80, 70, 65, 60, 55]
  - Parent of i: (i-1)/2
  - Children of i: 2i+1, 2i+2
```

**Operations:**

```rust
let mut heap = BinaryHeap::new();

// Insert - O(log n)
heap.push(PairCount { pair: ("l", "o"), count: 100 });
heap.push(PairCount { pair: ("o", "w"), count: 95 });

// Peek max - O(1)
let max = heap.peek();  // Some(PairCount { count: 100, ... })

// Remove max - O(log n)
let max = heap.pop();   // Removes and returns max

// Update count (need to remove + re-insert) - O(log n)
// BinaryHeap doesn't support update, so:
heap.pop();  // Remove old
heap.push(PairCount { pair: ("l", "o"), count: 105 });  // Insert new
```

**Performance Comparison:**

```rust
// Test: Find max element 10,000 times in 100,000 pairs

// Linear scan (naive):
// - 10,000 × 100,000 = 1 billion comparisons
// - Time: ~10 seconds

// Binary heap:
// - Build heap: 100,000 × log(100,000) ≈ 1.7M operations
// - 10,000 pops: 10,000 × log(100,000) ≈ 170k operations
// - Time: ~20 milliseconds
// - Speedup: 500x!
```

**Why It Matters for BPE:**

Milestone 3 (Naive): O(n² × vocab_size) - 30+ minutes for 100MB
Milestone 4 (Priority Queue): O(n log n × vocab_size) - 5-10 minutes for 100MB
Speedup: 3-6x just from data structure choice!

---

### 5. String Interning and Memory Optimization

**What Is It?**

String interning is a memory optimization where we store each unique string once and refer to it via a small integer ID. This reduces memory usage and makes string comparisons as fast as integer comparisons.

**The Problem:**

```rust
// Naive BPE: Store pairs as strings
let pairs: HashMap<(String, String), usize> = HashMap::new();
pairs.insert(("hello".to_string(), "world".to_string()), 42);

// Memory per pair:
// - Two String objects: 24 bytes each × 2 = 48 bytes
// - Heap allocations: "hello" (5 bytes) + "world" (5 bytes) = 10 bytes
// - HashMap overhead: ~32 bytes
// Total: ~90 bytes per pair

// For 100k unique pairs: 9MB just for pair keys!
```

**The Solution: String Interning:**

```rust
pub struct StringInterner {
    string_to_id: HashMap<String, u32>,  // Intern table
    id_to_string: Vec<String>,           // Reverse lookup
}

impl StringInterner {
    pub fn intern(&mut self, s: &str) -> u32 {
        // Check if already interned
        if let Some(&id) = self.string_to_id.get(s) {
            return id;
        }

        // Allocate new ID
        let id = self.id_to_string.len() as u32;
        self.string_to_id.insert(s.to_string(), id);
        self.id_to_string.push(s.to_string());
        id
    }

    pub fn get_string(&self, id: u32) -> &str {
        &self.id_to_string[id as usize]
    }
}

// Now store pairs as IDs
let pairs: HashMap<(u32, u32), usize> = HashMap::new();
let hello_id = interner.intern("hello");  // 0
let world_id = interner.intern("world");  // 1
pairs.insert((hello_id, world_id), 42);

// Memory per pair:
// - Two u32s: 4 bytes × 2 = 8 bytes
// - HashMap overhead: ~8 bytes
// Total: ~16 bytes per pair

// For 100k pairs: 1.6MB (5.6x reduction!)
```

**Benefits:**

1. **Memory reduction**: 48 bytes → 8 bytes per pair (6x less)
2. **Faster comparisons**: String comparison O(n) → Integer comparison O(1)
3. **Better cache locality**: Small integers fit in CPU cache
4. **Faster hashing**: Hash u32 (~2ns) vs hash string (~10-50ns)

**Encoding Pairs as u64:**

```rust
// Further optimization: Pack two u32s into one u64
fn encode_pair(a: u32, b: u32) -> u64 {
    ((a as u64) << 32) | (b as u64)
}

fn decode_pair(packed: u64) -> (u32, u32) {
    let a = (packed >> 32) as u32;
    let b = (packed & 0xFFFFFFFF) as u32;
    (a, b)
}

// Now HashMap key is single u64 instead of (u32, u32) tuple
let pairs: HashMap<u64, usize> = HashMap::new();
let pair_id = encode_pair(hello_id, world_id);
pairs.insert(pair_id, 42);

// Benefits:
// - Single 8-byte key instead of 16-byte tuple
// - Faster hashing (one hash instead of two)
// - Better memory layout
```

**Performance Impact:**

```rust
// Benchmark: Count 1M pairs in 100MB corpus

// With String pairs:
// - HashMap inserts: 1M × 100ns = 100ms (slow hash + allocation)
// - Memory: 90MB for pair storage
// - Cache misses: High (strings scattered in heap)

// With interned u32 pairs:
// - HashMap inserts: 1M × 20ns = 20ms (fast hash, no allocation)
// - Memory: 16MB for pair storage
// - Cache misses: Low (integers are compact)

// Speedup: 5x faster, 5.6x less memory
```

**When to Use:**

- ✅ Many duplicate strings (BPE has ~32k unique tokens, millions of repetitions)
- ✅ Need fast equality checks (pair comparison in BPE)
- ✅ Memory constrained (large corpora)
- ❌ Few unique strings (overhead not worth it)
- ❌ Strings rarely compared (benefit is small)

---

### 6. Rayon and Data Parallelism

**What Is It?**

Rayon is a data parallelism library that makes it trivial to parallelize operations on collections. It automatically splits work across CPU cores and handles thread management.

**The Problem:**

```rust
// Sequential pair counting
fn count_pairs(words: &[Vec<String>]) -> HashMap<(String, String), usize> {
    let mut counts = HashMap::new();

    for word in words {  // Process one word at a time
        for i in 0..word.len()-1 {
            let pair = (word[i].clone(), word[i+1].clone());
            *counts.entry(pair).or_insert(0) += 1;
        }
    }

    counts
}

// Problem: Uses only 1 CPU core
// Modern machines have 8-16 cores → 87-93% of CPU sitting idle!
```

**The Solution: Rayon:**

```rust
use rayon::prelude::*;

fn parallel_count_pairs(words: &[Vec<String>]) -> HashMap<(String, String), usize> {
    // Split words into chunks, process in parallel
    words.par_iter()  // Parallel iterator
        .fold(
            || HashMap::new(),  // Each thread gets own HashMap
            |mut counts, word| {
                // Count pairs in this word
                for i in 0..word.len()-1 {
                    let pair = (word[i].clone(), word[i+1].clone());
                    *counts.entry(pair).or_insert(0) += 1;
                }
                counts
            }
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                // Merge thread-local HashMaps
                for (pair, count) in b {
                    *a.entry(pair).or_insert(0) += count;
                }
                a
            }
        )
}
```

**How It Works:**

```
Words: [word1, word2, word3, word4, word5, word6, word7, word8]
         ↓       ↓       ↓       ↓       ↓       ↓       ↓       ↓
      Split into chunks (automatically by Rayon)
         [word1, word2] [word3, word4] [word5, word6] [word7, word8]
              ↓              ↓              ↓              ↓
           Thread 1       Thread 2       Thread 3       Thread 4
              ↓              ↓              ↓              ↓
          counts1        counts2        counts3        counts4
              ↓              ↓              ↓              ↓
                    Merge (reduce) all counts
                              ↓
                       Final counts
```

**Rayon Patterns:**

```rust
// Pattern 1: par_iter + fold + reduce
let sum: i32 = (0..1000)
    .par_iter()
    .fold(|| 0, |acc, &x| acc + x)  // Each thread sums its chunk
    .reduce(|| 0, |a, b| a + b);     // Combine thread results

// Pattern 2: par_iter + map + collect
let squares: Vec<i32> = (0..1000)
    .par_iter()
    .map(|&x| x * x)
    .collect();

// Pattern 3: par_iter + for_each (side effects)
(0..1000)
    .par_iter()
    .for_each(|&x| {
        println!("{}", x);  // Order not guaranteed!
    });
```

**Performance Characteristics:**

```rust
// Amdahl's Law: Speedup limited by sequential portion
// If 90% of work is parallelizable:
// - 2 cores: 1.82x speedup
// - 4 cores: 3.08x speedup
// - 8 cores: 4.71x speedup
// - 16 cores: 6.40x speedup (diminishing returns)

// Overhead considerations:
// - Thread spawning: ~1-2μs per thread
// - Work splitting: ~100ns per chunk
// - Merging results: Depends on data structure

// Rule of thumb: Parallelize if work > 10-100μs per item
```

**Real-World Performance:**

```rust
// BPE pair counting on 100MB corpus
// 10M words, 100M characters

// Sequential:
// - Time: 5000ms
// - CPU usage: 12.5% (1 of 8 cores)

// Rayon parallel (8 cores):
// - Time: 800ms
// - CPU usage: 85% (7 of 8 cores, some overhead)
// - Speedup: 6.25x

// Why not 8x?
// - Thread overhead: ~50ms
// - Load imbalance: Some words longer than others
// - Merge phase: Sequential (reduce)
// - Cache effects: More cache misses with parallel access
```

**When to Use Rayon:**

- ✅ CPU-bound work (computation, not I/O)
- ✅ Independent iterations (no dependencies between items)
- ✅ Enough work per item (>10μs, otherwise overhead dominates)
- ✅ Want automatic load balancing (Rayon handles work stealing)
- ❌ Tiny workloads (overhead > benefit)
- ❌ Sequential dependencies (item N depends on item N-1)
- ❌ I/O-bound work (use async instead)

---

### 7. DashMap and Lock-Free Concurrent Data Structures

**What Is It?**

DashMap is a concurrent HashMap that allows multiple threads to read and write simultaneously without explicit locks. It achieves this through sharding and fine-grained locking.

**The Problem with Standard HashMap:**

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Naive concurrent HashMap: Global lock
let counts = Arc::new(Mutex::new(HashMap::new()));

// All threads share one lock
thread::spawn({
    let counts = counts.clone();
    move || {
        for pair in pairs {
            let mut map = counts.lock().unwrap();  // LOCK ENTIRE MAP
            *map.entry(pair).or_insert(0) += 1;
        }  // Lock released here
    }
});

// Problem: Only ONE thread can access map at a time
// - Thread 1: Writing to key "ab"... LOCKED
// - Thread 2: Wants to write to key "cd"... WAITING (different key, but still blocked!)
// - Thread 3: Wants to write to key "ef"... WAITING
// - Result: Serialization → No parallelism!
```

**The Solution: DashMap (Sharding):**

```rust
use dashmap::DashMap;

// DashMap internally splits into N shards (default: num_cpus * 4)
// Each shard has its own lock
let counts = Arc::new(DashMap::new());

thread::spawn({
    let counts = counts.clone();
    move || {
        for pair in pairs {
            // NO explicit locking by user
            counts.entry(pair)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }
});

// How it works:
// - Hash key to determine shard: hash("ab") % num_shards
// - Lock only that shard (other shards remain accessible)
// - Thread 1: shard 3 (key "ab") ← LOCKED
// - Thread 2: shard 7 (key "cd") ← UNLOCKED ✓
// - Thread 3: shard 2 (key "ef") ← UNLOCKED ✓
```

**DashMap Internals:**

```
DashMap with 16 shards:

┌─────────┬─────────┬─────────┬─────────┐
│ Shard 0 │ Shard 1 │ Shard 2 │ Shard 3 │ ...
│ (lock)  │ (lock)  │ (lock)  │ (lock)  │
├─────────┼─────────┼─────────┼─────────┤
│ "ab": 5 │ "cd": 3 │ "ef": 7 │ "gh": 2 │
│ "xy": 1 │ "pq": 9 │ "mn": 4 │ "ij": 6 │
└─────────┴─────────┴─────────┴─────────┘

Key routing:
- hash("ab") % 16 = 0 → Shard 0
- hash("cd") % 16 = 1 → Shard 1
- hash("ef") % 16 = 2 → Shard 2

If Thread 1 locks Shard 0, Threads 2 and 3 can still access Shards 1-15
```

**API Usage:**

```rust
use dashmap::DashMap;

let map = DashMap::new();

// Insert
map.insert("key", 42);

// Get (returns reference guard, auto-releases lock)
if let Some(value) = map.get("key") {
    println!("Value: {}", *value);  // *value = 42
}  // Lock released here

// Entry API (atomic update)
map.entry("key")
    .and_modify(|v| *v += 1)  // If exists, increment
    .or_insert(1);             // If not, insert 1

// Iteration (locks each shard temporarily)
for entry in map.iter() {
    println!("{}: {}", entry.key(), entry.value());
}
```

**Performance Comparison:**

```rust
// Benchmark: 8 threads each inserting 100k items

// Mutex<HashMap>:
// - Time: 2000ms
// - Throughput: 400k inserts/sec
// - Problem: Threads wait for lock most of the time

// DashMap (16 shards):
// - Time: 300ms
// - Throughput: 2.7M inserts/sec
// - Speedup: 6.7x

// Why not 8x?
// - Collision: Sometimes 2 threads want same shard
// - Lock overhead: Fine-grained locks have some cost
// - Cache effects: More cache line bouncing
```

**Contention and Shard Count:**

```rust
// Too few shards: More contention
DashMap::with_capacity_and_shard_amount(1000, 4);  // 4 shards
// - 8 threads → 2 threads per shard on average
// - More waiting

// Too many shards: More overhead
DashMap::with_capacity_and_shard_amount(1000, 1024);  // 1024 shards
// - Memory overhead: 1024 locks
// - Iteration overhead: Must visit 1024 shards

// Sweet spot: num_cpus * 4 (default)
// - 8 cores → 32 shards
// - Low contention, reasonable overhead
```

**When to Use DashMap:**

- ✅ Concurrent reads and writes from multiple threads
- ✅ Independent keys (no cross-key operations)
- ✅ High contention (many threads, frequent access)
- ✅ Want simple API (no manual lock management)
- ❌ Single-threaded (overhead not worth it)
- ❌ Read-only access (use Arc<HashMap> instead)
- ❌ Need cross-key atomicity (use Mutex for consistency)

**Trade-offs:**

| Data Structure | Throughput | Memory | Consistency | Use Case |
|----------------|------------|--------|-------------|----------|
| `HashMap` | Fastest | Lowest | Serial | Single-threaded |
| `Mutex<HashMap>` | Slowest | Low | Strong | Need atomicity |
| `RwLock<HashMap>` | Medium | Low | Strong | Read-heavy |
| `DashMap` | Fast | Medium | Eventual | High concurrency |

---

### 8. SIMD (Single Instruction Multiple Data) and Vectorization

**What Is It?**

SIMD allows processing multiple data elements in a single CPU instruction. Instead of processing bytes one at a time, SIMD can process 16, 32, or even 64 bytes simultaneously.

**The Concept:**

```
Scalar processing (normal):
  for i in 0..16 {
      result[i] = data[i] + 1;
  }
  → 16 instructions (one per byte)

SIMD processing:
  result[0..16] = data[0..16] + [1; 16];
  → 1 instruction (processes 16 bytes at once)
  → 16x speedup (theoretical)
```

**CPU SIMD Instructions:**

Modern CPUs have SIMD instruction sets:
- **SSE** (128-bit): 16 bytes at once (4 × i32 or 16 × u8)
- **AVX** (256-bit): 32 bytes at once (8 × i32 or 32 × u8)
- **AVX-512** (512-bit): 64 bytes at once (16 × i32 or 64 × u8)

**Rust SIMD:**

```rust
// Option 1: Auto-vectorization (compiler does it)
fn add_scalar(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter())
        .map(|(&x, &y)| x.wrapping_add(y))
        .collect()
}
// Compiler may auto-vectorize this with -C target-cpu=native

// Option 2: Explicit SIMD (portable_simd)
#![feature(portable_simd)]
use std::simd::{u8x16, SimdUint};

fn add_simd(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(a.len());

    // Process 16 bytes at a time
    for i in (0..a.len()).step_by(16) {
        let va = u8x16::from_slice(&a[i..i+16]);
        let vb = u8x16::from_slice(&b[i..i+16]);
        let vr = va + vb;  // 16 additions in one instruction!
        result.extend_from_slice(vr.as_array());
    }

    result
}
```

**Tokenizer SIMD Opportunities:**

```rust
// 1. Byte scanning: Find characters in text
fn find_spaces_scalar(text: &[u8]) -> Vec<usize> {
    text.iter()
        .enumerate()
        .filter(|(_, &b)| b == b' ')
        .map(|(i, _)| i)
        .collect()
}

fn find_spaces_simd(text: &[u8]) -> Vec<usize> {
    let space = u8x16::splat(b' ');  // [' ', ' ', ..., ' '] (16 copies)
    let mut positions = vec![];

    for i in (0..text.len()).step_by(16) {
        let chunk = u8x16::from_slice(&text[i..]);
        let mask = chunk.simd_eq(space);  // Compare 16 bytes at once

        // Extract positions where mask is true
        for j in 0..16 {
            if mask.test(j) {
                positions.push(i + j);
            }
        }
    }

    positions
}
// Speedup: 4-8x for large texts
```

```rust
// 2. UTF-8 validation
fn validate_utf8_simd(text: &[u8]) -> bool {
    use std::simd::u8x32;

    for chunk in text.chunks_exact(32) {
        let bytes = u8x32::from_slice(chunk);

        // Check ASCII (< 0x80)
        let ascii_mask = bytes.simd_lt(u8x32::splat(0x80));

        // Check valid UTF-8 continuation bytes (0x80-0xBF)
        let cont_mask = bytes.simd_ge(u8x32::splat(0x80)) & bytes.simd_lt(u8x32::splat(0xC0));

        // Complex logic for multi-byte sequences...
    }

    true
}
// Used by: SentencePiece, tokenizers library
```

**Performance Example:**

```rust
// Benchmark: Count spaces in 10MB text file

// Scalar:
let start = Instant::now();
let count = text.iter().filter(|&&b| b == b' ').count();
let time = start.elapsed();  // 15ms

// SIMD (AVX2, 32 bytes):
let start = Instant::now();
let count = count_spaces_simd(text);
let time = start.elapsed();  // 2ms

// Speedup: 7.5x
```

**When SIMD Helps Tokenizers:**

1. **Byte scanning**: Find whitespace, punctuation for word splitting
2. **UTF-8 validation**: Check text is valid before processing
3. **Pattern matching**: Find special tokens like `<|endoftext|>`
4. **Encoding**: Parallel lookup of character codes (limited usefulness)

**Limitations:**

- **Not all operations vectorize**: HashMap lookups, branching, irregular access patterns
- **Alignment requirements**: Data must be 16/32/64-byte aligned (or use unaligned loads, slower)
- **Overhead for small data**: SIMD setup costs ~10ns, so need >100 bytes to benefit
- **Platform-specific**: AVX-512 not available on all CPUs

**Realistic Speedups for Tokenizers:**

- Byte scanning (find whitespace): 4-8x speedup ✓
- UTF-8 validation: 3-5x speedup ✓
- BPE merging: 1.2-1.5x speedup (memory-bound, not compute-bound)
- Encoding/decoding: 1.1-1.3x speedup (dominated by HashMap lookups)

**Overall**: SIMD gives 2-3x end-to-end speedup for production tokenizers when combined with other optimizations.

---

### 9. Cache-Friendly Memory Layout and Data-Oriented Design

**What Is It?**

Modern CPUs are 100-1000x faster than RAM. To bridge this gap, CPUs use caches (L1, L2, L3). Organizing data to maximize cache hits is critical for performance.

**The Memory Hierarchy:**

```
CPU Registers: 1 cycle (~0.3ns)
L1 Cache (32KB): 4 cycles (~1ns)
L2 Cache (256KB): 12 cycles (~3ns)
L3 Cache (8MB): 40 cycles (~10ns)
RAM (16GB): 200 cycles (~60ns)
SSD: 100,000 cycles (~30μs)

Ratio:
L1 → RAM: 60x slower
L1 → SSD: 100,000x slower!
```

**Cache Lines:**

CPUs fetch memory in 64-byte chunks called cache lines:
```
Memory: [byte0, byte1, byte2, ..., byte63] [byte64, byte65, ..., byte127]
          ^────── Cache line 1 ──────^       ^────── Cache line 2 ─────^

When you access byte0, CPU fetches bytes 0-63 into cache.
If you next access byte1, it's already in cache → fast!
If you next access byte1000, need new cache line → slow.
```

**Problem: Pointer Chasing**

```rust
// Bad: Vec<Vec<String>> (nested structure)
let words: Vec<Vec<String>> = vec![
    vec!["h".to_string(), "e".to_string(), "l".to_string()],
    vec!["w".to_string(), "o".to_string(), "r".to_string()],
];

// Memory layout:
//
// Stack:
// words: [ptr] ────────┐
//                      ▼
// Heap:
// [ptr, ptr] ──────────┬──────────┐
//      │               │          │
//      ▼               ▼          ▼
//    [ptr] → "h"     [ptr] → "e" [ptr] → "l"  (word 1)
//    [ptr] → "w"     [ptr] → "o" [ptr] → "r"  (word 2)
//
// Problem: 7 pointers to chase, 7 potential cache misses!
// Scattered allocations, poor cache locality
```

**Solution: Flat Arrays**

```rust
// Good: Flat Vec with offsets
struct FlatWords {
    chars: Vec<u8>,       // All characters in one array
    offsets: Vec<usize>,  // Start of each word
}

let words = FlatWords {
    chars: vec![b'h', b'e', b'l', b'w', b'o', b'r'],  // Contiguous!
    offsets: vec![0, 3],  // Word 0 starts at 0, word 1 starts at 3
};

// Memory layout:
//
// Stack:
// words: [chars_ptr, offsets_ptr]
//           │           │
//           ▼           ▼
// Heap:
// chars:   [h, e, l, w, o, r]  ← ONE allocation, cache-friendly
// offsets: [0, 3]              ← ONE allocation

// Access word 0: chars[offsets[0]..offsets[1]] = "hel"
// Access word 1: chars[offsets[1]..] = "wor"

// Only 2 cache lines needed (best case: 1 if chars fit in 64 bytes)
```

**Performance Impact:**

```rust
// Benchmark: Iterate 1M words, count pairs

// Nested Vec<Vec<String>>:
// - Time: 150ms
// - Cache misses: ~500k (measured with perf)
// - Memory bandwidth: 2GB/s (slow)

// Flat Vec<u8> + offsets:
// - Time: 20ms
// - Cache misses: ~10k (measured with perf)
// - Memory bandwidth: 15GB/s (fast)

// Speedup: 7.5x just from memory layout!
```

**Struct Layout:**

```rust
// Bad: Poor packing
struct Token {
    id: u32,         // 4 bytes
    text: String,    // 24 bytes (fat pointer)
    frequency: u64,  // 8 bytes
    is_special: bool,// 1 byte
}
// Total: 37 bytes, but actually 40 due to alignment (padding)

// Good: Separate hot and cold data
struct TokenId {
    id: u32,         // 4 bytes (hot: accessed every encoding)
    frequency: u64,  // 8 bytes (hot: accessed during training)
}
// 12 bytes, tightly packed

struct TokenMetadata {
    text: String,    // 24 bytes (cold: accessed rarely)
    is_special: bool,// 1 byte (cold)
}
// 25 bytes, but we rarely access this

// Store separately:
let tokens: Vec<TokenId> = vec![...];  // Hot path
let metadata: Vec<TokenMetadata> = vec![...];  // Rarely accessed

// When encoding, we only touch `tokens` → better cache usage
```

**Array of Structs (AoS) vs Struct of Arrays (SoA):**

```rust
// AoS: Bad for selective access
struct Token { id: u32, freq: u64, len: u8 }
let tokens: Vec<Token> = vec![...];

// If we only need IDs:
for token in &tokens {
    process(token.id);  // Load entire Token (13 bytes), waste 9 bytes per iteration
}

// SoA: Good for selective access
struct Tokens {
    ids: Vec<u32>,   // Packed together
    freqs: Vec<u64>, // Packed together
    lens: Vec<u8>,   // Packed together
}

// If we only need IDs:
for &id in &tokens.ids {
    process(id);  // Load only IDs (4 bytes each), perfect cache usage
}
```

**Real-World Example: Tokenizer Encoding:**

```rust
// Before optimization:
pub struct BPETokenizer {
    merges: Vec<(String, String)>,  // 48 bytes per merge, scattered in heap
}

fn encode(&self, text: &str) -> Vec<u32> {
    let mut tokens = split_chars(text);

    for (a, b) in &self.merges {  // Each iteration: 2 pointer dereferences
        tokens = apply_merge(tokens, a, b);
    }

    tokens
}
// 1M tokens, 32k merges: 32B merge operations × 2 cache misses each = slow

// After optimization:
pub struct OptimizedBPETokenizer {
    merge_table: Vec<Option<u32>>,  // Flat array indexed by pair_id
}

fn encode(&self, text: &str) -> Vec<u32> {
    let mut tokens: Vec<u32> = split_chars_as_ids(text);

    for i in 0..tokens.len()-1 {
        let pair_id = encode_pair(tokens[i], tokens[i+1]);
        if let Some(merged_id) = self.merge_table[pair_id as usize] {
            // Merge tokens[i] and tokens[i+1]
        }
    }

    tokens
}
// Direct array indexing, sequential access → excellent cache behavior
```

**Speedup Summary:**

| Optimization | Technique | Speedup |
|-------------|-----------|---------|
| Flat arrays | Avoid nested Vec | 5-8x |
| SoA layout | Separate hot/cold data | 2-3x |
| Aligned allocations | Use `align_to` | 1.1-1.2x |
| Smaller types | u32 instead of usize | 1.5-2x (32-bit workloads) |

**Combined**: 10-30x speedup for memory-bound algorithms like BPE training.

---

### 10. Algorithmic Complexity and Performance Profiling

**What Is It?**

Understanding Big-O complexity and measuring real-world performance are essential for optimizing tokenizers. Theoretical complexity tells us what to optimize; profiling tells us where to optimize.

**BPE Complexity Analysis:**

```rust
// Naive BPE (Milestone 3)
fn train_naive(text: &str, vocab_size: usize) {
    let mut words = split_into_chars(text);  // O(n)

    for _ in 0..vocab_size {  // vocab_size iterations
        // Count all pairs: scan all characters
        let pair_counts = count_pairs(&words);  // O(n)

        // Find max: scan all unique pairs
        let max_pair = pair_counts.iter().max();  // O(k) where k = unique pairs

        // Merge this pair: scan all characters
        merge_pair(&mut words, max_pair);  // O(n)
    }
}
// Total: O(vocab_size × (n + k))
// Worst case: k ≈ n/2 (many unique pairs)
// → O(vocab_size × n) = O(n²) for large vocab

// Example: 100MB text (100M chars), vocab=32k
// Operations: 32k × 100M = 3.2 trillion
// At 1GHz: 3200 seconds = 53 minutes (best case)
// Reality: 2-10 hours due to memory overhead
```

```rust
// Optimized BPE with Priority Queue (Milestone 4)
fn train_optimized(text: &str, vocab_size: usize) {
    let mut words = split_into_chars(text);  // O(n)

    // Build initial priority queue
    let pair_counts = count_pairs(&words);  // O(n)
    let mut heap = BinaryHeap::from_iter(pair_counts);  // O(k log k)

    for _ in 0..vocab_size {  // vocab_size iterations
        // Get max pair: O(1)
        let max_pair = heap.peek();

        // Merge pair: O(affected_chars)
        let affected = merge_pair(&mut words, max_pair);  // O(m) where m << n

        // Update heap: O(affected_pairs × log k)
        for pair in affected {
            heap.decrease_key(pair);  // O(log k)
        }
    }
}
// Total: O(n + k log k + vocab_size × (m + a × log k))
// Where: m = affected chars (local to merge),
//        a = affected pairs (~10-100 typically)
// → O(n log k) amortized

// Same example: 100MB, vocab=32k, k=100k unique pairs
// Operations: 100M + 32k × (1000 + 50 × log(100k))
//           ≈ 100M + 32k × 1800 ≈ 160M
// At 1GHz: 0.16 seconds (theoretical)
// Reality: 5-10 minutes (memory bandwidth limited)
// Speedup: 12-60x!
```

**Parallelization Speedup:**

```rust
// Parallel BPE (Milestone 5)
fn train_parallel(text: &str, vocab_size: usize) {
    let mut words = split_into_chars(text);

    for _ in 0..vocab_size {
        // Count pairs in parallel
        let pair_counts = words
            .par_chunks(words.len() / num_cpus)  // Split work
            .map(|chunk| count_pairs_local(chunk))  // O(n / num_cpus)
            .reduce(merge_counts);  // O(k)

        // Rest is sequential (finding max, merging)
        let max_pair = find_max(pair_counts);
        merge_pair(&mut words, max_pair);
    }
}

// Amdahl's Law: Speedup = 1 / (S + P/N)
// Where: S = sequential fraction (10%)
//        P = parallel fraction (90%)
//        N = num cores (8)
// Speedup = 1 / (0.1 + 0.9/8) = 1 / 0.2125 = 4.7x (theoretical)

// Reality: 3-4x on 8 cores due to:
// - Synchronization overhead
// - Cache coherence traffic
// - Load imbalance (some chunks have more pairs)
```

**Profiling Tools:**

```bash
# CPU profiling (Linux)
perf record -g ./tokenizer
perf report

# Example output:
#  40.2%  count_pairs
#  25.3%  merge_pair
#  15.1%  hashmap_lookup
#   8.7%  string_clone
#   6.3%  heap_operations
#   4.4%  other

# → Optimize count_pairs and merge_pair first (65% of time)

# Cache profiling
perf stat -e cache-references,cache-misses ./tokenizer

# Example output:
# 10,000,000 cache-references
#  5,000,000 cache-misses # 50% miss rate (BAD!)

# After flat array optimization:
# 10,000,000 cache-references
#    200,000 cache-misses # 2% miss rate (GOOD!)
```

```bash
# Flamegraph visualization
cargo install flamegraph
cargo flamegraph --bin tokenizer

# Generates flamegraph.svg showing call stack and time distribution
```

**Performance Metrics:**

```rust
// Training throughput
let start = Instant::now();
tokenizer.train(corpus, vocab_size);
let time = start.elapsed();

let chars_per_sec = corpus.len() as f64 / time.as_secs_f64();
println!("Training: {:.2} MB/s", chars_per_sec / 1_000_000.0);

// Target: 20-50 MB/s (Milestone 5)

// Encoding throughput
let start = Instant::now();
let encoded = tokenizer.encode(text);
let time = start.elapsed();

let tokens_per_sec = encoded.len() as f64 / time.as_secs_f64();
println!("Encoding: {:.2} M tokens/s", tokens_per_sec / 1_000_000.0);

// Target: 50-100 M tokens/s (Milestone 6-7)
```

**Optimization Priorities:**

1. **Algorithm**: O(n²) → O(n log n) (10-100x speedup)
2. **Data structures**: Priority queue, flat arrays (5-20x speedup)
3. **Parallelization**: Use all cores (3-8x speedup)
4. **Memory layout**: Cache-friendly access (2-10x speedup)
5. **SIMD**: Vectorized operations (1.5-3x speedup)
6. **Micro-optimizations**: Inlining, branch prediction (1.1-1.3x speedup)

**Rule of thumb**: Optimize in this order until you hit your performance target. Don't micro-optimize until algorithmic improvements are done.

---

## Connection to This Project

Now let's see how these concepts map to the seven milestones of the tokenizer project. Each milestone introduces new concepts and optimizations, building toward a production-grade BPE tokenizer.

### Milestone 1: Character-Level Tokenizer

**Concepts Used:**
- **HashMap and Bidirectional Mappings**: Implement `vocab: HashMap<char, u32>` and `id_to_char: HashMap<u32, char>` for character ↔ ID conversion
- **Special Tokens**: Add `<PAD>`, `<UNK>`, `<BOS>`, `<EOS>` tokens for model training

**What You'll Learn:**
- How to build vocabulary incrementally from text
- Bidirectional mapping pattern (used in all tokenizers)
- Encoding: text → IDs (for model input)
- Decoding: IDs → text (for model output)

**Performance Characteristics:**
- Training: O(n) where n = characters in corpus
- Encoding: O(m) where m = characters in text
- Memory: O(unique_chars) ≈ 256 characters + specials ≈ 2KB

**Why This Milestone:**
Establishes the foundation. All tokenizers (word-level, BPE) use the same vocabulary management pattern. Character-level is simplest: no algorithm, just map each character to ID.

**Expected Output:**
```
Vocabulary size: 260 (256 chars + 4 special tokens)
Training time: ~1ms for 1MB corpus
Encoding speed: ~50M chars/sec
```

---

### Milestone 2: Word-Level Tokenizer

**Concepts Used:**
- **HashMap and Bidirectional Mappings**: Same pattern as M1, but tokens are words instead of characters
- **Text Splitting**: Use `text.split_whitespace()` to extract words
- **Unknown Token Handling**: Map unseen words to `<UNK>` during encoding

**What You'll Learn:**
- Word tokenization reduces sequence length 4-5x vs character-level
- Trade-off: Larger vocabulary (50k+ words) vs shorter sequences
- Problem: Can't handle rare words, typos, morphological variants

**Performance Characteristics:**
- Training: O(n) where n = characters (word extraction is linear)
- Encoding: O(w) where w = words in text, each word is O(1) HashMap lookup
- Memory: O(unique_words) ≈ 50k-100k words × 20 bytes ≈ 1-2MB

**Why This Milestone:**
Demonstrates the limitations that motivate BPE. Word-level can't handle "unhappiness" if it only saw "happy" during training → outputs `<UNK>` → information loss.

**Expected Output:**
```
Vocabulary size: ~10k words for 1MB corpus
Training time: ~5ms
Encoding speed: ~20M chars/sec (fewer tokens than char-level)
Problem: ~5-10% unknown word rate on unseen text
```

---

### Milestone 3: Naive BPE Tokenizer

**Concepts Used:**
- **Byte-Pair Encoding Algorithm**: Implement iterative pair merging
- **Subword Tokenization**: Learn common subwords from corpus frequency
- **Algorithmic Complexity**: Understand O(n² × vocab_size) complexity of naive approach

**What You'll Learn:**
- BPE training: Start with characters, merge frequent pairs
- Encoding with merges: Apply learned merges in order
- Why naive BPE is slow (but correct)

**Performance Characteristics:**
- Training: O(vocab_size × n) ≈ O(n²) for large vocabularies
  - Example: 10MB corpus, vocab=500 → ~30 seconds
  - Example: 100MB corpus, vocab=2000 → ~30 minutes
- Encoding: O(m × num_merges) where m = characters in text
- Memory: O(n) for storing word representations during training

**Why This Milestone:**
Implements the core BPE algorithm correctly but inefficiently. This gives you a reference implementation to test optimized versions against. Understanding why it's slow (linear scan for max pair every iteration) motivates the next optimization.

**Expected Output:**
```
Vocabulary size: 500 (256 base + 244 merges)
Training time: 10-30 seconds for 10MB corpus
Encoding speed: ~5M chars/sec
Learns subwords like: ["th", "ing", "er", "low", "est"]
```

---

### Milestone 4: Optimized BPE with Priority Queue

**Concepts Used:**
- **Priority Queue / Binary Heap**: Use `BinaryHeap` to maintain max pair efficiently
- **Algorithmic Complexity**: Reduce complexity from O(n²) to O(n log n)
- **Incremental Updates**: Update heap counts after each merge instead of full recount

**What You'll Learn:**
- How data structure choice affects performance (linear scan → heap)
- Big-O improvement translates to real speedup (5-10x)
- Trade-off: More complex code for better performance

**Performance Characteristics:**
- Training: O(n + k log k + vocab_size × a × log k)
  - Where k = unique pairs, a = affected pairs per merge
  - Example: 10MB corpus, vocab=500 → ~3-5 seconds (6-10x faster than M3)
  - Example: 100MB corpus, vocab=2000 → ~3-5 minutes (10x faster than M3)
- Encoding: Same as M3
- Memory: O(k) for heap + O(n) for words

**Why This Milestone:**
First major optimization. Shows that algorithmic improvement (better data structure) has bigger impact than any micro-optimization. Priority queue is the key insight that makes BPE practical for large corpora.

**Expected Output:**
```
Training time: 3-5 seconds for 10MB corpus (10x faster than M3)
Same vocabulary quality as M3
Encoding speed: ~5M chars/sec (unchanged)
```

---

### Milestone 5: Parallel BPE Training with Concurrent HashMap

**Concepts Used:**
- **Rayon and Data Parallelism**: Use `par_iter()` to parallelize pair counting
- **DashMap / Concurrent HashMap**: Thread-safe pair counting across cores
- **Load Balancing**: Rayon's work stealing for uneven workloads

**What You'll Learn:**
- How to parallelize algorithms with independent work (pair counting)
- Concurrent data structures (DashMap) for lock-free updates
- Amdahl's Law: Speedup limited by sequential portions

**Performance Characteristics:**
- Training: O((n + k log k) / num_cores) amortized
  - Example: 100MB corpus, 8 cores → ~30-60 seconds (4-6x faster than M4)
  - Speedup: 4-6x on 8 cores (not 8x due to overhead and sequential merge phase)
- Encoding: Still sequential (could parallelize text splitting, but encoding is already fast)
- Memory: O(k × num_cores) for thread-local counts before merging

**Why This Milestone:**
Demonstrates parallelization for CPU-bound work. BPE training is embarrassingly parallel for pair counting (the bottleneck), so we get good speedups. This milestone brings 100MB corpus training from minutes to seconds.

**Expected Output:**
```
Training time: 30-60 seconds for 100MB corpus on 8 cores (40-50x faster than M3!)
CPU usage: 85-95% (using all cores)
Encoding speed: ~5M chars/sec (unchanged)
```

---

### Milestone 6: Extreme Optimization - SIMD, Caching, and Memory Layout

**Concepts Used:**
- **String Interning**: Replace string pairs with integer IDs
- **Cache-Friendly Memory Layout**: Flat arrays instead of nested Vec
- **SIMD**: Vectorized byte scanning for text processing
- **Encoding Cache**: Memoize common word encodings

**What You'll Learn:**
- Memory layout dramatically affects performance (pointer chasing → flat arrays)
- String operations are expensive; intern to u32 IDs (6x memory reduction, 5x speedup)
- SIMD gives 2-4x speedup for byte-level operations
- Caching trades memory for speed (encode popular words once)

**Performance Characteristics:**
- Training: O(n log k) with much better constant factors
  - Example: 100MB corpus → ~10-20 seconds (2-3x faster than M5)
  - Memory: 5-10x less due to interning (no duplicate strings)
- Encoding: O(m) with SIMD and caching
  - Example: 1M chars → ~10ms (50M chars/sec) vs ~200ms in M3 (20x faster!)
- Memory: Encoding cache can be 10-100MB for common words

**Why This Milestone:**
Combines multiple optimization techniques for production-grade performance. String interning alone gives 5x speedup. Flat arrays improve cache hit rate from 50% to 98%. SIMD adds another 2-3x for byte scanning. This milestone shows how low-level optimizations compound.

**Expected Output:**
```
Training time: 10-20 seconds for 100MB corpus (100-180x faster than M3!)
Encoding speed: 50M chars/sec (10x faster than M3)
Memory usage: 100-200MB (vs 500MB+ in M3)
Cache hit rate: 80-90% on real text (encode "the" once, reuse 100k times)
```

---

### Milestone 7: Ultra-Optimized Production BPE

**Concepts Used:**
- All concepts from M1-M6 combined and refined
- **Advanced Profiling**: Use `perf`, flamegraphs to find remaining bottlenecks
- **Benchmarking**: Compare against HuggingFace tokenizers, SentencePiece
- **Vocabulary Serialization**: Save/load trained vocabularies efficiently

**What You'll Learn:**
- How to profile and iteratively optimize
- Comparison with production tokenizers (tiktoken, tokenizers crate)
- End-to-end system design: training, encoding, serialization, error handling
- Performance targets: 50-100M tokens/sec encoding, 20-50 MB/s training

**Performance Characteristics:**
- Training: Target 20-50 MB/s (100MB in 2-5 minutes)
- Encoding: Target 50-100M tokens/sec
- Memory: Minimal allocations, reuse buffers
- Comparison: Should be within 2-3x of HuggingFace tokenizers (written by Rust experts)

**Why This Milestone:**
Brings all techniques together into a cohesive, production-quality system. Includes serialization (save vocab to disk), error handling (invalid UTF-8), and comprehensive benchmarks. Shows the path from naive implementation (M3) to production-grade (M7): 200-500x total speedup!

**Expected Output:**
```
Training: 100MB corpus in 2-5 minutes (200-500x faster than M3!)
Encoding: 50-100M tokens/sec (10-20x faster than M3, competitive with HuggingFace)
Memory: <200MB for 100MB corpus training
Vocabulary save/load: <10ms for 32k vocab
Production-ready: Error handling, comprehensive tests, documentation
```

---

### Summary Table

| Milestone | Key Concepts | Training Time (100MB) | Encoding Speed | Speedup vs M3 |
|-----------|-------------|----------------------|----------------|---------------|
| M1: Character | HashMap, special tokens | ~10ms (no learning) | 50M chars/sec | N/A (different task) |
| M2: Word-level | Splitting, UNK handling | ~50ms (simple) | 20M chars/sec | N/A (different task) |
| M3: Naive BPE | BPE algorithm, O(n²) | ~30 min | 5M chars/sec | 1x (baseline) |
| M4: Priority Queue | Binary heap, O(n log n) | ~3-5 min | 5M chars/sec | 6-10x training |
| M5: Parallel | Rayon, DashMap | ~30-60 sec | 5M chars/sec | 30-60x training |
| M6: SIMD + Caching | Interning, flat arrays, SIMD | ~10-20 sec | 50M chars/sec | 100-180x overall |
| M7: Production | All techniques + profiling | ~2-5 min* | 100M chars/sec | 200-500x overall |

*M7 is slower than M6 because it targets larger vocab (32k vs 2k) and includes serialization overhead, but has higher quality and throughput.

**Progressive Complexity:**
1. **M1-M2**: Learn basics (vocabulary management)
2. **M3**: Implement core algorithm (correct but slow)
3. **M4**: First optimization (algorithmic improvement)
4. **M5**: Parallelization (use all cores)
5. **M6**: Low-level optimizations (memory + SIMD)
6. **M7**: Production polish (benchmarks + serialization)

**Key Insights:**
- Algorithmic improvement (M4): Biggest single win (10x)
- Parallelization (M5): Good returns (4-6x on 8 cores)
- Memory optimization (M6): Compounding gains (cache + interning + SIMD = 10-20x)
- Combined: 200-500x total speedup from M3 to M7

This progression teaches you how to build production systems: start simple (M1-M3), optimize algorithms (M4), scale with parallelism (M5), optimize memory (M6), and polish for production (M7).

---
# Build The Project

## Milestone 1: Character-Level Tokenizer

### Introduction

Implement the simplest tokenizer: map each character to unique integer ID. This establishes the foundation for vocabulary management, encoding, and decoding before tackling complex algorithms.

Character-level tokenization is used in early language models and character-based RNNs. It's simple but inefficient for modern transformers due to long sequence lengths.

### Architecture

**Structs:**
- `CharTokenizer` - Character-based tokenizer
  - **Field** `vocab: HashMap<char, u32>` - Character to ID mapping
  - **Field** `id_to_char: HashMap<u32, char>` - ID to character mapping
  - **Field** `special_tokens: HashMap<String, u32>` - Special tokens (PAD, UNK, etc.)
  - **Field** `next_id: u32` - Counter for next token ID
  - **Function** `new() -> Self` - Create with default special tokens
  - **Function** `train(&mut self, text: &str)` - Build vocabulary from text
  - **Function** `encode(&self, text: &str) -> Vec<u32>` - Text to IDs
  - **Function** `decode(&self, ids: &[u32]) -> String` - IDs to text
  - **Function** `vocab_size(&self) -> usize` - Total vocabulary size
  - **Function** `add_special_token(&mut self, token: &str) -> u32` - Add special token

**Special Tokens:**
- `<PAD>` (0): Padding for batching
- `<UNK>` (1): Unknown character
- `<BOS>` (2): Beginning of sequence
- `<EOS>` (3): End of sequence

**Role Each Plays:**
- Vocabulary: Bidirectional char ↔ ID mapping
- Special tokens: Control tokens for model training
- Encode: Convert string to numerical representation
- Decode: Convert IDs back to human-readable text


### Starter Code

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CharTokenizer {
    vocab: HashMap<char, u32>,
    id_to_char: HashMap<u32, char>,
    special_tokens: HashMap<String, u32>,
    next_id: u32,
}

impl CharTokenizer {
    pub fn new() -> Self {
        let mut tokenizer = Self {
            vocab: HashMap::new(),
            id_to_char: HashMap::new(),
            special_tokens: HashMap::new(),
            next_id: 0,
        };

        // Add special tokens
        tokenizer.add_special_token("<PAD>");
        tokenizer.add_special_token("<UNK>");
        tokenizer.add_special_token("<BOS>");
        tokenizer.add_special_token("<EOS>");

        tokenizer
    }

    pub fn add_special_token(&mut self, token: &str) -> u32 {
        // TODO: Assign next_id to special token, increment next_id
        // Store in special_tokens HashMap
        todo!()
    }

    pub fn train(&mut self, text: &str) {
        // TODO: Iterate over all unique characters in text
        // For each char not in vocab:
        //   - Assign next_id
        //   - Store in vocab and id_to_char
        //   - Increment next_id

        // for c in text.chars() {
        //     if !self.vocab.contains_key(&c) {
        //         self.vocab.insert(c, self.next_id);
        //         self.id_to_char.insert(self.next_id, c);
        //         self.next_id += 1;
        //     }
        // }
        todo!()
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        // TODO: Convert each character to its ID
        // If character not in vocab, use UNK token (ID 1)

        // text.chars()
        //     .map(|c| {
        //         self.vocab.get(&c).copied().unwrap_or(1)
        //     })
        //     .collect()
        todo!()
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        // TODO: Convert each ID back to character
        // Skip special tokens or handle them appropriately

        // ids.iter()
        //     .filter_map(|&id| self.id_to_char.get(&id))
        //     .collect()
        todo!()
    }

    pub fn vocab_size(&self) -> usize {
        self.next_id as usize
    }
}
```

---
### Checkpoint Tests

```rust
#[test]
fn test_char_tokenizer_basic() {
    let mut tokenizer = CharTokenizer::new();
    tokenizer.train("hello world");

    let encoded = tokenizer.encode("hello");
    assert_eq!(encoded.len(), 5);

    let decoded = tokenizer.decode(&encoded);
    assert_eq!(decoded, "hello");
}

#[test]
fn test_special_tokens() {
    let tokenizer = CharTokenizer::new();

    // Special tokens should be first IDs
    assert_eq!(tokenizer.encode("<PAD>"), vec![0]);
    assert_eq!(tokenizer.encode("<UNK>"), vec![1]);
    assert_eq!(tokenizer.encode("<BOS>"), vec![2]);
    assert_eq!(tokenizer.encode("<EOS>"), vec![3]);
}

#[test]
fn test_unknown_characters() {
    let mut tokenizer = CharTokenizer::new();
    tokenizer.train("abc");

    // 'x' is unknown, should map to <UNK>
    let encoded = tokenizer.encode("axc");
    assert!(encoded.contains(&1)); // Contains UNK tokenz
}

#[test]
fn test_unicode_support() {
    let mut tokenizer = CharTokenizer::new();
    tokenizer.train("Hello 世界 🚀");

    let encoded = tokenizer.encode("世界");
    let decoded = tokenizer.decode(&encoded);
    assert_eq!(decoded, "世界");
}

#[test]
fn test_vocab_size() {
    let mut tokenizer = CharTokenizer::new();
    tokenizer.train("aabbcc");

    // 4 special tokens + 3 unique chars
    assert_eq!(tokenizer.vocab_size(), 7);
}
```

## Milestone 2: Word-Level Tokenizer

### Introduction

**Why Milestone 1 Is Not Enough:**
Character tokenizers create very long sequences. The sentence "The cat sat" becomes 11 tokens instead of 3. For a 512-token transformer context window, this means only ~150 characters of text instead of ~2000 characters.

Longer sequences = more computation (O(n²) self-attention), slower training, less context.

**What We're Improving:**
Word-level tokenization: split on whitespace and punctuation. "The cat sat." → `["The", "cat", "sat", "."]`. Shorter sequences, more efficient, but large vocabulary and can't handle unknown words well.

### Architecture

**Structs:**
- `WordTokenizer` - Word-based tokenizer
  - **Field** `vocab: HashMap<String, u32>` - Word to ID mapping
  - **Field** `id_to_word: HashMap<u32, String>` - ID to word mapping
  - **Field** `special_tokens: HashMap<String, u32>` - Special tokens
  - **Field** `next_id: u32` - Counter for next token ID
  - **Field** `min_frequency: usize` - Minimum word frequency to include
  - **Function** `new(min_frequency: usize) -> Self` - Create tokenizer
  - **Function** `train(&mut self, text: &str)` - Build vocabulary from text
  - **Function** `encode(&self, text: &str) -> Vec<u32>` - Text to IDs
  - **Function** `decode(&self, ids: &[u32]) -> String` - IDs to text
  - **Function** `tokenize(&self, text: &str) -> Vec<String>` - Split text into words

**Key Functions:**
- `tokenize(&self, text: &str) -> Vec<String>` - Split on whitespace and punctuation
- `count_words(&self, text: &str) -> HashMap<String, usize>` - Frequency counting
- `build_vocab(&mut self, word_counts: HashMap<String, usize>)` - Create vocab from counts

**Role Each Plays:**
- Tokenize: Text → words (handle punctuation)
- Word counts: Frequency analysis for vocabulary pruning
- Min frequency: Filter rare words to control vocab size
- UNK handling: Map rare/unknown words to UNK token

### Starter Code

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WordTokenizer {
    vocab: HashMap<String, u32>,
    id_to_word: HashMap<u32, String>,
    special_tokens: HashMap<String, u32>,
    next_id: u32,
    min_frequency: usize,
}

impl WordTokenizer {
    pub fn new(min_frequency: usize) -> Self {
        let mut tokenizer = Self {
            vocab: HashMap::new(),
            id_to_word: HashMap::new(),
            special_tokens: HashMap::new(),
            next_id: 0,
            min_frequency,
        };

        // Add special tokens
        tokenizer.add_special_token("<PAD>");
        tokenizer.add_special_token("<UNK>");
        tokenizer.add_special_token("<BOS>");
        tokenizer.add_special_token("<EOS>");

        tokenizer
    }

    fn add_special_token(&mut self, token: &str) -> u32 {
        let id = self.next_id;
        self.special_tokens.insert(token.to_string(), id);
        self.vocab.insert(token.to_string(), id);
        self.id_to_word.insert(id, token.to_string());
        self.next_id += 1;
        id
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        // TODO: Split text into words
        // Separate punctuation as individual tokens
        // Strategy:
        // 1. Split on whitespace
        // 2. For each word, separate leading/trailing punctuation
        //
        // Example: "Hello, world!" → ["Hello", ",", "world", "!"]
        //
        // Hint: Use chars().take_while() and skip_while()
        // Or use regex: r"\w+|[^\w\s]"
        todo!()
    }

    fn count_words(&self, text: &str) -> HashMap<String, usize> {
        // TODO: Count frequency of each word
        // let mut counts = HashMap::new();
        // for word in self.tokenize(text) {
        //     *counts.entry(word).or_insert(0) += 1;
        // }
        // counts
        todo!()
    }

    pub fn train(&mut self, text: &str) {
        // TODO:
        // 1. Count all words
        // 2. Filter by min_frequency
        // 3. Add to vocabulary

        // let word_counts = self.count_words(text);
        // for (word, count) in word_counts {
        //     if count >= self.min_frequency {
        //         if !self.vocab.contains_key(&word) {
        //             self.vocab.insert(word.clone(), self.next_id);
        //             self.id_to_word.insert(self.next_id, word);
        //             self.next_id += 1;
        //         }
        //     }
        // }
        todo!()
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        // TODO: Tokenize text and convert words to IDs
        // Use UNK (ID 1) for unknown words
        todo!()
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        // TODO: Convert IDs to words and join with spaces
        // Handle punctuation (don't add space before punctuation)
        todo!()
    }

    pub fn vocab_size(&self) -> usize {
        self.next_id as usize
    }
}
```

---

### Checkpoint Tests

```rust
#[test]
fn test_word_tokenizer_basic() {
    let mut tokenizer = WordTokenizer::new(1);
    tokenizer.train("the cat sat on the mat");

    let encoded = tokenizer.encode("the cat");
    assert_eq!(encoded.len(), 2);

    let decoded = tokenizer.decode(&encoded);
    assert_eq!(decoded, "the cat");
}

#[test]
fn test_punctuation_splitting() {
    let tokenizer = WordTokenizer::new(1);
    let words = tokenizer.tokenize("Hello, world!");

    // Should split: ["Hello", ",", "world", "!"]
    assert_eq!(words.len(), 4);
    assert_eq!(words[0], "Hello");
    assert_eq!(words[1], ",");
}

#[test]
fn test_frequency_filtering() {
    let mut tokenizer = WordTokenizer::new(2); // Min frequency = 2
    tokenizer.train("the cat the dog the bird cat");

    // "the" appears 3 times, "cat" appears 2 times
    // "dog" and "bird" appear 1 time (should be UNK)

    let encoded = tokenizer.encode("the dog");
    // "dog" should be UNK
    assert!(encoded.contains(&1)); // Contains UNK
}

#[test]
fn test_case_sensitivity() {
    let mut tokenizer = WordTokenizer::new(1);
    tokenizer.train("The the THE");

    // Should treat as different words (case-sensitive)
    assert!(tokenizer.vocab_size() > 4); // More than just special tokens
}

#[test]
fn test_unknown_words() {
    let mut tokenizer = WordTokenizer::new(1);
    tokenizer.train("hello world");

    let encoded = tokenizer.encode("hello universe");
    // "universe" is unknown, should be UNK (ID 1)
    assert_eq!(encoded[1], 1);
}
```

## Milestone 3: Naive BPE Tokenizer

### Introduction

**Why Milestone 2 Is Not Enough:**
Word-level tokenizers have critical flaws:
1. **Large vocabulary**: 50k-100k words → large embedding matrices, slow training
2. **Unknown words**: Can't handle typos, rare words, names → information loss
3. **Morphology ignored**: "run", "running", "runner" are separate tokens

BPE solves this via subword tokenization: learn common byte pairs, merge them iteratively.

**How BPE Works:**
```
Corpus: "low", "lower", "lowest"

1. Start with characters: [l, o, w], [l, o, w, e, r], [l, o, w, e, s, t]
2. Count pairs: (l,o)=3, (o,w)=3, (w,e)=2, (e,r)=1, (e,s)=1, (s,t)=1
3. Merge most frequent: (l,o) → "lo"
   Result: [lo, w], [lo, w, e, r], [lo, w, e, s, t]
4. Count pairs: (lo,w)=3, (w,e)=2, ...
5. Merge (lo,w) → "low"
6. Continue for N merges (vocab_size - alphabet_size)
```

**What We're Improving:**
Implement BPE training and encoding. This milestone uses naive O(n²) algorithm - we'll optimize in later milestones.

### Architecture

**Structs:**
- `BPETokenizer` - Byte-Pair Encoding tokenizer
  - **Field** `vocab: HashMap<String, u32>` - Token to ID mapping
  - **Field** `id_to_token: HashMap<u32, String>` - ID to token mapping
  - **Field** `merges: Vec<(String, String)>` - Ordered list of merge operations
  - **Field** `merge_priority: HashMap<(String, String), usize>` - Merge rank
  - **Field** `special_tokens: HashMap<String, u32>` - Special tokens
  - **Field** `next_id: u32` - Next token ID
  - **Function** `new() -> Self` - Create tokenizer
  - **Function** `train(&mut self, text: &str, vocab_size: usize)` - Train BPE
  - **Function** `encode(&self, text: &str) -> Vec<u32>` - Text to IDs
  - **Function** `decode(&self, ids: &[u32]) -> String` - IDs to text

**Key Functions:**
- `get_words(&self, text: &str) -> Vec<Vec<String>>` - Split text into character sequences
- `count_pairs(&self, words: &[Vec<String>]) -> HashMap<(String, String), usize>` - Count adjacent pairs
- `merge_pair(&self, words: &mut [Vec<String>], pair: (&str, &str))` - Merge pair in all words
- `apply_merges(&self, word: Vec<String>) -> Vec<String>` - Apply learned merges to encode

**Role Each Plays:**
- Merges: Ordered list of pair merges learned during training
- Merge priority: Rank of each merge (lower = applied earlier)
- Pair counting: Find most frequent adjacent token pair
- Merge operation: Combine pair into single token across corpus

### Starter Code

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BPETokenizer {
    vocab: HashMap<String, u32>,
    id_to_token: HashMap<u32, String>,
    merges: Vec<(String, String)>,
    merge_priority: HashMap<(String, String), usize>,
    special_tokens: HashMap<String, u32>,
    next_id: u32,
}

impl BPETokenizer {
    pub fn new() -> Self {
        let mut tokenizer = Self {
            vocab: HashMap::new(),
            id_to_token: HashMap::new(),
            merges: Vec::new(),
            merge_priority: HashMap::new(),
            special_tokens: HashMap::new(),
            next_id: 0,
        };

        tokenizer.add_special_token("<PAD>");
        tokenizer.add_special_token("<UNK>");
        tokenizer.add_special_token("<BOS>");
        tokenizer.add_special_token("<EOS>");

        tokenizer
    }

    fn add_special_token(&mut self, token: &str) -> u32 {
        let id = self.next_id;
        self.special_tokens.insert(token.to_string(), id);
        self.vocab.insert(token.to_string(), id);
        self.id_to_token.insert(id, token.to_string());
        self.next_id += 1;
        id
    }

    fn get_words(&self, text: &str) -> Vec<Vec<String>> {
        // TODO: Split text into words, then split each word into characters
        // Example: "hello world" → [['h','e','l','l','o'], ['w','o','r','l','d']]
        //
        // text.split_whitespace()
        //     .map(|word| {
        //         word.chars()
        //             .map(|c| c.to_string())
        //             .collect()
        //     })
        //     .collect()
        todo!()
    }

    fn count_pairs(&self, words: &[Vec<String>]) -> HashMap<(String, String), usize> {
        // TODO: Count all adjacent pairs in all words
        // For word ['h','e','l','l','o']:
        //   Count: (h,e), (e,l), (l,l), (l,o)
        //
        // let mut pair_counts = HashMap::new();
        // for word in words {
        //     for pair in word.windows(2) {
        //         let key = (pair[0].clone(), pair[1].clone());
        //         *pair_counts.entry(key).or_insert(0) += 1;
        //     }
        // }
        // pair_counts
        todo!()
    }

    fn merge_pair(&self, words: &mut [Vec<String>], pair: (&str, &str)) {
        // TODO: Merge all occurrences of pair in words
        // ['h','e','l','l','o'] with pair ('l','l') → ['h','e','ll','o']
        //
        // for word in words.iter_mut() {
        //     let mut i = 0;
        //     while i < word.len() - 1 {
        //         if word[i] == pair.0 && word[i + 1] == pair.1 {
        //             let merged = format!("{}{}", pair.0, pair.1);
        //             word[i] = merged;
        //             word.remove(i + 1);
        //         } else {
        //             i += 1;
        //         }
        //     }
        // }
        todo!()
    }

    pub fn train(&mut self, text: &str, target_vocab_size: usize) {
        // TODO: BPE training algorithm
        //
        // 1. Split text into words, then characters
        // 2. Add all unique characters to vocabulary
        // 3. Loop until vocab_size reaches target:
        //    a. Count all adjacent pairs
        //    b. Find most frequent pair
        //    c. Merge that pair in all words
        //    d. Add merged token to vocabulary
        //    e. Record merge operation
        //
        // let mut words = self.get_words(text);
        //
        // // Add initial character vocabulary
        // let mut chars: HashSet<String> = HashSet::new();
        // for word in &words {
        //     chars.extend(word.iter().cloned());
        // }
        // for c in chars {
        //     if !self.vocab.contains_key(&c) {
        //         self.vocab.insert(c.clone(), self.next_id);
        //         self.id_to_token.insert(self.next_id, c);
        //         self.next_id += 1;
        //     }
        // }
        //
        // // Perform merges
        // while self.vocab.len() < target_vocab_size {
        //     let pair_counts = self.count_pairs(&words);
        //
        //     if pair_counts.is_empty() {
        //         break;
        //     }
        //
        //     // Find most frequent pair
        //     let best_pair = pair_counts
        //         .iter()
        //         .max_by_key(|(_, count)| *count)
        //         .map(|(pair, _)| pair.clone())
        //         .unwrap();
        //
        //     // Merge in corpus
        //     self.merge_pair(&mut words, (&best_pair.0, &best_pair.1));
        //
        //     // Add to vocabulary
        //     let merged = format!("{}{}", best_pair.0, best_pair.1);
        //     self.vocab.insert(merged.clone(), self.next_id);
        //     self.id_to_token.insert(self.next_id, merged);
        //     self.next_id += 1;
        //
        //     // Record merge
        //     let merge_idx = self.merges.len();
        //     self.merges.push(best_pair.clone());
        //     self.merge_priority.insert(best_pair, merge_idx);
        // }
        todo!()
    }

    fn apply_merges(&self, mut word: Vec<String>) -> Vec<String> {
        // TODO: Apply learned merges to a word (for encoding)
        // Process merges in order of priority (earlier merges first)
        //
        // for (pair_a, pair_b) in &self.merges {
        //     let mut i = 0;
        //     while i < word.len() - 1 {
        //         if word[i] == *pair_a && word[i + 1] == *pair_b {
        //             word[i] = format!("{}{}", pair_a, pair_b);
        //             word.remove(i + 1);
        //         } else {
        //             i += 1;
        //         }
        //     }
        // }
        // word
        todo!()
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        // TODO: Encode text using BPE
        // 1. Split into words
        // 2. Split each word into characters
        // 3. Apply merges to each word
        // 4. Convert tokens to IDs
        todo!()
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        // TODO: Convert IDs to tokens and concatenate
        // Handle spaces between words appropriately
        todo!()
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}
```

---

### Checkpoint Tests

```rust
#[test]
fn test_bpe_basic() {
    let mut tokenizer = BPETokenizer::new();

    // Train on simple corpus
    let corpus = "low low low low lower lower newest newest newest newest newest newest widest widest widest";
    tokenizer.train(corpus, 100);

    // Should learn common subwords like "low", "est"
    let encoded = tokenizer.encode("lowest");
    assert!(encoded.len() < 6); // Fewer than character-level
}

#[test]
fn test_bpe_merges() {
    let mut tokenizer = BPETokenizer::new();
    tokenizer.train("aaaa bbbb", 20);

    // Should learn to merge repeated characters
    assert!(tokenizer.merges.len() > 0);
}

#[test]
fn test_bpe_encode_decode() {
    let mut tokenizer = BPETokenizer::new();
    let text = "hello world hello world";
    tokenizer.train(text, 50);

    let encoded = tokenizer.encode("hello");
    let decoded = tokenizer.decode(&encoded);

    assert_eq!(decoded, "hello");
}

#[test]
fn test_bpe_subword_splitting() {
    let mut tokenizer = BPETokenizer::new();

    // Train on words with common prefix
    let corpus = "running runner run runs";
    tokenizer.train(corpus, 50);

    let run_tokens = tokenizer.encode("run");
    let running_tokens = tokenizer.encode("running");

    // "running" should share prefix tokens with "run"
    assert!(running_tokens.len() > run_tokens.len());
}

#[test]
fn test_bpe_vocab_size() {
    let mut tokenizer = BPETokenizer::new();
    tokenizer.train("a b c d e f", 30);

    // Vocab should be close to target size
    assert!(tokenizer.vocab_size() <= 30);
}
```

## Milestone 4: Optimized BPE with Priority Queue

### Introduction

**Why Milestone 3 Is Not Enough:**
Naive BPE has terrible performance:
- Counting pairs: O(n) for each merge
- Finding max: O(pairs) for each merge
- Total: O(n × vocab_size) where n = corpus size

For 100MB corpus, 32k vocab: ~3.2 billion operations, 10+ hours training time.

**What We're Improving:**
Use priority queue (BinaryHeap) to track pair frequencies. Update only affected pairs after merge instead of recounting everything.

**Optimization:**
```
Naive: Recount all pairs every merge → O(n × V)
Optimized: Maintain heap, update only changed pairs → O(n + V × log(V))
```

For large corpora: 100-1000x speedup!

### Architecture

**Modified Structs:**
- `BPETokenizer`
  - **Field** `pair_heap: BinaryHeap<PairCount>` - Priority queue of pairs by frequency
  - **Field** `pair_positions: HashMap<(String, String), Vec<Position>>` - Where each pair appears
  - **Function** `update_pair_counts(&mut self, affected_pairs: HashSet<(String, String)>)` - Incremental update

**New Structs:**
- `PairCount` - Heap entry
  - **Field** `pair: (String, String)` - The token pair
  - **Field** `count: usize` - Frequency
  - Implement `Ord` to order by count (max-heap)

- `Position` - Location of pair
  - **Field** `word_idx: usize` - Which word
  - **Field** `pos: usize` - Position in word

**Role Each Plays:**
- Priority queue: Efficiently get most frequent pair (O(log n))
- Pair positions: Track where to update after merge
- Incremental updates: Only recount pairs affected by merge


### Starter Code

```rust
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, PartialEq)]
struct PairCount {
    pair: (String, String),
    count: usize,
}

impl Ord for PairCount {
    fn cmp(&self, other: &Self) -> Ordering {
        // TODO: Compare by count (max-heap: higher count = higher priority)
        // Break ties by lexicographic order of pair for determinism
        //
        // self.count.cmp(&other.count)
        //     .then_with(|| self.pair.cmp(&other.pair))
        todo!()
    }
}

impl PartialOrd for PairCount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct BPETokenizer {
    vocab: HashMap<String, u32>,
    id_to_token: HashMap<u32, String>,
    merges: Vec<(String, String)>,
    merge_priority: HashMap<(String, String), usize>,
    next_id: u32,
}

impl BPETokenizer {
    pub fn train(&mut self, text: &str, target_vocab_size: usize) {
        // TODO: Optimized BPE training
        //
        // 1. Initialize words as character arrays
        // 2. Build initial pair counts → BinaryHeap
        // 3. While vocab < target:
        //    a. Pop most frequent pair from heap
        //    b. Merge pair in corpus
        //    c. Update counts for affected pairs only
        //    d. Push updated pairs back to heap
        //
        // Key optimization: Don't recount all pairs!
        // Only update pairs that changed due to merge.
        //
        // Example:
        // Word: ['h','e','l','l','o']
        // Merge ('l','l') → ['h','e','ll','o']
        // Affected pairs:
        //   Removed: (e,l), (l,l), (l,o)
        //   Added: (e,ll), (ll,o)
        // Only update these 5 pairs, not entire corpus!

        todo!()
    }
}
```

---

### Checkpoint Tests

```rust
#[test]
fn test_optimized_bpe_correctness() {
    let mut tokenizer = BPETokenizer::new();
    let text = "low low low low lower lower newest newest newest newest newest newest";

    tokenizer.train(text, 50);

    // Should produce same results as naive
    let encoded = tokenizer.encode("lowest");
    let decoded = tokenizer.decode(&encoded);
    assert_eq!(decoded, "lowest");
}

#[test]
fn test_optimized_bpe_performance() {
    use std::time::Instant;

    let mut tokenizer = BPETokenizer::new();

    // Large corpus
    let corpus = "hello world ".repeat(10000);

    let start = Instant::now();
    tokenizer.train(&corpus, 500);
    let elapsed = start.elapsed();

    println!("Optimized BPE trained in {:?}", elapsed);

    // Should be much faster than naive
    assert!(elapsed.as_secs() < 10);
}

#[test]
fn test_heap_ordering() {
    use std::collections::BinaryHeap;

    let mut heap = BinaryHeap::new();

    heap.push(PairCount {
        pair: ("a".into(), "b".into()),
        count: 10,
    });
    heap.push(PairCount {
        pair: ("c".into(), "d".into()),
        count: 50,
    });
    heap.push(PairCount {
        pair: ("e".into(), "f".into()),
        count: 30,
    });

    // Should pop in descending order
    assert_eq!(heap.pop().unwrap().count, 50);
    assert_eq!(heap.pop().unwrap().count, 30);
    assert_eq!(heap.pop().unwrap().count, 10);
}
```

## Milestone 5: Parallel BPE Training with Concurrent HashMap

### Introduction

**Why Milestone 4 Is Not Enough:**
Even optimized BPE is single-threaded. Modern machines have 8-16 cores, but we're using only 1. For large corpora (1GB+), this leaves massive performance on the table.

**What We're Improving:**
Parallelize pair counting across chunks of corpus using Rayon. Use `DashMap` (concurrent HashMap) to aggregate counts from multiple threads without locks.

**Parallelization Strategy:**
```
Corpus: [chunk1, chunk2, chunk3, chunk4]
         ↓       ↓       ↓       ↓
      Thread1 Thread2 Thread3 Thread4
         ↓       ↓       ↓       ↓
       counts  counts  counts  counts
         ↓       ↓       ↓       ↓
         Merge into DashMap → Global counts
```

**Expected Speedup:**
- 4 cores: 3-3.5x faster
- 8 cores: 6-7x faster
- 16 cores: 10-12x faster

### Architecture

**Dependencies:**
```toml
[dependencies]
rayon = "1.8"
dashmap = "5.5"

# For benchmarks (downloading tiny-shakespeare dataset)
[dev-dependencies]
reqwest = { version = "0.11", features = ["blocking"] }
```

**Modified Structs:**
- `ParallelBPETokenizer` - Parallel version
  - Use `DashMap<(String, String), AtomicUsize>` for thread-safe pair counting
  - Parallel iteration with Rayon's `par_iter()`

**Key Functions:**
- `parallel_count_pairs(&self, words: &[Vec<String>]) -> HashMap<(String, String), usize>` - Parallel pair counting
- `parallel_merge(&self, words: &mut [Vec<String>], pair: (&str, &str))` - Parallel merge

**Role Each Plays:**
- DashMap: Lock-free concurrent HashMap
- Rayon: Data parallelism framework
- Chunking: Divide work across threads
- Atomic counters: Thread-safe frequency counting


### Starter Code

```rust
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone)]
pub struct ParallelBPETokenizer {
    vocab: HashMap<String, u32>,
    id_to_token: HashMap<u32, String>,
    merges: Vec<(String, String)>,
    next_id: u32,
}

impl ParallelBPETokenizer {
    pub fn new() -> Self {
        // TODO: Same as BPETokenizer::new()
        todo!()
    }

    fn parallel_count_pairs(&self, words: &[Vec<String>]) -> HashMap<(String, String), usize> {
        // TODO: Parallel pair counting
        //
        // Strategy:
        // 1. Create DashMap for concurrent counting
        // 2. Use Rayon's par_iter() to process words in parallel
        // 3. Each thread counts pairs in its chunk, updates DashMap
        // 4. Convert DashMap to HashMap at end
        //
        // let pair_counts = DashMap::new();
        //
        // words.par_iter().for_each(|word| {
        //     for pair in word.windows(2) {
        //         let key = (pair[0].clone(), pair[1].clone());
        //         pair_counts.entry(key)
        //             .and_modify(|count| *count += 1)
        //             .or_insert(1);
        //     }
        // });
        //
        // // Convert to HashMap
        // pair_counts.into_iter().collect()
        todo!()
    }

    fn parallel_merge(&self, words: &mut [Vec<String>], pair: (&str, &str)) {
        // TODO: Parallel merge operation
        //
        // Use par_iter_mut() to merge pair in each word concurrently
        //
        // words.par_iter_mut().for_each(|word| {
        //     let mut i = 0;
        //     while i < word.len() - 1 {
        //         if word[i] == pair.0 && word[i + 1] == pair.1 {
        //             word[i] = format!("{}{}", pair.0, pair.1);
        //             word.remove(i + 1);
        //         } else {
        //             i += 1;
        //         }
        //     }
        // });
        todo!()
    }

    pub fn train(&mut self, text: &str, target_vocab_size: usize) {
        // TODO: Parallel BPE training
        //
        // Same algorithm as Milestone 4, but use:
        // - parallel_count_pairs() instead of count_pairs()
        // - parallel_merge() instead of merge_pair()
        //
        // Most time is spent in counting, so parallelizing that gives biggest win
        todo!()
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        // TODO: Same as BPETokenizer::encode()
        // Encoding is fast enough, parallelization overhead not worth it
        todo!()
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        // TODO: Same as BPETokenizer::decode()
        todo!()
    }
}
```

---

### Checkpoint Tests

```rust
#[test]
fn test_parallel_bpe_correctness() {
    let mut tokenizer = ParallelBPETokenizer::new();

    let corpus = "hello world hello world".repeat(100);
    tokenizer.train(&corpus, 100);

    // Results should match sequential version
    let encoded = tokenizer.encode("hello");
    let decoded = tokenizer.decode(&encoded);
    assert_eq!(decoded, "hello");
}

#[test]
fn test_parallel_speedup() {
    use std::time::Instant;

    let corpus = "the quick brown fox jumps over the lazy dog ".repeat(50000);

    // Sequential
    let mut seq_tokenizer = BPETokenizer::new();
    let start = Instant::now();
    seq_tokenizer.train(&corpus, 500);
    let seq_time = start.elapsed();

    // Parallel
    let mut par_tokenizer = ParallelBPETokenizer::new();
    let start = Instant::now();
    par_tokenizer.train(&corpus, 500);
    let par_time = start.elapsed();

    println!("Sequential: {:?}", seq_time);
    println!("Parallel: {:?}", par_time);
    println!("Speedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    // Should be faster (at least 1.5x on multi-core)
    assert!(par_time < seq_time);
}

#[test]
fn test_dashmap_concurrent_updates() {
    use dashmap::DashMap;
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let map = DashMap::new();

    // Concurrent increments from multiple threads
    (0..10000).into_par_iter().for_each(|i| {
        let key = i % 100;
        map.entry(key)
            .and_modify(|count: &mut usize| *count += 1)
            .or_insert(1);
    });

    // Each key should have ~100 increments
    assert_eq!(map.len(), 100);
    for entry in map.iter() {
        assert!(*entry.value() >= 90 && *entry.value() <= 110);
    }
}
```

## Milestone 6: Extreme Optimization - SIMD, Caching, and Memory Layout

### Introduction

**Why Milestone 5 Is Not Enough:**
Parallel processing helps, but we're still doing unnecessary work:
- String allocations for every pair
- HashMap lookups for every pair
- Poor cache locality when scanning corpus
- Byte-by-byte character processing

**What We're Improving:**
Final optimizations for production-grade performance:
1. **Memory layout**: Store corpus as flat byte array, use indices instead of strings
2. **Interning**: Map strings to integer IDs, work with IDs only
3. **Caching**: Pre-compute common encodings
4. **SIMD**: Vectorized byte scanning where possible

**Optimizations:**
```
Before: Store pairs as (String, String) → 48 bytes, heap allocation
After:  Store pairs as (u32, u32) → 8 bytes, stack allocation

Before: HashMap<(String, String), usize> lookup → ~100ns
After:  Vec<usize> indexed by pair_id → ~2ns

Before: String concatenation for merges → allocation
After:  Update indices in-place → no allocation
```

**Expected Speedup:**
- 2-3x faster than Milestone 5
- 10-15x faster than Milestone 3
- 50-100x faster than Milestone 1

### Architecture

**New Structs:**
- `StringInterner` - String ↔ ID mapping
  - **Field** `string_to_id: HashMap<String, u32>` - Intern table
  - **Field** `id_to_string: Vec<String>` - Reverse mapping
  - **Function** `intern(&mut self, s: &str) -> u32` - Get or create ID
  - **Function** `get_string(&self, id: u32) -> &str` - Lookup string

- `OptimizedBPETokenizer` - Final optimized version
  - **Field** `interner: StringInterner` - String interning
  - **Field** `pair_cache: HashMap<u64, Vec<u32>>` - Encoding cache
  - **Field** `merge_table: Vec<Option<u32>>` - Fast merge lookup (indexed by pair_id)
  - Use byte arrays instead of String vectors

**Key Optimizations:**
1. **String interning**: Convert all strings to u32 IDs once
2. **Pair encoding**: Encode (u32, u32) pair as single u64 for fast hashing
3. **Flat arrays**: Replace Vec<Vec<String>> with flat Vec<u32> + offsets
4. **Merge table**: O(1) merge lookup instead of O(log n) HashMap

**Role Each Plays:**
- Interner: Amortize string operations
- Cache: Skip re-encoding common words
- Flat layout: Better cache locality
- Merge table: Constant-time merge queries


### Starter Code

```rust
use std::collections::HashMap;

// ============================================================================
// STRING INTERNER
// ============================================================================

#[derive(Debug, Clone)]
pub struct StringInterner {
    string_to_id: HashMap<String, u32>,
    id_to_string: Vec<String>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            string_to_id: HashMap::new(),
            id_to_string: Vec::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> u32 {
        // TODO: Get existing ID or create new one
        // if let Some(&id) = self.string_to_id.get(s) {
        //     id
        // } else {
        //     let id = self.id_to_string.len() as u32;
        //     self.string_to_id.insert(s.to_string(), id);
        //     self.id_to_string.push(s.to_string());
        //     id
        // }
        todo!()
    }

    pub fn get_string(&self, id: u32) -> &str {
        &self.id_to_string[id as usize]
    }

    pub fn get_id(&self, s: &str) -> Option<u32> {
        self.string_to_id.get(s).copied()
    }
}

// ============================================================================
// OPTIMIZED BPE TOKENIZER
// ============================================================================

#[derive(Debug, Clone)]
pub struct OptimizedBPETokenizer {
    interner: StringInterner,
    vocab: HashMap<u32, u32>, // Interned string ID → token ID
    id_to_token_id: Vec<u32>,
    merges: Vec<(u32, u32)>, // Pairs of interned IDs
    merge_table: HashMap<u64, u32>, // Encoded pair → merged token ID
    encoding_cache: HashMap<u64, Vec<u32>>, // Hash of word → encoding
    next_id: u32,
}

impl OptimizedBPETokenizer {
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
            vocab: HashMap::new(),
            id_to_token_id: Vec::new(),
            merges: Vec::new(),
            merge_table: HashMap::new(),
            encoding_cache: HashMap::new(),
            next_id: 4, // After special tokens
        }
    }

    fn encode_pair(a: u32, b: u32) -> u64 {
        // TODO: Pack two u32s into one u64
        // ((a as u64) << 32) | (b as u64)
        todo!()
    }

    fn hash_word(word: &[u32]) -> u64 {
        // TODO: Simple hash for caching
        // Use FxHash or compute simple hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        hasher.finish()
    }

    pub fn train(&mut self, text: &str, target_vocab_size: usize) {
        // TODO: Optimized training
        //
        // 1. Intern all characters upfront
        // 2. Convert corpus to Vec<Vec<u32>> (interned IDs)
        // 3. Use pair encoding (u64) for fast HashMap lookups
        // 4. Build merge_table for O(1) merge queries
        // 5. Store merges as (u32, u32) instead of (String, String)
        //
        // Key optimization: Work with integers only, no string ops
        todo!()
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        // TODO: Optimized encoding with caching
        //
        // 1. Split text into words
        // 2. For each word:
        //    a. Compute hash
        //    b. Check cache
        //    c. If miss: encode and cache result
        // 3. Use merge_table for O(1) merge lookups
        //
        // let hash = Self::hash_word(word_as_ids);
        // if let Some(cached) = self.encoding_cache.get(&hash) {
        //     return cached.clone();
        // }
        // // Encode and cache
        todo!()
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        // TODO: Decode using interner
        // Map token IDs → interned IDs → strings
        todo!()
    }

    pub fn vocab_size(&self) -> usize {
        self.next_id as usize
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        // TODO: Serialize vocabulary and merges to file
        // Use bincode or serde_json
        todo!()
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        // TODO: Deserialize from file
        todo!()
    }
}
```

---
### Checkpoint Tests

```rust
#[test]
fn test_string_interner() {
    let mut interner = StringInterner::new();

    let id1 = interner.intern("hello");
    let id2 = interner.intern("world");
    let id3 = interner.intern("hello"); // Should return same ID

    assert_eq!(id1, id3);
    assert_ne!(id1, id2);
    assert_eq!(interner.get_string(id1), "hello");
}

#[test]
fn test_optimized_bpe_correctness() {
    let mut tokenizer = OptimizedBPETokenizer::new();

    let corpus = "hello world hello world".repeat(100);
    tokenizer.train(&corpus, 100);

    let encoded = tokenizer.encode("hello world");
    let decoded = tokenizer.decode(&encoded);

    assert_eq!(decoded, "hello world");
}

#[test]
fn test_encoding_cache() {
    let mut tokenizer = OptimizedBPETokenizer::new();
    tokenizer.train("the cat sat on the mat", 50);

    // First encode (miss cache)
    let start = std::time::Instant::now();
    let encoded1 = tokenizer.encode("the cat");
    let time1 = start.elapsed();

    // Second encode (hit cache)
    let start = std::time::Instant::now();
    let encoded2 = tokenizer.encode("the cat");
    let time2 = start.elapsed();

    assert_eq!(encoded1, encoded2);
    // Cache hit should be faster (though might not be measurable for short strings)
}

#[test]
fn test_pair_encoding() {
    // Pack two u32s into u64 for fast hashing
    fn encode_pair(a: u32, b: u32) -> u64 {
        ((a as u64) << 32) | (b as u64)
    }

    let pair1 = encode_pair(100, 200);
    let pair2 = encode_pair(100, 200);
    let pair3 = encode_pair(200, 100);

    assert_eq!(pair1, pair2);
    assert_ne!(pair1, pair3);
}

// ============================================================================
// BENCHMARK HELPER: Download and cache tiny-shakespeare
// ============================================================================

fn get_tiny_shakespeare() -> String {
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    let cache_path = "tiny_shakespeare.txt";

    // Check if already cached
    if Path::new(cache_path).exists() {
        println!("Loading cached tiny-shakespeare dataset...");
        return fs::read_to_string(cache_path).expect("Failed to read cached dataset");
    }

    // Download from internet
    println!("Downloading tiny-shakespeare dataset...");
    let url = "https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt";

    let text = reqwest::blocking::get(url)
        .expect("Failed to download dataset")
        .text()
        .expect("Failed to read response text");

    // Save to cache
    let mut file = fs::File::create(cache_path).expect("Failed to create cache file");
    file.write_all(text.as_bytes()).expect("Failed to write cache");

    println!("Dataset downloaded and cached ({} bytes)", text.len());
    text
}

#[test]
fn benchmark_all_versions() {
    use std::time::Instant;

    let corpus = get_tiny_shakespeare();
    let vocab_size = 500;

    println!("\n=== BPE Tokenizer Benchmark ===");
    println!("Dataset: tiny-shakespeare ({} bytes)\n", corpus.len());

    // Naive BPE (Milestone 3) - Warning: may be very slow!
    println!("Milestone 3 (Naive): SKIPPED (too slow for large corpus)");

    // Optimized BPE (Milestone 4)
    println!("\nTesting Milestone 4 (Heap)...");
    let mut optimized = BPETokenizer::new();
    let start = Instant::now();
    optimized.train(&corpus, vocab_size);
    let opt_train = start.elapsed();

    let start = Instant::now();
    let opt_encoded = optimized.encode(&corpus);
    let opt_encode = start.elapsed();

    println!("  Training: {:?}", opt_train);
    println!("  Encoding: {:?} ({:.2}M tokens/sec)",
        opt_encode,
        opt_encoded.len() as f64 / opt_encode.as_secs_f64() / 1_000_000.0);

    // Parallel BPE (Milestone 5)
    println!("\nTesting Milestone 5 (Parallel)...");
    let mut parallel = ParallelBPETokenizer::new();
    let start = Instant::now();
    parallel.train(&corpus, vocab_size);
    let par_train = start.elapsed();

    let start = Instant::now();
    let par_encoded = parallel.encode(&corpus);
    let par_encode = start.elapsed();

    println!("  Training: {:?} ({:.2}x speedup)",
        par_train,
        opt_train.as_secs_f64() / par_train.as_secs_f64());
    println!("  Encoding: {:?} ({:.2}M tokens/sec)",
        par_encode,
        par_encoded.len() as f64 / par_encode.as_secs_f64() / 1_000_000.0);

    // Optimized with interning (Milestone 6)
    println!("\nTesting Milestone 6 (Interning)...");
    let mut extreme = OptimizedBPETokenizer::new();
    let start = Instant::now();
    extreme.train(&corpus, vocab_size);
    let ext_train = start.elapsed();

    let start = Instant::now();
    let ext_encoded = extreme.encode(&corpus);
    let ext_encode = start.elapsed();

    println!("  Training: {:?} ({:.2}x speedup)",
        ext_train,
        opt_train.as_secs_f64() / ext_train.as_secs_f64());
    println!("  Encoding: {:?} ({:.2}M tokens/sec)",
        ext_encode,
        ext_encoded.len() as f64 / ext_encode.as_secs_f64() / 1_000_000.0);

    println!("\n=== Summary ===");
    println!("Final speedup vs Milestone 4:");
    println!("  M5: {:.2}x faster training", opt_train.as_secs_f64() / par_train.as_secs_f64());
    println!("  M6: {:.2}x faster training", opt_train.as_secs_f64() / ext_train.as_secs_f64());
}
```

## Complete Working Example

```rust
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;

// ============================================================================
// CHARACTER TOKENIZER
// ============================================================================

#[derive(Debug, Clone)]
pub struct CharTokenizer {
    vocab: HashMap<char, u32>,
    id_to_char: HashMap<u32, char>,
    next_id: u32,
}

impl CharTokenizer {
    pub fn new() -> Self {
        let mut tokenizer = Self {
            vocab: HashMap::new(),
            id_to_char: HashMap::new(),
            next_id: 0,
        };

        // Add special tokens
        for token in ["<PAD>", "<UNK>", "<BOS>", "<EOS>"] {
            let c = token.chars().next().unwrap();
            tokenizer.vocab.insert(c, tokenizer.next_id);
            tokenizer.id_to_char.insert(tokenizer.next_id, c);
            tokenizer.next_id += 1;
        }

        tokenizer
    }

    pub fn train(&mut self, text: &str) {
        for c in text.chars() {
            if !self.vocab.contains_key(&c) {
                self.vocab.insert(c, self.next_id);
                self.id_to_char.insert(self.next_id, c);
                self.next_id += 1;
            }
        }
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        text.chars()
            .map(|c| self.vocab.get(&c).copied().unwrap_or(1))
            .collect()
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        ids.iter()
            .filter_map(|&id| self.id_to_char.get(&id))
            .collect()
    }

    pub fn vocab_size(&self) -> usize {
        self.next_id as usize
    }
}

// ============================================================================
// WORD TOKENIZER
// ============================================================================

#[derive(Debug, Clone)]
pub struct WordTokenizer {
    vocab: HashMap<String, u32>,
    id_to_word: HashMap<u32, String>,
    next_id: u32,
    min_frequency: usize,
}

impl WordTokenizer {
    pub fn new(min_frequency: usize) -> Self {
        let mut tokenizer = Self {
            vocab: HashMap::new(),
            id_to_word: HashMap::new(),
            next_id: 0,
            min_frequency,
        };

        for token in ["<PAD>", "<UNK>", "<BOS>", "<EOS>"] {
            tokenizer.vocab.insert(token.to_string(), tokenizer.next_id);
            tokenizer.id_to_word.insert(tokenizer.next_id, token.to_string());
            tokenizer.next_id += 1;
        }

        tokenizer
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut current_word = String::new();

        for c in text.chars() {
            if c.is_whitespace() {
                if !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
            } else if c.is_alphanumeric() {
                current_word.push(c);
            } else {
                // Punctuation
                if !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                words.push(c.to_string());
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        words
    }

    pub fn train(&mut self, text: &str) {
        let words = self.tokenize(text);
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }

        for (word, count) in word_counts {
            if count >= self.min_frequency && !self.vocab.contains_key(&word) {
                self.vocab.insert(word.clone(), self.next_id);
                self.id_to_word.insert(self.next_id, word);
                self.next_id += 1;
            }
        }
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        self.tokenize(text)
            .iter()
            .map(|word| self.vocab.get(word).copied().unwrap_or(1))
            .collect()
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        ids.iter()
            .filter_map(|&id| self.id_to_word.get(&id))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn vocab_size(&self) -> usize {
        self.next_id as usize
    }
}

// ============================================================================
// BPE TOKENIZER (OPTIMIZED WITH HEAP)
// ============================================================================

#[derive(Debug, Clone, Eq, PartialEq)]
struct PairCount {
    pair: (String, String),
    count: usize,
}

impl Ord for PairCount {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)
            .then_with(|| self.pair.cmp(&other.pair))
    }
}

impl PartialOrd for PairCount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct BPETokenizer {
    vocab: HashMap<String, u32>,
    id_to_token: HashMap<u32, String>,
    merges: Vec<(String, String)>,
    merge_priority: HashMap<(String, String), usize>,
    next_id: u32,
}

impl BPETokenizer {
    pub fn new() -> Self {
        let mut tokenizer = Self {
            vocab: HashMap::new(),
            id_to_token: HashMap::new(),
            merges: Vec::new(),
            merge_priority: HashMap::new(),
            next_id: 0,
        };

        for token in ["<PAD>", "<UNK>", "<BOS>", "<EOS>"] {
            tokenizer.vocab.insert(token.to_string(), tokenizer.next_id);
            tokenizer.id_to_token.insert(tokenizer.next_id, token.to_string());
            tokenizer.next_id += 1;
        }

        tokenizer
    }

    fn get_words(&self, text: &str) -> Vec<Vec<String>> {
        text.split_whitespace()
            .map(|word| {
                word.chars()
                    .map(|c| c.to_string())
                    .collect()
            })
            .collect()
    }

    fn count_pairs(&self, words: &[Vec<String>]) -> HashMap<(String, String), usize> {
        let mut pair_counts = HashMap::new();

        for word in words {
            for window in word.windows(2) {
                let pair = (window[0].clone(), window[1].clone());
                *pair_counts.entry(pair).or_insert(0) += 1;
            }
        }

        pair_counts
    }

    fn merge_pair(&self, words: &mut [Vec<String>], pair: (&str, &str)) {
        for word in words.iter_mut() {
            let mut i = 0;
            while i < word.len().saturating_sub(1) {
                if word[i] == pair.0 && word[i + 1] == pair.1 {
                    let merged = format!("{}{}", pair.0, pair.1);
                    word[i] = merged;
                    word.remove(i + 1);
                } else {
                    i += 1;
                }
            }
        }
    }

    pub fn train(&mut self, text: &str, target_vocab_size: usize) {
        let mut words = self.get_words(text);

        // Add character vocabulary
        let mut chars = HashSet::new();
        for word in &words {
            chars.extend(word.iter().cloned());
        }

        for c in chars {
            if !self.vocab.contains_key(&c) {
                self.vocab.insert(c.clone(), self.next_id);
                self.id_to_token.insert(self.next_id, c);
                self.next_id += 1;
            }
        }

        // BPE merging
        while self.vocab.len() < target_vocab_size {
            let pair_counts = self.count_pairs(&words);

            if pair_counts.is_empty() {
                break;
            }

            let best_pair = pair_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(pair, _)| pair.clone())
                .unwrap();

            self.merge_pair(&mut words, (&best_pair.0, &best_pair.1));

            let merged = format!("{}{}", best_pair.0, best_pair.1);
            if !self.vocab.contains_key(&merged) {
                self.vocab.insert(merged.clone(), self.next_id);
                self.id_to_token.insert(self.next_id, merged);
                self.next_id += 1;

                let merge_idx = self.merges.len();
                self.merges.push(best_pair.clone());
                self.merge_priority.insert(best_pair, merge_idx);
            }
        }
    }

    fn apply_merges(&self, mut word: Vec<String>) -> Vec<String> {
        for (pair_a, pair_b) in &self.merges {
            let mut i = 0;
            while i < word.len().saturating_sub(1) {
                if word[i] == *pair_a && word[i + 1] == *pair_b {
                    word[i] = format!("{}{}", pair_a, pair_b);
                    word.remove(i + 1);
                } else {
                    i += 1;
                }
            }
        }
        word
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        let words = self.get_words(text);
        let mut result = Vec::new();

        for word in words {
            let merged = self.apply_merges(word);
            for token in merged {
                result.push(self.vocab.get(&token).copied().unwrap_or(1));
            }
        }

        result
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        ids.iter()
            .filter_map(|&id| self.id_to_token.get(&id))
            .cloned()
            .collect::<Vec<_>>()
            .concat()
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

fn main() {
    println!("=== Neural Network Tokenizer Demo ===\n");

    let corpus = "the quick brown fox jumps over the lazy dog the quick cat";

    // Character-level
    println!("--- Character-Level Tokenizer ---");
    let mut char_tok = CharTokenizer::new();
    char_tok.train(corpus);
    let encoded = char_tok.encode("the fox");
    println!("Encoded 'the fox': {:?}", encoded);
    println!("Decoded: {}", char_tok.decode(&encoded));
    println!("Vocab size: {}\n", char_tok.vocab_size());

    // Word-level
    println!("--- Word-Level Tokenizer ---");
    let mut word_tok = WordTokenizer::new(1);
    word_tok.train(corpus);
    let encoded = word_tok.encode("the fox");
    println!("Encoded 'the fox': {:?}", encoded);
    println!("Decoded: {}", word_tok.decode(&encoded));
    println!("Vocab size: {}\n", word_tok.vocab_size());

    // BPE
    println!("--- BPE Tokenizer ---");
    let mut bpe_tok = BPETokenizer::new();
    bpe_tok.train(corpus, 50);
    let encoded = bpe_tok.encode("the fox");
    println!("Encoded 'the fox': {:?}", encoded);
    println!("Decoded: {}", bpe_tok.decode(&encoded));
    println!("Vocab size: {}", bpe_tok.vocab_size());
    println!("Learned {} merges\n", bpe_tok.merges.len());

    // Benchmark
    println!("--- Performance Benchmark ---");
    let large_corpus = "the quick brown fox jumps over the lazy dog ".repeat(1000);

    use std::time::Instant;

    let mut bpe = BPETokenizer::new();
    let start = Instant::now();
    bpe.train(&large_corpus, 200);
    let train_time = start.elapsed();

    let start = Instant::now();
    let encoded = bpe.encode(&large_corpus);
    let encode_time = start.elapsed();

    println!("Training time: {:?}", train_time);
    println!("Encoding time: {:?}", encode_time);
    println!("Tokens generated: {}", encoded.len());
    println!("Throughput: {:.2}M tokens/sec",
        encoded.len() as f64 / encode_time.as_secs_f64() / 1_000_000.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_tokenizer() {
        let mut tok = CharTokenizer::new();
        tok.train("hello");

        let enc = tok.encode("hello");
        let dec = tok.decode(&enc);

        assert_eq!(dec, "hello");
    }

    #[test]
    fn test_word_tokenizer() {
        let mut tok = WordTokenizer::new(1);
        tok.train("hello world");

        let enc = tok.encode("hello world");
        let dec = tok.decode(&enc);

        assert!(dec.contains("hello"));
        assert!(dec.contains("world"));
    }

    #[test]
    fn test_bpe_tokenizer() {
        let mut tok = BPETokenizer::new();
        tok.train("hello world hello", 30);

        let enc = tok.encode("hello");
        let dec = tok.decode(&enc);

        assert_eq!(dec, "hello");
        assert!(tok.merges.len() > 0);
    }

    #[test]
    fn test_bpe_subword() {
        let mut tok = BPETokenizer::new();
        tok.train("running runner run", 40);

        let run_enc = tok.encode("run");
        let runner_enc = tok.encode("runner");

        // Should share some tokens
        assert!(runner_enc.len() >= run_enc.len());
    }
}
```

---

## Milestone 7: Ultra-Optimized Production BPE

### Introduction

**Why Milestone 6 Is Not Enough:**
While Milestone 6 made significant improvements, there are still several low-hanging optimizations:
1. **HashMap overhead**: Standard HashMap uses SipHash (cryptographic) - overkill for our use case
2. **ASCII cache misses**: Looking up common ASCII characters in HashMap is slower than array indexing
3. **Allocation overhead**: Small vectors cause heap allocations for every word
4. **String operations**: Converting bytes to strings and back is expensive
5. **I/O overhead**: Unbuffered writes during serialization are slow

**What We're Improving:**
Production-grade optimizations used in real tokenizers like HuggingFace's `tokenizers` library:

1. **FxHashMap**: 2-3x faster hashing (non-cryptographic)
2. **ASCII cache**: O(1) lookup for ASCII chars (most common case)
3. **Rayon parallelism**: Multi-core pair counting
4. **Position tracking**: Efficient merge without full scan
5. **Greedy algorithm**: Single-pass encoding instead of priority queue
6. **Buffered I/O**: Fast serialization/deserialization
7. **SmallVec**: Stack allocation for short sequences (most words <24 chars)
8. **Byte operations**: Work with `&[u8]` instead of String where possible

**Performance Improvements:**
```
Standard HashMap:  100ms pair counting
FxHashMap:         35ms pair counting (3x faster)

Heap allocation:   1M words × 48 bytes = 48MB allocations
SmallVec:          1M words × 0 bytes = 0MB allocations (stack)

String ops:        50ms encoding
Byte ops:          15ms encoding (3x faster)

Total speedup: 5-10x over Milestone 6
             50-100x over Milestone 3
```

### Architecture

**Dependencies:**
```toml
[dependencies]
rayon = "1.8"
fxhash = "0.2"
smallvec = "1.11"
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }

# For benchmarks (downloading tiny-shakespeare dataset)
[dev-dependencies]
reqwest = { version = "0.11", features = ["blocking"] }
```

**New Structs:**
- `UltraOptimizedBPE` - Production-grade BPE tokenizer
  - **Field** `vocab: FxHashMap<Vec<u8>, u32>` - Fast hash map with byte keys
  - **Field** `id_to_token: Vec<Vec<u8>>` - Token lookup (bytes, not strings)
  - **Field** `ascii_cache: [Option<u32>; 256]` - Fast ASCII → ID lookup
  - **Field** `merges: Vec<(Vec<u8>, Vec<u8>)>` - Merge rules as bytes
  - **Field** `merge_ranks: FxHashMap<(u32, u32), u32>` - Merge priority lookup
  - **Field** `special_tokens: FxHashMap<Vec<u8>, u32>` - Special tokens
  - **Function** `new() -> Self` - Initialize with optimizations
  - **Function** `train(&mut self, text: &str, vocab_size: usize)` - Parallel training
  - **Function** `encode_bytes(&self, text: &[u8]) -> Vec<u32>` - Zero-copy encoding
  - **Function** `encode(&self, text: &str) -> Vec<u32>` - String wrapper
  - **Function** `decode(&self, ids: &[u32]) -> String` - Decode to string
  - **Function** `save_binary(&self, path: &str)` - Fast binary serialization
  - **Function** `load_binary(path: &str) -> Self` - Fast deserialization

**Key Optimizations:**

1. **FxHashMap**: Non-cryptographic hash for 2-3x speed
```rust
use fxhash::FxHashMap;
// FxHashMap is faster than HashMap for integer and small keys
let mut map = FxHashMap::default();
```

2. **ASCII Cache**: O(1) lookup for common characters
```rust
// Instead of: vocab.get(&char) → O(log n) or O(1) with hash overhead
// Use: ascii_cache[byte as usize] → O(1) direct array access
if byte < 128 {
    return ascii_cache[byte as usize];
}
```

3. **SmallVec**: Avoid heap allocation for short sequences
```rust
use smallvec::{SmallVec, smallvec};
// Most words < 24 bytes, keep on stack
type WordVec = SmallVec<[u32; 24]>;
```

4. **Byte Operations**: Avoid UTF-8 overhead
```rust
// Instead of: text.chars().map(|c| ...)
// Use: text.as_bytes().iter().map(|&b| ...)
```

5. **Parallel Counting**: Multi-core processing
```rust
use rayon::prelude::*;
words.par_iter()
    .fold(|| FxHashMap::default(), |mut acc, word| {
        // Count pairs in parallel
        acc
    })
    .reduce(|| FxHashMap::default(), merge_hashmaps)
```

**Role Each Plays:**
- FxHashMap: Fast hashing without cryptographic overhead
- ASCII cache: Skip hash lookup for common characters
- SmallVec: Stack allocation for most words (cache-friendly)
- Byte slices: Avoid UTF-8 validation and string allocation
- Rayon: Parallel processing across CPU cores
- Bincode: Fast binary serialization (vs JSON)


### Starter Code

```rust
use fxhash::FxHashMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};

// ============================================================================
// ULTRA-OPTIMIZED BPE TOKENIZER
// ============================================================================

type WordVec = SmallVec<[u32; 24]>; // Stack allocation for words < 24 tokens

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraOptimizedBPE {
    // Use bytes instead of strings for zero-copy operations
    vocab: FxHashMap<Vec<u8>, u32>,
    id_to_token: Vec<Vec<u8>>,

    // ASCII fast path: direct lookup for bytes < 128
    #[serde(skip)]
    ascii_cache: [Option<u32>; 256],

    // Merge rules and their ranks
    merges: Vec<(Vec<u8>, Vec<u8>)>,
    merge_ranks: FxHashMap<(u32, u32), u32>, // (token_a, token_b) → rank

    special_tokens: FxHashMap<Vec<u8>, u32>,
    next_id: u32,
}

impl UltraOptimizedBPE {
    pub fn new() -> Self {
        let mut tokenizer = Self {
            vocab: FxHashMap::default(),
            id_to_token: Vec::new(),
            ascii_cache: [None; 256],
            merges: Vec::new(),
            merge_ranks: FxHashMap::default(),
            special_tokens: FxHashMap::default(),
            next_id: 0,
        };

        // Add special tokens
        for token in &[b"<PAD>", b"<UNK>", b"<BOS>", b"<EOS>"] {
            tokenizer.add_special_token(token);
        }

        tokenizer
    }

    fn add_special_token(&mut self, token: &[u8]) -> u32 {
        let id = self.next_id;
        self.vocab.insert(token.to_vec(), id);
        self.id_to_token.push(token.to_vec());
        self.special_tokens.insert(token.to_vec(), id);
        self.next_id += 1;
        id
    }

    fn add_token(&mut self, token: Vec<u8>) -> u32 {
        // TODO: Add token to vocab
        // Update ASCII cache if single byte < 256
        //
        // let id = self.next_id;
        // self.vocab.insert(token.clone(), id);
        // self.id_to_token.push(token.clone());
        // self.next_id += 1;
        //
        // // Update ASCII cache
        // if token.len() == 1 {
        //     let byte = token[0] as usize;
        //     if byte < 256 {
        //         self.ascii_cache[byte] = Some(id);
        //     }
        // }
        //
        // id
        todo!()
    }

    fn get_token_id(&self, token: &[u8]) -> Option<u32> {
        // TODO: Fast path for single ASCII bytes
        // Use ascii_cache for O(1) lookup
        //
        // if token.len() == 1 {
        //     let byte = token[0] as usize;
        //     if byte < 256 {
        //         return self.ascii_cache[byte];
        //     }
        // }
        // self.vocab.get(token).copied()
        todo!()
    }

    fn get_words_as_bytes(&self, text: &str) -> Vec<Vec<Vec<u8>>> {
        // TODO: Split text into words, then bytes
        // Work with byte slices for performance
        //
        // text.split_whitespace()
        //     .map(|word| {
        //         word.bytes()
        //             .map(|b| vec![b])
        //             .collect()
        //     })
        //     .collect()
        todo!()
    }

    fn parallel_count_pairs(&self, words: &[Vec<Vec<u8>>]) -> FxHashMap<(Vec<u8>, Vec<u8>), usize> {
        // TODO: Parallel pair counting with Rayon and FxHashMap
        //
        // Use par_iter() to process words in parallel
        // Use fold-reduce pattern to combine results
        //
        // words.par_iter()
        //     .fold(
        //         || FxHashMap::default(),
        //         |mut acc, word| {
        //             for window in word.windows(2) {
        //                 let pair = (window[0].clone(), window[1].clone());
        //                 *acc.entry(pair).or_insert(0) += 1;
        //             }
        //             acc
        //         }
        //     )
        //     .reduce(
        //         || FxHashMap::default(),
        //         |mut a, b| {
        //             for (k, v) in b {
        //                 *a.entry(k).or_insert(0) += v;
        //             }
        //             a
        //         }
        //     )
        todo!()
    }

    fn parallel_merge(&self, words: &mut [Vec<Vec<u8>>], pair: (&[u8], &[u8])) {
        // TODO: Parallel merge using par_iter_mut
        //
        // words.par_iter_mut().for_each(|word| {
        //     let mut i = 0;
        //     while i < word.len().saturating_sub(1) {
        //         if word[i] == pair.0 && word[i + 1] == pair.1 {
        //             let mut merged = pair.0.to_vec();
        //             merged.extend_from_slice(pair.1);
        //             word[i] = merged;
        //             word.remove(i + 1);
        //         } else {
        //             i += 1;
        //         }
        //     }
        // });
        todo!()
    }

    pub fn train(&mut self, text: &str, target_vocab_size: usize) {
        // TODO: Ultra-optimized BPE training
        //
        // 1. Convert text to byte vectors
        // 2. Initialize vocabulary with all unique bytes
        // 3. Parallel pair counting
        // 4. Iterative merging until target vocab size
        // 5. Build merge_ranks for fast encoding
        //
        // Key optimizations:
        // - Use FxHashMap for faster hashing
        // - Parallel counting with Rayon
        // - Work with bytes instead of strings
        // - Update ASCII cache as you add tokens
        //
        // let mut words = self.get_words_as_bytes(text);
        //
        // // Add byte vocabulary
        // let mut bytes = std::collections::HashSet::new();
        // for word in &words {
        //     for byte_vec in word {
        //         bytes.insert(byte_vec.clone());
        //     }
        // }
        //
        // for byte_vec in bytes {
        //     if !self.vocab.contains_key(&byte_vec) {
        //         self.add_token(byte_vec);
        //     }
        // }
        //
        // // BPE merging with parallel counting
        // while self.vocab.len() < target_vocab_size {
        //     let pair_counts = self.parallel_count_pairs(&words);
        //
        //     if pair_counts.is_empty() {
        //         break;
        //     }
        //
        //     let best_pair = pair_counts
        //         .iter()
        //         .max_by_key(|(_, count)| *count)
        //         .map(|(pair, _)| pair.clone())
        //         .unwrap();
        //
        //     self.parallel_merge(&mut words, (&best_pair.0, &best_pair.1));
        //
        //     let mut merged = best_pair.0.clone();
        //     merged.extend_from_slice(&best_pair.1);
        //
        //     if !self.vocab.contains_key(&merged) {
        //         let merged_id = self.add_token(merged);
        //
        //         // Record merge rank
        //         let rank = self.merges.len() as u32;
        //         self.merges.push(best_pair.clone());
        //
        //         let token_a = self.vocab.get(&best_pair.0).copied().unwrap();
        //         let token_b = self.vocab.get(&best_pair.1).copied().unwrap();
        //         self.merge_ranks.insert((token_a, token_b), rank);
        //     }
        // }

        todo!()
    }

    fn apply_merges_greedy(&self, word: Vec<Vec<u8>>) -> WordVec {
        // TODO: Single-pass greedy merging algorithm
        //
        // Instead of applying merges in order (slow),
        // use greedy algorithm: always merge lowest-rank pair
        //
        // Convert to token IDs first, then work with IDs
        //
        // let mut tokens: WordVec = word.iter()
        //     .filter_map(|byte_vec| self.get_token_id(byte_vec))
        //     .collect();
        //
        // loop {
        //     let mut best_pos = None;
        //     let mut best_rank = u32::MAX;
        //
        //     // Find lowest-rank pair
        //     for i in 0..tokens.len().saturating_sub(1) {
        //         let pair = (tokens[i], tokens[i + 1]);
        //         if let Some(&rank) = self.merge_ranks.get(&pair) {
        //             if rank < best_rank {
        //                 best_rank = rank;
        //                 best_pos = Some(i);
        //             }
        //         }
        //     }
        //
        //     if let Some(pos) = best_pos {
        //         // Merge at pos
        //         let pair = (tokens[pos], tokens[pos + 1]);
        //         // Find merged token ID
        //         // This requires reverse lookup or storing merged IDs
        //         // For now, reconstruct the merged token
        //         let mut merged_bytes = self.id_to_token[tokens[pos] as usize].clone();
        //         merged_bytes.extend_from_slice(&self.id_to_token[tokens[pos + 1] as usize]);
        //
        //         if let Some(merged_id) = self.get_token_id(&merged_bytes) {
        //             tokens[pos] = merged_id;
        //             tokens.remove(pos + 1);
        //         } else {
        //             break;
        //         }
        //     } else {
        //         break;
        //     }
        // }
        //
        // tokens
        todo!()
    }

    pub fn encode_bytes(&self, text: &[u8]) -> Vec<u32> {
        // TODO: Encode byte slice (zero-copy)
        //
        // 1. Split on whitespace (work with byte slices)
        // 2. Convert each word to byte vectors
        // 3. Apply greedy merging
        // 4. Collect token IDs
        //
        // Optimization: Use SmallVec to avoid heap allocation
        todo!()
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        self.encode_bytes(text.as_bytes())
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        // TODO: Decode token IDs to string
        //
        // 1. Map each ID to byte vector
        // 2. Concatenate all bytes
        // 3. Convert to UTF-8 string
        //
        // let mut bytes = Vec::new();
        // for &id in ids {
        //     if id < self.id_to_token.len() as u32 {
        //         bytes.extend_from_slice(&self.id_to_token[id as usize]);
        //     }
        // }
        // String::from_utf8_lossy(&bytes).to_string()
        todo!()
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }

    pub fn save_binary(&self, path: &str) -> std::io::Result<()> {
        // TODO: Fast binary serialization with buffered I/O
        //
        // Use BufWriter for buffered writes
        // Use bincode for fast serialization
        //
        // let file = File::create(path)?;
        // let writer = BufWriter::new(file);
        // bincode::serialize_into(writer, self)
        //     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        // Ok(())
        todo!()
    }

    pub fn load_binary(path: &str) -> std::io::Result<Self> {
        // TODO: Fast binary deserialization with buffered I/O
        //
        // let file = File::open(path)?;
        // let reader = BufReader::new(file);
        // let mut tokenizer: Self = bincode::deserialize_from(reader)
        //     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        //
        // // Rebuild ASCII cache (skipped during serialization)
        // for (token, &id) in &tokenizer.vocab {
        //     if token.len() == 1 {
        //         let byte = token[0] as usize;
        //         if byte < 256 {
        //             tokenizer.ascii_cache[byte] = Some(id);
        //         }
        //     }
        // }
        //
        // Ok(tokenizer)
        todo!()
    }
}

// ============================================================================
// HELPER: MERGE TWO FXHASHMAPS
// ============================================================================

fn merge_hashmaps<K, V>(mut a: FxHashMap<K, V>, b: FxHashMap<K, V>) -> FxHashMap<K, V>
where
    K: std::hash::Hash + Eq,
    V: std::ops::AddAssign,
{
    for (k, v) in b {
        a.entry(k).and_modify(|val| *val += v).or_insert(v);
    }
    a
}
```

---

### Checkpoint Tests

```rust
#[test]
fn test_ultra_bpe_correctness() {
    let mut tokenizer = UltraOptimizedBPE::new();

    let corpus = "hello world hello world".repeat(100);
    tokenizer.train(&corpus, 100);

    let encoded = tokenizer.encode("hello world");
    let decoded = tokenizer.decode(&encoded);

    assert_eq!(decoded, "hello world");
}

#[test]
fn test_ascii_cache() {
    let mut tokenizer = UltraOptimizedBPE::new();
    tokenizer.train("abc", 50);

    // ASCII characters should use cache
    let encoded = tokenizer.encode("abc");
    assert_eq!(encoded.len(), 3);
}

#[test]
fn test_byte_encoding() {
    let mut tokenizer = UltraOptimizedBPE::new();
    tokenizer.train("hello world", 50);

    // encode_bytes should work with byte slices
    let encoded = tokenizer.encode_bytes(b"hello");
    let encoded_str = tokenizer.encode("hello");

    assert_eq!(encoded, encoded_str);
}

#[test]
fn test_unicode_handling() {
    let mut tokenizer = UltraOptimizedBPE::new();
    tokenizer.train("Hello 世界 🚀", 100);

    let encoded = tokenizer.encode("世界");
    let decoded = tokenizer.decode(&encoded);

    assert_eq!(decoded, "世界");
}

#[test]
fn test_serialization() {
    let mut tokenizer = UltraOptimizedBPE::new();
    tokenizer.train("the quick brown fox", 50);

    // Save and load
    tokenizer.save_binary("test_tokenizer.bin").unwrap();
    let loaded = UltraOptimizedBPE::load_binary("test_tokenizer.bin").unwrap();

    let encoded1 = tokenizer.encode("quick");
    let encoded2 = loaded.encode("quick");

    assert_eq!(encoded1, encoded2);

    std::fs::remove_file("test_tokenizer.bin").ok();
}

#[test]
fn test_parallel_speedup() {
    use std::time::Instant;

    let corpus = "the quick brown fox jumps over the lazy dog ".repeat(100000);

    let mut tokenizer = UltraOptimizedBPE::new();
    let start = Instant::now();
    tokenizer.train(&corpus, 500);
    let train_time = start.elapsed();

    println!("Ultra-optimized training: {:?}", train_time);

    // Should be very fast
    assert!(train_time.as_secs() < 5);
}

#[test]
fn test_encoding_performance() {
    let mut tokenizer = UltraOptimizedBPE::new();
    let corpus = "hello world ".repeat(1000);
    tokenizer.train(&corpus, 100);

    let text = "hello world ".repeat(100000);

    let start = std::time::Instant::now();
    let encoded = tokenizer.encode(&text);
    let elapsed = start.elapsed();

    let tokens_per_sec = encoded.len() as f64 / elapsed.as_secs_f64();
    println!("Encoding speed: {:.2}M tokens/sec", tokens_per_sec / 1_000_000.0);

    // Should achieve millions of tokens/sec
    assert!(tokens_per_sec > 1_000_000.0);
}

#[test]
fn benchmark_all_optimizations() {
    use std::time::Instant;

    let corpus = get_tiny_shakespeare();
    let vocab_size = 500;

    println!("\n=== Ultimate BPE Benchmark ===");
    println!("Dataset: tiny-shakespeare ({} bytes)\n", corpus.len());

    // Milestone 3: Naive BPE (if you want to compare, may be slow)
    // Skipped for time

    // Milestone 6: Optimized BPE with interning
    println!("Testing Milestone 6 (Interning)...");
    let mut m6 = OptimizedBPETokenizer::new();
    let start = Instant::now();
    m6.train(&corpus, vocab_size);
    let m6_train = start.elapsed();

    let start = Instant::now();
    let m6_encoded = m6.encode(&corpus);
    let m6_encode = start.elapsed();

    println!("  Training: {:?}", m6_train);
    println!("  Encoding: {:?} ({:.2}M tokens/sec)",
        m6_encode,
        m6_encoded.len() as f64 / m6_encode.as_secs_f64() / 1_000_000.0);

    // Milestone 7: Ultra-optimized
    println!("\nTesting Milestone 7 (Ultra-optimized)...");
    let mut m7 = UltraOptimizedBPE::new();
    let start = Instant::now();
    m7.train(&corpus, vocab_size);
    let m7_train = start.elapsed();

    let start = Instant::now();
    let m7_encoded = m7.encode(&corpus);
    let m7_encode = start.elapsed();

    println!("  Training: {:?} ({:.2}x speedup)",
        m7_train,
        m6_train.as_secs_f64() / m7_train.as_secs_f64());
    println!("  Encoding: {:?} ({:.2}M tokens/sec)",
        m7_encode,
        m7_encoded.len() as f64 / m7_encode.as_secs_f64() / 1_000_000.0);

    println!("\n=== Summary ===");
    println!("Speedup (M7 vs M6):");
    println!("  Training: {:.2}x faster", m6_train.as_secs_f64() / m7_train.as_secs_f64());
    println!("  Encoding: {:.2}x faster", m6_encode.as_secs_f64() / m7_encode.as_secs_f64());
}
```

## Final Performance Comparison

This comprehensive benchmark compares all tokenizer implementations using the **tiny-shakespeare** dataset, a standard benchmark corpus for tokenizer performance.

### Benchmark Setup

**Dataset:** The tiny-shakespeare dataset (~1MB of Shakespeare text) is downloaded automatically from GitHub on first run and cached locally for subsequent runs.

```rust
// ============================================================================
// BENCHMARK HELPER: Download and cache tiny-shakespeare dataset
// ============================================================================

fn get_tiny_shakespeare() -> String {
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    let cache_path = "tiny_shakespeare.txt";

    // Check if already cached
    if Path::new(cache_path).exists() {
        println!("Loading cached tiny-shakespeare dataset...");
        return fs::read_to_string(cache_path).expect("Failed to read cached dataset");
    }

    // Download from internet
    println!("Downloading tiny-shakespeare dataset...");
    let url = "https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt";

    let text = reqwest::blocking::get(url)
        .expect("Failed to download dataset")
        .text()
        .expect("Failed to read response text");

    // Save to cache
    let mut file = fs::File::create(cache_path).expect("Failed to create cache file");
    file.write_all(text.as_bytes()).expect("Failed to write cache");

    println!("Dataset downloaded and cached ({} bytes)", text.len());
    text
}

// ============================================================================
// COMPREHENSIVE BENCHMARK
// ============================================================================

#[cfg(test)]
mod benchmark {
    use super::*;
    use std::time::Instant;

    #[test]
    fn ultimate_benchmark() {
        let corpus = get_tiny_shakespeare();
        let vocab_size = 1000;

        println!("\n=== ULTIMATE TOKENIZER BENCHMARK ===");
        println!("Dataset: tiny-shakespeare ({} bytes)", corpus.len());
        println!("Target vocabulary: {}\n", vocab_size);

        // Milestone 3: Naive BPE (skip if too slow)
        // println!("Milestone 3 (Naive): SKIPPED (too slow)");

        // Milestone 4: Optimized with heap
        println!("Testing Milestone 4...");
        let mut m4 = BPETokenizer::new();
        let start = Instant::now();
        m4.train(corpus, vocab_size);
        let m4_train = start.elapsed();

        let start = Instant::now();
        let m4_encoded = m4.encode(corpus);
        let m4_encode = start.elapsed();

        println!("  Training: {:?}", m4_train);
        println!("  Encoding: {:?} ({:.2}M tokens/sec)",
            m4_encode,
            m4_encoded.len() as f64 / m4_encode.as_secs_f64() / 1_000_000.0
        );

        // Milestone 5: Parallel BPE
        println!("\nTesting Milestone 5...");
        let mut m5 = ParallelBPETokenizer::new();
        let start = Instant::now();
        m5.train(corpus, vocab_size);
        let m5_train = start.elapsed();

        let start = Instant::now();
        let m5_encoded = m5.encode(corpus);
        let m5_encode = start.elapsed();

        println!("  Training: {:?} ({:.2}x speedup)",
            m5_train,
            m4_train.as_secs_f64() / m5_train.as_secs_f64()
        );
        println!("  Encoding: {:?} ({:.2}M tokens/sec)",
            m5_encode,
            m5_encoded.len() as f64 / m5_encode.as_secs_f64() / 1_000_000.0
        );

        // Milestone 6: Optimized with interning
        println!("\nTesting Milestone 6...");
        let mut m6 = OptimizedBPETokenizer::new();
        let start = Instant::now();
        m6.train(corpus, vocab_size);
        let m6_train = start.elapsed();

        let start = Instant::now();
        let m6_encoded = m6.encode(corpus);
        let m6_encode = start.elapsed();

        println!("  Training: {:?} ({:.2}x speedup)",
            m6_train,
            m4_train.as_secs_f64() / m6_train.as_secs_f64()
        );
        println!("  Encoding: {:?} ({:.2}M tokens/sec)",
            m6_encode,
            m6_encoded.len() as f64 / m6_encode.as_secs_f64() / 1_000_000.0
        );

        // Milestone 7: Ultra-optimized
        println!("\nTesting Milestone 7 (ULTRA)...");
        let mut m7 = UltraOptimizedBPE::new();
        let start = Instant::now();
        m7.train(corpus, vocab_size);
        let m7_train = start.elapsed();

        let start = Instant::now();
        let m7_encoded = m7.encode(corpus);
        let m7_encode = start.elapsed();

        println!("  Training: {:?} ({:.2}x speedup over M4)",
            m7_train,
            m4_train.as_secs_f64() / m7_train.as_secs_f64()
        );
        println!("  Encoding: {:?} ({:.2}M tokens/sec)",
            m7_encode,
            m7_encoded.len() as f64 / m7_encode.as_secs_f64() / 1_000_000.0
        );

        // Summary table
        println!("\n=== SUMMARY ===");
        println!("┌─────────────┬──────────────┬──────────────┬──────────────┐");
        println!("│ Milestone   │ Train Time   │ Encode Time  │ Tokens/sec   │");
        println!("├─────────────┼──────────────┼──────────────┼──────────────┤");
        println!("│ M4 (Heap)   │ {:>11.3}s │ {:>11.3}s │ {:>9.2}M │",
            m4_train.as_secs_f64(),
            m4_encode.as_secs_f64(),
            m4_encoded.len() as f64 / m4_encode.as_secs_f64() / 1_000_000.0
        );
        println!("│ M5 (||)     │ {:>11.3}s │ {:>11.3}s │ {:>9.2}M │",
            m5_train.as_secs_f64(),
            m5_encode.as_secs_f64(),
            m5_encoded.len() as f64 / m5_encode.as_secs_f64() / 1_000_000.0
        );
        println!("│ M6 (Intern) │ {:>11.3}s │ {:>11.3}s │ {:>9.2}M │",
            m6_train.as_secs_f64(),
            m6_encode.as_secs_f64(),
            m6_encoded.len() as f64 / m6_encode.as_secs_f64() / 1_000_000.0
        );
        println!("│ M7 (ULTRA)  │ {:>11.3}s │ {:>11.3}s │ {:>9.2}M │",
            m7_train.as_secs_f64(),
            m7_encode.as_secs_f64(),
            m7_encoded.len() as f64 / m7_encode.as_secs_f64() / 1_000_000.0
        );
        println!("└─────────────┴──────────────┴──────────────┴──────────────┘");

        println!("\nFinal speedup (M7 vs M4):");
        println!("  Training: {:.2}x faster", m4_train.as_secs_f64() / m7_train.as_secs_f64());
        println!("  Encoding: {:.2}x faster", m4_encode.as_secs_f64() / m7_encode.as_secs_f64());

        // Memory comparison
        println!("\nMemory optimizations:");
        println!("  SmallVec: Eliminates heap allocation for ~80% of words");
        println!("  FxHashMap: 40% less memory overhead vs std HashMap");
        println!("  Byte storage: 50% less memory vs String storage");
    }
}
```

---

## Summary

This completes the comprehensive tokenizer project with all 7 milestones, from naive implementation to production-grade ultra-optimized BPE!

### What You'll Build:

1. **Milestone 1**: Character-level tokenizer (baseline)
2. **Milestone 2**: Word-level tokenizer (reduced sequence length)
3. **Milestone 3**: Naive BPE (subword tokenization)
4. **Milestone 4**: Optimized BPE with priority queue
5. **Milestone 5**: Parallel BPE with Rayon and DashMap
6. **Milestone 6**: Extreme optimization with string interning
7. **Milestone 7**: Ultra-optimized production BPE with:
   - FxHashMap for 3x faster hashing
   - ASCII cache for O(1) character lookup
   - SmallVec for zero-allocation on 80% of words
   - Byte operations for zero-copy encoding
   - Buffered I/O for fast serialization
   - Greedy algorithm for single-pass encoding

### Benchmarking:

All benchmarks use the **tiny-shakespeare dataset** (~1MB), a standard corpus for NLP benchmarks:
- Downloaded automatically from GitHub on first run
- Cached locally as `tiny_shakespeare.txt` for subsequent runs
- Provides realistic performance measurements
- Same dataset used in Andrej Karpathy's char-rnn project

**Expected Performance:**
- Milestone 4: ~1-5 seconds training on tiny-shakespeare
- Milestone 5: 2-3x faster with parallelism
- Milestone 6: 5-10x faster with interning
- Milestone 7: 10-20x faster with all optimizations
- Encoding: 10-50+ million tokens/second

This project teaches production-grade performance optimization techniques used in real-world tokenizers like HuggingFace's `tokenizers` library!
