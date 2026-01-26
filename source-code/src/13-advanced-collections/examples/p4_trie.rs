//! Pattern 4: Trie and Radix Tree Structures
//! Trie for Autocomplete and Prefix Search
//!
//! Run with: cargo run --example p4_trie

use std::collections::HashMap;

#[derive(Default, Debug)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end: bool,
    count: usize, // Frequency/popularity
}

struct Trie {
    root: TrieNode,
}

impl Trie {
    fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    fn insert(&mut self, word: &str) {
        self.insert_with_count(word, 1);
    }

    fn insert_with_count(&mut self, word: &str, count: usize) {
        let mut node = &mut self.root;

        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }

        node.is_end = true;
        node.count += count;
    }

    fn search(&self, word: &str) -> bool {
        self.find_node(word).map_or(false, |node| node.is_end)
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.find_node(prefix).is_some()
    }

    fn find_node(&self, prefix: &str) -> Option<&TrieNode> {
        let mut node = &self.root;

        for ch in prefix.chars() {
            node = node.children.get(&ch)?;
        }

        Some(node)
    }

    fn autocomplete(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();

        if let Some(node) = self.find_node(prefix) {
            Self::collect_words(node, prefix.to_string(), &mut results);
        }

        results
    }

    fn collect_words(node: &TrieNode, current: String, results: &mut Vec<String>) {
        if node.is_end {
            results.push(current.clone());
        }

        for (&ch, child) in &node.children {
            let mut next = current.clone();
            next.push(ch);
            Self::collect_words(child, next, results);
        }
    }

    fn top_k_autocomplete(&self, prefix: &str, k: usize) -> Vec<(String, usize)> {
        let mut results = Vec::new();

        if let Some(node) = self.find_node(prefix) {
            Self::collect_words_with_count(node, prefix.to_string(), &mut results);
        }

        // Sort by count (descending) and take top k
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(k);
        results
    }

    fn collect_words_with_count(
        node: &TrieNode,
        current: String,
        results: &mut Vec<(String, usize)>,
    ) {
        if node.is_end {
            results.push((current.clone(), node.count));
        }

        for (&ch, child) in &node.children {
            let mut next = current.clone();
            next.push(ch);
            Self::collect_words_with_count(child, next, results);
        }
    }

    fn delete(&mut self, word: &str) -> bool {
        Self::delete_recursive(&mut self.root, word, 0)
    }

    fn delete_recursive(node: &mut TrieNode, word: &str, index: usize) -> bool {
        if index == word.len() {
            if !node.is_end {
                return false;
            }
            node.is_end = false;
            return node.children.is_empty();
        }

        let ch = word.chars().nth(index).unwrap();

        if let Some(child) = node.children.get_mut(&ch) {
            let should_delete = Self::delete_recursive(child, word, index + 1);

            if should_delete {
                node.children.remove(&ch);
                return !node.is_end && node.children.is_empty();
            }
        }

        false
    }
}

//=======================================
// Real-world: Search engine autocomplete
//=======================================
struct SearchAutocomplete {
    trie: Trie,
}

impl SearchAutocomplete {
    fn new() -> Self {
        Self {
            trie: Trie::new(),
        }
    }

    fn add_search_query(&mut self, query: &str) {
        // Normalize: lowercase
        let normalized = query.to_lowercase();
        self.trie.insert_with_count(&normalized, 1);
    }

    fn suggest(&self, prefix: &str, limit: usize) -> Vec<(String, usize)> {
        let normalized = prefix.to_lowercase();
        self.trie.top_k_autocomplete(&normalized, limit)
    }
}

//========================================
// Real-world: Dictionary with spell check
//========================================
struct Dictionary {
    trie: Trie,
}

impl Dictionary {
    fn new() -> Self {
        Self {
            trie: Trie::new(),
        }
    }

    fn add_word(&mut self, word: &str) {
        self.trie.insert(&word.to_lowercase());
    }

    fn contains(&self, word: &str) -> bool {
        self.trie.search(&word.to_lowercase())
    }

    fn suggest_corrections(&self, word: &str, max_suggestions: usize) -> Vec<String> {
        let word = word.to_lowercase();

        // Try prefixes of increasing length
        for len in (1..=word.len()).rev() {
            let prefix = &word[..len];
            let suggestions = self.trie.autocomplete(prefix);

            if !suggestions.is_empty() {
                let mut results: Vec<_> = suggestions
                    .into_iter()
                    .filter(|s| Self::edit_distance(s, &word) <= 2)
                    .collect();

                results.truncate(max_suggestions);

                if !results.is_empty() {
                    return results;
                }
            }
        }

        vec![]
    }

    fn edit_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        let mut dp = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            dp[i][0] = i;
        }
        for j in 0..=len2 {
            dp[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };

                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }

        dp[len1][len2]
    }
}

fn main() {
    println!("=== Autocomplete ===\n");

    let mut autocomplete = SearchAutocomplete::new();

    // Simulate search queries
    autocomplete.add_search_query("rust programming");
    autocomplete.add_search_query("rust tutorial");
    autocomplete.add_search_query("rust tutorial");
    autocomplete.add_search_query("rust book");
    autocomplete.add_search_query("python programming");

    println!("Suggestions for 'rust':");
    for (query, count) in autocomplete.suggest("rust", 5) {
        println!("  {} (searched {} times)", query, count);
    }

    println!("\n=== Dictionary ===\n");

    let mut dict = Dictionary::new();

    for word in ["hello", "help", "helper", "world", "word", "work"] {
        dict.add_word(word);
    }

    let test_word = "helo";
    println!("Is '{}' in dictionary? {}", test_word, dict.contains(test_word));

    println!("Suggestions for '{}':", test_word);
    for suggestion in dict.suggest_corrections(test_word, 3) {
        println!("  {}", suggestion);
    }

    println!("\n=== Key Points ===");
    println!("1. Trie insert/search: O(m) where m = word length");
    println!("2. Space: O(ALPHABET_SIZE * N * M) worst case");
    println!("3. Ideal for autocomplete and spell checking");
    println!("4. Prefix search is O(m + k) where k = results");
}
