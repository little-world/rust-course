# Project 3: Fast String Search with Boyer-Moore Algorithm

## Problem Statement

Implement the Boyer-Moore string search algorithm for finding patterns in text efficiently. This algorithm is used in grep, text editors, and search systems because it can skip large portions of text, making it faster than naive search for most cases.

Your implementation should:
- Build bad character and good suffix tables
- Search for pattern in text with O(n/m) average case (faster than O(n))
- Support case-insensitive search
- Find all occurrences efficiently
- Benchmark against naive search

## Why It Matters

String search is fundamental to text processing. Naive search is O(n*m), checking every position. Boyer-Moore is O(n/m) average case, skipping text based on mismatches. For searching "pattern" in 1MB text:
- Naive: ~1M comparisons
- Boyer-Moore: ~150K comparisons (7x faster)

This algorithm is used in grep, text editors (find functionality), DNA sequence matching, plagiarism detection.

## Use Cases

- Text editors (find/replace)
- Log analysis (grep-style search)
- DNA sequence matching (bioinformatics)
- Intrusion detection (packet inspection)
- Plagiarism detection
- Search engines (document scanning)

---

## Introduction to String Search and Boyer-Moore Concepts

String searching is one of the most fundamental operations in computer science, yet naive approaches are surprisingly inefficient. The Boyer-Moore algorithm revolutionized text searching by introducing the counterintuitive idea of scanning patterns right-to-left and using mismatches to skip large portions of text—achieving sublinear average-case performance.

### 1. The String Search Problem

String search finds all occurrences of a pattern in a text:

**Problem Definition**:
```
Text:    "ABCABDABCABC"
Pattern: "ABC"
Output:  [0, 6, 9]  // Starting positions of matches
```

**Naive Solution** (Brute Force):
```rust
for i in 0..=(text.len() - pattern.len()) {
    if text[i..i+pattern.len()] == pattern {
        matches.push(i);
    }
}
```

**Complexity**:
- **Time**: O(n * m) where n = text length, m = pattern length
- **Space**: O(1)

**Why It's Slow**:
- Checks every position in text
- On mismatch, shifts by only 1 position
- For text of 1MB and pattern "ABCD", makes ~1M * 4 = 4M character comparisons

### 2. Boyer-Moore Algorithm Overview

Boyer-Moore achieves faster search by:

1. **Scanning right-to-left**: Start comparing from end of pattern
2. **Using mismatches**: Extract information from failures to skip positions
3. **Preprocessing pattern**: Build tables once, use for all searches

**Key Insight**: A mismatch tells us where the pattern CANNOT match, allowing safe skips.

**Example**:
```
Text:    "HERE IS A SIMPLE EXAMPLE"
Pattern: "EXAMPLE"

Position 0:
Text:    HERE IS...
Pattern: EXAMPLE
              ^ Mismatch at 'E' (text) vs 'E' (pattern)

Text has 'E' at position 6, pattern has 'E' at position 6
Character 'E' in text doesn't match 'E' in pattern at this alignment
Can skip to next possible alignment
```

**Average Case**: O(n/m) - Yes, FASTER than O(n)! Can skip m characters at a time.

### 3. Bad Character Heuristic

When a mismatch occurs, use the mismatched character in the text to determine skip distance:

**Rule**: If character `c` from text doesn't match pattern, shift pattern to align with rightmost occurrence of `c` in pattern.

**Example**:
```
Text:    "ANPANMAN"
Pattern: "NANB"

Step 1:
Text:    ANPANMAN
Pattern: NANB
            ^ Mismatch: 'A' (text) vs 'B' (pattern)

Last occurrence of 'A' in pattern is at position 1
Shift pattern to align:

Step 2:
Text:    ANPANMAN
Pattern:  NANB
             ^ Continue searching...
```

**Bad Character Table**:
```rust
// For pattern "NANB"
{
  'N': 2,  // Rightmost 'N' at index 2
  'A': 1,  // Rightmost 'A' at index 1
  'B': 3,  // Rightmost 'B' at index 3
}

// Characters not in pattern: skip entire pattern length
```

**Skip Calculation**:
```rust
let skip = match bad_char_table.get(&mismatched_char) {
    Some(&last_occurrence) => {
        // Distance from mismatch position to last occurrence
        max(1, mismatch_index - last_occurrence)
    }
    None => {
        // Character not in pattern, skip entire pattern
        pattern.len()
    }
};
```

### 4. Good Suffix Heuristic

When a mismatch occurs after some matches, use the matched suffix to determine skip distance:

**Rule**: If suffix `S` of pattern matched but prefix mismatched, shift pattern to align with another occurrence of `S` in pattern, or shift to align prefix with suffix.

**Example**:
```
Text:    "ABCXXXABC"
Pattern: "XXXABC"

Matching from right:
Text:    ABCXXXABC
Pattern: XXXABC
         ^^^
         Matched suffix "ABC", mismatch at 'C' vs 'X'

"ABC" doesn't occur elsewhere in "XXX"
But "ABC" prefix could align with next occurrence
Skip to align appropriately
```

**Good Suffix Table**: Stores shift distances for each position.

**Why It Helps**: Bad character might give small skip (shift by 1), but good suffix can give large skip.

**Combined Heuristic**: Take maximum of bad character and good suffix shifts:
```rust
let skip = max(bad_char_skip, good_suffix_skip);
```

### 5. Right-to-Left Scanning

Boyer-Moore scans patterns from right to left, contrary to intuition:

**Why Right-to-Left**:
- Mismatches near pattern end give more information
- Can skip more characters based on suffix information
- Bad character heuristic is more effective with right-to-left

**Example**:
```
Pattern: "EXAMPLE" (length 7)

Scan order: 6 → 5 → 4 → 3 → 2 → 1 → 0
            E   L   P   M   A   X   E
```

**On Early Mismatch** (near right end):
```
Text:    "...EXAM[Q]LE..."
Pattern: "EXAMPLE"
                ^ Mismatch at position 6

Character 'Q' not in pattern → skip 7 positions!
```

**On Late Mismatch** (near left end):
```
Text:    "...[Q]XAMPLE..."
Pattern: "EXAMPLE"
            ^ Mismatch at position 0

Matched "XAMPLE" (6 chars), but mismatched at 'Q'
Use good suffix to determine skip
```

### 6. Preprocessing Pattern vs Text

Boyer-Moore preprocesses the **pattern**, not the text:

**Preprocessing** (one-time cost):
```rust
let searcher = BoyerMoore::new("pattern");
// Builds bad character table: O(m + σ) where σ = alphabet size
// Builds good suffix table: O(m)
```

**Search** (used repeatedly):
```rust
searcher.search(text1);  // Uses prebuilt tables
searcher.search(text2);  // Reuses same tables
searcher.search(text3);  // No re-preprocessing
```

**Why This Matters**:
- Same pattern, multiple texts: Preprocess once, search many times
- grep searches one pattern in many files
- Text editor searches one pattern in large document

**Amortized Cost**: Preprocessing cost amortized over multiple searches.

### 7. Complexity Analysis

Boyer-Moore has different complexities for different scenarios:

**Best Case**: O(n/m)
```
Text:    "AAAAAAAAAA" (10 A's)
Pattern: "BAAA"

Every comparison: 'A' vs 'A' → mismatch
Skip 4 positions each time
Comparisons: 10/4 = 2.5 → 3 checks instead of 10!
```

**Worst Case**: O(n * m)
```
Text:    "AAAAAAAAAA"
Pattern: "AAAA"

Many partial matches before full match
Similar to naive search in pathological cases
```

**Average Case**: O(n) with low constant factor
- For random text and pattern: ~3 comparisons per position checked
- Much faster than naive's m comparisons per position

**Practical Performance**:
- English text: 3-5x faster than naive
- Random data: 5-10x faster than naive
- Long patterns: Even better (can skip more)

### 8. Handling Case Sensitivity

Case-insensitive search requires normalization:

**Approach 1**: Normalize Both (Simple)
```rust
let pattern_lower = pattern.to_lowercase();
let text_lower = text.to_lowercase();
// Search pattern_lower in text_lower
```
- **Pros**: Simple, reuses exact search
- **Cons**: Allocates new strings (2x memory)

**Approach 2**: Normalize Tables (Efficient)
```rust
// Build bad character table with both cases
for ch in pattern.chars() {
    table.insert(ch.to_lowercase(), position);
    table.insert(ch.to_uppercase(), position);
}
// Compare case-insensitively during search
```
- **Pros**: No text allocation
- **Cons**: More complex table

**Trade-off**: Simplicity vs memory efficiency.

### 9. Streaming Search for Large Files

Processing files larger than memory requires streaming:

**Chunk-Based Search**:
```rust
loop {
    let chunk = read_chunk(file, CHUNK_SIZE);
    let matches = search(chunk);
    // Problem: Pattern might span chunks!
}
```

**Solution**: Overlap Chunks
```
Chunk 1: "ABCDEFGH"
Chunk 2: "IJKLMNOP"

With overlap (pattern length - 1):
Chunk 1: "ABCDEFGH"
Chunk 2:      "GHIJKLMNOP"  // Overlap "GH"

Pattern "FGHI" spanning chunks is now found!
```

**Memory**: Constant (chunk size + pattern length) regardless of file size.

### 10. Real-World Optimizations

Production implementations add optimizations:

**Turbo Boyer-Moore**:
- Remembers last match position
- Skips even more on repeated searches
- Used in some text editors

**Horspool Simplification**:
- Uses only bad character heuristic
- Simpler, often 90% of Boyer-Moore speed
- Easier to implement correctly

**SIMD Optimization**:
- Parallel character comparison using SIMD instructions
- 4-8x speedup for patterns fitting in SIMD registers
- Used in high-performance grep implementations

**Alphabet Size Optimization**:
- Array instead of HashMap for ASCII (256 entries)
- O(1) lookups vs O(log n) for HashMap
- Critical for inner loop performance

### Connection to This Project

This Boyer-Moore implementation demonstrates advanced string search optimization in practice:

**Naive Baseline (Step 1)**: You'll implement brute-force search to establish a performance baseline. This O(n*m) algorithm checks every position, making the improvements from Boyer-Moore dramatic and measurable.

**Bad Character Heuristic (Step 2)**: Building the bad character table demonstrates preprocessing patterns for fast lookups. The HashMap maps each character to its rightmost position, enabling O(1) skip distance calculation during search.

**Right-to-Left Scanning (Step 2)**: The search loop compares from pattern end backward (`j = m - 1` down to `0`). Early mismatches near the pattern end yield large skips—the key to sublinear average-case performance.

**Skip Distance Calculation (Step 2)**: On mismatch, the algorithm calculates skip as `max(1, mismatch_pos - last_occurrence)`. This ensures forward progress even when bad character gives negative skip (character occurs to the right).

**Good Suffix Heuristic (Step 3)**: The good suffix table handles patterns with internal repetition. Building this table is complex (uses suffix arrays) but enables optimal skipping when bad character heuristic gives small shifts.

**Case Insensitivity (Step 4)**: Normalizing to lowercase before searching demonstrates the trade-off between simplicity and memory. For large texts, allocating lowercase copies doubles memory—but simplifies implementation.

**Streaming Search (Step 5)**: Overlapping chunks by `pattern.len() - 1` ensures patterns spanning boundaries are found. The overlap buffer (`buffer.copy_within`) avoids allocating new chunks—critical for processing GB-sized logs.

**Performance Benchmarking (Step 6)**: Comparing against naive search reveals 5-10x speedups for typical text. Benchmarking against Rust's built-in `str::find()` (which uses optimized Boyer-Moore-Horspool) shows how close your implementation is to production quality.

By the end of this project, you'll have built a **production-grade string search** matching grep's performance characteristics—understanding both the algorithm (Boyer-Moore heuristics) and engineering concerns (streaming, memory efficiency, cache optimization).

---

## Solution Outline

### Step 1: Naive String Search (Baseline)
**Goal**: Implement naive search for comparison.

**What to implement**:
- Search pattern in text character by character
- Return all match positions
- Measure operations count

**Why this step**: Establish baseline for comparison. Understanding naive approach makes Boyer-Moore improvements clear.

**Testing hint**: Test with various patterns and texts. Verify all matches found. Count comparisons.

```rust
pub fn naive_search(text: &str, pattern: &str) -> Vec<usize> {
    let text_bytes = text.as_bytes();
    let pattern_bytes = pattern.as_bytes();
    let mut matches = Vec::new();

    if pattern.is_empty() || pattern.len() > text.len() {
        return matches;
    }

    for i in 0..=(text.len() - pattern.len()) {
        let mut match_found = true;

        for j in 0..pattern.len() {
            if text_bytes[i + j] != pattern_bytes[j] {
                match_found = false;
                break;
            }
        }

        if match_found {
            matches.push(i);
        }
    }

    matches
}
```

---

### Step 2: Build Bad Character Table
**Goal**: Implement bad character heuristic for skipping.

**What to implement**:
- Build table mapping each character to last occurrence in pattern
- On mismatch, skip based on bad character
- Handle characters not in pattern

**Why the previous step is not enough**: Naive search checks every position. Bad character heuristic skips positions based on mismatched character.

**What's the improvement**: When mismatch occurs, skip to align pattern with last occurrence of mismatched character. This can skip multiple positions:
- Naive: Always advances by 1
- Bad character: Can skip by pattern length

**Testing hint**: Test table construction. Verify skipping logic. Test with patterns having repeated characters.

```rust
use std::collections::HashMap;

pub struct BoyerMoore {
    pattern: Vec<u8>,
    bad_char_table: HashMap<u8, usize>,
}

impl BoyerMoore {
    pub fn new(pattern: &str) -> Self {
        let pattern_bytes = pattern.as_bytes().to_vec();
        let bad_char_table = Self::build_bad_char_table(&pattern_bytes);

        BoyerMoore {
            pattern: pattern_bytes,
            bad_char_table,
        }
    }

    fn build_bad_char_table(pattern: &[u8]) -> HashMap<u8, usize> {
        let mut table = HashMap::new();

        // For each character, store its rightmost position
        for (i, &ch) in pattern.iter().enumerate() {
            table.insert(ch, i);
        }

        table
    }

    pub fn search(&self, text: &str) -> Vec<usize> {
        let text_bytes = text.as_bytes();
        let mut matches = Vec::new();
        let m = self.pattern.len();
        let n = text_bytes.len();

        if m > n {
            return matches;
        }

        let mut i = 0;
        while i <= n - m {
            let mut j = m as isize - 1;

            // Match pattern from right to left
            while j >= 0 && self.pattern[j as usize] == text_bytes[i + j as usize] {
                j -= 1;
            }

            if j < 0 {
                // Pattern found
                matches.push(i);
                i += m;
            } else {
                // Mismatch - use bad character heuristic
                let bad_char = text_bytes[i + j as usize];
                let shift = if let Some(&last_occurrence) = self.bad_char_table.get(&bad_char) {
                    let shift = j as usize - last_occurrence;
                    shift.max(1)
                } else {
                    j as usize + 1
                };

                i += shift;
            }
        }

        matches
    }
}
```

---

### Step 3: Add Good Suffix Heuristic
**Goal**: Implement good suffix table for additional skipping.

**What to implement**:
- Build good suffix table
- On mismatch, use both bad character and good suffix
- Take maximum skip from both heuristics

**Why the previous step is not enough**: Bad character heuristic alone doesn't handle all cases optimally. Good suffix adds another skipping strategy.

**What's the improvement**: Good suffix handles cases where bad character gives small skip. Using both heuristics gives maximum skip, making algorithm faster.

**Testing hint**: Test with patterns where good suffix provides larger skip. Verify both heuristics are used.

```rust
impl BoyerMoore {
    pub fn new_with_good_suffix(pattern: &str) -> Self {
        let pattern_bytes = pattern.as_bytes().to_vec();
        let bad_char_table = Self::build_bad_char_table(&pattern_bytes);
        let good_suffix_table = Self::build_good_suffix_table(&pattern_bytes);

        BoyerMoore {
            pattern: pattern_bytes,
            bad_char_table,
            // good_suffix_table, // Add this field
        }
    }

    fn build_good_suffix_table(pattern: &[u8]) -> Vec<usize> {
        let m = pattern.len();
        let mut table = vec![0; m];
        let mut suffix = vec![0; m];

        // Build suffix array
        suffix[m - 1] = m;
        let mut g = m - 1;
        let mut f = 0;

        for i in (0..m - 1).rev() {
            if i > g && suffix[i + m - 1 - f] < i - g {
                suffix[i] = suffix[i + m - 1 - f];
            } else {
                if i < g {
                    g = i;
                }
                f = i;
                while g > 0 && pattern[g - 1] == pattern[g + m - 1 - f] {
                    g -= 1;
                }
                suffix[i] = f - g + 1;
            }
        }

        // Build good suffix table from suffix array
        for i in 0..m {
            table[i] = m;
        }

        let mut j = 0;
        for i in (0..m - 1).rev() {
            if suffix[i] == i + 1 {
                while j < m - 1 - i {
                    if table[j] == m {
                        table[j] = m - 1 - i;
                    }
                    j += 1;
                }
            }
        }

        for i in 0..m - 1 {
            table[m - 1 - suffix[i]] = m - 1 - i;
        }

        table
    }
}
```

---

### Step 4: Case-Insensitive Search
**Goal**: Support case-insensitive search efficiently.

**What to implement**:
- Normalize pattern and text to lowercase
- Use same Boyer-Moore algorithm
- Alternative: modify tables to handle case

**Why the previous step is not enough**: Case-sensitive search doesn't match "Hello" with "hello". Users often want case-insensitive.

**What's the improvement**: Case-insensitive search broadens matches. Normalizing to lowercase is simplest approach.

**Testing hint**: Test matches across different cases. Verify performance is similar to case-sensitive.

```rust
impl BoyerMoore {
    pub fn new_case_insensitive(pattern: &str) -> Self {
        let normalized = pattern.to_lowercase();
        Self::new(&normalized)
    }

    pub fn search_case_insensitive(&self, text: &str) -> Vec<usize> {
        let normalized_text = text.to_lowercase();
        self.search(&normalized_text)
    }
}
```

---

### Step 5: Find All Occurrences with Streaming
**Goal**: Find matches in large files using streaming.

**What to implement**:
- Process file in chunks
- Handle pattern spanning chunk boundaries
- Use iterator for memory efficiency

**Why the previous step is not enough**: Loading entire file into memory fails for large files.

**What's the improvement**: Streaming enables processing files of any size with constant memory. Pattern boundary handling ensures no matches are missed.

**Testing hint**: Test with large files. Test patterns spanning chunks. Verify all matches found.

```rust
use std::io::{BufReader, Read};
use std::fs::File;

pub fn search_file_streaming(
    path: &str,
    pattern: &str,
    chunk_size: usize,
) -> std::io::Result<Vec<usize>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let searcher = BoyerMoore::new(pattern);
    let mut matches = Vec::new();
    let mut buffer = vec![0u8; chunk_size + pattern.len()];
    let mut overlap = 0;
    let mut total_bytes_read = 0;

    loop {
        let bytes_read = reader.read(&mut buffer[overlap..])?;
        if bytes_read == 0 {
            break;
        }

        let search_len = overlap + bytes_read;
        let text = std::str::from_utf8(&buffer[..search_len]).unwrap_or("");

        // Search in current chunk
        for match_pos in searcher.search(text) {
            matches.push(total_bytes_read + match_pos - overlap);
        }

        total_bytes_read += bytes_read;

        // Keep overlap for pattern spanning chunks
        if search_len >= pattern.len() {
            overlap = pattern.len() - 1;
            buffer.copy_within(search_len - overlap..search_len, 0);
        } else {
            overlap = search_len;
        }
    }

    Ok(matches)
}
```

---

### Step 6: Benchmark and Optimization
**Goal**: Compare performance against naive search and optimize.

**What to implement**:
- Benchmark with various pattern and text sizes
- Measure: operations count, time, cache misses
- Optimize: table lookups, memory layout
- Identify best and worst cases

**Why the previous step is not enough**: Implementation is complete, but understanding performance characteristics is essential.

**What's the improvement**: Benchmarks reveal:
- Best case: O(n/m) when pattern doesn't occur
- Worst case: O(n*m) with many false matches
- Average: Much faster than naive for most real-world text

**Optimization focus**: Understanding when Boyer-Moore excels vs when to use alternatives (e.g., KMP for small alphabets).

**Testing hint**: Benchmark with realistic text (code, prose, DNA). Test with short and long patterns. Compare with Rust's str::find().

```rust
use std::time::Instant;

pub fn benchmark_search_algorithms() {
    let text = include_str!("large_text.txt"); // 1MB text
    let pattern = "target";

    // Naive search
    let start = Instant::now();
    let _matches = naive_search(text, pattern);
    let naive_time = start.elapsed();
    println!("Naive search: {:?}", naive_time);

    // Boyer-Moore
    let searcher = BoyerMoore::new(pattern);
    let start = Instant::now();
    let _matches = searcher.search(text);
    let bm_time = start.elapsed();
    println!("Boyer-Moore: {:?}", bm_time);

    // Rust's built-in
    let start = Instant::now();
    let _matches: Vec<usize> = text.match_indices(pattern).map(|(i, _)| i).collect();
    let builtin_time = start.elapsed();
    println!("Built-in find: {:?}", builtin_time);

    println!("Speedup: {:.2}x", naive_time.as_secs_f64() / bm_time.as_secs_f64());
}
```

---

## Testing Strategies

1. **Correctness Tests**: Compare results with naive search
2. **Edge Cases**: Empty pattern, pattern longer than text, no matches
3. **Performance Tests**: Benchmark with various inputs
4. **Property Tests**: Verify all matches are found
5. **Streaming Tests**: Test chunk boundary handling
6. **Real-World Tests**: Test with code, prose, structured data
