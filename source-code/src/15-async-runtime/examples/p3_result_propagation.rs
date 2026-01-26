//! Pattern 3: Async/Await Patterns
//! Result propagation with ?
//!
//! Run with: cargo run --example p3_result_propagation

async fn fetch_user_data(user_id: u64) -> Result<String, String> {
    if user_id == 0 {
        return Err("Invalid ID".to_string());
    }
    Ok(format!("User {}", user_id))
}

async fn get_user_profile(user_id: u64) -> Result<String, String> {
    let data = fetch_user_data(user_id).await?;
    let profile = format!("Profile: {}", data);
    Ok(profile)
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let profile = get_user_profile(42).await?;
    println!("{}", profile);
    Ok(())
}
