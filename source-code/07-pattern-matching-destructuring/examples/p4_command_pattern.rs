//! Pattern 4: State Machines with Type-State Pattern
//! Example: Command Pattern with Enums
//!
//! Run with: cargo run --example p4_command_pattern

#[derive(Debug)]
enum Command {
    CreateUser { username: String, email: String },
    DeleteUser { user_id: u64 },
    UpdateEmail { user_id: u64, new_email: String },
    ChangePassword { user_id: u64, new_hash: String },
    GrantRole { user_id: u64, role: String },
    RevokeRole { user_id: u64, role: String },
}

#[derive(Debug)]
enum CommandResult {
    Success { message: String },
    Failure { error: String },
}

fn execute_command(command: Command) -> CommandResult {
    // When you add a new Command variant, this match MUST be updated.
    // The compiler enforces handling all cases!
    match command {
        Command::CreateUser { username, email } => {
            println!("Creating user '{}' with email '{}'", username, email);
            // In real code: insert into database
            CommandResult::Success {
                message: format!("User '{}' created", username),
            }
        }
        Command::DeleteUser { user_id } => {
            println!("Deleting user {}", user_id);
            // In real code: delete from database
            CommandResult::Success {
                message: format!("User {} deleted", user_id),
            }
        }
        Command::UpdateEmail { user_id, new_email } => {
            println!("Updating email for user {} to '{}'", user_id, new_email);
            CommandResult::Success {
                message: format!("Email updated for user {}", user_id),
            }
        }
        Command::ChangePassword { user_id, .. } => {
            println!("Changing password for user {}", user_id);
            CommandResult::Success {
                message: format!("Password changed for user {}", user_id),
            }
        }
        Command::GrantRole { user_id, role } => {
            println!("Granting role '{}' to user {}", role, user_id);
            CommandResult::Success {
                message: format!("Role '{}' granted to user {}", role, user_id),
            }
        }
        Command::RevokeRole { user_id, role } => {
            println!("Revoking role '{}' from user {}", role, user_id);
            CommandResult::Success {
                message: format!("Role '{}' revoked from user {}", role, user_id),
            }
        }
    }
}

// Command validation using pattern matching
fn validate_command(command: &Command) -> Result<(), String> {
    match command {
        Command::CreateUser { username, email } => {
            if username.is_empty() {
                return Err("Username cannot be empty".to_string());
            }
            if !email.contains('@') {
                return Err("Invalid email format".to_string());
            }
            Ok(())
        }
        Command::UpdateEmail { new_email, .. } => {
            if !new_email.contains('@') {
                return Err("Invalid email format".to_string());
            }
            Ok(())
        }
        // Other commands don't need special validation
        _ => Ok(()),
    }
}

// Command queue processing
fn process_command_queue(commands: Vec<Command>) {
    for command in commands {
        print!("Validating {:?}... ", command);
        match validate_command(&command) {
            Ok(()) => {
                println!("OK");
                let result = execute_command(command);
                println!("  Result: {:?}", result);
            }
            Err(e) => {
                println!("FAILED: {}", e);
            }
        }
        println!();
    }
}

fn main() {
    println!("=== Command Pattern with Enums ===\n");

    // Usage: dispatch commands via pattern matching
    let commands = vec![
        Command::CreateUser {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        Command::CreateUser {
            username: "".to_string(), // Invalid: empty username
            email: "bob@example.com".to_string(),
        },
        Command::UpdateEmail {
            user_id: 1,
            new_email: "alice_new@example.com".to_string(),
        },
        Command::GrantRole {
            user_id: 1,
            role: "admin".to_string(),
        },
        Command::DeleteUser { user_id: 2 },
    ];

    process_command_queue(commands);

    println!("=== Benefits of Enum-Based Commands ===");
    println!("1. All commands are typed and documented");
    println!("2. Adding a variant forces updates to all match expressions");
    println!("3. Serialization/deserialization is straightforward");
    println!("4. Easy to implement command logging and replay");
    println!("5. Type-safe: can't pass wrong arguments");

    println!("\n=== Compared to OOP Command Pattern ===");
    println!("OOP: interface Command {{ execute(); }}");
    println!("     class CreateUser implements Command {{ ... }}");
    println!("     class DeleteUser implements Command {{ ... }}");
    println!();
    println!("Rust enum: single type, exhaustive matching");
    println!("     - All commands visible in one place");
    println!("     - Compiler enforces handling all cases");
    println!("     - No virtual dispatch overhead");
}
