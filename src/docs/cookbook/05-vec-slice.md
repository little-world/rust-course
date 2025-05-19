

## Rust Vec<T> vs. &[T] Method Matrix

| Method                 | `Vec<T>` | `&[T]` | Consumes? | Mutates? | Purpose / Common Use Case             |
| ---------------------- | -------- | ------ | --------- | -------- | ------------------------------------- |
| `iter()`               | ✅        | ✅      | ❌         | ❌        | Immutable iteration                   |
| `iter_mut()`           | ✅        | ✅      | ❌         | ✅        | Mutable iteration                     |
| `into_iter()`          | ✅        | ⚠️¹    | ✅         | ❌        | Consumes vector, owns items           |
| `map()` (via iterator) | ✅        | ✅      | ❌         | ❌        | Transform items                       |
| `filter()`             | ✅        | ✅      | ❌         | ❌        | Select matching items                 |
| `fold()`               | ✅        | ✅      | ❌         | ❌        | Reduce to one value                   |
| `collect()`            | ✅        | ✅      | ❌         | ❌        | Turn iterator into collection         |
| `clone()`              | ✅        | ⚠️²    | ✅         | ❌        | Duplicate owned data                  |
| `push()`               | ✅        | ❌      | ❌         | ✅        | Append element                        |
| `pop()`                | ✅        | ❌      | ❌         | ✅        | Remove last element                   |
| `insert()`             | ✅        | ❌      | ❌         | ✅        | Insert at position                    |
| `remove()`             | ✅        | ❌      | ❌         | ✅        | Remove by index                       |
| `retain()`             | ✅        | ❌      | ❌         | ✅        | Retain elements by predicate          |
| `clear()`              | ✅        | ❌      | ❌         | ✅        | Empty the vector                      |
| `truncate()`           | ✅        | ❌      | ❌         | ✅        | Remove elements after N               |
| `resize()`             | ✅        | ❌      | ❌         | ✅        | Resize with default                   |
| `sort()`               | ✅        | ❌      | ❌         | ✅        | In-place sort                         |
| `sort_by()`            | ✅        | ❌      | ❌         | ✅        | Custom sort                           |
| `dedup()`              | ✅        | ❌      | ❌         | ✅        | Remove consecutive duplicates         |
| `reverse()`            | ✅        | ❌      | ❌         | ✅        | Reverse in-place                      |
| `windows(n)`           | ✅        | ✅      | ❌         | ❌        | Sliding windows of size `n`           |
| `chunks(n)`            | ✅        | ✅      | ❌         | ❌        | Break into chunks                     |
| `zip()` (via iterator) | ✅        | ✅      | ❌         | ❌        | Combine two iterators                 |
| `any()` / `all()`      | ✅        | ✅      | ❌         | ❌        | Boolean checks                        |
| `find()`               | ✅        | ✅      | ❌         | ❌        | Find first match                      |
| `partition()`          | ✅        | ✅      | ❌         | ❌        | Split into two groups by predicate    |
| `drain()`              | ✅        | ❌      | ✅         | ✅        | Remove and return part of the vector  |
| `clone_from_slice()`   | ✅        | ✅      | ❌         | ✅        | Copy data from slice (with same size) |
| `to_vec()`             | ❌        | ✅      | ✅         | ❌        | Create owned `Vec` from slice         |

---

### Notes:

* ⚠️¹ `&[T]::into_iter()` produces `&T` items, not owned `T`.
* ⚠️² Slices don't have `clone()` directly, but you can clone their content: `slice.to_vec()`.


