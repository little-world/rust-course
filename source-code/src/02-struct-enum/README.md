# Struct & Enum Patterns - Runnable Examples

This directory contains runnable Rust examples for all patterns from Chapter 2: Struct & Enum Patterns.

## Running Examples

Run any example with:

```bash
cargo run --example <example_name>
```

Or list all available examples:

```bash
cargo run --example
```

## Example Index

### Pattern 1: Struct Design Patterns

| Example | Description |
|---------|-------------|
| `p1_named_struct` | Named field structs for complex data models |
| `p1_tuple_struct` | Tuple structs and the newtype pattern |
| `p1_unit_struct` | Unit structs as markers with PhantomData |

### Pattern 2: Newtype and Wrapper Patterns

| Example | Description |
|---------|-------------|
| `p2_newtype` | Type-safe wrappers for primitives (UserId, OrderId) |
| `p2_deref_wrapper` | Transparent wrappers with Deref |

### Pattern 3: Struct Memory and Update Patterns

| Example | Description |
|---------|-------------|
| `p3_struct_update` | Struct update syntax with `..other` |
| `p3_partial_moves` | Understanding partial moves of struct fields |

### Pattern 4: Enum Design Patterns

| Example | Description |
|---------|-------------|
| `p4_http_response` | Enum with pattern matching for HTTP responses |
| `p4_state_machine` | Enum-based state machine for order processing |

### Pattern 5: Advanced Enum Techniques

| Example | Description |
|---------|-------------|
| `p5_recursive_enum` | Recursive enums with Box (Tree, AST) |
| `p5_efficient_enum` | Memory-efficient enums with boxed large variants |

### Pattern 6: Visitor Pattern

| Example | Description |
|---------|-------------|
| `p6_visitor` | Complete visitor pattern with AST, PrettyPrinter, and Evaluator |

## Quick Start

```bash
# Run struct examples
cargo run --example p1_named_struct
cargo run --example p1_tuple_struct
cargo run --example p1_unit_struct

# Run enum state machine
cargo run --example p4_state_machine

# Run the complete visitor pattern example
cargo run --example p6_visitor
```

## Key Concepts

- **Named structs**: Self-documenting code with explicit field names
- **Tuple structs**: Lightweight wrappers, newtype pattern
- **Unit structs**: Zero-sized markers for type-level programming
- **Newtypes**: Prevent mixing semantically different values
- **Struct update**: Create variations with `..base`
- **Enum state machines**: Type-safe state transitions
- **Box for enums**: Handle recursion and large variants
- **Visitor pattern**: Separate operations from data structures
