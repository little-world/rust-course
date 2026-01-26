//! Pattern 5: Room-based WebSocket Chat
//!
//! Demonstrates a more sophisticated WebSocket server with rooms/channels.
//! Users can join specific rooms and only receive messages from those rooms.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use serde::{Deserialize, Serialize};

type RoomId = String;
type UserId = String;

#[derive(Clone, Debug, Serialize)]
struct ChatMessage {
    room: String,
    user: String,
    content: String,
}

struct Room {
    // Broadcast channel for this room
    tx: broadcast::Sender<ChatMessage>,
    // Connected users
    users: HashMap<UserId, String>, // user_id -> username
}

struct ChatServer {
    rooms: Arc<RwLock<HashMap<RoomId, Room>>>,
}

impl ChatServer {
    fn new() -> Self {
        ChatServer {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn join_room(&self, room_id: &str, user_id: &str, username: &str) -> broadcast::Receiver<ChatMessage> {
        let mut rooms = self.rooms.write().await;

        let room = rooms.entry(room_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            println!("Created room: {}", room_id);
            Room {
                tx,
                users: HashMap::new(),
            }
        });

        room.users.insert(user_id.to_string(), username.to_string());
        println!("User '{}' joined room '{}'", username, room_id);

        // Send join notification
        let _ = room.tx.send(ChatMessage {
            room: room_id.to_string(),
            user: "System".to_string(),
            content: format!("{} joined the room", username),
        });

        room.tx.subscribe()
    }

    async fn send_message(&self, room_id: &str, user_id: &str, content: &str) -> Result<(), String> {
        let rooms = self.rooms.read().await;

        if let Some(room) = rooms.get(room_id) {
            let username = room.users.get(user_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown");

            let msg = ChatMessage {
                room: room_id.to_string(),
                user: username.to_string(),
                content: content.to_string(),
            };

            room.tx.send(msg).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err(format!("Room '{}' not found", room_id))
        }
    }

    async fn leave_room(&self, room_id: &str, user_id: &str) {
        let mut rooms = self.rooms.write().await;

        if let Some(room) = rooms.get_mut(room_id) {
            if let Some(username) = room.users.remove(user_id) {
                println!("User '{}' left room '{}'", username, room_id);

                // Send leave notification
                let _ = room.tx.send(ChatMessage {
                    room: room_id.to_string(),
                    user: "System".to_string(),
                    content: format!("{} left the room", username),
                });

                // Clean up empty rooms
                if room.users.is_empty() {
                    rooms.remove(room_id);
                    println!("Room '{}' closed (empty)", room_id);
                }
            }
        }
    }

    async fn list_rooms(&self) -> Vec<(String, usize)> {
        let rooms = self.rooms.read().await;
        rooms.iter()
            .map(|(id, room)| (id.clone(), room.users.len()))
            .collect()
    }
}

#[tokio::main]
async fn main() {
    println!("=== Pattern 5: Room-based WebSocket Chat ===\n");

    let server = ChatServer::new();

    // Simulate some chat activity
    println!("--- Simulating Chat Activity ---\n");

    // Users join rooms
    let mut alice_rx = server.join_room("general", "user1", "Alice").await;
    let mut bob_rx = server.join_room("general", "user2", "Bob").await;
    let mut charlie_rx = server.join_room("rust", "user3", "Charlie").await;

    // Alice also joins rust room
    let mut alice_rust_rx = server.join_room("rust", "user1", "Alice").await;

    // List rooms
    println!("\nCurrent rooms:");
    for (room, users) in server.list_rooms().await {
        println!("  {} ({} users)", room, users);
    }
    println!();

    // Send some messages
    server.send_message("general", "user1", "Hello everyone!").await.unwrap();
    server.send_message("general", "user2", "Hey Alice!").await.unwrap();
    server.send_message("rust", "user3", "Anyone want to discuss async/await?").await.unwrap();
    server.send_message("rust", "user1", "Yes! I love Tokio!").await.unwrap();

    // Receive messages (drain the channels)
    println!("--- Messages received by Alice in 'general' ---");
    while let Ok(msg) = alice_rx.try_recv() {
        println!("  [{}] {}: {}", msg.room, msg.user, msg.content);
    }

    println!("\n--- Messages received by Bob in 'general' ---");
    while let Ok(msg) = bob_rx.try_recv() {
        println!("  [{}] {}: {}", msg.room, msg.user, msg.content);
    }

    println!("\n--- Messages received by Charlie in 'rust' ---");
    while let Ok(msg) = charlie_rx.try_recv() {
        println!("  [{}] {}: {}", msg.room, msg.user, msg.content);
    }

    println!("\n--- Messages received by Alice in 'rust' ---");
    while let Ok(msg) = alice_rust_rx.try_recv() {
        println!("  [{}] {}: {}", msg.room, msg.user, msg.content);
    }

    // Users leave
    println!("\n--- Users leaving ---");
    server.leave_room("general", "user2").await;
    server.leave_room("rust", "user3").await;

    // Final room status
    println!("\nFinal rooms:");
    for (room, users) in server.list_rooms().await {
        println!("  {} ({} users)", room, users);
    }

    println!("\nRoom-based chat demo completed!");
    println!("\nNote: This example shows the data structures and logic.");
    println!("In a real app, you would integrate this with axum WebSocket handlers.");
}
