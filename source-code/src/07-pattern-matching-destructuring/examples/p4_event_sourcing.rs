//! Pattern 4: State Machines with Type-State Pattern
//! Example: Event Sourcing with Enums
//!
//! Run with: cargo run --example p4_event_sourcing

use std::time::{SystemTime, UNIX_EPOCH};

// All possible events in the system are defined as enum variants.
// Each event captures the data needed to describe what happened.
#[derive(Debug, Clone)]
enum UserEvent {
    UserRegistered {
        user_id: u64,
        username: String,
        email: String,
        timestamp: u64,
    },
    EmailVerified {
        user_id: u64,
        timestamp: u64,
    },
    EmailChanged {
        user_id: u64,
        old_email: String,
        new_email: String,
        timestamp: u64,
    },
    PasswordChanged {
        user_id: u64,
        timestamp: u64,
    },
    AccountLocked {
        user_id: u64,
        reason: String,
        timestamp: u64,
    },
    AccountUnlocked {
        user_id: u64,
        timestamp: u64,
    },
}

// The aggregate state is rebuilt by applying events in order.
#[derive(Debug, Default)]
struct User {
    id: u64,
    username: String,
    email: String,
    is_verified: bool,
    is_locked: bool,
    password_changed_count: u32,
    last_updated: u64,
}

impl User {
    // Rebuild user state from a sequence of events.
    fn from_events(events: &[UserEvent]) -> Self {
        let mut user = User::default();
        for event in events {
            user.apply(event);
        }
        user
    }

    // Apply a single event to update state.
    // This match MUST handle all event types - compiler enforces it!
    fn apply(&mut self, event: &UserEvent) {
        match event {
            UserEvent::UserRegistered {
                user_id,
                username,
                email,
                timestamp,
            } => {
                self.id = *user_id;
                self.username = username.clone();
                self.email = email.clone();
                self.is_verified = false;
                self.is_locked = false;
                self.last_updated = *timestamp;
            }
            UserEvent::EmailVerified { timestamp, .. } => {
                self.is_verified = true;
                self.last_updated = *timestamp;
            }
            UserEvent::EmailChanged {
                new_email,
                timestamp,
                ..
            } => {
                self.email = new_email.clone();
                self.is_verified = false; // Re-verification needed
                self.last_updated = *timestamp;
            }
            UserEvent::PasswordChanged { timestamp, .. } => {
                self.password_changed_count += 1;
                self.last_updated = *timestamp;
            }
            UserEvent::AccountLocked { timestamp, .. } => {
                self.is_locked = true;
                self.last_updated = *timestamp;
            }
            UserEvent::AccountUnlocked { timestamp, .. } => {
                self.is_locked = false;
                self.last_updated = *timestamp;
            }
        }
    }
}

// Event store (simplified)
struct EventStore {
    events: Vec<UserEvent>,
}

impl EventStore {
    fn new() -> Self {
        EventStore { events: Vec::new() }
    }

    fn append(&mut self, event: UserEvent) {
        println!("Storing event: {:?}", event);
        self.events.push(event);
    }

    fn get_events(&self) -> &[UserEvent] {
        &self.events
    }

    fn get_events_for_user(&self, user_id: u64) -> Vec<&UserEvent> {
        self.events
            .iter()
            .filter(|e| match e {
                UserEvent::UserRegistered { user_id: id, .. }
                | UserEvent::EmailVerified { user_id: id, .. }
                | UserEvent::EmailChanged { user_id: id, .. }
                | UserEvent::PasswordChanged { user_id: id, .. }
                | UserEvent::AccountLocked { user_id: id, .. }
                | UserEvent::AccountUnlocked { user_id: id, .. } => *id == user_id,
            })
            .collect()
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn main() {
    println!("=== Event Sourcing with Enums ===\n");

    let mut store = EventStore::new();

    // Simulate a sequence of events
    println!("Recording events...\n");

    store.append(UserEvent::UserRegistered {
        user_id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        timestamp: now(),
    });

    store.append(UserEvent::EmailVerified {
        user_id: 1,
        timestamp: now(),
    });

    store.append(UserEvent::EmailChanged {
        user_id: 1,
        old_email: "alice@example.com".to_string(),
        new_email: "alice@newdomain.com".to_string(),
        timestamp: now(),
    });

    store.append(UserEvent::PasswordChanged {
        user_id: 1,
        timestamp: now(),
    });

    store.append(UserEvent::AccountLocked {
        user_id: 1,
        reason: "Suspicious activity".to_string(),
        timestamp: now(),
    });

    store.append(UserEvent::AccountUnlocked {
        user_id: 1,
        timestamp: now(),
    });

    // Rebuild state from events
    println!("\n=== Rebuilding User State ===");
    let user = User::from_events(store.get_events());
    println!("{:#?}", user);

    println!("\n=== Event Sourcing Benefits ===");
    println!("1. Complete audit trail - every change is recorded");
    println!("2. Time travel - rebuild state at any point in time");
    println!("3. Type-safe events - compiler ensures all events handled");
    println!("4. Decoupled - events can be consumed by multiple systems");
    println!("5. Testable - replay events to verify behavior");

    println!("\n=== Pattern Matching in Event Sourcing ===");
    println!("- Each event variant captures relevant data");
    println!("- apply() MUST handle every event type (exhaustive)");
    println!("- Adding new events forces update to all handlers");
    println!("- Or-patterns help filter events by user_id");
}
