# Chapter 25: FFI & C Interop — Programming Projects

## Project 2: Python Interop with PyO3 (Native Extensions, GIL, and Object Lifecycle)

### Introduction to Python FFI Concepts

Python interoperability opens Rust's performance to the vast Python ecosystem. Unlike C FFI, Python interop requires understanding Python's runtime model, memory management, and unique constraints. The PyO3 library provides safe, ergonomic bindings between Rust and Python.

#### 1. The Global Interpreter Lock (GIL)

Python's Global Interpreter Lock ensures only one thread executes Python bytecode at a time. This has profound implications for Rust interop:

**Holding the GIL**: When calling from Python into Rust, you hold the GIL by default. This means:
- Only one Python thread can execute at a time
- No parallelism benefit for pure-Python code
- CPU-intensive Rust code blocks all Python threads

**Releasing the GIL**: PyO3's `py.allow_threads(|| { ... })` temporarily releases the GIL, allowing:
- True parallelism for CPU-intensive Rust work
- Other Python threads to run concurrently
- Risk of deadlock if you re-acquire the GIL inside

**GIL Strategy**:
```rust
#[pyfunction]
fn compute_heavy(py: Python, data: Vec<f64>) -> PyResult<f64> {
    // Release GIL for CPU-intensive work
    py.allow_threads(|| {
        expensive_rust_computation(&data)
    })
}
```

#### 2. Python Object Lifecycle and Reference Counting

Python uses reference counting for memory management (with cycle detection). Rust must respect Python's ownership model:

**`Py<T>` vs `&PyAny`**:
- `Py<T>`: Owned Python object reference (can cross the GIL boundary, lives beyond function scope)
- `&PyAny`: Borrowed Python object (tied to GIL token lifetime, cannot escape function)

**Reference Semantics**:
```rust
// Borrowed - cannot store or return without cloning
fn process(obj: &PyDict) { }

// Owned - can store in Rust structs, return from functions
fn create(py: Python) -> Py<PyDict> {
    PyDict::new(py).into()
}
```

**Manual Reference Management**: PyO3 handles most refcounting automatically, but when crossing boundaries or storing Python objects in Rust structs, you must explicitly manage `Py<T>` objects.

#### 3. Exception Handling Between Rust and Python

Rust's `Result<T, E>` maps naturally to Python exceptions via `PyResult<T>`:

**Error Propagation**:
```rust
use pyo3::exceptions::PyValueError;

#[pyfunction]
fn parse_data(s: &str) -> PyResult<i32> {
    s.parse::<i32>()
        .map_err(|_| PyValueError::new_err("Invalid integer"))
}
```

**Panic Behavior**: Panics in Rust code called from Python are caught and converted to Python exceptions. However, this is expensive—prefer `PyResult` for error handling.

**Custom Exceptions**:
```rust
create_exception!(mymodule, CustomError, PyException);

#[pyfunction]
fn risky_op() -> PyResult<()> {
    Err(CustomError::new_err("Something went wrong"))
}
```

#### 4. Type Conversion with `FromPyObject` and `IntoPy`

PyO3 provides automatic conversion between Rust and Python types:

**Common Conversions**:
- `i32`, `f64`, `bool` ↔ Python `int`, `float`, `bool`
- `String`, `&str` ↔ Python `str`
- `Vec<T>` ↔ Python `list`
- `HashMap<K, V>` ↔ Python `dict`

**Custom Conversions**:
```rust
#[pyclass]
struct Point {
    #[pyo3(get, set)]
    x: f64,
    #[pyo3(get, set)]
    y: f64,
}

impl FromPyObject<'_> for Point {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        // Custom extraction logic
    }
}
```

**Zero-Copy with `PyBytes`**: For binary data, use `PyBytes` to avoid copies:
```rust
fn process_bytes(data: &[u8]) -> Vec<u8> { }

#[pyfunction]
fn wrapper(py: Python, data: &[u8]) -> PyResult<Py<PyBytes>> {
    let result = process_bytes(data);
    Ok(PyBytes::new(py, &result).into())
}
```

#### 5. `#[pyclass]`, `#[pymethods]`, and `#[pyfunction]`

PyO3's attribute macros expose Rust items to Python:

**`#[pyclass]`**: Makes a Rust struct a Python class
```rust
#[pyclass]
struct Counter {
    count: i32,
}
```

**`#[pymethods]`**: Exposes methods to Python
```rust
#[pymethods]
impl Counter {
    #[new]
    fn new() -> Self { Counter { count: 0 } }

    fn increment(&mut self) { self.count += 1; }

    #[getter]
    fn count(&self) -> i32 { self.count }
}
```

**`#[pyfunction]`**: Exposes standalone functions
```rust
#[pyfunction]
fn add(a: i32, b: i32) -> i32 { a + b }
```

#### 6. Module Definition and Registration

A Python extension module bundles functions and classes:

```rust
#[pymodule]
fn mymodule(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_class::<Counter>()?;
    Ok(())
}
```

**Building with `maturin`**: PyO3 projects use `maturin` to build and install:
```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"
```

#### 7. Interior Mutability in `#[pyclass]`

Python expects mutable methods (`self.value = x`) but Rust's `&self` is immutable. Solutions:

**`RefCell` for Single-Threaded**:
```rust
#[pyclass]
struct State {
    data: RefCell<Vec<i32>>,
}

#[pymethods]
impl State {
    fn push(&self, val: i32) {
        self.data.borrow_mut().push(val);
    }
}
```

**`Mutex` for Thread-Safe**:
```rust
#[pyclass]
struct SharedState {
    data: Arc<Mutex<HashMap<String, i32>>>,
}
```

#### 8. Iterators and Python Protocols

Implement Python's iterator protocol in Rust:

```rust
#[pyclass]
struct RangeIter {
    current: i32,
    end: i32,
}

#[pymethods]
impl RangeIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> { slf }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<i32> {
        if slf.current < slf.end {
            let val = slf.current;
            slf.current += 1;
            Some(val)
        } else {
            None
        }
    }
}
```

#### 9. Async/Await Bridging (pyo3-asyncio)

Bridge Rust async with Python's `asyncio`:

```rust
use pyo3_asyncio::tokio::future_into_py;

#[pyfunction]
fn async_fetch(py: Python, url: String) -> PyResult<&PyAny> {
    future_into_py(py, async move {
        let response = reqwest::get(&url).await?;
        Ok(response.text().await?)
    })
}
```

This returns a Python `coroutine` that can be `await`ed in Python code.

#### 10. Buffer Protocol for Zero-Copy NumPy Integration

The buffer protocol enables zero-copy sharing with NumPy arrays:

```rust
use numpy::{PyArray1, PyReadonlyArray1};

#[pyfunction]
fn process_array(array: PyReadonlyArray1<f64>) -> PyResult<Py<PyArray1<f64>>> {
    let slice = array.as_slice()?;
    // Process without copying
}
```

### Connection to This Project

This project builds a Python extension module that exposes Rust functionality through a safe, idiomatic Python API. Here's how each concept applies:

**GIL Management**: You'll implement CPU-intensive processing functions that release the GIL, allowing Python programs to benefit from true parallelism while maintaining safety.

**Object Lifecycle**: The project creates stateful Python classes backed by Rust structs, requiring careful management of `Py<T>` references and understanding when objects cross the GIL boundary.

**Exception Handling**: All Rust errors map to Python exceptions using `PyResult`, demonstrating proper error propagation between languages.

**Type Conversion**: Functions accept Python lists, dicts, and strings, automatically converting them to Rust types, then returning converted results.

**`#[pyclass]` Architecture**: You'll design Python classes with methods, properties, and lifecycle hooks (`__init__`, `__repr__`), all implemented in Rust.

**Interior Mutability**: Stateful classes use `RefCell` for single-threaded mutation, showing how to reconcile Python's mutable-by-default semantics with Rust's borrowing rules.

**Module Building**: The project culminates in a complete, installable Python package with proper metadata, type stubs, and testing infrastructure.

By the end, you'll have created a **production-ready Python extension** that exposes high-performance Rust code through a natural Python interface, understanding both the technical mechanics and design considerations for Python interop.

---

### Problem Statement

Build a Python extension module using PyO3 that exposes a text processing library. Implement a stateful analyzer class that can parse documents, extract statistics, and notify callbacks on events. Demonstrate GIL management, proper error handling, Python object lifecycle management, and zero-copy operations where possible. Provide type stubs for IDE support and comprehensive tests in Python.

### Why It Matters

- Python is ubiquitous in data science, ML, and scripting. Rust extensions can accelerate critical paths by 10-100x.
- PyO3 provides memory-safe Python bindings without the fragility of raw FFI or ctypes.
- Understanding GIL semantics and object lifecycle prevents subtle bugs in production systems.

### Use Cases

- Accelerating data processing pipelines (parsing, validation, transformation).
- Building Python-friendly APIs for Rust libraries (cryptography, compression, parsing).
- Creating drop-in replacements for slow pure-Python modules.

---

## Solution Outline (Didactic, not full implementation)

1) Set up a PyO3 project with `maturin`, define a minimal module with a single function.
2) Create a stateful `#[pyclass]` that manages internal state with `RefCell`.
3) Add methods that process Python strings, return statistics as Python dicts.
4) Implement callback registration that stores a Python callable and invokes it from Rust.
5) Add GIL release for CPU-intensive operations; benchmark vs. pure Python.
6) Error handling: custom exceptions, validation with `PyResult`.
7) Generate type stubs (`.pyi`) and write comprehensive Python tests with `pytest`.

---

## Milestone 1: Project Setup and Basic Module

### Introduction
Create a new PyO3 project with `maturin`, expose a simple function, and verify it's callable from Python.

Why previous step is not enough: We need a working build pipeline before implementing features.

### Architecture

- Project structure using `maturin init`.
- Functions:
  - `#[pyfunction] fn hello(name: &str) -> String` — Returns a greeting.
- Module:
  - `#[pymodule] fn text_processor(_py: Python, m: &PyModule) -> PyResult<()>`

### Checkpoint Tests

```python
# test_basic.py
import text_processor

def test_hello():
    result = text_processor.hello("World")
    assert result == "Hello, World!"
```

### Starter Code

**Cargo.toml**:
```toml
[package]
name = "text_processor"
version = "0.1.0"
edition = "2021"

[lib]
name = "text_processor"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }
```

**src/lib.rs**:
```rust
use pyo3::prelude::*;

#[pyfunction]
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[pymodule]
fn text_processor(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    Ok(())
}
```

**Build and install**:
```bash
maturin develop  # development install
python -c "import text_processor; print(text_processor.hello('Rust'))"
```

---

## Milestone 2: Stateful Analyzer Class with `#[pyclass]`

### Introduction
Create a `TextAnalyzer` class that stores processed documents and maintains statistics.

Why previous step is not enough: Real applications need stateful objects that persist across calls.

### Architecture

- Structs:
  - `#[pyclass] struct TextAnalyzer { documents: RefCell<Vec<String>>, word_count: RefCell<usize> }`
- Methods:
  - `#[new] fn new() -> Self`
  - `fn add_document(&self, text: &str)`
  - `fn total_words(&self) -> usize`
  - `fn document_count(&self) -> usize`

### Checkpoint Tests

```python
def test_analyzer_state():
    analyzer = text_processor.TextAnalyzer()
    analyzer.add_document("hello world")
    analyzer.add_document("foo bar baz")
    assert analyzer.document_count() == 2
    assert analyzer.total_words() == 5
```

### Starter Code

```rust
use pyo3::prelude::*;
use std::cell::RefCell;

#[pyclass]
struct TextAnalyzer {
    documents: RefCell<Vec<String>>,
    word_count: RefCell<usize>,
}

#[pymethods]
impl TextAnalyzer {
    #[new]
    fn new() -> Self {
        TextAnalyzer {
            documents: RefCell::new(Vec::new()),
            word_count: RefCell::new(0),
        }
    }

    fn add_document(&self, text: &str) {
        let words = text.split_whitespace().count();
        self.documents.borrow_mut().push(text.to_string());
        *self.word_count.borrow_mut() += words;
    }

    fn total_words(&self) -> usize {
        *self.word_count.borrow()
    }

    fn document_count(&self) -> usize {
        self.documents.borrow().len()
    }

    fn __repr__(&self) -> String {
        format!(
            "TextAnalyzer(documents={}, words={})",
            self.document_count(),
            self.total_words()
        )
    }
}
```

Register in module:
```rust
#[pymodule]
fn text_processor(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    m.add_class::<TextAnalyzer>()?;
    Ok(())
}
```

---

## Milestone 3: Return Rich Python Objects (Dicts and Lists)

### Introduction
Add methods that return statistics as Python dictionaries and lists for natural Python consumption.

Why previous step is not enough: Simple scalars are limiting; Python users expect dicts, lists, and tuples.

### Architecture

- Methods:
  - `fn get_statistics(&self, py: Python) -> PyResult<PyObject>` — Returns a dict with various stats.
  - `fn get_documents(&self) -> Vec<String>` — Returns all documents.
  - `fn word_frequency(&self, py: Python) -> PyResult<PyObject>` — Returns word→count mapping.

### Checkpoint Tests

```python
def test_statistics():
    analyzer = text_processor.TextAnalyzer()
    analyzer.add_document("hello world hello")
    stats = analyzer.get_statistics()
    assert stats["total_words"] == 3
    assert stats["unique_words"] == 2

    freq = analyzer.word_frequency()
    assert freq["hello"] == 2
    assert freq["world"] == 1
```

### Starter Code

```rust
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

#[pymethods]
impl TextAnalyzer {
    fn get_statistics(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("total_words", self.total_words())?;
        dict.set_item("document_count", self.document_count())?;

        // Calculate unique words
        let mut unique = std::collections::HashSet::new();
        for doc in self.documents.borrow().iter() {
            for word in doc.split_whitespace() {
                unique.insert(word.to_lowercase());
            }
        }
        dict.set_item("unique_words", unique.len())?;

        Ok(dict.into())
    }

    fn get_documents(&self) -> Vec<String> {
        self.documents.borrow().clone()
    }

    fn word_frequency(&self, py: Python) -> PyResult<PyObject> {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for doc in self.documents.borrow().iter() {
            for word in doc.split_whitespace() {
                let word = word.to_lowercase();
                *freq.entry(word).or_insert(0) += 1;
            }
        }

        let dict = PyDict::new(py);
        for (word, count) in freq {
            dict.set_item(word, count)?;
        }
        Ok(dict.into())
    }
}
```

---

## Milestone 4: Callback Registration and Invocation

### Introduction
Allow Python users to register callbacks that Rust invokes on certain events (e.g., when a document is added).

Why previous step is not enough: Event-driven APIs are common in Python; we need bidirectional calls.

### Architecture

- Structs:
  - Add `callback: RefCell<Option<Py<PyAny>>>` to `TextAnalyzer`.
- Methods:
  - `fn set_callback(&self, callback: PyObject)` — Store the Python callable.
  - `fn clear_callback(&self)` — Remove the callback.
  - Modify `add_document` to invoke the callback with document info.

### Checkpoint Tests

```python
def test_callback():
    events = []

    def on_document(info):
        events.append(info)

    analyzer = text_processor.TextAnalyzer()
    analyzer.set_callback(on_document)
    analyzer.add_document("test doc")

    assert len(events) == 1
    assert events[0]["text"] == "test doc"
    assert events[0]["word_count"] == 2
```

### Starter Code

```rust
#[pyclass]
struct TextAnalyzer {
    documents: RefCell<Vec<String>>,
    word_count: RefCell<usize>,
    callback: RefCell<Option<Py<PyAny>>>,
}

#[pymethods]
impl TextAnalyzer {
    #[new]
    fn new() -> Self {
        TextAnalyzer {
            documents: RefCell::new(Vec::new()),
            word_count: RefCell::new(0),
            callback: RefCell::new(None),
        }
    }

    fn set_callback(&self, callback: PyObject) {
        *self.callback.borrow_mut() = Some(callback);
    }

    fn clear_callback(&self) {
        *self.callback.borrow_mut() = None;
    }

    fn add_document(&self, py: Python, text: &str) -> PyResult<()> {
        let words = text.split_whitespace().count();
        self.documents.borrow_mut().push(text.to_string());
        *self.word_count.borrow_mut() += words;

        // Invoke callback if registered
        if let Some(ref callback) = *self.callback.borrow() {
            let dict = PyDict::new(py);
            dict.set_item("text", text)?;
            dict.set_item("word_count", words)?;
            dict.set_item("total_documents", self.document_count())?;
            callback.call1(py, (dict,))?;
        }

        Ok(())
    }
}
```

---

## Milestone 5: GIL Release for CPU-Intensive Operations

### Introduction
Add a CPU-intensive analysis function that releases the GIL, allowing true parallelism when called from multiple Python threads.

Why previous step is not enough: Without GIL release, CPU-heavy Rust code blocks all Python threads, negating performance benefits.

### Architecture

- Functions:
  - `fn analyze_sentiment(&self, py: Python) -> PyResult<f64>` — Performs expensive analysis with GIL release.
- Pattern:
  ```rust
  let data = self.prepare_data(); // with GIL
  let result = py.allow_threads(|| {
      expensive_computation(&data) // without GIL
  });
  ```

### Checkpoint Tests

```python
import threading
import time

def test_parallelism():
    analyzer = text_processor.TextAnalyzer()
    for i in range(1000):
        analyzer.add_document("word " * 100)

    start = time.time()
    threads = []
    for _ in range(4):
        t = threading.Thread(target=analyzer.analyze_sentiment)
        threads.append(t)
        t.start()

    for t in threads:
        t.join()

    duration = time.time() - start
    # Should complete faster than 4x single-threaded time
    print(f"Parallel execution: {duration:.2f}s")
```

### Starter Code

```rust
#[pymethods]
impl TextAnalyzer {
    fn analyze_sentiment(&self, py: Python) -> PyResult<f64> {
        // Prepare data while holding GIL
        let documents = self.documents.borrow().clone();

        // Release GIL for CPU-intensive work
        let score = py.allow_threads(|| {
            let mut total = 0.0;
            for doc in &documents {
                // Simulate expensive computation
                for word in doc.split_whitespace() {
                    total += compute_word_sentiment(word);
                }
                // Artificial delay to demonstrate parallelism
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            total / documents.len() as f64
        });

        Ok(score)
    }
}

fn compute_word_sentiment(word: &str) -> f64 {
    // Simplified sentiment scoring
    match word.to_lowercase().as_str() {
        "good" | "great" | "excellent" => 1.0,
        "bad" | "terrible" | "awful" => -1.0,
        _ => 0.0,
    }
}
```

---

## Milestone 6: Error Handling with Custom Exceptions

### Introduction
Create custom Python exceptions for domain-specific errors and demonstrate proper error propagation.

Why previous step is not enough: Generic exceptions provide poor user experience; domain-specific errors are clearer.

### Architecture

- Exceptions:
  - `EmptyAnalyzerError` — raised when operations require documents but none exist.
  - `InvalidDocumentError` — raised for malformed input.
- Methods that validate and return `PyResult`.

### Checkpoint Tests

```python
import pytest

def test_empty_analyzer_error():
    analyzer = text_processor.TextAnalyzer()
    with pytest.raises(text_processor.EmptyAnalyzerError):
        analyzer.analyze_sentiment()

def test_invalid_document_error():
    analyzer = text_processor.TextAnalyzer()
    with pytest.raises(text_processor.InvalidDocumentError):
        analyzer.add_document("")  # empty document
```

### Starter Code

```rust
use pyo3::create_exception;
use pyo3::exceptions::PyException;

create_exception!(text_processor, EmptyAnalyzerError, PyException);
create_exception!(text_processor, InvalidDocumentError, PyException);

#[pymethods]
impl TextAnalyzer {
    fn add_document(&self, py: Python, text: &str) -> PyResult<()> {
        if text.trim().is_empty() {
            return Err(InvalidDocumentError::new_err("Document cannot be empty"));
        }

        // ... existing logic ...
        Ok(())
    }

    fn analyze_sentiment(&self, py: Python) -> PyResult<f64> {
        if self.documents.borrow().is_empty() {
            return Err(EmptyAnalyzerError::new_err(
                "Cannot analyze sentiment of empty analyzer"
            ));
        }

        // ... existing logic ...
    }
}

#[pymodule]
fn text_processor(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    m.add_class::<TextAnalyzer>()?;
    m.add("EmptyAnalyzerError", _py.get_type::<EmptyAnalyzerError>())?;
    m.add("InvalidDocumentError", _py.get_type::<InvalidDocumentError>())?;
    Ok(())
}
```

---

## Milestone 7: Type Stubs and Complete Python Package

### Introduction
Generate type stubs (`.pyi` files) for IDE support and create a complete, installable Python package with tests.

Why previous step is not enough: Without type hints, Python users lose autocomplete and type checking benefits.

### Architecture

- Create `text_processor.pyi` with type annotations.
- Add `pyproject.toml` with package metadata.
- Write comprehensive `pytest` test suite.
- Document the API in docstrings accessible via `help()`.

### Starter Code

**text_processor.pyi**:
```python
from typing import Optional, Callable, Dict, List, Any

class TextAnalyzer:
    def __init__(self) -> None: ...
    def add_document(self, text: str) -> None: ...
    def total_words(self) -> int: ...
    def document_count(self) -> int: ...
    def get_statistics(self) -> Dict[str, int]: ...
    def get_documents(self) -> List[str]: ...
    def word_frequency(self) -> Dict[str, int]: ...
    def set_callback(self, callback: Callable[[Dict[str, Any]], None]) -> None: ...
    def clear_callback(self) -> None: ...
    def analyze_sentiment(self) -> float: ...

class EmptyAnalyzerError(Exception): ...
class InvalidDocumentError(Exception): ...

def hello(name: str) -> str: ...
```

**pyproject.toml**:
```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "text_processor"
version = "0.1.0"
description = "Fast text processing library written in Rust"
authors = [{name = "Your Name", email = "you@example.com"}]
requires-python = ">=3.8"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: 3",
]

[project.optional-dependencies]
dev = ["pytest>=7.0", "pytest-benchmark"]
```

**tests/test_text_processor.py**:
```python
import pytest
import text_processor

class TestTextAnalyzer:
    def test_initial_state(self):
        analyzer = text_processor.TextAnalyzer()
        assert analyzer.document_count() == 0
        assert analyzer.total_words() == 0

    def test_add_documents(self):
        analyzer = text_processor.TextAnalyzer()
        analyzer.add_document("hello world")
        assert analyzer.document_count() == 1
        assert analyzer.total_words() == 2

    def test_word_frequency(self):
        analyzer = text_processor.TextAnalyzer()
        analyzer.add_document("hello world hello")
        freq = analyzer.word_frequency()
        assert freq["hello"] == 2
        assert freq["world"] == 1

    def test_empty_document_error(self):
        analyzer = text_processor.TextAnalyzer()
        with pytest.raises(text_processor.InvalidDocumentError):
            analyzer.add_document("")

    def test_callback_invocation(self):
        events = []
        analyzer = text_processor.TextAnalyzer()
        analyzer.set_callback(lambda info: events.append(info))
        analyzer.add_document("test")
        assert len(events) == 1
        assert events[0]["word_count"] == 1

def test_hello_function():
    result = text_processor.hello("Python")
    assert result == "Hello, Python!"
```

**Build and test**:
```bash
maturin develop
pytest tests/
```

---

## Complete Working Example

See the accumulated code from all milestones above. The complete module includes:

- Basic function (`hello`)
- Stateful class (`TextAnalyzer`)
- Interior mutability with `RefCell`
- Rich Python return types (dicts, lists)
- Callback registration and invocation
- GIL release for parallelism
- Custom exception types
- Type stubs for IDE support

---

## Performance Comparison and Benchmarking

### Introduction
Compare the Rust implementation against equivalent pure Python code to demonstrate performance benefits.

Why it matters: Quantifying speedup justifies the complexity of FFI and guides optimization efforts.

### Pure Python Baseline

```python
# pure_python.py
from collections import Counter

class PythonAnalyzer:
    def __init__(self):
        self.documents = []
        self.word_count = 0

    def add_document(self, text):
        if not text.strip():
            raise ValueError("Empty document")
        words = text.split()
        self.documents.append(text)
        self.word_count += len(words)

    def word_frequency(self):
        counter = Counter()
        for doc in self.documents:
            counter.update(word.lower() for word in doc.split())
        return dict(counter)

    def analyze_sentiment(self):
        if not self.documents:
            raise ValueError("Empty analyzer")
        total = 0.0
        for doc in self.documents:
            for word in doc.split():
                total += self._word_sentiment(word)
        return total / len(self.documents)

    @staticmethod
    def _word_sentiment(word):
        word = word.lower()
        if word in {"good", "great", "excellent"}:
            return 1.0
        elif word in {"bad", "terrible", "awful"}:
            return -1.0
        return 0.0
```

### Benchmark Script

```python
# benchmark.py
import time
import text_processor
from pure_python import PythonAnalyzer

def benchmark_add_documents(analyzer_class, n=10000):
    analyzer = analyzer_class()
    start = time.time()
    for i in range(n):
        analyzer.add_document(f"document {i} with some words here")
    return time.time() - start

def benchmark_word_frequency(analyzer_class, n=1000):
    analyzer = analyzer_class()
    for i in range(n):
        analyzer.add_document(f"word{i % 100} repeated multiple times")

    start = time.time()
    freq = analyzer.word_frequency()
    return time.time() - start

def benchmark_sentiment(analyzer_class, n=1000):
    analyzer = analyzer_class()
    for i in range(n):
        analyzer.add_document("good bad neutral excellent terrible")

    start = time.time()
    score = analyzer.analyze_sentiment()
    return time.time() - start

if __name__ == "__main__":
    print("Add Documents:")
    py_time = benchmark_add_documents(PythonAnalyzer)
    rust_time = benchmark_add_documents(text_processor.TextAnalyzer)
    print(f"  Python: {py_time:.3f}s")
    print(f"  Rust:   {rust_time:.3f}s")
    print(f"  Speedup: {py_time/rust_time:.2f}x\n")

    print("Word Frequency:")
    py_time = benchmark_word_frequency(PythonAnalyzer)
    rust_time = benchmark_word_frequency(text_processor.TextAnalyzer)
    print(f"  Python: {py_time:.3f}s")
    print(f"  Rust:   {rust_time:.3f}s")
    print(f"  Speedup: {py_time/rust_time:.2f}x\n")

    print("Sentiment Analysis:")
    py_time = benchmark_sentiment(PythonAnalyzer)
    rust_time = benchmark_sentiment(text_processor.TextAnalyzer)
    print(f"  Python: {py_time:.3f}s")
    print(f"  Rust:   {rust_time:.3f}s")
    print(f"  Speedup: {py_time/rust_time:.2f}x")
```

Expected output shows 5-50x speedup depending on operation complexity.

---

## Advanced Topics: NumPy Integration

For projects processing numerical data, integrate with NumPy using the `numpy` crate:

```rust
use numpy::{PyArray1, PyReadonlyArray1, ToPyArray};

#[pyfunction]
fn process_array(py: Python, arr: PyReadonlyArray1<f64>) -> Py<PyArray1<f64>> {
    let input = arr.as_slice().unwrap();
    let output: Vec<f64> = input.iter().map(|x| x * 2.0).collect();
    output.to_pyarray(py).to_owned()
}
```

This enables zero-copy operations on NumPy arrays from Python.
