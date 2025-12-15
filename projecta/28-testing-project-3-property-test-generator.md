# Property-Based Test Generator

### Problem Statement

Build an intelligent property-based test generator that analyzes function signatures and automatically generates property tests using proptest. Your generator should infer appropriate test properties from type signatures, identify mathematical invariants, create custom value generators, and produce comprehensive test suites that explore edge cases far beyond what manual testing would cover.

Your property test generator should support:
- Automatic property inference from function signatures
- Custom value generator creation based on constraints
- Invariant detection (commutativity, associativity, identity, etc.)
- Round-trip property generation (encode/decode, serialize/deserialize)
- Shrinking strategies for minimal failing cases
- Integration with existing test infrastructure

## Why Property-Based Testing Automation Matters

### The Problem with Example-Based Tests

**Manual Test Limitations**:
```rust
fn reverse<T>(vec: Vec<T>) -> Vec<T> {
    let mut result = vec.clone();
    result.reverse();
    result
}

// Traditional tests - only check specific examples
#[test]
fn test_reverse() {
    assert_eq!(reverse(vec![1, 2, 3]), vec![3, 2, 1]);
    assert_eq!(reverse(vec![5]), vec![5]);
    assert_eq!(reverse(vec![]), vec![]);
}
// Tested 3 cases out of infinite possibilities!
// What about vec with 1000 elements? Duplicates? MAX_INT?
```

**Properties capture universal truths**:
```rust
// Property: Reversing twice returns original
proptest! {
    #[test]
    fn prop_reverse_twice(vec: Vec<i32>) {
        let reversed = reverse(reverse(vec.clone()));
        prop_assert_eq!(reversed, vec);
    }
}
// Tests 100 random cases by default
// Finds edge cases humans miss: [i32::MIN], very long vectors, etc.
```

### What is Property-Based Testing?

**Core Concept**: Instead of testing specific examples, verify properties that should hold for ALL inputs.

**Example Properties**:
| Function | Property | Mathematical Name |
|----------|----------|------------------|
| `reverse(reverse(x)) == x` | Double reverse is identity | Involution |
| `add(a, b) == add(b, a)` | Order doesn't matter | Commutativity |
| `serialize(deserialize(x)) == x` | Round-trip preservation | Isomorphism |
| `sorted.len() == input.len()` | Length preservation | Conservation |
| `max(a, b) >= a && max(a, b) >= b` | Result bounds | Ordering |

### Why Automatic Generation?

**The Manual Problem**:
```rust
// Developer must manually identify properties
fn manual_approach(func: fn(i32, i32) -> i32) {
    // What properties does this function have?
    // Is it commutative? Associative? Does it have an identity?
    // Developer must figure this out manually
}
```

**The Automated Solution**:
```rust
// Generator infers properties from signature and tests
let generator = PropertyTestGenerator::new();
let tests = generator.analyze_function(add_function);
// Automatically generates:
// - Commutativity test: add(a,b) == add(b,a)
// - Associativity test: add(add(a,b),c) == add(a,add(b,c))
// - Identity test: add(x, 0) == x
```

**Value**:
- **Completeness**: Finds all applicable properties
- **Consistency**: Never forgets edge cases
- **Speed**: Generates tests in seconds vs hours of manual work
- **Education**: Teaches developers about mathematical properties

### Real-World Impact

**Case Study: Sorting Function**
```
Manual tests: 5 test cases, 20 lines of code, 30 minutes to write
→ Found: 0 bugs

Auto-generated property tests: 8 properties, 40 lines, 2 minutes to generate
→ Found: 3 bugs (stability violation, comparison error, empty vec panic)
```

**Bug Discovery Rate**:
```
Project A: 100 manual tests
→ Edge case coverage: ~15%
→ Bugs found in dev: 8

Project B: 20 auto-generated property tests
→ Edge case coverage: ~85%
→ Bugs found in dev: 24 (3x more bugs caught!)
```

## Use Cases

### 1. Library Development
- **API validation**: Ensure public APIs satisfy expected properties
- **Regression prevention**: Properties catch bugs across refactors
- **Documentation**: Generated properties serve as formal specifications

### 2. Data Structure Implementation
- **Invariant verification**: BST ordering, heap property, balance factors
- **Operation properties**: Insert/remove commute with lookup
- **Memory safety**: No use-after-free, no double-free

### 3. Serialization/Networking
- **Round-trip testing**: Encode then decode returns original
- **Compatibility**: Old version can read new format
- **Error handling**: Invalid inputs produce errors, not crashes

### 4. Mathematical Code
- **Numerical stability**: Results within epsilon bounds
- **Algebraic properties**: Commutativity, distributivity, etc.
- **Edge cases**: Infinity, NaN, zero, overflow

---

## Building the Project

### Milestone 1: Function Signature Analysis

**Goal**: Parse Rust function signatures to extract types, parameters, and return values for property inference.

**Why we start here**: Before generating properties, we need to understand what the function does based on its type signature.

#### Architecture

**Structs:**
- `FunctionSignature` - Parsed function information
  - **Field**: `name: String` - Function name
  - **Field**: `parameters: Vec<Parameter>` - Input parameters
  - **Field**: `return_type: Option<Type>` - Return type
  - **Field**: `generics: Vec<String>` - Generic type parameters
  - **Field**: `constraints: Vec<Constraint>` - Trait bounds

- `Parameter` - Function parameter
  - **Field**: `name: String` - Parameter name
  - **Field**: `param_type: Type` - Parameter type
  - **Field**: `is_mutable: bool` - Whether mutable

- `Type` - Rust type representation
  - **Variants**: `Primitive(String)`, `Generic(String)`, `Vec(Box<Type>)`, `Option(Box<Type>)`, `Result(Box<Type>, Box<Type>)`, `Custom(String)`

**Functions:**
- `parse_function(source: &str) -> Result<FunctionSignature, Error>` - Parse function
- `extract_parameters(sig: &str) -> Vec<Parameter>` - Get parameters
- `extract_return_type(sig: &str) -> Option<Type>` - Get return type
- `infer_constraints(sig: &FunctionSignature) -> Vec<Constraint>` - Infer trait bounds

**Starter Code**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(String),      // i32, bool, etc.
    Generic(String),        // T, U, etc.
    Vec(Box<Type>),        // Vec<T>
    Option(Box<Type>),     // Option<T>
    Result(Box<Type>, Box<Type>),  // Result<T, E>
    Tuple(Vec<Type>),      // (T, U, V)
    Custom(String),        // User-defined types
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub type_param: String,
    pub trait_bound: String,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub generics: Vec<String>,
    pub constraints: Vec<Constraint>,
}

impl FunctionSignature {
    pub fn parse_function(source: &str) -> Result<Self, String> {
        // TODO: Parse function definition
        // TODO: Extract fn name
        // TODO: Parse generic parameters <T, U>
        // TODO: Parse parameters (name: type, ...)
        // TODO: Parse return type -> Type
        // TODO: Extract where clauses
        todo!("Parse function signature")
    }

    fn extract_parameters(param_str: &str) -> Vec<Parameter> {
        // TODO: Split by commas (respecting nested types)
        // TODO: Parse each "name: type" pair
        // TODO: Detect &mut for mutability
        todo!("Extract parameters")
    }

    fn extract_return_type(sig_str: &str) -> Option<Type> {
        // TODO: Find -> in signature
        // TODO: Parse type after ->
        // TODO: Handle unit type () vs other types
        todo!("Extract return type")
    }

    fn parse_type(type_str: &str) -> Type {
        // TODO: Match primitive types (i32, bool, etc.)
        // TODO: Parse generic types Vec<T>, Option<T>, etc.
        // TODO: Handle nested generics Vec<Vec<T>>
        // TODO: Parse tuples (T, U, V)
        todo!("Parse type")
    }

    pub fn infer_constraints(&self) -> Vec<Constraint> {
        // TODO: Infer required traits from operations
        // TODO: If comparing, need PartialEq
        // TODO: If using in HashMap, need Hash + Eq
        todo!("Infer constraints")
    }

    pub fn is_generic(&self) -> bool {
        !self.generics.is_empty()
    }

    pub fn has_side_effects(&self) -> bool {
        // TODO: Check if parameters are mutable
        // TODO: Check return type (() often indicates side effects)
        self.parameters.iter().any(|p| p.is_mutable) || self.return_type.is_none()
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = "fn add(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert_eq!(sig.name, "add");
        assert_eq!(sig.parameters.len(), 2);
        assert_eq!(sig.parameters[0].name, "a");
        assert_eq!(sig.parameters[0].param_type, Type::Primitive("i32".to_string()));
        assert!(matches!(sig.return_type, Some(Type::Primitive(_))));
    }

    #[test]
    fn test_parse_generic_function() {
        let source = "fn reverse<T>(vec: Vec<T>) -> Vec<T>";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert_eq!(sig.name, "reverse");
        assert_eq!(sig.generics, vec!["T"]);
        assert!(matches!(sig.parameters[0].param_type, Type::Vec(_)));
    }

    #[test]
    fn test_parse_with_constraints() {
        let source = "fn max<T: Ord>(a: T, b: T) -> T";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert_eq!(sig.generics, vec!["T"]);
        let constraints = sig.infer_constraints();
        assert!(constraints.iter().any(|c| c.trait_bound.contains("Ord")));
    }

    #[test]
    fn test_mutable_parameter_detection() {
        let source = "fn increment(x: &mut i32)";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert!(sig.parameters[0].is_mutable);
        assert!(sig.has_side_effects());
    }

    #[test]
    fn test_complex_return_type() {
        let source = "fn parse(s: &str) -> Result<i32, String>";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert!(matches!(sig.return_type, Some(Type::Result(_, _))));
    }

    #[test]
    fn test_tuple_parameters() {
        let source = "fn swap<T>(pair: (T, T)) -> (T, T)";
        let sig = FunctionSignature::parse_function(source).unwrap();

        assert!(matches!(sig.parameters[0].param_type, Type::Tuple(_)));
    }
}
```

**Check Your Understanding**:
- Why is type information crucial for property generation?
- How do generics affect property inference?
- What properties can be inferred from pure functions vs side-effecting functions?

---

#### Why Milestone 1 Isn't Enough

**Limitation**: We can parse signatures but don't know which properties to test. A function `add(i32, i32) -> i32` could be commutative, associative, have identity—but we need to detect these.

**What we're adding**: Property inference engine that analyzes function semantics to determine applicable mathematical properties.

**Improvement**:
- **Intelligence**: Automatically detects patterns
- **Completeness**: Never misses applicable properties
- **Correctness**: Only generates valid properties
- **Versatility**: Handles various function types

---

### Milestone 2: Property Inference Engine

**Goal**: Automatically infer testable properties from function signatures and semantics.

**Why this matters**: The power of property-based testing comes from testing the right properties. Manual property selection is error-prone and incomplete.

#### Architecture

**Structs:**
- `PropertyInference` - Infers properties for a function
  - **Field**: `signature: FunctionSignature` - Function to analyze
  - **Field**: `inferred_properties: Vec<Property>` - Discovered properties

- `Property` - A testable property
  - **Field**: `name: String` - Property name (e.g., "Commutativity")
  - **Field**: `description: String` - Human-readable description
  - **Field**: `property_type: PropertyType` - Category
  - **Field**: `test_code: String` - Generated proptest code

**Enums:**
- `PropertyType` - Categories of properties
  - **Variants**:
    - `Commutativity` - f(a,b) == f(b,a)
    - `Associativity` - f(f(a,b),c) == f(a,f(b,c))
    - `Identity` - f(a, identity) == a
    - `Idempotence` - f(f(a)) == f(a)
    - `Involution` - f(f(a)) == a
    - `Monotonicity` - a < b → f(a) < f(b)
    - `RoundTrip` - decode(encode(a)) == a
    - `LengthPreservation` - len(f(a)) == len(a)
    - `Invariant` - Some condition always holds

**Functions:**
- `infer_properties(sig: &FunctionSignature) -> Vec<Property>` - Find all properties
- `test_commutativity(sig: &FunctionSignature) -> Option<Property>` - Check if commutative
- `test_associativity(sig: &FunctionSignature) -> Option<Property>` - Check if associative
- `find_identity_element(sig: &FunctionSignature) -> Option<Property>` - Find identity
- `detect_involution(sig: &FunctionSignature) -> Option<Property>` - Detect involution

**Starter Code**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
    Commutativity,
    Associativity,
    Identity(String),  // Identity element value
    Idempotence,
    Involution,
    Monotonicity,
    RoundTrip,
    LengthPreservation,
    Invariant(String),  // Invariant condition
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub description: String,
    pub property_type: PropertyType,
    pub test_code: String,
}

pub struct PropertyInference {
    signature: FunctionSignature,
    inferred_properties: Vec<Property>,
}

impl PropertyInference {
    pub fn new(signature: FunctionSignature) -> Self {
        // TODO: Initialize inference engine
        todo!("Create property inference")
    }

    pub fn infer_properties(&mut self) -> Vec<Property> {
        // TODO: Try each property type
        // TODO: Collect all applicable properties
        let mut properties = Vec::new();

        if let Some(prop) = self.test_commutativity() {
            properties.push(prop);
        }

        if let Some(prop) = self.test_associativity() {
            properties.push(prop);
        }

        if let Some(prop) = self.find_identity_element() {
            properties.push(prop);
        }

        if let Some(prop) = self.detect_involution() {
            properties.push(prop);
        }

        if let Some(prop) = self.test_idempotence() {
            properties.push(prop);
        }

        if let Some(prop) = self.test_length_preservation() {
            properties.push(prop);
        }

        properties
    }

    fn test_commutativity(&self) -> Option<Property> {
        // TODO: Check if function has 2 params of same type
        // TODO: Check if return type matches param type
        // TODO: If yes, generate commutativity test
        // TODO: Property: f(a, b) == f(b, a)
        todo!("Test commutativity")
    }

    fn test_associativity(&self) -> Option<Property> {
        // TODO: Check if function takes 2 params of type T and returns T
        // TODO: Property: f(f(a, b), c) == f(a, f(b, c))
        todo!("Test associativity")
    }

    fn find_identity_element(&self) -> Option<Property> {
        // TODO: Based on function name/signature, guess identity
        // TODO: "add" → 0, "mul" → 1, "concat" → ""
        // TODO: Generate test: f(x, identity) == x
        todo!("Find identity element")
    }

    fn detect_involution(&self) -> Option<Property> {
        // TODO: Check if function is T -> T
        // TODO: Check function name for hints: "reverse", "not", "negate"
        // TODO: Property: f(f(x)) == x
        todo!("Detect involution")
    }

    fn test_idempotence(&self) -> Option<Property> {
        // TODO: Check if T -> T
        // TODO: Property: f(f(x)) == f(x)
        // TODO: Common in: normalization, deduplication
        todo!("Test idempotence")
    }

    fn test_length_preservation(&self) -> Option<Property> {
        // TODO: Check if input/output are collections (Vec, String, etc.)
        // TODO: Property: len(output) == len(input)
        // TODO: Common in: reverse, shuffle, sort
        todo!("Test length preservation")
    }

    fn generate_round_trip_test(&self) -> Option<Property> {
        // TODO: Check if function name suggests encoding: serialize, encode, marshal
        // TODO: Find corresponding decoder
        // TODO: Property: decode(encode(x)) == x
        todo!("Generate round-trip test")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_commutativity() {
        let source = "fn add(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        assert!(properties.iter().any(|p| {
            matches!(p.property_type, PropertyType::Commutativity)
        }));
    }

    #[test]
    fn test_infer_involution() {
        let source = "fn reverse<T>(vec: Vec<T>) -> Vec<T>";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        assert!(properties.iter().any(|p| {
            matches!(p.property_type, PropertyType::Involution)
        }));
    }

    #[test]
    fn test_infer_identity() {
        let source = "fn multiply(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        // Should infer identity element is 1 for multiply
        assert!(properties.iter().any(|p| {
            matches!(p.property_type, PropertyType::Identity(_))
        }));
    }

    #[test]
    fn test_length_preservation() {
        let source = "fn shuffle<T>(vec: Vec<T>) -> Vec<T>";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        assert!(properties.iter().any(|p| {
            matches!(p.property_type, PropertyType::LengthPreservation)
        }));
    }

    #[test]
    fn test_no_false_positives() {
        // Division is NOT commutative
        let source = "fn divide(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        // Should NOT infer commutativity for divide
        // (This is a simplification - real implementation would check semantics)
        // For now, we might over-generate and filter later
    }

    #[test]
    fn test_associativity_inference() {
        let source = "fn max(a: i32, b: i32) -> i32";
        let sig = FunctionSignature::parse_function(source).unwrap();
        let mut inference = PropertyInference::new(sig);

        let properties = inference.infer_properties();

        // max is associative: max(max(a,b),c) == max(a,max(b,c))
        assert!(properties.iter().any(|p| {
            matches!(p.property_type, PropertyType::Associativity)
        }));
    }
}
```

---

#### Why Milestone 2 Isn't Enough

**Limitation**: We infer properties but don't generate actual proptest code. Need to translate properties into executable tests.

**What we're adding**: Code generation engine that produces complete, runnable proptest test functions.

**Improvement**:
- **Automation**: One-click test generation
- **Correctness**: Generated tests are syntactically valid
- **Customization**: Tests follow project conventions
- **Integration**: Works with existing test infrastructure

---

### Milestone 3: Proptest Code Generation

**Goal**: Generate complete, compilable proptest code from inferred properties.

**Why this matters**: Inferred properties are useless without executable tests. We need to generate idiomatic Rust test code.

#### Architecture

**Structs:**
- `CodeGenerator` - Generates proptest code
  - **Field**: `properties: Vec<Property>` - Properties to generate tests for
  - **Field**: `config: GeneratorConfig` - Code generation settings

- `GeneratorConfig` - Configuration options
  - **Field**: `num_cases: usize` - Number of test cases (default 100)
  - **Field**: `max_shrink_iters: usize` - Shrinking iterations
  - **Field**: `use_custom_generators: bool` - Whether to create custom generators
  - **Field**: `test_module_name: String` - Name for test module

**Functions:**
- `generate_tests(properties: Vec<Property>) -> String` - Generate all tests
- `generate_property_test(prop: &Property) -> String` - Generate one test
- `create_value_generator(typ: &Type) -> String` - Create proptest generator
- `generate_test_module(tests: Vec<String>) -> String` - Wrap in module

**Starter Code**:

```rust
pub struct GeneratorConfig {
    pub num_cases: usize,
    pub max_shrink_iters: usize,
    pub use_custom_generators: bool,
    pub test_module_name: String,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        GeneratorConfig {
            num_cases: 100,
            max_shrink_iters: 1000,
            use_custom_generators: true,
            test_module_name: "generated_property_tests".to_string(),
        }
    }
}

pub struct CodeGenerator {
    properties: Vec<Property>,
    config: GeneratorConfig,
}

impl CodeGenerator {
    pub fn new(properties: Vec<Property>, config: GeneratorConfig) -> Self {
        // TODO: Initialize code generator
        todo!("Create code generator")
    }

    pub fn generate_tests(&self) -> String {
        // TODO: Generate proptest! macro block
        // TODO: For each property, generate test function
        // TODO: Wrap in module with use statements
        todo!("Generate all tests")
    }

    fn generate_property_test(&self, prop: &Property) -> String {
        // TODO: Generate test function based on property type
        // TODO: Create appropriate proptest code
        match &prop.property_type {
            PropertyType::Commutativity => self.gen_commutativity_test(prop),
            PropertyType::Associativity => self.gen_associativity_test(prop),
            PropertyType::Identity(_) => self.gen_identity_test(prop),
            PropertyType::Involution => self.gen_involution_test(prop),
            PropertyType::Idempotence => self.gen_idempotence_test(prop),
            PropertyType::LengthPreservation => self.gen_length_test(prop),
            PropertyType::RoundTrip => self.gen_roundtrip_test(prop),
            PropertyType::Invariant(_) => self.gen_invariant_test(prop),
            _ => String::new(),
        }
    }

    fn gen_commutativity_test(&self, prop: &Property) -> String {
        // TODO: Generate: prop_assert_eq!(f(a, b), f(b, a))
        todo!("Generate commutativity test")
    }

    fn gen_associativity_test(&self, prop: &Property) -> String {
        // TODO: Generate: prop_assert_eq!(f(f(a,b),c), f(a,f(b,c)))
        todo!("Generate associativity test")
    }

    fn gen_identity_test(&self, prop: &Property) -> String {
        // TODO: Generate: prop_assert_eq!(f(x, IDENTITY), x)
        todo!("Generate identity test")
    }

    fn gen_involution_test(&self, prop: &Property) -> String {
        // TODO: Generate: prop_assert_eq!(f(f(x)), x)
        todo!("Generate involution test")
    }

    fn gen_idempotence_test(&self, prop: &Property) -> String {
        // TODO: Generate: prop_assert_eq!(f(f(x)), f(x))
        todo!("Generate idempotence test")
    }

    fn gen_length_test(&self, prop: &Property) -> String {
        // TODO: Generate: prop_assert_eq!(f(x).len(), x.len())
        todo!("Generate length preservation test")
    }

    fn gen_roundtrip_test(&self, prop: &Property) -> String {
        // TODO: Generate: prop_assert_eq!(decode(encode(x)), x)
        todo!("Generate round-trip test")
    }

    fn gen_invariant_test(&self, prop: &Property) -> String {
        // TODO: Generate: prop_assert!(invariant_condition)
        todo!("Generate invariant test")
    }

    fn create_value_generator(&self, typ: &Type) -> String {
        // TODO: Map type to proptest generator
        // TODO: i32 → any::<i32>()
        // TODO: Vec<T> → prop::collection::vec(strategy, size)
        // TODO: String → ".*" or "[a-z]{1,100}"
        // TODO: Option<T> → prop::option::of(strategy)
        todo!("Create value generator")
    }

    fn generate_test_module(&self, tests: Vec<String>) -> String {
        // TODO: Create module with:
        // TODO: use proptest::prelude::*;
        // TODO: proptest! { ... all tests ... }
        todo!("Generate test module")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_commutativity_test() {
        let prop = Property {
            name: "test_add_commutative".to_string(),
            description: "Addition is commutative".to_string(),
            property_type: PropertyType::Commutativity,
            test_code: String::new(),
        };

        let generator = CodeGenerator::new(vec![prop], GeneratorConfig::default());
        let code = generator.generate_property_test(&generator.properties[0]);

        // Should contain proptest assertions
        assert!(code.contains("prop_assert_eq!"));
        // Should swap arguments
        assert!(code.contains("(a, b)") && code.contains("(b, a)")
            || code.contains("swap"));
    }

    #[test]
    fn test_generate_involution_test() {
        let prop = Property {
            name: "test_reverse_involution".to_string(),
            description: "Reversing twice returns original".to_string(),
            property_type: PropertyType::Involution,
            test_code: String::new(),
        };

        let generator = CodeGenerator::new(vec![prop], GeneratorConfig::default());
        let code = generator.generate_property_test(&generator.properties[0]);

        // Should apply function twice
        assert!(code.contains("reverse(reverse") || code.contains("f(f("));
    }

    #[test]
    fn test_create_primitive_generator() {
        let generator = CodeGenerator::new(vec![], GeneratorConfig::default());

        let gen = generator.create_value_generator(&Type::Primitive("i32".to_string()));
        assert!(gen.contains("any::<i32>()") || gen.contains("i32"));
    }

    #[test]
    fn test_create_vec_generator() {
        let generator = CodeGenerator::new(vec![], GeneratorConfig::default());

        let gen = generator.create_value_generator(&Type::Vec(
            Box::new(Type::Primitive("i32".to_string()))
        ));

        assert!(gen.contains("vec") || gen.contains("collection"));
    }

    #[test]
    fn test_full_module_generation() {
        let props = vec![
            Property {
                name: "test_prop1".to_string(),
                description: "Test 1".to_string(),
                property_type: PropertyType::Commutativity,
                test_code: String::new(),
            },
            Property {
                name: "test_prop2".to_string(),
                description: "Test 2".to_string(),
                property_type: PropertyType::Involution,
                test_code: String::new(),
            },
        ];

        let generator = CodeGenerator::new(props, GeneratorConfig::default());
        let module = generator.generate_tests();

        // Should have proptest macro
        assert!(module.contains("proptest!"));
        // Should have use statement
        assert!(module.contains("use proptest"));
        // Should have both tests
        assert!(module.contains("test_prop1"));
        assert!(module.contains("test_prop2"));
    }

    #[test]
    fn test_compilable_output() {
        // This would ideally compile the generated code to verify it's valid
        // For now, check syntax elements are present
        let prop = Property {
            name: "test_example".to_string(),
            description: "Example".to_string(),
            property_type: PropertyType::Commutativity,
            test_code: String::new(),
        };

        let generator = CodeGenerator::new(vec![prop], GeneratorConfig::default());
        let code = generator.generate_tests();

        // Should be valid Rust syntax
        assert!(code.contains("#[test]") || code.contains("proptest!"));
        assert!(code.contains("fn test_"));
    }
}
```

---

#### Why Milestone 3 Isn't Enough

**Limitation**: Default generators (any::<i32>()) produce random values, but many functions have constraints. Need custom generators for bounded values, valid states, etc.

**What we're adding**: Smart generator creation that respects domain constraints and generates realistic test values.

**Improvement**:
- **Realism**: Test values match actual use cases
- **Coverage**: Explore valid input space thoroughly
- **Efficiency**: Avoid wasting time on invalid inputs
- **Shrinking**: Better minimal examples when tests fail

---

### Milestone 4: Custom Value Generators

**Goal**: Create domain-specific value generators that produce realistic, constrained test inputs.

**Why this matters**: Testing `age(Person)` with `age = -1000` or `age = i32::MAX` wastes time. Custom generators ensure meaningful test inputs.

#### Architecture

**Structs:**
- `CustomGenerator` - Domain-specific value generator
  - **Field**: `generator_name: String` - Generator identifier
  - **Field**: `constraints: Vec<Constraint>` - Value constraints
  - **Field**: `strategy_code: String` - Proptest strategy code

- `Constraint` - Value constraint
  - **Variants**:
    - `Range(min, max)` - Bounded numeric range
    - `Length(min, max)` - Collection length bounds
    - `Pattern(regex)` - String pattern
    - `Predicate(condition)` - Custom validation

**Functions:**
- `infer_constraints(sig: &FunctionSignature) -> Vec<Constraint>` - Detect constraints
- `create_bounded_generator(constraint: &Constraint) -> String` - Create constrained generator
- `create_regex_generator(pattern: &str) -> String` - String pattern generator
- `create_struct_generator(struct_def: &str) -> String` - Composite generator

**Starter Code**:

```rust
#[derive(Debug, Clone)]
pub enum Constraint {
    Range { min: i64, max: i64 },
    Length { min: usize, max: usize },
    Pattern(String),
    NonZero,
    Positive,
    NonEmpty,
    Predicate(String),  // Custom condition as string
}

#[derive(Debug, Clone)]
pub struct CustomGenerator {
    pub generator_name: String,
    pub base_type: Type,
    pub constraints: Vec<Constraint>,
    pub strategy_code: String,
}

impl CustomGenerator {
    pub fn from_parameter(param: &Parameter) -> Option<Self> {
        // TODO: Analyze parameter for constraints
        // TODO: Check parameter name for hints: "age", "count", "index"
        // TODO: Check type for natural constraints
        // TODO: Create appropriate generator
        todo!("Create custom generator from parameter")
    }

    pub fn infer_constraints(param: &Parameter) -> Vec<Constraint> {
        // TODO: Use parameter name to infer constraints
        // TODO: "age" → Range(0, 150)
        // TODO: "count" → Positive
        // TODO: "index" → NonZero
        // TODO: "email" → Pattern(email_regex)
        todo!("Infer parameter constraints")
    }

    fn create_bounded_generator(constraint: &Constraint) -> String {
        // TODO: Generate proptest strategy code
        // TODO: Range(0, 100) → prop::num::i32::Range::new(0, 100)
        // TODO: Length(1, 10) → prop::collection::vec(..., 1..=10)
        todo!("Create bounded generator")
    }

    fn create_regex_generator(pattern: &str) -> String {
        // TODO: Generate: prop::string::string_regex(pattern)
        format!("prop::string::string_regex(\"{}\").unwrap()", pattern)
    }

    fn create_struct_generator(fields: &[(String, CustomGenerator)]) -> String {
        // TODO: Generate strategy that produces valid struct instances
        // TODO: Combine field generators with prop_compose! or strategy combinators
        todo!("Create struct generator")
    }

    pub fn generate_strategy_code(&self) -> String {
        // TODO: Combine all constraints into single strategy
        // TODO: Use .prop_filter() for predicates
        // TODO: Use .prop_map() for transformations
        todo!("Generate strategy code")
    }
}

// Predefined generators for common patterns
pub mod common_generators {
    use super::*;

    pub fn email_generator() -> CustomGenerator {
        CustomGenerator {
            generator_name: "email".to_string(),
            base_type: Type::Primitive("String".to_string()),
            constraints: vec![
                Constraint::Pattern("[a-z0-9.]+@[a-z0-9]+\\.[a-z]{2,}".to_string())
            ],
            strategy_code: r#"prop::string::string_regex("[a-z0-9.]+@[a-z0-9]+\\.[a-z]{2,}").unwrap()"#.to_string(),
        }
    }

    pub fn age_generator() -> CustomGenerator {
        // TODO: Create generator for realistic ages (0-150)
        todo!()
    }

    pub fn non_empty_string_generator() -> CustomGenerator {
        // TODO: Create generator for non-empty strings
        todo!()
    }

    pub fn positive_int_generator() -> CustomGenerator {
        // TODO: Create generator for positive integers
        todo!()
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_age_constraints() {
        let param = Parameter {
            name: "age".to_string(),
            param_type: Type::Primitive("i32".to_string()),
            is_mutable: false,
        };

        let constraints = CustomGenerator::infer_constraints(&param);

        // Should infer age is in range 0-150
        assert!(constraints.iter().any(|c| {
            matches!(c, Constraint::Range { min: 0, max: 150 })
                || matches!(c, Constraint::Positive)
        }));
    }

    #[test]
    fn test_infer_email_pattern() {
        let param = Parameter {
            name: "email".to_string(),
            param_type: Type::Primitive("String".to_string()),
            is_mutable: false,
        };

        let constraints = CustomGenerator::infer_constraints(&param);

        // Should infer email pattern
        assert!(constraints.iter().any(|c| {
            matches!(c, Constraint::Pattern(_))
        }));
    }

    #[test]
    fn test_range_generator_code() {
        let constraint = Constraint::Range { min: 1, max: 100 };
        let code = CustomGenerator::create_bounded_generator(&constraint);

        assert!(code.contains("1") && code.contains("100"));
        assert!(code.contains("Range") || code.contains(".."));
    }

    #[test]
    fn test_regex_generator_code() {
        let pattern = "[a-z]{3,10}";
        let code = CustomGenerator::create_regex_generator(pattern);

        assert!(code.contains("string_regex"));
        assert!(code.contains(pattern));
    }

    #[test]
    fn test_email_generator() {
        use common_generators::*;

        let gen = email_generator();
        assert_eq!(gen.generator_name, "email");
        assert!(gen.strategy_code.contains("@"));
    }

    #[test]
    fn test_non_zero_constraint() {
        let param = Parameter {
            name: "divisor".to_string(),
            param_type: Type::Primitive("i32".to_string()),
            is_mutable: false,
        };

        let constraints = CustomGenerator::infer_constraints(&param);

        // Divisor should be non-zero
        assert!(constraints.iter().any(|c| matches!(c, Constraint::NonZero)));
    }

    #[test]
    fn test_index_constraints() {
        let param = Parameter {
            name: "index".to_string(),
            param_type: Type::Primitive("usize".to_string()),
            is_mutable: false,
        };

        let gen = CustomGenerator::from_parameter(&param).unwrap();

        // Index should be non-negative (usize ensures this)
        // Might also be bounded by collection size
        assert!(gen.constraints.len() > 0);
    }
}
```

---

#### Why Milestone 4 Isn't Enough

**Limitation**: Generated tests are isolated—one property per test. Many bugs only appear when multiple properties interact.

**What we're adding**: Composite property testing that verifies multiple properties simultaneously and tests property combinations.

**Improvement**:
- **Interaction bugs**: Find bugs in property combinations
- **Efficiency**: Test multiple properties per run
- **Realism**: Mirror real-world usage patterns
- **Coverage**: Test edge case interactions

---

### Milestone 5: Shrinking Strategy Optimization

**Goal**: Optimize shrinking strategies to find minimal failing examples quickly.

**Why this matters**: When a property fails with a complex input, shrinking finds the simplest case. Better shrinking = faster debugging.

#### Architecture

**Structs:**
- `ShrinkStrategy` - Defines how to simplify values
  - **Field**: `value_type: Type` - Type being shrunk
  - **Field**: `shrink_steps: Vec<ShrinkStep>` - Simplification steps

**Enums:**
- `ShrinkStep` - One shrinking transformation
  - **Variants**:
    - `TowardsZero` - Reduce numbers toward 0
    - `RemoveElements` - Remove items from collections
    - `SimplifyStructure` - Flatten nested structures
    - `ReplaceWithDefault` - Use default/simple values

**Functions:**
- `create_shrink_strategy(typ: &Type) -> ShrinkStrategy` - Create strategy
- `optimize_for_type(typ: &Type) -> Vec<ShrinkStep>` - Best steps for type
- `generate_shrink_code(strategy: &ShrinkStrategy) -> String` - Generate code

**Starter Code**:

```rust
#[derive(Debug, Clone)]
pub enum ShrinkStep {
    TowardsZero,
    RemoveElements,
    ShortenString,
    SimplifyStructure,
    ReplaceWithDefault,
    BinarySearch,  // For finding exact boundary
}

#[derive(Debug, Clone)]
pub struct ShrinkStrategy {
    pub value_type: Type,
    pub shrink_steps: Vec<ShrinkStep>,
    pub strategy_code: String,
}

impl ShrinkStrategy {
    pub fn create_shrink_strategy(typ: &Type) -> Self {
        // TODO: Determine best shrinking approach for type
        // TODO: Numeric → TowardsZero
        // TODO: Collections → RemoveElements, ShortenString
        // TODO: Structs → SimplifyStructure
        todo!("Create shrink strategy")
    }

    fn optimize_for_type(typ: &Type) -> Vec<ShrinkStep> {
        // TODO: Return optimal shrinking steps for type
        match typ {
            Type::Primitive(name) if name.contains("i") || name.contains("u") => {
                vec![ShrinkStep::TowardsZero, ShrinkStep::BinarySearch]
            }
            Type::Vec(_) => {
                vec![ShrinkStep::RemoveElements, ShrinkStep::SimplifyStructure]
            }
            Type::Primitive(name) if name == "String" => {
                vec![ShrinkStep::ShortenString]
            }
            _ => vec![ShrinkStep::ReplaceWithDefault],
        }
    }

    pub fn generate_shrink_code(&self) -> String {
        // TODO: Generate proptest shrinking strategy code
        // TODO: Use BoxedStrategy for complex shrinking
        todo!("Generate shrink code")
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_shrink_strategy() {
        let typ = Type::Primitive("i32".to_string());
        let strategy = ShrinkStrategy::create_shrink_strategy(&typ);

        assert!(strategy.shrink_steps.contains(&ShrinkStep::TowardsZero));
    }

    #[test]
    fn test_vec_shrink_strategy() {
        let typ = Type::Vec(Box::new(Type::Primitive("i32".to_string())));
        let strategy = ShrinkStrategy::create_shrink_strategy(&typ);

        assert!(strategy.shrink_steps.contains(&ShrinkStep::RemoveElements));
    }

    #[test]
    fn test_string_shrink_strategy() {
        let typ = Type::Primitive("String".to_string());
        let strategy = ShrinkStrategy::create_shrink_strategy(&typ);

        assert!(strategy.shrink_steps.contains(&ShrinkStep::ShortenString));
    }
}
```

---

#### Why Milestone 5 Isn't Enough

**Limitation**: Generated tests are files on disk. Need CLI tool to integrate with development workflow.

**What we're adding**: Command-line interface for easy project integration and automation.

**Improvement**:
- **Usability**: Simple commands to generate tests
- **Integration**: Works with existing projects
- **Automation**: Can be used in CI/CD
- **Flexibility**: Configurable options

---

### Milestone 6: CLI Tool and Project Integration

**Goal**: Create a command-line tool that integrates property test generation into development workflow.

**Why this matters**: Developers need an easy way to generate tests without writing custom code. A polished CLI makes the tool practical.

#### Architecture

**CLI Commands:**
- `generate --file <path>` - Generate tests for one file
- `generate --project` - Generate for entire project
- `analyze --function <name>` - Analyze one function
- `list-properties --file <path>` - Show inferred properties without generating

**Configuration:**
- `.proptestgen.toml` - Project-wide configuration
- Command-line flags override config file

**Functions:**
- `cli_main(args: Vec<String>)` - CLI entry point
- `generate_for_file(path: &Path, config: &Config)` - Process one file
- `generate_for_project(root: &Path, config: &Config)` - Process project
- `write_generated_tests(tests: String, output_path: &Path)` - Save tests

**Starter Code**:

```rust
use clap::{App, Arg, SubCommand};
use std::path::Path;

pub struct Config {
    pub num_cases: usize,
    pub output_dir: String,
    pub parallel: bool,
    pub verbose: bool,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, std::io::Error> {
        // TODO: Load from .proptestgen.toml
        todo!("Load config from file")
    }

    pub fn merge_with_args(&mut self, args: &ArgMatches) {
        // TODO: Override config with CLI args
        todo!("Merge CLI args")
    }
}

pub fn cli_main() {
    let matches = App::new("Property Test Generator")
        .version("1.0")
        .about("Automatically generates property-based tests")
        .subcommand(
            SubCommand::with_name("generate")
                .about("Generate property tests")
                .arg(Arg::with_name("file")
                    .long("file")
                    .value_name("FILE")
                    .help("Source file to analyze"))
                .arg(Arg::with_name("project")
                    .long("project")
                    .help("Analyze entire project"))
                .arg(Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .value_name("DIR")
                    .help("Output directory for tests"))
        )
        .subcommand(
            SubCommand::with_name("analyze")
                .about("Analyze functions without generating tests")
                .arg(Arg::with_name("file")
                    .long("file")
                    .required(true)
                    .value_name("FILE"))
        )
        .get_matches();

    // TODO: Handle subcommands
    // TODO: Load config
    // TODO: Execute requested action
}

pub fn generate_for_file(path: &Path, config: &Config) -> Result<(), Error> {
    // TODO: Parse file
    // TODO: Extract functions
    // TODO: Infer properties
    // TODO: Generate tests
    // TODO: Write to output
    todo!("Generate tests for file")
}

pub fn generate_for_project(root: &Path, config: &Config) -> Result<(), Error> {
    // TODO: Find all .rs files
    // TODO: Process each file
    // TODO: Aggregate results
    todo!("Generate tests for project")
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_file_generation() {
        // Test generating tests for a single file
        let temp = create_temp_rust_file();
        let config = Config::default();

        let result = generate_for_file(temp.path(), &config);
        assert!(result.is_ok());

        // Check that tests were generated
        let output_path = Path::new(&config.output_dir).join("generated_tests.rs");
        assert!(output_path.exists());
    }

    #[test]
    fn test_project_generation() {
        let temp_project = create_temp_project();
        let config = Config::default();

        let result = generate_for_project(temp_project.path(), &config);
        assert!(result.is_ok());
    }
}
```

---

## Testing Strategies

### 1. Unit Tests
- Test each component: parsing, inference, generation
- Verify edge cases: empty functions, complex generics
- Validate generated code syntax

### 2. Integration Tests
- End-to-end: parse → infer → generate → compile
- Test on real Rust projects
- Verify generated tests actually run

### 3. Meta-Testing
- Generate properties for the generator itself
- Ensure generated tests are deterministic
- Verify no false positives

### 4. Performance Tests
- Measure generation speed
- Test scalability (1000+ functions)
- Benchmark shrinking efficiency

---

## Complete Working Example

Due to space constraints, the complete implementation is provided in separate modules. The full system demonstrates:
- **Intelligent inference**: Automatically detects applicable properties
- **Code generation**: Produces idiomatic, compilable Rust tests
- **Custom generators**: Creates realistic test values
- **CLI integration**: Easy-to-use command-line tool

Run the generator:
```bash
proptestgen generate --project --output tests/generated
```

This creates comprehensive property-based tests that explore your code's behavior far more thoroughly than manual testing ever could.
