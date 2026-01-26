//! Pattern 9: Gap Buffer Implementation
//! Text Editor Data Structure
//!
//! Run with: cargo run --example p9_gap_buffer

fn main() {
    println!("=== Gap Buffer Implementation ===\n");

    let mut gb = GapBuffer::from_str("Hello World");

    println!("Initial: '{}'", gb.to_string());
    println!("Cursor at: {}", gb.cursor_position());
    println!("Length: {}", gb.len());

    // Move to position 5 (after "Hello")
    println!("\n=== Edit Operations ===\n");

    gb.move_to(6);
    println!("After move_to(6): cursor at {}", gb.cursor_position());

    gb.delete_backward();  // Delete space
    gb.insert(',');
    gb.insert(' ');

    println!("After delete + insert: '{}'", gb.to_string());

    // Insert at beginning
    gb.move_to(0);
    gb.insert('>');
    gb.insert(' ');

    println!("After insert at start: '{}'", gb.to_string());

    // Delete from end
    gb.move_to(gb.len());
    gb.delete_backward();
    gb.delete_backward();
    gb.delete_backward();

    println!("After delete from end: '{}'", gb.to_string());

    // Test empty buffer
    println!("\n=== Edge Cases ===\n");

    let mut empty = GapBuffer::new();
    println!("Empty buffer: '{}'", empty.to_string());
    println!("Delete from empty: {:?}", empty.delete_backward());

    empty.insert('A');
    empty.insert('B');
    empty.insert('C');
    println!("After inserts: '{}'", empty.to_string());

    println!("\n=== Key Points ===");
    println!("1. O(1) insert/delete at cursor position");
    println!("2. O(distance) cursor movement");
    println!("3. Gap grows when full (exponential reallocation)");
    println!("4. Efficient for localized, sequential edits");
}

struct GapBuffer {
    buffer: Vec<char>,
    gap_start: usize,   // Read position
    gap_end: usize,     // Write position
}

impl GapBuffer {
    fn new() -> Self {
        GapBuffer::with_capacity(64)
    }

    fn with_capacity(capacity: usize) -> Self {
        GapBuffer {
            buffer: vec!['\0'; capacity],
            gap_start: 0,
            gap_end: capacity,
        }
    }

    fn from_str(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        let capacity = (len * 2).max(64);

        let mut buffer = vec!['\0'; capacity];
        buffer[..len].copy_from_slice(&chars);

        GapBuffer {
            buffer,
            gap_start: len,
            gap_end: capacity,
        }
    }

    // Insert character at cursor (gap_start)
    fn insert(&mut self, ch: char) {
        if self.gap_start == self.gap_end {
            self.grow();
        }

        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;
    }

    // Delete character before cursor
    fn delete_backward(&mut self) -> Option<char> {
        if self.gap_start == 0 {
            return None;
        }

        self.gap_start -= 1;
        Some(self.buffer[self.gap_start])
    }

    // Delete character after cursor
    fn delete_forward(&mut self) -> Option<char> {
        if self.gap_end == self.buffer.len() {
            return None;
        }

        let ch = self.buffer[self.gap_end];
        self.gap_end += 1;
        Some(ch)
    }

    // Move cursor left
    fn move_left(&mut self) {
        if self.gap_start > 0 {
            self.gap_start -= 1;
            self.gap_end -= 1;
            self.buffer[self.gap_end] = self.buffer[self.gap_start];
        }
    }

    // Move cursor right
    fn move_right(&mut self) {
        if self.gap_end < self.buffer.len() {
            self.buffer[self.gap_start] = self.buffer[self.gap_end];
            self.gap_start += 1;
            self.gap_end += 1;
        }
    }

    // Move cursor to position
    fn move_to(&mut self, pos: usize) {
        let current_pos = self.gap_start;

        if pos < current_pos {
            for _ in 0..(current_pos - pos) {
                self.move_left();
            }
        } else if pos > current_pos {
            for _ in 0..(pos - current_pos) {
                self.move_right();
            }
        }
    }

    fn grow(&mut self) {
        let new_capacity = self.buffer.len() * 2;
        let additional = new_capacity - self.buffer.len();

        // Extend buffer
        self.buffer.resize(new_capacity, '\0');

        // Move content after gap to end
        let content_after_gap = self.buffer.len() - self.gap_end - additional;
        for i in (0..content_after_gap).rev() {
            self.buffer[new_capacity - 1 - i] = self.buffer[self.gap_end + i];
        }

        self.gap_end = new_capacity - content_after_gap;
    }

    fn to_string(&self) -> String {
        let mut result = String::new();

        for i in 0..self.gap_start {
            result.push(self.buffer[i]);
        }

        for i in self.gap_end..self.buffer.len() {
            result.push(self.buffer[i]);
        }

        result
    }

    fn len(&self) -> usize {
        self.gap_start + (self.buffer.len() - self.gap_end)
    }

    fn cursor_position(&self) -> usize {
        self.gap_start
    }
}
