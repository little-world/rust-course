//! Pattern 2: Typestate Pattern
//! Example: Typestate for a Connection
//!
//! Run with: cargo run --example p2_typestate_connection

use std::marker::PhantomData;

// State marker types are zero-sized structs.
#[derive(Debug)]
struct Disconnected;
#[derive(Debug)]
struct Connected;

// The Connection is generic over its state.
struct Connection<State> {
    address: Option<String>,
    data_sent: usize,
    _state: PhantomData<State>,
}

// In the `Disconnected` state, we can only connect.
impl Connection<Disconnected> {
    fn new() -> Self {
        Connection {
            address: None,
            data_sent: 0,
            _state: PhantomData,
        }
    }

    fn connect(self, addr: &str) -> Connection<Connected> {
        println!("Connecting to {}...", addr);
        Connection {
            address: Some(addr.to_string()),
            data_sent: 0,
            _state: PhantomData,
        }
    }
}

// In the `Connected` state, we can send data and disconnect.
impl Connection<Connected> {
    fn send(&mut self, data: &[u8]) {
        let addr = self.address.as_ref().unwrap();
        println!("Sending {} bytes to {}", data.len(), addr);
        self.data_sent += data.len();
    }

    fn bytes_sent(&self) -> usize {
        self.data_sent
    }

    fn disconnect(self) -> Connection<Disconnected> {
        let addr = self.address.as_ref().unwrap();
        println!("Disconnecting from {} (sent {} bytes total)", addr, self.data_sent);
        Connection {
            address: None,
            data_sent: 0,
            _state: PhantomData,
        }
    }
}

fn main() {
    println!("=== Typestate Pattern: Connection ===");
    // Usage: send() only exists on Connected state; compile error if called on Disconnected.

    // Start in Disconnected state
    let conn = Connection::<Disconnected>::new();
    println!("Created new connection (Disconnected state)");

    // This would NOT compile - send() doesn't exist on Disconnected:
    // conn.send(b"hello"); // ERROR: no method named `send`

    // Transition to Connected state
    let mut connected = conn.connect("192.168.1.100:8080");
    println!("Now in Connected state");

    // Now we can send data
    connected.send(b"Hello, ");
    connected.send(b"World!");
    println!("Total bytes sent: {}", connected.bytes_sent());

    // Transition back to Disconnected
    let disconnected = connected.disconnect();
    println!("Back to Disconnected state");

    // This would NOT compile - send() doesn't exist on Disconnected:
    // disconnected.send(b"test"); // ERROR: no method named `send`

    // But we can connect again
    let mut reconnected = disconnected.connect("192.168.1.200:9090");
    reconnected.send(b"New connection data");
    let _ = reconnected.disconnect();

    println!("\n=== Why Typestate Matters ===");
    println!("- Invalid operations are compile-time errors, not runtime panics");
    println!("- State transitions are explicit and documented in the type system");
    println!("- No runtime state checking needed");
    println!("- The compiler enforces correct usage patterns");

    println!("\n=== State Transition Diagram ===");
    println!("  Disconnected --connect()--> Connected");
    println!("  Connected --disconnect()--> Disconnected");
    println!("  Connected --send()--> Connected (self-loop)");
}
