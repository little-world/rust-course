# Project 2: Text Editor Buffer with Gap Buffer

## Problem Statement

Build a text editor buffer data structure that efficiently handles text insertion and deletion at cursor position. Implement gap buffer algorithm for O(1) insert/delete at cursor with minimal memory overhead.

Your text editor should:
- Support cursor movement (forward, backward, start, end)
- Insert character at cursor position in O(1)
- Delete character at cursor in O(1)
- Support multi-line text
- Undo/redo functionality
- Efficient memory usage (no reallocation on most operations)

## Why It Matters

Text editors need to handle millions of characters with frequent insertions/deletions. Naive approaches (Vec\<char>, String) require O(n) operations to insert in middle. Gap buffer achieves O(1) insertion at cursor by maintaining a gap at cursor position.

This pattern is used by: Emacs, many terminal emulators, and high-performance text editors. Understanding gap buffer teaches memory layout optimization and amortized analysis.

## Use Cases

- Text editors (Emacs, vim-style)
- Terminal emulators (handling backspace, insert mode)
- Command-line input buffers
- Rich text editors
- Code editors with syntax highlighting

---

## Introduction to Gap Buffer and Text Editor Concepts

Building an efficient text editor requires understanding data structures optimized for a specific access pattern: most edits happen at or near the cursor position. Gap buffer exploits this locality to achieve O(1) insertions and deletions at the cursor, making it ideal for interactive text editing.

### 1. The Gap Buffer Algorithm

Gap buffer is a data structure that maintains a contiguous array with a "gap" at the cursor position:

**Structure**:
```
Content: "Hello world"
Cursor at position 5 (after "Hello")

Buffer layout:
[H][e][l][l][o][_][_][_][_][_][w][o][r][l][d]
              ^gap_start    ^gap_end

gap_start = 5
gap_end = 10
gap_size = 5
```

**Key Operations**:

**Insert at cursor** (O(1)):
```rust
// Insert 'X' at cursor
buffer[gap_start] = 'X';
gap_start += 1;

// Result: "HelloX world"
[H][e][l][l][o][X][_][_][_][_][w][o][r][l][d]
                ^gap_start
```

**Delete at cursor** (O(1)):
```rust
// Delete character before cursor
gap_start -= 1;

// Result: "Hell world"
[H][e][l][l][_][_][_][_][_][_][w][o][r][l][d]
            ^gap_start
```

**Move cursor** (O(n) worst case, O(1) amortized):
```rust
// Move gap from position 5 to position 8
// Copy "o w" from after gap to before gap
[H][e][l][l][o][ ][w][_][_][_][o][r][l][d]
                    ^gap_start
```

**Why This Works**: Sequential editing (typing, backspace) keeps cursor at gap, making most operations O(1).

### 2. Comparison: Gap Buffer vs Alternatives

Different data structures have different trade-offs for text editing:

**Vec<char> (Naive Approach)**:
```rust
chars.insert(cursor, 'x');  // O(n) - shifts all chars after cursor
chars.remove(cursor);       // O(n) - shifts all chars after cursor
```
- **Insert**: O(n) - must shift all elements after insertion point
- **Delete**: O(n) - must shift all elements after deletion point
- **Memory**: Compact, no wasted space
- **Use case**: Small texts or append-only scenarios

**Gap Buffer**:
```rust
buffer[gap_start] = 'x';  // O(1) at cursor
gap_start += 1;
```
- **Insert at cursor**: O(1) - just fill gap
- **Delete at cursor**: O(1) - expand gap
- **Move cursor**: O(distance) - amortized O(1) for sequential edits
- **Memory**: Wastes space for gap (typically 10-25% overhead)
- **Use case**: Interactive editing (typing, backspace)

**Rope (Tree-Based)**:
```rust
rope.insert(position, 'x');  // O(log n)
```
- **Insert anywhere**: O(log n) - tree balancing
- **Delete anywhere**: O(log n)
- **Memory**: High overhead (tree nodes)
- **Use case**: Large files with edits scattered throughout

**Performance for Sequential Editing** (1000 insertions at cursor):
- **Gap Buffer**: ~1ms (O(1) per insert)
- **Vec<char>**: ~500ms (O(n) per insert)
- **Rope**: ~10ms (O(log n) per insert)

**Conclusion**: Gap buffer wins for typical text editing (sequential insertions/deletions).

### 3. Gap Management and Growth Strategy

Managing the gap size involves trade-offs:

**Small Gap**:
- **Pros**: Less memory waste, better cache locality
- **Cons**: Frequent reallocation when gap fills

**Large Gap**:
- **Pros**: Fewer reallocations, fast consecutive inserts
- **Cons**: Memory waste, poor cache locality

**Growth Strategies**:

**Fixed Gap** (Emacs-style):
```rust
const GAP_SIZE: usize = 128;
// Always maintain gap of 128 bytes
```
- Simple, predictable
- Good for average edit patterns

**Proportional Gap**:
```rust
let gap_size = buffer.len() / 4;
// Gap is 25% of total buffer size
```
- Scales with document size
- Larger docs get larger gaps

**Adaptive Gap**:
```rust
// Track insertion frequency
if insertion_rate_high {
    grow_gap();
} else {
    shrink_gap();
}
```
- Adjusts to user behavior
- Complex to implement correctly

### 4. UTF-8 Handling in Gap Buffers

Rust strings are UTF-8, where characters can be 1-4 bytes. Gap buffers must handle this:

**Character Boundaries**:
```rust
// ❌ Wrong - can split UTF-8 character
buffer[gap_start] = bytes[0];  // Only part of multi-byte char

// ✅ Correct - insert complete character
for byte in char.to_string().bytes() {
    buffer[gap_start] = byte;
    gap_start += 1;
}
```

**Cursor Positioning**:
```rust
// Cursor position is in BYTES, not characters
let char = '世';  // 3 bytes in UTF-8
insert_char(char);
cursor += char.len_utf8();  // Advance by 3 bytes
```

**Why This Matters**: Moving cursor by 1 byte might land in the middle of a multi-byte character (invalid UTF-8). Must track character boundaries.

### 5. Cursor Abstraction and User Model

Users think in terms of "cursor position in text," not "gap position in buffer":

**User Model**:
```
"Hello world"
      ^ Cursor at position 6 (visible position)
```

**Buffer Model**:
```
[H][e][l][l][o][ ][_][_][_][w][o][r][l][d]
                  ^gap_start = 6
```

**Abstraction Layer**:
```rust
struct TextBuffer {
    gap_buffer: GapBuffer,
    cursor: usize,  // Logical cursor position
}

impl TextBuffer {
    fn insert_char(&mut self, ch: char) {
        self.gap_buffer.move_gap_to(self.cursor);  // Align gap with cursor
        self.gap_buffer.insert(ch);
        self.cursor += ch.len_utf8();
    }
}
```

**Key Insight**: Gap is an implementation detail. Cursor provides user-friendly API.

### 6. Multi-Line Indexing

Text editors need to map between byte positions and (line, column) coordinates:

**Line Starts Index**:
```rust
line_starts: Vec<usize>  // Byte position of each line start

// Example: "hello\nworld\nrust"
line_starts = [0, 6, 12]
//             ^  ^  ^
//             |  |  "rust"
//             |  "world"
//             "hello"
```

**Byte Position → (Line, Column)**:
```rust
fn cursor_to_line_col(cursor: usize) -> (usize, usize) {
    let line = line_starts.partition_point(|&pos| pos <= cursor);
    let line_start = line_starts[line];
    let column = cursor - line_start;
    (line, column)
}
// Binary search: O(log lines)
```

**Why This Matters**: Editors display "Line 42, Column 15" in status bar. Efficient line indexing enables this.

### 7. Command Pattern for Undo/Redo

Undo/redo requires storing edit history without storing entire buffer snapshots:

**Command Pattern**:
```rust
enum EditCommand {
    InsertChar { position: usize, ch: char },
    DeleteChar { position: usize, ch: char },
}

impl EditCommand {
    fn inverse(&self) -> EditCommand {
        match self {
            InsertChar { pos, ch } => DeleteChar { position: pos, ch },
            DeleteChar { pos, ch } => InsertChar { position: pos, ch },
        }
    }
}
```

**Undo Stack**:
```rust
// User types "abc"
undo_stack: [Insert('a'), Insert('b'), Insert('c')]

// User presses undo
let cmd = undo_stack.pop();  // Insert('c')
cmd.inverse().execute();     // Delete('c')
redo_stack.push(cmd);

// Text: "ab"
```

**Memory Efficiency**:
- **Naive**: Store entire buffer per edit (1MB per snapshot)
- **Command pattern**: Store only command (16 bytes per edit)
- For 1000 edits: 1GB vs 16KB (62,500x less memory!)

### 8. Gap Movement Optimization

Moving gap involves copying memory. Optimizations:

**Copy Within Buffer** (No Allocation):
```rust
// Move gap from 5 to 8
// Instead of: allocate temp buffer, copy, deallocate
// Use: buffer.copy_within(examples, dest)

buffer.copy_within(
    gap_end..gap_end + distance,  // Source: after gap
    gap_start                      // Dest: before gap
);
```

**Batch Gap Movements**:
```rust
// ❌ Inefficient: move gap for each char
for ch in "hello" {
    move_gap_to(cursor);
    insert(ch);
    cursor += 1;
}
// 5 gap movements!

// ✅ Efficient: move gap once
move_gap_to(cursor);
for ch in "hello" {
    insert(ch);
    cursor += 1;
}
// 1 gap movement!
```

### 9. Cache Efficiency and Memory Layout

Gap buffers are cache-friendly for sequential access:

**Cache Line** (typically 64 bytes):
```
[H][e][l][l][o][ ][w][o][r][l][d][...][...][...]
 ← Cache line loaded on first access →
```

**Sequential Insertion**:
- First insert: Load cache line containing gap_start
- Next inserts: Data already in cache (cache hit!)
- Result: ~100 cycles for first, ~3 cycles for subsequent

**Gap Size and Cache**:
- **Small gap** (<64 bytes): Fits in single cache line
- **Large gap** (>1KB): May cause cache misses when jumping across gap

**Optimal Gap Size**: 128-512 bytes balances memory waste vs cache efficiency.

### 10. Amortized Analysis of Gap Buffer

Gap buffer operations have **amortized O(1)** complexity for sequential edits:

**Worst Case** (Random Edits):
```rust
// Edit at position 0, then position 1000, then 0, ...
// Each edit requires O(n) gap movement
// Total: O(n * edits)
```

**Best Case** (Sequential Edits):
```rust
// Typing normally: cursor moves forward 1 position at a time
// Gap stays at cursor, no movement needed
// Total: O(edits)
```

**Amortized Analysis**:
- For N sequential edits: 0 gap movements
- For 1 random edit: 1 gap movement of average distance N/2
- Average: O(N/2) / N = O(1) per edit

**Real-World**: Typing is 99%+ sequential, making gap buffer very efficient in practice.

### Connection to This Project

This text editor project demonstrates gap buffer design and optimizations in a complete implementation:

**Gap Buffer Structure (Step 1)**: You'll implement the core gap buffer with `gap_start` and `gap_end` indices. Inserting at cursor is O(1) by writing to `buffer[gap_start]` and incrementing. This single-array design eliminates pointer-chasing.

**Memory Layout (Step 1)**: The `Vec<u8>` backing storage is contiguous in memory, providing excellent cache locality. The `copy_within` method moves gap without allocating temporary buffers—critical for performance.

**Cursor Abstraction (Step 2)**: The `TextBuffer` wrapper provides a user-friendly API (insert_char, move_cursor_left) while hiding gap management. Moving cursor triggers `move_gap_to`, aligning implementation with user intent.

**UTF-8 Handling (Step 2)**: Inserting multi-byte characters (`ch.to_string().bytes()`) demonstrates proper UTF-8 handling. Cursor advances by `char.len_utf8()` bytes, not 1, maintaining valid character boundaries.

**Line Indexing (Step 3)**: The `line_starts` vector enables O(log n) line lookups via binary search (`partition_point`). Essential for implementing "goto line 42" and displaying line numbers.

**Command Pattern (Step 4)**: Edit commands store minimal data (position + character), not entire buffer states. The `inverse()` method generates undo operations automatically, making undo/redo straightforward.

**Performance Comparison (Step 5)**: Benchmarking against `Vec<char>` and `String` reveals why gap buffer matters—100x speedup for sequential editing patterns typical of human typing.

**Cache Optimization (Step 6)**: Experimenting with gap sizes (fixed vs proportional) teaches how algorithmic complexity (O(1)) doesn't tell the whole story—cache misses can dominate performance.

By the end of this project, you'll have built an **editor buffer** matching the design of Emacs and other professional editors, understanding both the algorithm (gap buffer) and systems concerns (cache efficiency, memory layout, UTF-8).

---

## Build The Project

### Milestone 1: Basic Gap Buffer Structure
**Goal**: Implement gap buffer with insert and delete at cursor.

**What to implement**:
- `GapBuffer` with `Vec<u8>` backing storage
- Gap start and gap end indices
- `insert_at_cursor()` places char in gap
- `delete_at_cursor()` expands gap
- Move gap to cursor position when needed

**Why this step**: Core gap buffer algorithm. Understanding gap concept is essential.

**Testing hint**: Test insert/delete at various positions. Verify gap is maintained correctly. Test edge cases (empty buffer, full buffer).

```rust
pub struct GapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer {
    pub fn new(capacity: usize) -> Self {
        GapBuffer {
            buffer: vec![0; capacity],
            gap_start: 0,
            gap_end: capacity,
        }
    }

    pub fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    pub fn len(&self) -> usize {
        self.buffer.len() - self.gap_size()
    }

    pub fn move_gap_to(&mut self, position: usize) {
        if position < self.gap_start {
            // Move gap backward
            let distance = self.gap_start - position;
            self.buffer.copy_within(position..self.gap_start, self.gap_end - distance);
            self.gap_end -= distance;
            self.gap_start = position;
        } else if position > self.gap_start {
            // Move gap forward
            let distance = position - self.gap_start;
            self.buffer.copy_within(self.gap_end..self.gap_end + distance, self.gap_start);
            self.gap_start += distance;
            self.gap_end += distance;
        }
    }

    pub fn insert(&mut self, ch: u8) {
        if self.gap_size() == 0 {
            self.grow();
        }

        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;
    }

    pub fn delete(&mut self) -> Option<u8> {
        if self.gap_start == 0 {
            return None;
        }

        self.gap_start -= 1;
        Some(self.buffer[self.gap_start])
    }

    fn grow(&mut self) {
        let new_capacity = self.buffer.len() * 2;
        let old_gap_size = self.gap_size();

        self.buffer.resize(new_capacity, 0);
        self.gap_end = self.buffer.len();

        // Move content after gap to end
        let content_after_gap = self.buffer.len() - old_gap_size - self.gap_start;
        if content_after_gap > 0 {
            self.buffer.copy_within(
                self.gap_start..self.gap_start + content_after_gap,
                self.gap_end - content_after_gap
            );
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        result.push_str(std::str::from_utf8(&self.buffer[..self.gap_start]).unwrap());
        result.push_str(std::str::from_utf8(&self.buffer[self.gap_end..]).unwrap());
        result
    }
}
```

---

### Milestone 2: Cursor Management and Operations
**Goal**: Add cursor abstraction for user-friendly interface.

**What to implement**:
- `Cursor` struct tracking position
- Move cursor (left, right, start, end)
- Insert/delete operations relative to cursor
- Ensure gap follows cursor

**Why the previous step is not enough**: Raw gap buffer works but is low-level. Cursor abstraction provides intuitive interface.

**What's the improvement**: Cursor makes gap buffer usable like real text editor. Moving cursor efficiently moves gap, maintaining O(1) insert/delete.

**Testing hint**: Test cursor movements. Verify gap moves with cursor. Test insert/delete at cursor.

```rust
pub struct TextBuffer {
    gap_buffer: GapBuffer,
    cursor: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        TextBuffer {
            gap_buffer: GapBuffer::new(128),
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.gap_buffer.move_gap_to(self.cursor);

        for byte in ch.to_string().bytes() {
            self.gap_buffer.insert(byte);
        }

        self.cursor += ch.len_utf8();
    }

    pub fn delete_char(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        self.gap_buffer.move_gap_to(self.cursor);

        // Handle UTF-8 character boundaries
        if let Some(_) = self.gap_buffer.delete() {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.gap_buffer.len() {
            self.cursor += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.gap_buffer.len();
    }

    pub fn text(&self) -> String {
        self.gap_buffer.to_string()
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }
}
```

---

### Milestone 3: Multi-Line Support with Line Index
**Goal**: Add efficient line-based operations (goto line, insert line).

**What to implement**:
- Track line boundaries (newline positions)
- Map cursor position to (line, column)
- Operations: goto_line, insert_newline, delete_line
- Update line index on edits

**Why the previous step is not enough**: Single-line buffer works for simple cases, but real editors need multi-line support.

**What's the improvement**: Line index enables O(log n) line lookups and line-based operations. Essential for displaying line numbers, goto line commands.

**Testing hint**: Test multi-line text. Verify line boundaries are tracked. Test goto_line accuracy.

```rust
pub struct MultiLineBuffer {
    buffer: TextBuffer,
    line_starts: Vec<usize>,  // Positions of line starts
}

impl MultiLineBuffer {
    pub fn new() -> Self {
        MultiLineBuffer {
            buffer: TextBuffer::new(),
            line_starts: vec![0],
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let cursor = self.buffer.cursor_position();
        self.buffer.insert_char(ch);

        if ch == '\n' {
            // Find insertion point in line_starts
            let line_index = self.line_starts.partition_point(|&pos| pos <= cursor);
            self.line_starts.insert(line_index, cursor + 1);

            // Update all line starts after insertion
            for pos in &mut self.line_starts[line_index + 1..] {
                *pos += 1;
            }
        } else {
            // Update all line starts after insertion
            let line_index = self.line_starts.partition_point(|&pos| pos <= cursor);
            for pos in &mut self.line_starts[line_index..] {
                *pos += ch.len_utf8();
            }
        }
    }

    pub fn cursor_to_line_col(&self, cursor: usize) -> (usize, usize) {
        let line = self.line_starts.partition_point(|&pos| pos <= cursor);
        let line_start = if line > 0 {
            self.line_starts[line - 1]
        } else {
            0
        };
        let column = cursor - line_start;
        (line, column)
    }

    pub fn line_col_to_cursor(&self, line: usize, column: usize) -> Option<usize> {
        if line >= self.line_starts.len() {
            return None;
        }

        let line_start = self.line_starts[line];
        Some(line_start + column)
    }

    pub fn goto_line(&mut self, line: usize) {
        if let Some(line_start) = self.line_starts.get(line) {
            self.buffer.cursor = *line_start;
        }
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}
```

---

### Milestone 4: Undo/Redo with Command Pattern
**Goal**: Implement undo/redo functionality.

**What to implement**:
- `EditCommand` enum (Insert, Delete, etc.)
- Command history stack
- Undo: reverse command and add to redo stack
- Redo: replay command
- Batch commands (group multiple edits as single undo)

**Why the previous step is not enough**: Real editors need undo/redo. Users expect to revert mistakes.

**What's the improvement**: Command pattern enables undo/redo with minimal overhead. Each edit stores command (type + data), not entire buffer state.

**Testing hint**: Test undo/redo sequences. Verify state is restored correctly. Test undo limit.

```rust
#[derive(Clone)]
pub enum EditCommand {
    InsertChar { position: usize, ch: char },
    DeleteChar { position: usize, ch: char },
    InsertText { position: usize, text: String },
    DeleteText { position: usize, text: String },
}

impl EditCommand {
    pub fn inverse(&self) -> EditCommand {
        match self {
            EditCommand::InsertChar { position, ch } => {
                EditCommand::DeleteChar { position: *position, ch: *ch }
            }
            EditCommand::DeleteChar { position, ch } => {
                EditCommand::InsertChar { position: *position, ch: *ch }
            }
            EditCommand::InsertText { position, text } => {
                EditCommand::DeleteText { position: *position, text: text.clone() }
            }
            EditCommand::DeleteText { position, text } => {
                EditCommand::InsertText { position: *position, text: text.clone() }
            }
        }
    }
}

pub struct EditorWithUndo {
    buffer: MultiLineBuffer,
    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
    max_undo: usize,
}

impl EditorWithUndo {
    pub fn new() -> Self {
        EditorWithUndo {
            buffer: MultiLineBuffer::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo: 1000,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let position = self.buffer.buffer.cursor_position();
        self.buffer.insert_char(ch);

        let command = EditCommand::InsertChar { position, ch };
        self.add_to_undo(command);
    }

    pub fn delete_char(&mut self) {
        let position = self.buffer.buffer.cursor_position();
        if position == 0 {
            return;
        }

        // Get character being deleted (simplified)
        let ch = ' '; // Would need to extract actual char from buffer

        if self.buffer.buffer.delete_char() {
            let command = EditCommand::DeleteChar { position, ch };
            self.add_to_undo(command);
        }
    }

    fn add_to_undo(&mut self, command: EditCommand) {
        self.undo_stack.push(command);
        self.redo_stack.clear();  // Clear redo stack on new edit

        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(command) = self.undo_stack.pop() {
            self.execute_command(&command.inverse());
            self.redo_stack.push(command);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(command) = self.redo_stack.pop() {
            self.execute_command(&command);
            self.undo_stack.push(command);
            true
        } else {
            false
        }
    }

    fn execute_command(&mut self, command: &EditCommand) {
        // Execute command without adding to undo stack
        match command {
            EditCommand::InsertChar { position, ch } => {
                self.buffer.buffer.cursor = *position;
                self.buffer.insert_char(*ch);
            }
            EditCommand::DeleteChar { position, .. } => {
                self.buffer.buffer.cursor = *position;
                self.buffer.buffer.delete_char();
            }
            _ => {}
        }
    }
}
```

---

### Milestone 5: Performance Comparison with Alternatives
**Goal**: Benchmark gap buffer vs Vec\<char>, String, Rope.

**What to implement**:
- Implement same operations with Vec\<char>
- Implement with String
- Benchmark: random insertions, sequential insertions, deletions
- Compare memory usage

**Why the previous step is not enough**: Understanding why gap buffer is chosen requires comparing alternatives.

**What's the improvement**: Benchmarks reveal trade-offs:
- Vec: O(n) insert in middle, simple
- Gap buffer: O(1) insert at cursor, O(n) gap movement
- Rope: O(log n) insert anywhere, complex

Gap buffer wins for sequential editing (typical text editing pattern).

**Testing hint**: Test with realistic editing patterns (typing, backspace, cursor movement). Measure operations/second.

```rust
use std::time::Instant;

// Vec<char> implementation
pub struct VecBuffer {
    chars: Vec<char>,
    cursor: usize,
}

impl VecBuffer {
    pub fn new() -> Self {
        VecBuffer {
            chars: Vec::new(),
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.chars.insert(self.cursor, ch);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) -> bool {
        if self.cursor > 0 {
            self.chars.remove(self.cursor - 1);
            self.cursor -= 1;
            true
        } else {
            false
        }
    }
}

pub fn benchmark_editors() {
    let operations = 10_000;

    // Gap buffer
    let start = Instant::now();
    let mut gap_buffer = TextBuffer::new();
    for i in 0..operations {
        gap_buffer.insert_char('a');
        if i % 2 == 0 {
            gap_buffer.move_cursor_left();
        }
    }
    println!("Gap buffer: {:?}", start.elapsed());

    // Vec buffer
    let start = Instant::now();
    let mut vec_buffer = VecBuffer::new();
    for i in 0..operations {
        vec_buffer.insert_char('a');
        if i % 2 == 0 && vec_buffer.cursor > 0 {
            vec_buffer.cursor -= 1;
        }
    }
    println!("Vec buffer: {:?}", start.elapsed());

    // String buffer
    let start = Instant::now();
    let mut string_buffer = String::new();
    for _ in 0..operations {
        string_buffer.insert(string_buffer.len() / 2, 'a');
    }
    println!("String buffer: {:?}", start.elapsed());
}
```

---

### Milestone 6: Optimize Memory Layout for Cache
**Goal**: Optimize gap buffer for cache efficiency.

**What to implement**:
- Measure cache misses with different gap sizes
- Experiment with gap size strategy (fixed vs dynamic)
- Add prefetching hints (advanced)
- Profile cache performance

**Why the previous step is not enough**: Algorithmic complexity is O(1), but constant factors matter. Cache efficiency can provide 2-10x speedup.

**What's the improvement**: Smaller gaps fit in cache, larger gaps reduce gap movement frequency. Optimal gap size balances these trade-offs.

**Optimization focus**: Speed through cache optimization.

**Testing hint**: Use perf tools (Linux) or Instruments (macOS) to measure cache misses. Test different gap sizes.

```rust
// Advanced: configurable gap growth strategy
pub struct OptimizedGapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
    growth_strategy: GrowthStrategy,
}

pub enum GrowthStrategy {
    Fixed(usize),      // Fixed gap size
    Proportional(f32), // Gap size as proportion of buffer
    Adaptive,          // Adjust based on edit pattern
}

impl OptimizedGapBuffer {
    pub fn new_with_strategy(capacity: usize, strategy: GrowthStrategy) -> Self {
        let initial_gap = match strategy {
            GrowthStrategy::Fixed(size) => size,
            GrowthStrategy::Proportional(ratio) => (capacity as f32 * ratio) as usize,
            GrowthStrategy::Adaptive => capacity / 4,
        };

        OptimizedGapBuffer {
            buffer: vec![0; capacity],
            gap_start: 0,
            gap_end: initial_gap,
            growth_strategy: strategy,
        }
    }

    // Optimized gap movement with prefetch
    pub fn move_gap_optimized(&mut self, position: usize) {
        // Implementation with prefetch hints
        // Would use platform-specific intrinsics in production
    }
}
```

---

### Complete Working Example

```rust
use std::time::Instant;

// =============================================================================
// Milestone 1: Basic Gap Buffer Structure
// =============================================================================

pub struct GapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer {
    pub fn new(capacity: usize) -> Self {
        let adjusted = capacity.max(1);
        GapBuffer {
            buffer: vec![0; adjusted],
            gap_start: 0,
            gap_end: adjusted,
        }
    }

    pub fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    pub fn len(&self) -> usize {
        self.buffer.len() - self.gap_size()
    }

    pub fn move_gap_to(&mut self, position: usize) {
        let position = position.min(self.len());
        if position < self.gap_start {
            let distance = self.gap_start - position;
            self.buffer
                .copy_within(position..self.gap_start, self.gap_end - distance);
            self.gap_start -= distance;
            self.gap_end -= distance;
        } else if position > self.gap_start {
            let distance = position - self.gap_start;
            self.buffer
                .copy_within(self.gap_end..self.gap_end + distance, self.gap_start);
            self.gap_start += distance;
            self.gap_end += distance;
        }
    }

    pub fn insert(&mut self, byte: u8) {
        if self.gap_size() == 0 {
            self.grow();
        }
        self.buffer[self.gap_start] = byte;
        self.gap_start += 1;
    }

    pub fn delete(&mut self) -> Option<u8> {
        if self.gap_start == 0 {
            return None;
        }
        self.gap_start -= 1;
        Some(self.buffer[self.gap_start])
    }

    fn grow(&mut self) {
        let new_capacity = (self.buffer.len().max(1)) * 2;
        let mut new_buffer = vec![0; new_capacity];
        let before = self.gap_start;
        let after_len = self.buffer.len() - self.gap_end;
        new_buffer[..before].copy_from_slice(&self.buffer[..before]);
        if after_len > 0 {
            let start = new_capacity - after_len;
            new_buffer[start..].copy_from_slice(&self.buffer[self.gap_end..]);
            self.gap_end = start;
        } else {
            self.gap_end = new_capacity;
        }
        self.buffer = new_buffer;
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.len());
        result.push_str(std::str::from_utf8(&self.buffer[..self.gap_start]).unwrap());
        result.push_str(std::str::from_utf8(&self.buffer[self.gap_end..]).unwrap());
        result
    }
}

// =============================================================================
// Milestone 2: Cursor Management and Operations
// =============================================================================

pub struct TextBuffer {
    gap_buffer: GapBuffer,
    cursor: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        TextBuffer {
            gap_buffer: GapBuffer::new(128),
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.gap_buffer.move_gap_to(self.cursor);
        let mut encoded = [0u8; 4];
        let bytes = ch.encode_utf8(&mut encoded);
        for byte in bytes.as_bytes() {
            self.gap_buffer.insert(*byte);
            self.cursor += 1;
        }
    }

    pub fn delete_char(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.gap_buffer.move_gap_to(self.cursor);
        let mut removed = false;
        while let Some(byte) = self.gap_buffer.delete() {
            removed = true;
            self.cursor -= 1;
            if !Self::is_continuation_byte(byte) {
                break;
            }
            if self.cursor == 0 {
                break;
            }
        }
        removed
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        while self.cursor > 0 {
            if let Some(byte) = self.byte_at(self.cursor) {
                if Self::is_continuation_byte(byte) {
                    self.cursor -= 1;
                    continue;
                }
            }
            break;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor >= self.gap_buffer.len() {
            return;
        }
        if let Some(byte) = self.byte_at(self.cursor) {
            let advance = Self::char_len_from_first_byte(byte);
            self.cursor = (self.cursor + advance).min(self.gap_buffer.len());
        } else {
            self.cursor += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.gap_buffer.len();
    }

    pub fn text(&self) -> String {
        self.gap_buffer.to_string()
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }

    pub fn len(&self) -> usize {
        self.gap_buffer.len()
    }

    fn byte_at(&self, logical_index: usize) -> Option<u8> {
        if logical_index >= self.gap_buffer.len() {
            return None;
        }
        if logical_index < self.gap_buffer.gap_start {
            Some(self.gap_buffer.buffer[logical_index])
        } else {
            let offset = logical_index + self.gap_buffer.gap_size();
            Some(self.gap_buffer.buffer[offset])
        }
    }

    fn is_continuation_byte(byte: u8) -> bool {
        (byte & 0b1100_0000) == 0b1000_0000
    }

    fn char_len_from_first_byte(byte: u8) -> usize {
        if byte & 0b1000_0000 == 0 {
            1
        } else if byte & 0b1110_0000 == 0b1100_0000 {
            2
        } else if byte & 0b1111_0000 == 0b1110_0000 {
            3
        } else {
            4
        }
    }
}

// =============================================================================
// Milestone 3: Multi-Line Support with Line Index
// =============================================================================

pub struct MultiLineBuffer {
    buffer: TextBuffer,
    line_starts: Vec<usize>,
}

impl MultiLineBuffer {
    pub fn new() -> Self {
        MultiLineBuffer {
            buffer: TextBuffer::new(),
            line_starts: vec![0],
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.buffer.insert_char(ch);
        self.rebuild_line_starts();
    }

    pub fn delete_char(&mut self) -> Option<char> {
        let cursor = self.buffer.cursor_position();
        if cursor == 0 {
            return None;
        }
        let text = self.buffer.text();
        let mut slice = text[..cursor].chars();
        let ch = slice.next_back()?;
        if self.buffer.delete_char() {
            self.rebuild_line_starts();
            Some(ch)
        } else {
            None
        }
    }

    pub fn cursor_to_line_col(&self, cursor: usize) -> (usize, usize) {
        let clamped_cursor = cursor.min(self.buffer.len());
        let line = self
            .line_starts
            .partition_point(|&pos| pos <= clamped_cursor);
        let line_index = line.saturating_sub(1);
        let line_start = self.line_starts[line_index];
        let column = clamped_cursor - line_start;
        (line_index, column)
    }

    pub fn line_col_to_cursor(&self, line: usize, column: usize) -> Option<usize> {
        if line >= self.line_starts.len() {
            return None;
        }
        let line_start = self.line_starts[line];
        let line_end = if line + 1 < self.line_starts.len() {
            self.line_starts[line + 1]
        } else {
            self.buffer.len()
        };
        if column > line_end.saturating_sub(line_start) {
            return None;
        }
        Some(line_start + column)
    }

    pub fn goto_line(&mut self, line: usize) {
        if self.line_starts.is_empty() {
            return;
        }
        let clamped_line = line.min(self.line_starts.len() - 1);
        let cursor = self.line_starts[clamped_line];
        self.buffer.cursor = cursor;
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn cursor_position(&self) -> usize {
        self.buffer.cursor_position()
    }

    fn rebuild_line_starts(&mut self) {
        let text = self.buffer.text();
        self.line_starts.clear();
        self.line_starts.push(0);
        for (idx, ch) in text.char_indices() {
            if ch == '\n' {
                let next_start = idx + ch.len_utf8();
                self.line_starts.push(next_start.min(text.len()));
            }
        }
    }
}

// =============================================================================
// Milestone 4: Undo/Redo with Command Pattern
// =============================================================================

#[derive(Clone, Debug)]
pub enum EditCommand {
    InsertChar { position: usize, ch: char },
    DeleteChar { position: usize, ch: char },
    InsertText { position: usize, text: String },
    DeleteText { position: usize, text: String },
}

impl EditCommand {
    pub fn inverse(&self) -> EditCommand {
        match self {
            EditCommand::InsertChar { position, ch } => EditCommand::DeleteChar {
                position: *position + ch.len_utf8(),
                ch: *ch,
            },
            EditCommand::DeleteChar { position, ch } => EditCommand::InsertChar {
                position: position.saturating_sub(ch.len_utf8()),
                ch: *ch,
            },
            EditCommand::InsertText { position, text } => EditCommand::DeleteText {
                position: *position + text.len(),
                text: text.clone(),
            },
            EditCommand::DeleteText { position, text } => EditCommand::InsertText {
                position: position.saturating_sub(text.len()),
                text: text.clone(),
            },
        }
    }
}

pub struct EditorWithUndo {
    buffer: MultiLineBuffer,
    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
    max_undo: usize,
}

impl EditorWithUndo {
    pub fn new() -> Self {
        EditorWithUndo {
            buffer: MultiLineBuffer::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo: 1000,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let position = self.buffer.cursor_position();
        self.buffer.insert_char(ch);
        self.add_to_undo(EditCommand::InsertChar { position, ch });
    }

    pub fn delete_char(&mut self) {
        let position = self.buffer.cursor_position();
        if let Some(ch) = self.buffer.delete_char() {
            let command = EditCommand::DeleteChar {
                position,
                ch,
            };
            self.add_to_undo(command);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(command) = self.undo_stack.pop() {
            let inverse = command.inverse();
            self.execute_command(&inverse);
            self.redo_stack.push(command);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(command) = self.redo_stack.pop() {
            self.execute_command(&command);
            self.undo_stack.push(command);
            true
        } else {
            false
        }
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    fn add_to_undo(&mut self, command: EditCommand) {
        self.undo_stack.push(command);
        self.redo_stack.clear();
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.remove(0);
        }
    }

    fn execute_command(&mut self, command: &EditCommand) {
        match command {
            EditCommand::InsertChar { position, ch } => {
                self.buffer.buffer.cursor = *position;
                self.buffer.insert_char(*ch);
            }
            EditCommand::DeleteChar { position, .. } => {
                self.buffer.buffer.cursor = *position;
                let _ = self.buffer.delete_char();
            }
            EditCommand::InsertText { position, text } => {
                self.buffer.buffer.cursor = *position;
                for ch in text.chars() {
                    self.buffer.insert_char(ch);
                }
            }
            EditCommand::DeleteText { position, text } => {
                self.buffer.buffer.cursor = *position;
                for _ in text.chars() {
                    let _ = self.buffer.delete_char();
                }
            }
        }
    }
}

// =============================================================================
// Milestone 5: Performance Comparison with Alternatives
// =============================================================================

pub struct VecBuffer {
    chars: Vec<char>,
    cursor: usize,
}

impl VecBuffer {
    pub fn new() -> Self {
        VecBuffer {
            chars: Vec::new(),
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.chars.insert(self.cursor, ch);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) -> bool {
        if self.cursor > 0 {
            self.chars.remove(self.cursor - 1);
            self.cursor -= 1;
            true
        } else {
            false
        }
    }
}

pub fn benchmark_editors() {
    let operations = 5_000;

    let start = Instant::now();
    let mut gap_buffer = TextBuffer::new();
    for i in 0..operations {
        gap_buffer.insert_char('a');
        if i % 2 == 0 {
            gap_buffer.move_cursor_left();
        }
    }
    let gap_time = start.elapsed();

    let start = Instant::now();
    let mut vec_buffer = VecBuffer::new();
    for i in 0..operations {
        vec_buffer.insert_char('a');
        if i % 2 == 0 {
            let _ = vec_buffer.delete_char();
        }
    }
    let vec_time = start.elapsed();

    let start = Instant::now();
    let mut string_buffer = String::new();
    for _ in 0..operations {
        let mid = string_buffer.len() / 2;
        string_buffer.insert(mid, 'a');
    }
    let string_time = start.elapsed();

    println!(
        "Gap buffer: {:?}, Vec<char>: {:?}, String insert: {:?}",
        gap_time, vec_time, string_time
    );
}

// =============================================================================
// Milestone 6: Optimize Memory Layout for Cache
// =============================================================================

pub struct OptimizedGapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
    growth_strategy: GrowthStrategy,
}

#[derive(Clone, Copy)]
pub enum GrowthStrategy {
    Fixed(usize),
    Proportional(f32),
    Adaptive,
}

impl OptimizedGapBuffer {
    pub fn new_with_strategy(capacity: usize, strategy: GrowthStrategy) -> Self {
        let initial_capacity = capacity.max(1);
        OptimizedGapBuffer {
            buffer: vec![0; initial_capacity],
            gap_start: 0,
            gap_end: initial_capacity,
            growth_strategy: strategy,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len() - (self.gap_end - self.gap_start)
    }

    pub fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    pub fn move_gap_optimized(&mut self, position: usize) {
        let position = position.min(self.len());
        if position < self.gap_start {
            let distance = self.gap_start - position;
            self.buffer
                .copy_within(position..self.gap_start, self.gap_end - distance);
            self.gap_start -= distance;
            self.gap_end -= distance;
        } else if position > self.gap_start {
            let distance = position - self.gap_start;
            self.buffer
                .copy_within(self.gap_end..self.gap_end + distance, self.gap_start);
            self.gap_start += distance;
            self.gap_end += distance;
        }
    }

    pub fn push_str(&mut self, text: &str) {
        self.move_gap_optimized(self.len());
        for byte in text.as_bytes() {
            if self.gap_size() == 0 {
                self.grow_gap();
            }
            self.buffer[self.gap_start] = *byte;
            self.gap_start += 1;
        }
    }

    fn grow_gap(&mut self) {
        let old_capacity = self.buffer.len();
        let additional = match self.growth_strategy {
            GrowthStrategy::Fixed(size) => size.max(1),
            GrowthStrategy::Proportional(ratio) => {
                ((old_capacity as f32 * ratio).round() as usize).max(1)
            }
            GrowthStrategy::Adaptive => (old_capacity / 2).max(1),
        };
        let new_capacity = old_capacity + additional;
        let mut new_buffer = vec![0; new_capacity];
        let before = self.gap_start;
        let after_len = old_capacity - self.gap_end;
        new_buffer[..before].copy_from_slice(&self.buffer[..before]);
        if after_len > 0 {
            let new_gap_end = new_capacity - after_len;
            new_buffer[new_gap_end..].copy_from_slice(&self.buffer[self.gap_end..]);
            self.gap_end = new_gap_end;
        } else {
            self.gap_end = new_capacity;
        }
        self.buffer = new_buffer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gap_buffer_insert_delete_sequence() {
        let mut buffer = GapBuffer::new(8);
        for byte in b"hello" {
            buffer.insert(*byte);
        }
        assert_eq!(buffer.to_string(), "hello");
        buffer.move_gap_to(2);
        buffer.insert(b'X');
        assert_eq!(buffer.to_string(), "heXllo");
        buffer.move_gap_to(buffer.len());
        assert!(buffer.delete().is_some());
        assert_eq!(buffer.to_string(), "heXll");
    }

    #[test]
    fn text_buffer_handles_utf8_deletion() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char('你');
        buffer.insert_char('好');
        assert_eq!(buffer.cursor_position(), buffer.len());
        assert!(buffer.delete_char());
        assert_eq!(buffer.text(), "你");
        buffer.move_cursor_start();
        buffer.insert_char('😊');
        assert_eq!(buffer.text(), "😊你");
    }

    #[test]
    fn multiline_buffer_tracks_lines() {
        let mut buffer = MultiLineBuffer::new();
        for ch in "one\ntwo\nthree".chars() {
            buffer.insert_char(ch);
        }
        assert_eq!(buffer.line_count(), 3);
        let cursor = buffer.cursor_position();
        let (line, col) = buffer.cursor_to_line_col(cursor);
        assert_eq!((line, col), (2, 5));
        buffer.goto_line(1);
        assert_eq!(buffer.cursor_position(), 4);
        let pos = buffer.line_col_to_cursor(2, 2).unwrap();
        assert_eq!(pos, 10);
    }

    #[test]
    fn editor_undo_redo_flow() {
        let mut editor = EditorWithUndo::new();
        editor.insert_char('a');
        editor.insert_char('b');
        editor.insert_char('c');
        assert_eq!(editor.text(), "abc");
        assert!(editor.undo());
        assert_eq!(editor.text(), "ab");
        assert!(editor.redo());
        assert_eq!(editor.text(), "abc");
        editor.delete_char();
        assert_eq!(editor.text(), "ab");
        assert!(editor.undo());
        assert_eq!(editor.text(), "abc");
    }

    #[test]
    fn vec_buffer_behaviour_matches_expectations() {
        let mut buffer = VecBuffer::new();
        buffer.insert_char('x');
        buffer.insert_char('y');
        assert!(buffer.delete_char());
        assert_eq!(buffer.cursor, 1);
    }

    #[test]
    fn optimized_gap_buffer_moves_gap() {
        let mut buffer = OptimizedGapBuffer::new_with_strategy(32, GrowthStrategy::Fixed(8));
        buffer.push_str("abcdef");
        assert_eq!(buffer.len(), 6);
        buffer.move_gap_optimized(3);
        assert_eq!(buffer.gap_start, 3);
    }
}

```