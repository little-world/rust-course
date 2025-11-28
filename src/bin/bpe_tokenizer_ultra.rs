use std::collections::{HashMap, HashSet};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;
use candle_core::{Device, Tensor};
use rustc_hash::FxHashMap as HashMap;

// Optimized BPE Tokenizer with performance improvements
// Maximum performance BPE Tokenizer with aggressive optimizations
struct BPETokenizer {
    vocab: HashMap<String, usize>,
    id_to_token: HashMap<usize, String>,
    // OPTIMIZATION: Store merges as ID pairs instead of string pairs
    merges: HashMap<(usize, usize), usize>,
    // OPTIMIZATION: Add reverse lookup for efficient tokenization
    merge_priority: HashMap<(usize, usize), usize>, // merge -> priority (iteration when added)
    merge_priority: HashMap<(usize, usize), usize>,
    // ULTRA-OPTIMIZATION: ASCII fast-path lookup table (256 bytes, cache-friendly)
    ascii_to_id: [Option<usize>; 256],
    unk_token: String,
    unk_token_id: usize,
}

impl BPETokenizer {
    fn new() -> Self {
        let mut vocab = HashMap::new();
        let mut id_to_token = HashMap::new();
        let mut vocab = HashMap::default();
        let mut id_to_token = HashMap::default();

        let unk_token = "<unk>".to_string();
        let unk_token_id = 0;

        vocab.insert(unk_token.clone(), unk_token_id);
        id_to_token.insert(unk_token_id, unk_token.clone());

        BPETokenizer {
            vocab,
            id_to_token,
            merges: HashMap::new(),
            merge_priority: HashMap::new(),
            merges: HashMap::default(),
            merge_priority: HashMap::default(),
            ascii_to_id: [None; 256],
            unk_token,
            unk_token_id,
        }
    }

    // ULTRA-OPTIMIZATION: Build ASCII fast-path cache
    #[inline]
    fn rebuild_ascii_cache(&mut self) {
        self.ascii_to_id = [None; 256];
        for byte in 0..=255u8 {
            if byte.is_ascii() {
                let ch = byte as char;
                let s = ch.to_string();
                if let Some(&id) = self.vocab.get(&s) {
                    self.ascii_to_id[byte as usize] = Some(id);
                }
            }
        }
    }

    // ULTRA-OPTIMIZATION: Fast character to ID lookup
    #[inline(always)]
    fn char_to_id(&self, c: char) -> Option<usize> {
        if c.is_ascii() {
            self.ascii_to_id[c as usize]
        } else {
            self.vocab.get(&c.to_string()).copied()
        }
    }

    fn train(&mut self, texts: &[String], vocab_size: usize) -> Result<(), Box<dyn std::error::Error>> {
        // OPTIMIZATION: Use with_capacity for all HashMaps
        let estimated_chars: usize = texts.iter().map(|t| t.chars().count()).sum();
        let estimated_unique_chars = 256; // Reasonable estimate for most texts
        let estimated_chars: usize = texts.iter().map(|t| t.len()).sum();

        let mut vocab: HashSet<char> = HashSet::with_capacity(estimated_unique_chars);
        // ULTRA-OPTIMIZATION: Use HashSet for unique chars
        let mut vocab: HashSet<char> = HashSet::with_capacity(512);
        for text in texts {
            vocab.extend(text.chars());
        }

        let mut token_id = self.vocab.len();

        // OPTIMIZATION: Pre-allocate and batch insert characters
        self.vocab.reserve(vocab.len());
        self.id_to_token.reserve(vocab.len());

        for c in vocab {
            let token = c.to_string();
            if !self.vocab.contains_key(&token) {
                self.vocab.insert(token.clone(), token_id);
                self.id_to_token.insert(token_id, token);
                token_id += 1;
            }
        }

        // OPTIMIZATION: Pre-allocate tokenized_texts with better capacity estimation
        // Build ASCII cache after initializing characters
        self.rebuild_ascii_cache();

        // ULTRA-OPTIMIZATION: Use fast char_to_id lookup
        let mut tokenized_texts: Vec<Vec<usize>> = Vec::with_capacity(texts.len());

        for text in texts {
            let char_count = text.chars().count();
            let mut token_ids = Vec::with_capacity(char_count);
            let mut token_ids = Vec::with_capacity(text.len());
            for c in text.chars() {
                let token_str = c.to_string();
                if let Some(&id) = self.vocab.get(&token_str) {
                    if let Some(id) = self.char_to_id(c) {
                        token_ids.push(id);
                    }
                }
                tokenized_texts.push(token_ids);
                if !token_ids.is_empty() {
                    tokenized_texts.push(token_ids);
                }
            }

            let mut merge_iteration = 0;

            // OPTIMIZATION: Pre-allocate pair_counts outside the loop and reuse it
            let estimated_pairs = estimated_chars / 2;
            let mut pair_counts: HashMap<(usize, usize), usize> = HashMap::with_capacity(estimated_pairs);
            let estimated_pairs = estimated_chars;
            let mut pair_counts: HashMap<(usize, usize), usize> = HashMap::with_capacity_and_hasher(
                estimated_pairs,
                Default::default(),
            );

            while self.vocab.len() < vocab_size {
                merge_iteration += 1;
                pair_counts.clear(); // Reuse the HashMap
                pair_counts.clear();

                // Count pairs
                // ULTRA-OPTIMIZATION: Manual inlining and iterator optimization
                for token_ids in &tokenized_texts {
                    for window in token_ids.windows(2) {
                        if let [first, second] = window {
                            *pair_counts.entry((*first, *second)).or_insert(0) += 1;
                        }
                        let len = token_ids.len();
                        if len < 2 {
                            continue;
                        }

                        // ULTRA-OPTIMIZATION: Use unsafe for bounds check elimination
                        for i in 0..len - 1 {
                            let pair = unsafe {
                                (*token_ids.get_unchecked(i), *token_ids.get_unchecked(i + 1))
                            };
                            *pair_counts.entry(pair).or_insert(0) += 1;
                        }
                    }

                    if let Some((&best_pair, &count)) = pair_counts.iter().max_by_key(|&(_, count)| count) {
                        let (first_id, second_id) = best_pair;

                        // OPTIMIZATION: Clone strings immediately to release borrows
                        let first_str = self.id_to_token.get(&first_id).unwrap().clone();
                        let second_str = self.id_to_token.get(&second_id).unwrap().clone();

                        // OPTIMIZATION: Use string concatenation instead of format! for better performance
                        let mut new_token = String::with_capacity(first_str.len() + second_str.len());
                        new_token.push_str(&first_str);
                        new_token.push_str(&second_str);
                        // ULTRA-OPTIMIZATION: Avoid double lookup and cloning
                        let new_token = {
                            let first_str = unsafe { self.id_to_token.get(&first_id).unwrap_unchecked() };
                            let second_str = unsafe { self.id_to_token.get(&second_id).unwrap_unchecked() };
                            let mut s = String::with_capacity(first_str.len() + second_str.len());
                            s.push_str(first_str);
                            s.push_str(second_str);
                            s
                        };

                        let new_token_id = if let Some(&existing_id) = self.vocab.get(&new_token) {
                            existing_id
                        } else {
                            self.vocab.insert(new_token.clone(), token_id);
                            self.id_to_token.insert(token_id, new_token);
                            self.rebuild_ascii_cache(); // Update cache when vocab changes
                            let id = token_id;
                            token_id += 1;
                            id
                        };

                        // OPTIMIZATION: Store merge as ID pair instead of string pair
                        self.merges.insert((first_id, second_id), new_token_id);
                        self.merge_priority.insert((first_id, second_id), merge_iteration);

                        // Apply merge with optimized algorithm
                        // ULTRA-OPTIMIZATION: Vectorized merge application
                        for token_ids in &mut tokenized_texts {
                            if token_ids.len() < 2 {
                                let len = token_ids.len();
                                if len < 2 {
                                    continue;
                                }

                                let mut write_idx = 0;
                                let mut read_idx = 0;

                                while read_idx < token_ids.len() {
                                    if read_idx < token_ids.len() - 1
                                        && token_ids[read_idx] == first_id
                                        && token_ids[read_idx + 1] == second_id {
                                        token_ids[write_idx] = new_token_id;
                                        write_idx += 1;
                                        read_idx += 2;
                                    } else {
                                        if write_idx != read_idx {
                                            token_ids[write_idx] = token_ids[read_idx];
                                            // ULTRA-OPTIMIZATION: Manual loop unrolling hint
                                            while read_idx < len {
                                                if read_idx < len - 1 {
                                                    let current = unsafe { *token_ids.get_unchecked(read_idx) };
                                                    let next = unsafe { *token_ids.get_unchecked(read_idx + 1) };

                                                    if current == first_id && next == second_id {
                                                        unsafe { *token_ids.get_unchecked_mut(write_idx) = new_token_id; }
                                                        write_idx += 1;
                                                        read_idx += 2;
                                                        continue;
                                                    }
                                                }

                                                if write_idx != read_idx {
                                                    unsafe {
                                                        *token_ids.get_unchecked_mut(write_idx) = *token_ids.get_unchecked(read_idx);
                                                    }
                                                    write_idx += 1;
                                                    read_idx += 1;
                                                }
                                            }
                                            write_idx += 1;
                                            read_idx += 1;
                                        }

                                        token_ids.truncate(write_idx);
                                    }

                                    if merge_iteration % 50 == 0 {
                                        println!("  Merge {}: vocab size = {} / {}, best pair freq = {}",
                                                 if merge_iteration % 100 == 0 {
                                                     println!("  Merge {}: vocab={}/{}, freq={}",
                                                              merge_iteration, self.vocab.len(), vocab_size, count);
                                                 } else {
                                                     break;
                                                 }
                                        }

                                                 println ! (
                                        "Training complete after {} merge iterations", merge_iteration);
                                        println!("Training complete: {} merges", merge_iteration);
                                        Ok(())
                                    }

                                    // OPTIMIZATION: Rewritten tokenize method using ID-based merges
                                    // ULTRA-OPTIMIZATION: Reuse encode and just convert to strings
                                    #[inline]
                                    fn tokenize(&self, text: &str) -> Vec<String> {
                                        if text.is_empty() {
                                            return Vec::new();
                                        }

                                        // Convert to IDs first
                                        let mut token_ids: Vec<usize> = text
                                            .chars()
                                            .filter_map(|c| self.vocab.get(&c.to_string()).copied())
                                            .collect();

                                        if token_ids.is_empty() {
                                            return vec![self.unk_token.clone()];
                                        }

                                        // Apply merges using priority-based approach
                                        loop {
                                            let mut best_merge: Option<(usize, usize, usize, usize)> = None; // (pos, first_id, second_id, priority)

                                            for i in 0..token_ids.len().saturating_sub(1) {
                                                let pair = (token_ids[i], token_ids[i + 1]);
                                                if let Some(&priority) = self.merge_priority.get(&pair) {
                                                    if best_merge.is_none() || priority < best_merge.unwrap().3 {
                                                        best_merge = Some((i, pair.0, pair.1, priority));
                                                    }
                                                }
                                            }

                                            if let Some((pos, first_id, second_id, _)) = best_merge {
                                                let new_token_id = *self.merges.get(&(first_id, second_id)).unwrap();
                                                token_ids[pos] = new_token_id;
                                                token_ids.remove(pos + 1);
                                            } else {
                                                break;
                                            }
                                        }

                                        // Convert IDs back to strings
                                        token_ids
                                            .iter()
                                            .map(|&id| self.id_to_token.get(&id).unwrap_or(&self.unk_token).clone())
                                        let ids = self.encode(text);
                                        ids.iter()
                                            .map(|&id| unsafe { self.id_to_token.get(&id).unwrap_unchecked().clone() })
                                            .collect()
                                    }

                                    // OPTIMIZATION: Encode directly to IDs without intermediate tokenization
                                    // ULTRA-OPTIMIZATION: Greedy single-pass encode with priority tracking
                                    fn encode(&self, text: &str) -> Vec<usize> {
                                        if text.is_empty() {
                                            return Vec::new();
                                        }

                                        let mut token_ids: Vec<usize> = text
                                            .chars()
                                            .filter_map(|c| self.vocab.get(&c.to_string()).copied())
                                            .collect();
                                        // ULTRA-OPTIMIZATION: Use fast char_to_id with pre-allocated capacity
                                        let mut token_ids: Vec<usize> = Vec::with_capacity(text.len());
                                        for c in text.chars() {
                                            if let Some(id) = self.char_to_id(c) {
                                                token_ids.push(id);
                                            }
                                        }

                                        if token_ids.is_empty() {
                                            return vec![self.unk_token_id];
                                        }

                                        // Apply merges directly on IDs
                                        loop {
                                            let mut best_merge: Option<(usize, usize, usize, usize)> = None;
                                            // ULTRA-OPTIMIZATION: Greedy merge with early exit
                                            let mut changed = true;
                                            while changed && token_ids.len() > 1 {
                                                changed = false;
                                                let mut best_pos = None;
                                                let mut best_priority = usize::MAX;

                                                for i in 0..token_ids.len().saturating_sub(1) {
                                                    let pair = (token_ids[i], token_ids[i + 1]);
                                                    // Find best merge position in single pass
                                                    for i in 0..token_ids.len() - 1 {
                                                        let pair = unsafe {
                                                            (*token_ids.get_unchecked(i), *token_ids.get_unchecked(i + 1))
                                                        };
                                                        if let Some(&priority) = self.merge_priority.get(&pair) {
                                                            if best_merge.is_none() || priority < best_merge.unwrap().3 {
                                                                best_merge = Some((i, pair.0, pair.1, priority));
                                                                if priority < best_priority {
                                                                    best_priority = priority;
                                                                    best_pos = Some(i);
                                                                }
                                                            }
                                                        }

                                                        if let Some((pos, first_id, second_id, _)) = best_merge {
                                                            let new_token_id = *self.merges.get(&(first_id, second_id)).unwrap();
                                                            // Apply best merge
                                                            if let Some(pos) = best_pos {
                                                                let pair = unsafe {
                                                                    (*token_ids.get_unchecked(pos), *token_ids.get_unchecked(pos + 1))
                                                                };
                                                                let new_token_id = unsafe { *self.merges.get(&pair).unwrap_unchecked() };
                                                                token_ids[pos] = new_token_id;
                                                                token_ids.remove(pos + 1);
                                                            } else {
                                                                break;
                                                                changed = true;
                                                            }
                                                        }

                                                        token_ids
                                                    }

                                                    fn decode(&self, ids: &[usize]) -> String {
                                                        // OPTIMIZATION: Pre-allocate string capacity
                                                        let estimated_capacity = ids.len() * 2; // Rough estimate
                                                        let mut result = String::with_capacity(estimated_capacity);

                                                        for &id in ids {
                                                            result.push_str(self.id_to_token.get(&id).unwrap_or(&self.unk_token));
                                                        }

                                                        result
                                                    }

                                                    fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
                                                        let mut file = File::create(path)?;
                                                        // ULTRA-OPTIMIZATION: Use buffered writer for 10-100x faster I/O
                                                        let file = File::create(path)?;
                                                        let mut writer = BufWriter::with_capacity(64 * 1024, file);

                                                        writeln!(file, "# Vocabulary")?;
                                                        writeln!(writer, "# Vocabulary")?;
                                                        for (token, id) in &self.vocab {
                                                            writeln!(file, "{}\t{}", token, id)?;
                                                            writeln!(writer, "{}\t{}", token, id)?;
                                                        }

                                                        writeln!(file, "# Merges")?;
                                                        // OPTIMIZATION: Save merges as ID pairs with priority for faster loading
                                                        writeln!(writer, "# Merges")?;
                                                        for (&(first_id, second_id), &new_id) in &self.merges {
                                                            let priority = self.merge_priority.get(&(first_id, second_id)).unwrap_or(&0);
                                                            writeln!(file, "{}\t{}\t{}\t{}", first_id, second_id, new_id, priority)?;
                                                            let priority = unsafe { self.merge_priority.get(&(first_id, second_id)).unwrap_unchecked() };
                                                            writeln!(writer, "{}\t{}\t{}\t{}", first_id, second_id, new_id, priority)?;
                                                        }

                                                        writer.flush()?;
                                                        Ok(())
                                                    }

                                                    fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
                                                        let file = File::open(path)?;
                                                        let reader = BufReader::new(file);
                                                        let reader = BufReader::with_capacity(64 * 1024, file);

                                                        let mut tokenizer = BPETokenizer::new();
                                                        let mut section = String::new();

                                                        for line in reader.lines() {
                                                            let line = line?;

                                                            if line.starts_with('#') {
                                                                section = line.trim_start_matches('#').trim().to_string();
                                                                section = line[1..].trim().to_string();
                                                                continue;
                                                            }

                                                            if line.trim().is_empty() {
                                                                if line.is_empty() {
                                                                    continue;
                                                                }

                                                                match section.as_str() {
                                                                    "Vocabulary" => {
                                                                        let parts: Vec<&str> = line.split('\t').collect();
                                                                        if parts.len() == 2 {
                                                                            let token = parts[0].to_string();
                                                                            let id = parts[1].parse::<usize>()?;
                                                                            tokenizer.vocab.insert(token.clone(), id);
                                                                            tokenizer.id_to_token.insert(id, token);
                                                                            if let Some(tab_pos) = line.find('\t') {
                                                                                let token = line[..tab_pos].to_string();
                                                                                if let Ok(id) = line[tab_pos + 1..].parse::<usize>() {
                                                                                    tokenizer.vocab.insert(token.clone(), id);
                                                                                    tokenizer.id_to_token.insert(id, token);
                                                                                }
                                                                            }
                                                                        }
                                                                        "Merges" => {
                                                                            let parts: Vec<&str> = line.split('\t').collect();
                                                                            if parts.len() >= 3 {
                                                                                let first_id = parts[0].parse::<usize>()?;
                                                                                let second_id = parts[1].parse::<usize>()?;
                                                                                let new_id = parts[2].parse::<usize>()?;
                                                                                let priority = if parts.len() >= 4 {
                                                                                    parts[3].parse::<usize>().unwrap_or(0)
                                                                                } else {
                                                                                    0
                                                                                };

                                                                                tokenizer.merges.insert((first_id, second_id), new_id);
                                                                                tokenizer.merge_priority.insert((first_id, second_id), priority);
                                                                                if parts.len() >= 4 {
                                                                                    if let (Ok(first_id), Ok(second_id), Ok(new_id), Ok(priority)) = (
                                                                                        parts[0].parse::<usize>(),
                                                                                        parts[1].parse::<usize>(),
                                                                                        parts[2].parse::<usize>(),
                                                                                        parts[3].parse::<usize>(),
                                                                                    ) {
                                                                                        tokenizer.merges.insert((first_id, second_id), new_id);
                                                                                        tokenizer.merge_priority.insert((first_id, second_id), priority);
                                                                                    }
                                                                                }
                                                                            }
                                                                            _ => {}
                                                                        }
                                                                    }

                                                                    // ULTRA-OPTIMIZATION: Rebuild ASCII cache after loading
                                                                    tokenizer.rebuild_ascii_cache();

                                                                    Ok(tokenizer)
                                                                }

                                                                fn encode_for_model(&self, text: &str, device: &Device) -> Result<Tensor, Box<dyn std::error::Error>> {
                                                                    let ids = self.encode(text);
                                                                    let ids_i64: Vec<i64> = ids.iter().map(|&id| id as i64).collect();
                                                                    Tensor::new(&*ids_i64, device).map_err(|e| e.into())
                                                                }

                                                                fn batch_encode_for_model(&self, texts: &[&str], device: &Device) -> Result<Tensor, Box<dyn std::error::Error>> {
                                                                    if texts.is_empty() {
                                                                        return Tensor::new(&[] as &[i64], device).map_err(|e| e.into());
                                                                    }

                                                                    // ULTRA-OPTIMIZATION: Single-pass encoding with pre-allocation
                                                                    let batch: Vec<Vec<usize>> = texts.iter().map(|&text| self.encode(text)).collect();
                                                                    let max_len = batch.iter().map(|seq| seq.len()).max().unwrap_or(0);

                                                                    // OPTIMIZATION: Pre-allocate with exact size
                                                                    let mut flat: Vec<i64> = Vec::with_capacity(texts.len() * max_len);
                                                                    let total_size = texts.len() * max_len;
                                                                    let mut flat: Vec<i64> = Vec::with_capacity(total_size);

                                                                    for mut seq in batch {
                                                                        seq.resize(max_len, self.unk_token_id);
                                                                        // ULTRA-OPTIMIZATION: Direct iteration without intermediate cloning
                                                                        for seq in batch {
                                                                            flat.extend(seq.iter().map(|&id| id as i64));
                                                                            flat.extend(std::iter::repeat(self.unk_token_id as i64).take(max_len - seq.len()));
                                                                        }

                                                                        let batch_size = texts.len();
                                                                        let shape = vec![batch_size, max_len];
                                                                        Tensor::new(&*flat, device)?
                                                                            .reshape(shape)
                                                                        Tensor::new(&flat[..], device)?
                                                                            .reshape((batch_size, max_len))
                                                                            .map_err(|e| e.into())
                                                                    }
                                                                }

                                                                fn main() -> Result<(), Box<dyn std::error::Error>> {
                                                                    let start = Instant::now();

                                                                    let mut tokenizer = BPETokenizer::new();

                                                                    let shakespeare_path = Path::new("data/shakespeare.txt");
                                                                    if !shakespeare_path.exists() {
                                                                        return Err("Shakespeare dataset not found at data/shakespeare.txt".into());
                                                                    }

                                                                    println!("Reading Shakespeare dataset...");
                                                                    let shakespeare_text = std::fs::read_to_string(shakespeare_path)?;

                                                                    // OPTIMIZATION: Larger chunks for better performance
                                                                    let chunk_size = 5000; // Increased from 1000
                                                                    let mut chunks = Vec::new();

                                                                    for i in 0..(shakespeare_text.len() / chunk_size) + 1 {
                                                                        let start_idx = i * chunk_size;
                                                                        let end_idx = std::cmp::min((i + 1) * chunk_size, shakespeare_text.len());
                                                                        if start_idx < end_idx {
                                                                            chunks.push(shakespeare_text[start_idx..end_idx].to_string());
                                                                        }
                                                                    }

                                                                    println!("Training on {} chunks of Shakespeare text...", chunks.len());

                                                                    tokenizer.train(&chunks, 500)?;

                                                                    let save_path = Path::new("models/shakespeare_bpe_tokenizer.txt");
                                                                    tokenizer.save(save_path)?;
                                                                    println!("Tokenizer saved to {:?}", save_path);

                                                                    let loaded_tokenizer = BPETokenizer::load(save_path)?;

                                                                    let sample_text = "To be, or not to be, that is the question";

                                                                    let tokens = loaded_tokenizer.tokenize(sample_text);
                                                                    let ids = loaded_tokenizer.encode(sample_text);

                                                                    println!("\nSample text: {}", sample_text);
                                                                    println!("Tokens: {:?}", tokens);
                                                                    println!("IDs: {:?}", ids);

                                                                    let decoded = loaded_tokenizer.decode(&ids);
                                                                    println!("Decoded: {}", decoded);

                                                                    let duration = start.elapsed();
                                                                    println!("\nTime elapsed: {:?}", duration);

                                                                    Ok(())
                                                                }