//! Pattern 3: Message Passing with Channels
//! The Actor Pattern
//!
//! Run with: cargo run --example p3_actor_pattern

use crossbeam::channel::{unbounded, Sender, Receiver};
use std::thread;

// Messages the actor can receive.
enum ActorMessage {
    Process(String),
    GetState(Sender<String>), // Message to request the actor's state.
    Shutdown,
}

// The actor itself.
struct Actor {
    inbox: Receiver<ActorMessage>,
    state: String,
}

impl Actor {
    fn new(inbox: Receiver<ActorMessage>) -> Self {
        Self { inbox, state: String::new() }
    }

    // The actor's main loop.
    fn run(mut self) {
        while let Ok(msg) = self.inbox.recv() {
            match msg {
                ActorMessage::Process(data) => {
                    self.state.push_str(&data);
                    println!("Actor state updated.");
                }
                ActorMessage::GetState(reply_to) => {
                    reply_to.send(self.state.clone()).unwrap();
                }
                ActorMessage::Shutdown => {
                    println!("Actor shutting down.");
                    break;
                }
            }
        }
    }
}

fn actor_pattern_example() {
    let (tx, rx) = unbounded();
    let actor_handle = thread::spawn(move || Actor::new(rx).run());

    // Send messages to the actor.
    tx.send(ActorMessage::Process("Hello, ".to_string())).unwrap();
    tx.send(ActorMessage::Process("Actor!".to_string())).unwrap();

    // Send a message to get the actor's state.
    let (reply_tx, reply_rx) = unbounded();
    tx.send(ActorMessage::GetState(reply_tx)).unwrap();
    let state = reply_rx.recv().unwrap();
    println!("Retrieved actor state: '{}'", state);

    // Shut down the actor.
    tx.send(ActorMessage::Shutdown).unwrap();
    actor_handle.join().unwrap();
}

fn main() {
    println!("=== The Actor Pattern ===\n");
    actor_pattern_example();

    println!("\n=== Key Points ===");
    println!("1. Actors communicate only through messages");
    println!("2. State is encapsulated within the actor");
    println!("3. Request-reply pattern using a reply channel");
    println!("4. Clean shutdown via Shutdown message");
}
