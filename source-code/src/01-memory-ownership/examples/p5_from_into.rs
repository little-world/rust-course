// Pattern 5: From/Into Implementation
struct UserId(u64);

// Implement From, get Into for free
impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        UserId(id)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        UserId(s.parse().unwrap_or(0))
    }
}

fn create_user(id: impl Into<UserId>) {
    let user_id: UserId = id.into();
    println!("User ID: {}", user_id.0);
}

fn use_from_into() {
    create_user(42u64);      // u64 -> UserId
    create_user("12345");    // &str -> UserId
}

fn main() {
    use_from_into();
    println!("From/Into example completed");
}
