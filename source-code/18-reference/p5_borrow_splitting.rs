// Pattern 5: Borrow Splitting
struct Data {
    left: Vec<i32>,
    right: Vec<i32>,
}

impl Data {
    // Returns mutable references to both fields simultaneously
    fn split(&mut self) -> (&mut Vec<i32>, &mut Vec<i32>) {
        (&mut self.left, &mut self.right)
    }
}

// Slice splitting
fn slice_split() {
    let mut arr = [1, 2, 3, 4, 5];

    // split_at_mut returns two non-overlapping mutable slices
    let (left, right) = arr.split_at_mut(2);
    // left: &mut [1, 2], right: &mut [3, 4, 5]

    left[0] = 10;
    right[0] = 30;
    // Both mutations are valid because slices don't overlap

    println!("Array after split: {:?}", arr);
}

// Manual split with unsafe (when safe API isn't available)
fn manual_split<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    assert!(mid <= slice.len());

    let ptr = slice.as_mut_ptr();
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), slice.len() - mid),
        )
    }
}

fn main() {
    let mut data = Data {
        left: vec![1, 2, 3],
        right: vec![4, 5, 6],
    };

    let (l, r) = data.split();
    l.push(10);
    r.push(20);
    println!("Left: {:?}, Right: {:?}", data.left, data.right);

    slice_split();

    let mut arr = [1, 2, 3, 4, 5];
    let (a, b) = manual_split(&mut arr, 2);
    a[0] = 100;
    b[0] = 300;
    println!("Manual split result: {:?}", arr);

    println!("Borrow splitting example completed");
}
