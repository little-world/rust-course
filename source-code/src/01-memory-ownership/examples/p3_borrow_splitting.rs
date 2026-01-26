// Pattern 3: Borrow Splitting
struct Player {
    name: String,
    health: i32,
    position: (f32, f32),
}

fn borrow_splitting() {
    let mut player = Player {
        name: "Hero".into(),
        health: 100,
        position: (0.0, 0.0),
    };

    // Can borrow different fields mutably at the same time
    let name = &player.name;           // Immutable borrow of name
    let health = &mut player.health;   // Mutable borrow of health

    *health -= 10;
    println!("{} has {} health", name, health);

    // Use position to avoid warning
    println!("Position: {:?}", player.position);
}

// Works with slices too
fn slice_splitting() {
    let mut arr = [1, 2, 3, 4, 5];
    let (left, right) = arr.split_at_mut(2);

    left[0] = 10;   // Mutate left half
    right[0] = 30;  // Mutate right half simultaneously

    println!("{:?}", arr);  // [10, 2, 30, 4, 5]
}

fn main() {
    borrow_splitting();
    slice_splitting();
    println!("Borrow splitting example completed");
}
