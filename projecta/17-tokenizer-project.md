# Chapter 17: Advanced String Processing - Neural Network Tokenizers

## Project: High-Performance Text Tokenizer for Neural Networks

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
