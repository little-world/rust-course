//! Pattern 6: Phantom Types and Type-Level State
//! Example: Type-State for Protocol States
//!
//! Run with: cargo run --example p6_protocol_state

use std::marker::PhantomData;

// State marker types (zero-sized)
struct Disconnected;
struct Connected;
struct Authenticated;

// Connection with phantom type parameter for state
struct Connection<State> {
    socket: String,
    _state: PhantomData<State>,
}

// Methods only available in Disconnected state
impl Connection<Disconnected> {
    fn new() -> Self {
        println!("Creating new disconnected connection");
        Connection {
            socket: String::new(),
            _state: PhantomData,
        }
    }

    fn connect(self, addr: &str) -> Connection<Connected> {
        println!("Connecting to {}...", addr);
        Connection {
            socket: addr.to_string(),
            _state: PhantomData,
        }
    }
}

// Methods only available in Connected state
impl Connection<Connected> {
    fn authenticate(self, credentials: &str) -> Connection<Authenticated> {
        println!("Authenticating with credentials: {}...", credentials);
        Connection {
            socket: self.socket,
            _state: PhantomData,
        }
    }

    fn disconnect(self) -> Connection<Disconnected> {
        println!("Disconnecting...");
        Connection {
            socket: String::new(),
            _state: PhantomData,
        }
    }
}

// Methods only available in Authenticated state
impl Connection<Authenticated> {
    fn send(&self, data: &[u8]) {
        println!("Sending {} bytes to {}", data.len(), self.socket);
    }

    fn receive(&self) -> Vec<u8> {
        println!("Receiving data from {}", self.socket);
        vec![1, 2, 3, 4, 5] // Simulated data
    }

    fn logout(self) -> Connection<Connected> {
        println!("Logging out...");
        Connection {
            socket: self.socket,
            _state: PhantomData,
        }
    }
}

// Methods available in any state
impl<State> Connection<State> {
    fn socket_info(&self) -> &str {
        if self.socket.is_empty() {
            "not connected"
        } else {
            &self.socket
        }
    }
}

fn main() {
    println!("=== Type-State Protocol Demo ===\n");

    // Create disconnected connection
    let conn = Connection::<Disconnected>::new();
    println!("Socket info: {}", conn.socket_info());

    // connect() only available on Disconnected
    let conn = conn.connect("localhost:8080");
    println!("Socket info: {}", conn.socket_info());

    // authenticate() only available on Connected
    let mut conn = conn.authenticate("user:password");
    println!("Socket info: {}", conn.socket_info());

    // send() and receive() only available on Authenticated
    conn.send(b"Hello, server!");
    let data = conn.receive();
    println!("Received {} bytes", data.len());

    // logout() returns to Connected state
    let conn = conn.logout();
    println!("Socket info: {}", conn.socket_info());

    // disconnect() only available on Connected
    let _conn = conn.disconnect();

    println!("\n=== Compile-Time Guarantees ===");
    println!("The following would NOT compile:");
    println!("  conn_disconnected.send(...)  // No send on Disconnected");
    println!("  conn_connected.send(...)     // No send on Connected");
    println!("  conn_authenticated.connect(...) // No connect on Authenticated");
    println!("\nType state ensures the protocol is followed correctly!");

    // Demonstration of what wouldn't compile:
    // let conn = Connection::<Disconnected>::new();
    // conn.send(b"data"); // ERROR: no method named `send`
    // conn.authenticate("creds"); // ERROR: no method named `authenticate`
}
