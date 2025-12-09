**What to implement**:
- `binary_search_exact()`: Find exact match, return index
- `binary_search_lower_bound()`: Find first element >= target
- `binary_search_upper_bound()`: Find first element > target
- Generic implementations that work with any ordered type

---

### Understanding Binary Search

**What is Binary Search?**

Binary search is an efficient algorithm for finding a target value in a **sorted array**. Instead of checking each element sequentially (linear search), binary search repeatedly divides the search space in half, eliminating half of the remaining elements in each step.

**Key Requirements:**
- The array **must be sorted** (ascending or descending)
- Random access to elements (arrays/vectors work well)

**Time Complexity:**
- Binary search: **O(log n)** - logarithmic time
- Linear search: **O(n)** - linear time
- For 1 million elements: ~20 comparisons (binary) vs 1 million comparisons (linear)

**How It Works:**

1. Start with two pointers: `left = 0` and `right = array.length - 1`
2. Calculate the middle index: `mid = (left + right) / 2`
3. Compare the middle element with the target:
   - If `array[mid] == target`: Found! Return the index
   - If `array[mid] < target`: Target is in the right half, set `left = mid + 1`
   - If `array[mid] > target`: Target is in the left half, set `right = mid - 1`
4. Repeat until `left > right` (target not found)

**Visual Example:**
