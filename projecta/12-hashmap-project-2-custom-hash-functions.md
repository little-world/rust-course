# Project 2: Custom Hash Functions for Semantic Correctness

## Problem Statement

Build a spatial indexing system for geographic data that requires custom hash implementations for correct behavior. The system must handle case-insensitive lookups, floating-point coordinates with tolerance, and composite keys - all requiring custom `Hash` implementations.

Your spatial index should:
- Store locations with approximate coordinate matching (±0.0001 degrees)
- Support case-insensitive place name lookups
- Handle composite keys (category + location)
- Use fast hashers (FxHash) for performance-critical paths
- Benchmark hash function performance

Example use cases:
- "Find all restaurants near (37.7749, -122.4194)" with tolerance
- Case-insensitive: "San Francisco" == "san francisco"
- Query by category + region combinations

## Why It Matters

Default hash functions don't match business semantics. Floating-point coordinates can't be HashMap keys (NaN != NaN, rounding errors). Case-sensitive matching rejects valid lookups. Custom `Hash` implementations encode domain semantics into the type system, making "correct by construction" code possible.

Hasher selection impacts performance: SipHash (default, secure, slow) vs FxHash (fast, trusted keys only) can differ by 10×. Wrong hasher = unnecessary CPU waste.

## Use Cases

- Geographic information systems (GIS)
- Location-based services (restaurant finders, ride-sharing)
- Content-addressable storage
- Case-insensitive caching (HTTP headers, DNS)
- Approximate deduplication
- Performance-critical integer key maps

---

## Introduction to Custom Hash Functions and Semantic Correctness

Default hash implementations optimize for the common case, but business logic often requires custom semantics: case-insensitive matching, approximate coordinates, or content-based identity. Understanding the Hash trait contract and implementing custom hashers enables "correct by construction" code where the type system enforces business rules.

### 1. The Hash Trait Contract

The Hash trait has a critical invariant that must be maintained:

**The Contract**:
```rust
// If a == b, then hash(a) MUST equal hash(b)
if a == b {
    assert_eq!(hash(a), hash(b));
}
```

**Why This Matters**:
```rust
// Violating the contract breaks HashMap
struct Bad {
    value: i32,
}

impl PartialEq for Bad {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value  // Equal if values match
    }
}

impl Hash for Bad {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // BUG: Hashing random value!
        rand::random::<i32>().hash(state);
    }
}

// Same object hashes differently each time!
let b = Bad { value: 42 };
map.insert(b, "data");
map.get(&b);  // Might not find it! Random hash changed the bucket
```

**Correct Implementation**:
```rust
impl Hash for Good {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash same fields used in PartialEq
        self.value.hash(state);
    }
}
```

**Key Rule**: Hash exactly the fields compared in `PartialEq` / `Eq`.

### 2. Hash Distribution and Collisions

Good hash functions distribute keys evenly across buckets:

**Perfect Distribution** (no collisions):
```
Keys: [1, 2, 3, 4, 5, 6, 7, 8]
Buckets (8): [1] [2] [3] [4] [5] [6] [7] [8]
Lookup: O(1) always
```

**Poor Distribution** (many collisions):
```
Keys: [1, 2, 3, 4, 5, 6, 7, 8]
Buckets (8): [] [1,2,3,4,5,6,7,8] [] [] [] [] [] []
Lookup: O(n) - degrades to linked list search
```

**Example of Bad Hash**:
```rust
impl Hash for BadHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        42.hash(state);  // Every key hashes to 42!
    }
}
// All keys map to same bucket → O(n) lookups
```

**Quality Metrics**:
- **Avalanche effect**: Small input change → large hash change
- **Uniform distribution**: Equal probability for each bucket
- **Low collision rate**: Different keys → different hashes (mostly)

### 3. Case-Insensitive Hashing

Business logic often requires case-insensitive matching:

**The Problem**:
```rust
let mut headers = HashMap::new();
headers.insert("Content-Type", "application/json");

// User queries with different case
headers.get("content-type");  // None - case mismatch!
```

**Solution - Normalize in Hash**:
```rust
struct CaseInsensitive(String);

impl Hash for CaseInsensitive {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash lowercase version
        self.0.to_lowercase().hash(state);
    }
}

impl PartialEq for CaseInsensitive {
    fn eq(&self, other: &Self) -> bool {
        // Compare case-insensitively
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
```

**Result**: "Content-Type" and "content-type" hash to same bucket and compare equal.

**Performance Consideration**: Allocating lowercase string on every hash is expensive. Optimization:
```rust
// Better: hash bytes directly in lowercase
for byte in self.0.bytes() {
    byte.to_ascii_lowercase().hash(state);
}
```

### 4. The Floating-Point Equality Problem

Floating-point values cannot be HashMap keys directly:

**Why Not**:
```rust
let mut map = HashMap::new();
map.insert(0.1 + 0.2, "value");  // Stores 0.30000000000000004

map.get(&0.3);  // None! 0.3 ≠ 0.30000000000000004
```

**Additional Issues**:
```rust
// NaN is never equal to itself
assert!(f64::NAN != f64::NAN);  // True!

// Can't be HashMap key - violates Eq contract
```

**Solution: Quantization**:
```rust
// Round to fixed precision
struct QuantizedFloat {
    value: i32,  // Store as integer (e.g., cents instead of dollars)
}

impl QuantizedFloat {
    fn from_float(f: f64, precision: f64) -> Self {
        QuantizedFloat {
            value: (f / precision).round() as i32
        }
    }
}

// 0.1 + 0.2 and 0.3 both round to same integer
```

### 5. Coordinate Quantization for Geographic Lookup

GPS coordinates need approximate matching due to precision limits:

**The Problem**:
```rust
// GPS readings of same location
let loc1 = (37.7749, -122.4194);   // Reading 1
let loc2 = (37.77491, -122.41939); // Reading 2 (1 meter away)

// Can't use as HashMap key - never match exactly
```

**Grid Quantization**:
```rust
// Divide world into grid cells
// Precision 0.0001 ≈ 11 meters

struct GridCell {
    x: i32,  // (longitude / 0.0001).round()
    y: i32,  // (latitude / 0.0001).round()
}

// Both readings map to same cell
let cell1 = GridCell::from(37.7749, -122.4194);
let cell2 = GridCell::from(37.77491, -122.41939);
assert_eq!(cell1, cell2);  // Same grid cell!
```

**Trade-offs**:
- **Finer grid** (0.00001): More cells, fewer false matches, higher memory
- **Coarser grid** (0.001): Fewer cells, more false matches, lower memory

**Typical**: 0.0001 degrees ≈ 11 meters is good balance.

### 6. Selective Field Hashing for Composite Keys

Not all struct fields should affect equality/hashing:

**The Problem**:
```rust
struct Record {
    id: u32,           // Key field
    name: String,      // Key field
    timestamp: u64,    // NOT a key field (metadata)
}

// Derive would hash ALL fields
#[derive(Hash, PartialEq)]  // Wrong! Timestamp affects equality
```

**Solution: Manual Implementation**:
```rust
impl Hash for Record {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Only hash key fields
        self.id.hash(state);
        self.name.hash(state);
        // Explicitly omit timestamp
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
        // Timestamp differences don't affect equality
    }
}
```

**Why This Matters**: Database-style queries where some fields are "keys" and others are "values."

### 7. Hasher Selection: SipHash vs FxHash

Rust's default HashMap uses SipHash-1-3, a cryptographic hash function:

**SipHash Characteristics**:
- **Cryptographically secure**: Resistant to hash collision DoS attacks
- **Slow**: ~10-15 CPU cycles per byte
- **Key-dependent**: Each HashMap uses random seed

**When to Use**: Untrusted keys (user input, network data)

**FxHash Characteristics**:
- **Non-cryptographic**: Vulnerable to collision attacks
- **Fast**: ~1-2 CPU cycles per byte (10× faster)
- **Simple**: Just XOR and multiply

**When to Use**: Trusted keys (internal IDs, counters)

**Attack Scenario (SipHash prevents)**:
```rust
// Attacker crafts keys that all hash to same bucket
let malicious_keys = craft_colliding_keys();

// SipHash: Random seed makes attack infeasible
// FxHash: All keys collide → O(n) lookups → DoS!
```

**Performance Comparison** (1M insertions):
```rust
HashMap<u64, u64>:      150ms (SipHash)
FxHashMap<u64, u64>:     15ms (FxHash)
// 10× speedup!
```

### 8. Cryptographic Hashing for Content Addressing

Content-addressable storage uses cryptographic hashes as identifiers:

**SHA-256 Properties**:
- **Deterministic**: Same input → same hash
- **Unique**: Different inputs → different hashes (collision probability ≈ 0)
- **One-way**: Hash → input is infeasible

**Pattern**:
```rust
let content = b"Hello, World!";
let hash = sha256(content);  // 32-byte hash

storage.insert(hash, content);
// Later: retrieve by hash
let retrieved = storage.get(&hash);
```

**Automatic Deduplication**:
```rust
// Store same content twice
let hash1 = storage.store(b"data");
let hash2 = storage.store(b"data");

assert_eq!(hash1, hash2);  // Same hash!
assert_eq!(storage.len(), 1);  // Stored only once
```

**Use Cases**:
- **Git**: Commits identified by SHA-1
- **Docker**: Layers identified by SHA-256
- **IPFS**: Content-addressed file system

### 9. The Newtype Pattern for Type Safety

Wrapping types prevents accidental mixing:

**Without Newtype** (error-prone):
```rust
let mut user_cache: HashMap<String, User> = HashMap::new();
let mut session_cache: HashMap<String, Session> = HashMap::new();

// Bug: Wrong cache!
user_cache.insert(session_id, user);  // Compiles but wrong!
```

**With Newtype** (type-safe):
```rust
struct UserId(String);
struct SessionId(String);

let mut user_cache: HashMap<UserId, User> = HashMap::new();
let mut session_cache: HashMap<SessionId, Session> = HashMap::new();

user_cache.insert(SessionId("..."), user);  // Won't compile!
```

**Additional Benefits**:
- Custom Hash/Eq implementations
- Clear intent in API signatures
- Prevents type confusion bugs

### 10. Hash Invariants and Debugging

Violating Hash/Eq contract causes subtle bugs:

**Symptom**: "I inserted X but can't retrieve it!"

**Diagnostic**:
```rust
#[test]
fn verify_hash_eq_invariant() {
    let a = MyType::new(...);
    let b = MyType::new(...);

    if a == b {
        // Hash MUST be equal
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish(), "Hash/Eq contract violated!");
    }
}
```

**Common Violations**:
- Hashing fields not in `Eq` comparison
- Mutable fields affecting hash (HashMap keys must be immutable)
- Floating-point comparisons with tolerance in Eq but not Hash

### Connection to This Project

This custom hash functions project demonstrates how to encode business semantics into the type system:

**Case-Insensitive Strings (Step 1)**: The `CaseInsensitiveString` wrapper implements Hash by normalizing to lowercase, ensuring "Content-Type" and "content-type" map to the same HashMap bucket. This prevents lookup failures common in HTTP header processing.

**Quantized Coordinates (Step 2)**: Converting floating-point GPS coordinates to integer grid cells solves the "never exactly equal" problem. Two readings 1 meter apart map to the same `QuantizedPoint`, enabling O(1) spatial queries instead of O(n) distance calculations.

**Selective Field Hashing (Step 3)**: The `LocationKey` demonstrates hashing only business-relevant fields (category, region) while ignoring metadata. This enables database-style composite keys where some fields are part of the key, others are values.

**FxHash Performance (Step 4)**: Benchmarking SipHash vs FxHash reveals 10× speedup for trusted integer keys. For internal ID lookups (user IDs, request IDs), FxHash eliminates cryptographic overhead without security risks.

**Content-Addressable Storage (Step 5)**: Using SHA-256 hashes as keys enables automatic deduplication—identical content gets the same hash. This pattern, used by Git and Docker, can reduce storage by 100-1000× for redundant data.

**Performance Validation (Step 6)**: Comprehensive benchmarks measure real-world impact of each optimization, moving from "X should be faster" to "X is 10× faster for our workload."

By the end of this project, you'll have built **type-safe abstractions** that prevent entire classes of bugs (case-sensitivity errors, floating-point equality issues, type confusion) while achieving dramatic performance improvements through hasher selection—the same techniques used in production systems like HTTP caches, GIS databases, and content distribution networks.

---

## Build The Project

## Step 1: Case-Insensitive String Wrapper

### Introduction

HTTP headers, usernames, DNS records need case-insensitive matching: "Content-Type" should equal "content-type". This requires custom `Hash` and `Eq` implementations.

### Architecture

**Structs:**
- `CaseInsensitiveString` - Newtype wrapper around String
  - **Field** `inner: String` - Actual string storage

**Traits to Implement:**
- `Hash` - Hash lowercase version
- `PartialEq / Eq` - Compare case-insensitively
- `From<String>`, `AsRef<str>` - Conversions

**Role Each Plays:**
- Newtype pattern prevents accidental usage of wrong comparison
- `Hash` must match `Eq`: if `a == b` then `hash(a) == hash(b)`
- Hashing lowercase ensures consistent buckets

### Checkpoint Tests

```rust
#[test]
fn test_case_insensitive_equality() {
    let s1 = CaseInsensitiveString::from("Hello");
    let s2 = CaseInsensitiveString::from("hello");
    let s3 = CaseInsensitiveString::from("HELLO");

    assert_eq!(s1, s2);
    assert_eq!(s2, s3);
}

#[test]
fn test_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let s1 = CaseInsensitiveString::from("Test");
    let s2 = CaseInsensitiveString::from("test");

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    s1.hash(&mut hasher1);
    s2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_hashmap_usage() {
    let mut map = HashMap::new();
    map.insert(CaseInsensitiveString::from("Content-Type"), "application/json");

    assert_eq!(
        map.get(&CaseInsensitiveString::from("content-type")),
        Some(&"application/json")
    );
}
```

### Starter Code

```rust
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct CaseInsensitiveString {
    inner: String,
}

impl CaseInsensitiveString {
    pub fn new(s: impl Into<String>) -> Self {
        CaseInsensitiveString { inner: s.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<String> for CaseInsensitiveString {
    fn from(s: String) -> Self {
        CaseInsensitiveString::new(s)
    }
}

impl From<&str> for CaseInsensitiveString {
    fn from(s: &str) -> Self {
        CaseInsensitiveString::new(s)
    }
}

impl Hash for CaseInsensitiveString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TODO: Hash the lowercase version
        // Hint: self.inner.to_lowercase().hash(state)
        unimplemented!()
    }
}

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Compare case-insensitively
        // Hint: self.inner.eq_ignore_ascii_case(&other.inner)
        unimplemented!()
    }
}

impl Eq for CaseInsensitiveString {}

// Additional challenge: implement Ord for sorted maps
```

**Why previous step is not enough:** N/A - Foundation step.

**What's the improvement:** Custom Hash enables semantic correctness. Alternative (normalizing strings before insert) is error-prone - forgetting normalization breaks lookups. Type-safe wrapper prevents mistakes at compile time.

---

## Step 2: Quantized Float Coordinates

### Introduction

Floating-point coordinates can't be HashMap keys directly (NaN != NaN, 37.77490 != 37.77491 due to precision). Quantization rounds to grid cells, enabling approximate matching with tolerance.

### Architecture

**Structs:**
- `QuantizedPoint` - Grid-aligned coordinate
  - **Field** `x: i32` - Quantized X (degrees × 10000)
  - **Field** `y: i32` - Quantized Y

- `SpatialIndex<T>` - Geographic lookup table
  - **Field** `locations: HashMap<QuantizedPoint, Vec<T>>` - Items per grid cell

**Key Functions:**
- `QuantizedPoint::from_coords(lat: f64, lon: f64, precision: f64)` - Convert float to quantized
- `SpatialIndex::insert(lat, lon, item)` - Add item at location
- `SpatialIndex::query_near(lat, lon, tolerance)` - Find items within tolerance

**Role Each Plays:**
- Quantization: `(lat × 10000).round() as i32` converts float to integer
- Tolerance queries check surrounding grid cells
- Vec per cell handles multiple items at same location

### Checkpoint Tests

```rust
#[test]
fn test_quantization() {
    let p1 = QuantizedPoint::from_coords(37.7749, -122.4194, 0.0001);
    let p2 = QuantizedPoint::from_coords(37.77491, -122.41941, 0.0001);

    // Should be same grid cell
    assert_eq!(p1, p2);
}

#[test]
fn test_different_cells() {
    let p1 = QuantizedPoint::from_coords(37.7749, -122.4194, 0.0001);
    let p2 = QuantizedPoint::from_coords(37.7750, -122.4194, 0.0001);

    // Different grid cells
    assert_ne!(p1, p2);
}

#[test]
fn test_spatial_index() {
    let mut index = SpatialIndex::new(0.0001);
    index.insert(37.7749, -122.4194, "Location A");
    index.insert(37.77491, -122.41939, "Location B"); // Very close

    let results = index.query_exact(37.7749, -122.4194);
    assert_eq!(results.len(), 2); // Both in same cell
}

#[test]
fn test_tolerance_query() {
    let mut index = SpatialIndex::new(0.0001);
    index.insert(37.7749, -122.4194, "A");
    index.insert(37.7751, -122.4194, "B"); // Nearby cell

    // Should find both with tolerance
    let results = index.query_near(37.7750, -122.4194, 0.0002);
    assert!(results.len() >= 2);
}
```

### Starter Code

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuantizedPoint {
    x: i32,
    y: i32,
}

impl QuantizedPoint {
    pub fn from_coords(lat: f64, lon: f64, precision: f64) -> Self {
        // TODO: Quantize coordinates
        // Convert lat/lon to integer grid cells
        // Hint: (lat / precision).round() as i32
        unimplemented!()
    }

    pub fn neighbors(&self) -> Vec<QuantizedPoint> {
        // TODO: Return 8 surrounding cells + self (9 total)
        // For tolerance queries
        unimplemented!()
    }
}

pub struct SpatialIndex<T> {
    locations: HashMap<QuantizedPoint, Vec<T>>,
    precision: f64,
}

impl<T> SpatialIndex<T> {
    pub fn new(precision: f64) -> Self {
        // TODO: Create index with given precision
        unimplemented!()
    }

    pub fn insert(&mut self, lat: f64, lon: f64, item: T) {
        // TODO: Quantize point and insert into HashMap
        // Hint: Use entry API to append to Vec
        unimplemented!()
    }

    pub fn query_exact(&self, lat: f64, lon: f64) -> Vec<&T> {
        // TODO: Return items at exact grid cell
        unimplemented!()
    }

    pub fn query_near(&self, lat: f64, lon: f64, tolerance: f64) -> Vec<&T> {
        // TODO: Query point + neighbors for tolerance matching
        // Hint: Use neighbors() to get adjacent cells
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Case-insensitive strings work for exact matches. Geographic data needs approximate matching - coordinates never match exactly due to GPS precision, rounding.

**What's the improvement:** Quantization enables O(1) approximate lookups:
- Naive (scan all points, compute distance): O(n) per query
- Quantized grid: O(1) to find cell, O(k) items in cell where k << n

For 1M locations, finding nearby points:
- Naive: 1M distance calculations
- Quantized: ~10 items in cell (100,000× faster)

---

## Step 3: Composite Keys with Selective Hashing

### Introduction

Business queries often combine dimensions: "revenue by product+region" or "users by (age_group, country)". Composite keys must hash only relevant fields for correct semantics.

### Architecture

**Structs:**
- `LocationKey` - Composite geographic key
  - **Field** `category: String` - Business category
  - **Field** `region: String` - Geographic region
  - **Field** `_metadata: String` - Ignored in hash/eq (for display only)

**Role Each Plays:**
- Only hash category + region (not metadata)
- Critical: metadata differences don't affect HashMap placement
- Demonstrates selective field hashing

### Checkpoint Tests

```rust
#[test]
fn test_composite_key_equality() {
    let k1 = LocationKey {
        category: "restaurant".into(),
        region: "downtown".into(),
        _metadata: "details A".into(),
    };

    let k2 = LocationKey {
        category: "restaurant".into(),
        region: "downtown".into(),
        _metadata: "details B".into(), // Different metadata
    };

    // Should be equal (metadata ignored)
    assert_eq!(k1, k2);
}

#[test]
fn test_hash_ignores_metadata() {
    use std::collections::hash_map::DefaultHasher;

    let k1 = LocationKey {
        category: "cafe".into(),
        region: "north".into(),
        _metadata: "A".into(),
    };

    let k2 = LocationKey {
        category: "cafe".into(),
        region: "north".into(),
        _metadata: "B".into(),
    };

    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    k1.hash(&mut h1);
    k2.hash(&mut h2);

    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn test_hashmap_with_composite_keys() {
    let mut map = HashMap::new();

    let key = LocationKey {
        category: "restaurant".into(),
        region: "downtown".into(),
        _metadata: "".into(),
    };

    map.insert(key.clone(), vec!["Location 1", "Location 2"]);

    let query_key = LocationKey {
        category: "restaurant".into(),
        region: "downtown".into(),
        _metadata: "different metadata".into(),
    };

    assert!(map.contains_key(&query_key));
}
```

### Starter Code

```rust
#[derive(Debug, Clone)]
pub struct LocationKey {
    pub category: String,
    pub region: String,
    pub _metadata: String, // Not used in Hash/Eq
}

impl Hash for LocationKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TODO: Only hash category and region, NOT metadata
        // This is critical for correct behavior
        unimplemented!()
    }
}

impl PartialEq for LocationKey {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Only compare category and region
        unimplemented!()
    }
}

impl Eq for LocationKey {}
```

**Why previous step is not enough:** Single-field keys can't represent complex business dimensions. Queries like "restaurants in downtown" need composite keys.

**What's the improvement:** Selective field hashing enables semantic correctness:
- Hash all fields: metadata changes break lookups (wrong!)
- Hash selective fields: only business-relevant fields affect equality (correct!)

This is critical for database-style queries where some fields are keys, others are values.

---

## Step 4: Fast Integer Hasher (FxHash)

### Introduction

Default SipHash is cryptographically secure but slow. For trusted integer keys (IDs, counters), FxHash is 10× faster without security overhead.

### Architecture

**Dependencies:** Add `rustc-hash = "1.1"` to Cargo.toml

**Usage:**
- `FxHashMap<K, V>` instead of `HashMap<K, V>`
- Faster hashing for integer keys
- Benchmark comparison

### Checkpoint Tests

```rust
use rustc_hash::FxHashMap;
use std::time::Instant;

#[test]
fn test_fxhash_correctness() {
    let mut map: FxHashMap<u64, String> = FxHashMap::default();
    map.insert(1, "one".into());
    map.insert(2, "two".into());

    assert_eq!(map.get(&1), Some(&"one".into()));
    assert_eq!(map.len(), 2);
}

#[test]
fn benchmark_hashers() {
    const N: u64 = 1_000_000;

    // Standard HashMap (SipHash)
    let start = Instant::now();
    let mut std_map = HashMap::new();
    for i in 0..N {
        std_map.insert(i, i * 2);
    }
    let std_duration = start.elapsed();

    // FxHashMap
    let start = Instant::now();
    let mut fx_map = FxHashMap::default();
    for i in 0..N {
        fx_map.insert(i, i * 2);
    }
    let fx_duration = start.elapsed();

    println!("Standard HashMap: {:?}", std_duration);
    println!("FxHashMap: {:?}", fx_duration);
    println!("Speedup: {:.2}x", std_duration.as_secs_f64() / fx_duration.as_secs_f64());

    // FxHash should be significantly faster (3-10x)
    assert!(fx_duration < std_duration);
}
```

### Starter Code

```rust
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::time::Instant;

pub struct HasherBenchmark;

impl HasherBenchmark {
    pub fn compare_insertion(n: usize) -> (Duration, Duration) {
        // TODO: Benchmark HashMap vs FxHashMap insertion
        // Return (std_duration, fx_duration)
        unimplemented!()
    }

    pub fn compare_lookup(n: usize, queries: usize) -> (Duration, Duration) {
        // TODO: Benchmark lookup performance
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Custom Hash implementations enable correctness, but hasher selection impacts performance. SipHash protects against DoS but has overhead for trusted keys.

**What's the improvement:** FxHash for integer keys:
- SipHash: Secure, ~10-15 cycles per hash
- FxHash: Fast, ~1-2 cycles per hash (10× faster)

For 1M insertions:
- SipHash: ~150ms
- FxHash: ~15ms (10× faster)

**Critical:** Only use FxHash with trusted keys. Untrusted keys (user input, network data) need SipHash to prevent DoS attacks.

---

## Step 5: Content-Addressable Storage

### Introduction

Hash-based deduplication: store data once, reference by content hash. Identical content gets same hash, enabling automatic deduplication.

### Architecture

**Structs:**
- `ContentHash` - SHA256 hash wrapper
  - **Field** `hash: [u8; 32]`

- `ContentStore` - Deduplicated storage
  - **Field** `storage: HashMap<ContentHash, Vec<u8>>` - Content by hash
  - **Field** `stats: StoreStats` - Deduplication statistics

**Key Functions:**
- `store(data: &[u8]) -> ContentHash` - Store data, return hash
- `retrieve(hash: &ContentHash) -> Option<&[u8]>` - Get data by hash
- `dedup_ratio() -> f64` - Measure deduplication effectiveness

**Role Each Plays:**
- SHA256 ensures unique hash per unique content
- HashMap automatically deduplicates (same hash = same bucket)
- Stats track space savings

### Checkpoint Tests

```rust
#[test]
fn test_content_deduplication() {
    let mut store = ContentStore::new();

    let data = b"Hello, World!";

    let hash1 = store.store(data);
    let hash2 = store.store(data); // Duplicate

    assert_eq!(hash1, hash2);
    assert_eq!(store.unique_contents(), 1); // Only stored once
}

#[test]
fn test_different_content() {
    let mut store = ContentStore::new();

    let hash1 = store.store(b"Content A");
    let hash2 = store.store(b"Content B");

    assert_ne!(hash1, hash2);
    assert_eq!(store.unique_contents(), 2);
}

#[test]
fn test_dedup_ratio() {
    let mut store = ContentStore::new();

    // Store same 1KB content 10 times
    let data = vec![0u8; 1024];
    for _ in 0..10 {
        store.store(&data);
    }

    // Should have 10KB logical, 1KB physical
    assert_eq!(store.dedup_ratio(), 10.0);
}
```

### Starter Code

```rust
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash {
    hash: [u8; 32],
}

impl ContentHash {
    pub fn from_data(data: &[u8]) -> Self {
        // TODO: Compute SHA256 hash
        // Hint: Sha256::digest(data).into()
        unimplemented!()
    }
}

pub struct ContentStore {
    storage: HashMap<ContentHash, Vec<u8>>,
    total_stored_bytes: usize,   // Logical size (with duplicates)
    unique_bytes: usize,           // Physical size (after dedup)
}

impl ContentStore {
    pub fn new() -> Self {
        // TODO: Initialize store
        unimplemented!()
    }

    pub fn store(&mut self, data: &[u8]) -> ContentHash {
        // TODO: Hash data and store if not present
        // Update statistics
        // Hint: Use entry API to avoid duplicate storage
        unimplemented!()
    }

    pub fn retrieve(&self, hash: &ContentHash) -> Option<&[u8]> {
        // TODO: Return data for hash
        unimplemented!()
    }

    pub fn unique_contents(&self) -> usize {
        self.storage.len()
    }

    pub fn dedup_ratio(&self) -> f64 {
        // TODO: Return total_stored / unique_bytes
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Fast hashers help performance, but don't enable deduplication. Content-addressable storage needs cryptographic hashes to ensure uniqueness.

**What's the improvement:** Automatic deduplication through hashing:
- Explicit dedup checks: O(n) comparisons to find duplicates
- Hash-based: O(1) lookup to detect duplicate

For storing 1000 duplicate 1MB files:
- Naive: 1GB storage
- Content-addressed: 1MB storage (1000× savings)

Git, Docker, and backup systems use this pattern for massive space savings.

---

## Step 6: Performance Comparison Dashboard

### Introduction

Benchmark all custom hash implementations to understand trade-offs and validate optimization claims.

### Architecture

**Benchmarks:**
1. Case-insensitive vs case-sensitive HashMap
2. Quantized vs raw float HashMap attempts
3. FxHash vs SipHash for integers
4. Content-addressable dedup effectiveness

**Output:**
- Operations/second for each approach
- Memory usage comparison
- Deduplication ratios

### Starter Code

```rust
pub struct HashBenchmarks;

impl HashBenchmarks {
    pub fn run_all() {
        Self::bench_case_insensitive();
        Self::bench_spatial_index();
        Self::bench_hashers();
        Self::bench_content_dedup();
    }

    fn bench_case_insensitive() {
        // TODO: Compare case-sensitive vs case-insensitive performance
        println!("=== Case-Insensitive Benchmark ===");
        // Measure insertion and lookup times
    }

    fn bench_spatial_index() {
        // TODO: Compare quantized vs linear scan
        println!("=== Spatial Index Benchmark ===");
    }

    fn bench_hashers() {
        // TODO: SipHash vs FxHash
        println!("=== Hasher Comparison ===");
    }

    fn bench_content_dedup() {
        // TODO: Measure dedup effectiveness
        println!("=== Content Deduplication ===");
    }
}
```

**Why previous step is not enough:** Understanding techniques theoretically is insufficient. Measurements validate claims and reveal real-world performance.

**What's the improvement:** Data-driven decisions:
- Claims: "FxHash is 10× faster"
- Benchmark: Proves it's true for your workload
- Reveals when optimizations matter (hot paths) vs don't (cold paths)

---

## Complete Working Example

```rust
// See companion file: hashmap-custom-hash-complete.rs
// Includes full implementations of all steps with benchmarks

fn main() {
    println!("=== Custom Hash Functions Demo ===\n");

    // Step 1: Case-insensitive
    demo_case_insensitive();

    // Step 2: Spatial indexing
    demo_spatial_index();

    // Step 3: Composite keys
    demo_composite_keys();

    // Step 4: Fast hashers
    demo_hashers();

    // Step 5: Content-addressable storage
    demo_content_store();

    // Step 6: Benchmarks
    HashBenchmarks::run_all();
}
```

### Complete Working Code

```rust
use rustc_hash::FxHashMap;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

// =============================================================================
// Milestone 1: Case-Insensitive String Hashing
// =============================================================================

#[derive(Debug, Clone)]
pub struct CaseInsensitiveString {
    inner: String,
}

impl CaseInsensitiveString {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self { inner: s.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<&str> for CaseInsensitiveString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for CaseInsensitiveString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl Hash for CaseInsensitiveString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.inner.bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
    }
}

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq_ignore_ascii_case(&other.inner)
    }
}

impl Eq for CaseInsensitiveString {}

// =============================================================================
// Milestone 2: Quantized Float Coordinates
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuantizedPoint {
    x: i32,
    y: i32,
}

impl QuantizedPoint {
    pub fn from_coords(lat: f64, lon: f64, precision: f64) -> Self {
        let factor = 1.0 / precision;
        let x = (lat * factor).round() as i32;
        let y = (lon * factor).round() as i32;
        Self { x, y }
    }

    pub fn neighbors(&self) -> Vec<QuantizedPoint> {
        let mut cells = Vec::with_capacity(9);
        for dx in -1..=1 {
            for dy in -1..=1 {
                cells.push(QuantizedPoint {
                    x: self.x + dx,
                    y: self.y + dy,
                });
            }
        }
        cells
    }
}

pub struct SpatialIndex<T> {
    locations: HashMap<QuantizedPoint, Vec<T>>,
    precision: f64,
}

impl<T> SpatialIndex<T> {
    pub fn new(precision: f64) -> Self {
        Self {
            locations: HashMap::new(),
            precision,
        }
    }

    pub fn insert(&mut self, lat: f64, lon: f64, item: T) {
        let point = QuantizedPoint::from_coords(lat, lon, self.precision);
        self.locations.entry(point).or_insert_with(Vec::new).push(item);
    }

    pub fn query_exact(&self, lat: f64, lon: f64) -> Vec<&T> {
        let point = QuantizedPoint::from_coords(lat, lon, self.precision);
        self.locations
            .get(&point)
            .map(|items| items.iter().collect())
            .unwrap_or_default()
    }

    pub fn query_near(&self, lat: f64, lon: f64, tolerance: f64) -> Vec<&T> {
        let base = QuantizedPoint::from_coords(lat, lon, self.precision);
        let mut results = Vec::new();
        let max_offset = (tolerance / self.precision).ceil() as i32;
        for dx in -max_offset..=max_offset {
            for dy in -max_offset..=max_offset {
                let point = QuantizedPoint {
                    x: base.x + dx,
                    y: base.y + dy,
                };
                if let Some(items) = self.locations.get(&point) {
                    results.extend(items.iter());
                }
            }
        }
        results
    }
}

// =============================================================================
// Milestone 3: Composite Keys with Selective Hashing
// =============================================================================

#[derive(Debug, Clone)]
pub struct LocationKey {
    pub category: String,
    pub region: String,
    pub _metadata: String,
}

impl Hash for LocationKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.category.hash(state);
        self.region.hash(state);
    }
}

impl PartialEq for LocationKey {
    fn eq(&self, other: &Self) -> bool {
        self.category == other.category && self.region == other.region
    }
}

impl Eq for LocationKey {}

// =============================================================================
// Milestone 4: Fast Integer Hasher (FxHash)
// =============================================================================

pub struct HasherBenchmark;

impl HasherBenchmark {
    pub fn compare_insertion(n: usize) -> (Duration, Duration) {
        let start = Instant::now();
        let mut std_map = HashMap::new();
        for i in 0..n {
            std_map.insert(i, i);
        }
        let std_duration = start.elapsed();

        let start = Instant::now();
        let mut fx_map = FxHashMap::default();
        for i in 0..n {
            fx_map.insert(i, i);
        }
        let fx_duration = start.elapsed();

        (std_duration, fx_duration)
    }

    pub fn compare_lookup(n: usize, queries: usize) -> (Duration, Duration) {
        let mut std_map = HashMap::new();
        let mut fx_map = FxHashMap::default();
        for i in 0..n {
            std_map.insert(i, i * 2);
            fx_map.insert(i, i * 2);
        }

        let start = Instant::now();
        let mut sum = 0usize;
        for i in 0..queries {
            if let Some(v) = std_map.get(&(i % n)) {
                sum += *v;
            }
        }
        let std_duration = start.elapsed();

        let start = Instant::now();
        let mut sum_fx = 0usize;
        for i in 0..queries {
            if let Some(v) = fx_map.get(&(i % n)) {
                sum_fx += *v;
            }
        }
        let fx_duration = start.elapsed();
        assert_eq!(sum, sum_fx);

        (std_duration, fx_duration)
    }
}

// =============================================================================
// Milestone 5: Content-Addressable Storage
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash {
    hash: [u8; 32],
}

impl ContentHash {
    pub fn from_data(data: &[u8]) -> Self {
        let digest = Sha256::digest(data);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest);
        Self { hash }
    }
}

pub struct ContentStore {
    storage: HashMap<ContentHash, Vec<u8>>,
    total_stored_bytes: usize,
    unique_bytes: usize,
}

impl ContentStore {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            total_stored_bytes: 0,
            unique_bytes: 0,
        }
    }

    pub fn store(&mut self, data: &[u8]) -> ContentHash {
        let hash = ContentHash::from_data(data);
        self.total_stored_bytes += data.len();
        self.storage
            .entry(hash)
            .or_insert_with(|| {
                self.unique_bytes += data.len();
                data.to_vec()
            });
        hash
    }

    pub fn retrieve(&self, hash: &ContentHash) -> Option<&[u8]> {
        self.storage.get(hash).map(|data| data.as_slice())
    }

    pub fn unique_contents(&self) -> usize {
        self.storage.len()
    }

    pub fn dedup_ratio(&self) -> f64 {
        if self.unique_bytes == 0 {
            0.0
        } else {
            self.total_stored_bytes as f64 / self.unique_bytes as f64
        }
    }
}

// =============================================================================
// Milestone 6: Benchmark Dashboard
// =============================================================================

pub struct HashBenchmarks;

impl HashBenchmarks {
    pub fn run_all() {
        Self::bench_case_insensitive();
        Self::bench_spatial_index();
        Self::bench_hashers();
        Self::bench_content_dedup();
    }

    fn bench_case_insensitive() {
        println!("=== Case-Insensitive Benchmark ===");
        let mut map_sensitive = HashMap::new();
        let mut map_insensitive = HashMap::new();
        for i in 0..10_000 {
            let key = format!("Header{}", i);
            map_sensitive.insert(key.clone(), i);
            map_insensitive.insert(CaseInsensitiveString::from(key), i);
        }
        let lookup_key = "header500";
        let start = Instant::now();
        let _ = map_sensitive.get(lookup_key);
        let sensitive = start.elapsed();
        let start = Instant::now();
        let _ = map_insensitive.get(&CaseInsensitiveString::from(lookup_key));
        let insensitive = start.elapsed();
        println!("Sensitive: {:?}, Case-insensitive: {:?}", sensitive, insensitive);
    }

    fn bench_spatial_index() {
        println!("=== Spatial Index Benchmark ===");
        let mut index = SpatialIndex::new(0.0001);
        for i in 0..10_000 {
            index.insert(37.0 + (i as f64 * 0.00001), -122.0, i);
        }
        let start = Instant::now();
        let results = index.query_near(37.5, -122.0, 0.0002);
        let duration = start.elapsed();
        println!("Found {} results in {:?}", results.len(), duration);
    }

    fn bench_hashers() {
        println!("=== Hasher Comparison ===");
        let (std_duration, fx_duration) = HasherBenchmark::compare_insertion(200_000);
        println!("Insertion - Std: {:?}, Fx: {:?}", std_duration, fx_duration);
        let (std_lookup, fx_lookup) = HasherBenchmark::compare_lookup(200_000, 400_000);
        println!("Lookup - Std: {:?}, Fx: {:?}", std_lookup, fx_lookup);
    }

    fn bench_content_dedup() {
        println!("=== Content Deduplication ===");
        let mut store = ContentStore::new();
        let data = vec![1u8; 1024];
        for _ in 0..100 {
            store.store(&data);
        }
        println!("Dedup ratio: {:.2}", store.dedup_ratio());
    }
}

fn main() {
    HashBenchmarks::run_all();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_insensitive_hash() {
        use std::collections::hash_map::DefaultHasher;
        let s1 = CaseInsensitiveString::from("Content-Type");
        let s2 = CaseInsensitiveString::from("content-type");
        assert_eq!(s1, s2);
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        s1.hash(&mut h1);
        s2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn quantized_points_same_cell() {
        let p1 = QuantizedPoint::from_coords(37.7749, -122.4194, 0.0001);
        let p2 = QuantizedPoint::from_coords(37.77491, -122.41941, 0.0001);
        assert_eq!(p1, p2);
    }

    #[test]
    fn spatial_index_queries() {
        let mut index = SpatialIndex::new(0.0001);
        index.insert(37.7749, -122.4194, "A");
        index.insert(37.7750, -122.4194, "B");
        assert_eq!(index.query_exact(37.7749, -122.4194).len(), 1);
        assert!(index.query_near(37.77495, -122.4194, 0.0002).len() >= 1);
    }

    #[test]
    fn composite_key_hash() {
        use std::collections::hash_map::DefaultHasher;
        let k1 = LocationKey {
            category: "restaurant".into(),
            region: "downtown".into(),
            _metadata: "A".into(),
        };
        let k2 = LocationKey {
            category: "restaurant".into(),
            region: "downtown".into(),
            _metadata: "B".into(),
        };
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        k1.hash(&mut h1);
        k2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
        assert_eq!(k1, k2);
    }

    #[test]
    fn content_store_dedup() {
        let mut store = ContentStore::new();
        let hash1 = store.store(b"hello");
        let hash2 = store.store(b"hello");
        assert_eq!(hash1, hash2);
        assert_eq!(store.unique_contents(), 1);
        assert!(store.dedup_ratio() >= 2.0);
        assert_eq!(store.retrieve(&hash1).unwrap(), b"hello");
    }

    #[test]
    fn hasher_benchmarks_compare() {
        let (std_insert, fx_insert) = HasherBenchmark::compare_insertion(10_000);
        let (std_lookup, fx_lookup) = HasherBenchmark::compare_lookup(10_000, 20_000);
        assert!(fx_insert <= std_insert * 2);
        assert!(fx_lookup <= std_lookup * 2);
    }
}
```