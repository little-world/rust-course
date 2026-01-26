//! Pattern 2: Slice Algorithms
//! Example: Common Slice Operations
//!
//! Run with: cargo run --example p2_slice_ops

fn main() {
    println!("=== Common Slice Operations ===\n");

    // Rotation
    println!("=== Cyclic Rotation ===\n");

    fn rotate_buffer(buffer: &mut [u8], offset: usize) {
        buffer.rotate_left(offset % buffer.len());
    }

    let mut data = vec![1u8, 2, 3, 4, 5];
    println!("Original: {:?}", data);

    rotate_buffer(&mut data, 2);
    println!("Rotate left 2: {:?}", data);

    data.rotate_right(2);
    println!("Rotate right 2: {:?}", data);

    // Deduplication
    println!("\n=== Deduplication on Sorted Data ===\n");

    fn unique_sorted(items: &mut Vec<i32>) {
        items.sort_unstable();
        items.dedup();
    }

    let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
    println!("Original: {:?}", data);

    unique_sorted(&mut data);
    println!("After sort + dedup: {:?}", data);

    // In-place filtering with retain
    println!("\n=== In-Place Filtering with Retain ===\n");

    #[derive(Debug)]
    struct Item {
        id: usize,
        valid: bool,
    }

    impl Item {
        fn is_valid(&self) -> bool {
            self.valid
        }
    }

    let mut items: Vec<Item> = (0..10)
        .map(|i| Item { id: i, valid: i % 2 == 0 })
        .collect();

    println!("Before retain: {} items", items.len());
    items.retain(|item| item.is_valid());
    println!("After retain: {} items", items.len());
    println!("Remaining: {:?}", items.iter().map(|i| i.id).collect::<Vec<_>>());

    // Reverse operations
    println!("\n=== Reverse Operations ===\n");

    fn reverse_segments(data: &mut [u8], segment_size: usize) {
        for chunk in data.chunks_mut(segment_size) {
            chunk.reverse();
        }
    }

    let mut data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    println!("Original: {:?}", data);

    reverse_segments(&mut data, 4);
    println!("Segments reversed (size 4): {:?}", data);

    // Fill operations
    println!("\n=== Fill Operations ===\n");

    fn initialize_buffer(buffer: &mut [u8], pattern: u8) {
        buffer.fill(pattern);
    }

    fn initialize_with_indices(buffer: &mut [usize]) {
        let mut counter = 0;
        buffer.fill_with(|| {
            let val = counter;
            counter += 1;
            val
        });
    }

    let mut buffer = vec![0u8; 8];
    initialize_buffer(&mut buffer, 0xFF);
    println!("Filled with 0xFF: {:?}", buffer);

    let mut buffer = vec![0usize; 8];
    initialize_with_indices(&mut buffer);
    println!("Filled with indices: {:?}", buffer);

    // Swap operations
    println!("\n=== Swap Operations ===\n");

    fn swap_halves(data: &mut [u8]) {
        let mid = data.len() / 2;
        let (left, right) = data.split_at_mut(mid);
        let min_len = left.len().min(right.len());
        left[..min_len].swap_with_slice(&mut right[..min_len]);
    }

    let mut data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    println!("Original: {:?}", data);

    swap_halves(&mut data);
    println!("Halves swapped: {:?}", data);

    // Pattern matching
    println!("\n=== Pattern Matching ===\n");

    fn has_magic_header(data: &[u8]) -> bool {
        const MAGIC: &[u8] = b"\x89PNG";
        data.starts_with(MAGIC)
    }

    let png_data = b"\x89PNGrest of file";
    let jpg_data = b"\xFF\xD8\xFFsome jpeg";

    println!("PNG header check: {}", has_magic_header(png_data));
    println!("JPG header check: {}", has_magic_header(jpg_data));

    // Finding subsequences
    println!("\n=== Finding Subsequences ===\n");

    fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len())
            .position(|window| window == needle)
    }

    let data = b"hello world, hello rust";
    let pattern = b"hello";

    match find_pattern(data, pattern) {
        Some(idx) => println!("Found '{}' at index {}",
            String::from_utf8_lossy(pattern), idx),
        None => println!("Pattern not found"),
    }

    // Find all occurrences
    let indices: Vec<usize> = data.windows(pattern.len())
        .enumerate()
        .filter_map(|(i, window)| if window == pattern { Some(i) } else { None })
        .collect();
    println!("All occurrences: {:?}", indices);

    println!("\n=== Key Points ===");
    println!("1. rotate_left/right for cyclic shifts");
    println!("2. sort + dedup for removing duplicates");
    println!("3. retain for in-place filtering");
    println!("4. fill/fill_with for initialization");
    println!("5. windows for subsequence search");
}
