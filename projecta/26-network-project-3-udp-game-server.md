# Chapter 26: Network Programming

## Project 3: UDP Game Server with Reliable Messaging

### Problem Statement

Build a real-time multiplayer game server that evolves from a simple UDP echo to a production-ready system with service discovery, reliable message delivery, and hybrid protocols. You'll start with basic UDP datagrams, add continuous position broadcasting for real-time gameplay, implement broadcast-based service discovery, layer reliable messaging with acknowledgments for critical events, add retransmission with timeout for lost packets, and finish with a hybrid protocol that combines unreliable fast updates with reliable critical messages.

### Why It Matters

**Real-World Impact**: UDP-based game servers power the most popular multiplayer games:
- **Fortnite**: 100 players per match, position updates 30-60 times/sec via UDP
- **Call of Duty**: 64-player matches, weapon fire/movement over UDP with <50ms latency
- **Minecraft**: Multiplayer servers handle 1000+ players, chunk updates via UDP
- **Rocket League**: Physics simulation synced at 120Hz, UDP for position/rotation
- **PUBG**: 100 players, vehicles, projectiles—all UDP for minimal latency

**Performance Numbers**:
- **TCP latency**: 50-100ms (handshake + acknowledgments + retransmission delays)
- **UDP latency**: 10-30ms (direct send, no handshake, no forced retransmission)
- **Packet rate**: 30-120 updates/sec per player (TCP can't sustain this without head-of-line blocking)
- **Bandwidth**: Position update = ~20 bytes, 60 updates/sec = 1.2KB/sec per player
- **Reliability cost**: Reliable UDP adds ~10-20ms latency (ack + potential retransmit)

**Rust-Specific Challenge**: UDP is connectionless and unreliable—packets can be lost, duplicated, or arrive out of order. Games need low latency for position updates (tolerate loss) but reliability for critical events (player joined, score changed). Rust's ownership system helps you build safe concurrent packet handling without data races. This project teaches you to design custom protocols that layer reliability on top of UDP when needed, handle packet loss gracefully, and use broadcast/multicast for discovery.

### Use Cases

**When you need this pattern**:
1. **Real-time multiplayer games** - FPS, racing, sports games (low latency critical)
2. **MMO servers** - World of Warcraft, EVE Online (position sync for thousands of entities)
3. **Physics simulation sync** - VR applications, robotics control (high-frequency updates)
4. **Voice chat in games** - Team communication (audio packets, loss acceptable)
5. **Live sports data** - Real-time scores, player positions (ESPN, NFL apps)
6. **IoT sensor networks** - Temperature, motion sensors (frequent updates, some loss OK)
7. **Stock market data feeds** - Price updates, order book changes (low latency critical)

**Real Examples**:
- **Valve Source Engine**: Uses UDP for movement/shooting, custom reliability layer for critical events
- **Unity Netcode**: MLAPI uses UDP with selective reliability (position = unreliable, RPC = reliable)
- **Photon Engine**: Real-time multiplayer framework, UDP with reliability options
- **QUIC protocol**: Google's UDP-based transport (replaces TCP), powers HTTP/3

### Learning Goals

- Master UDP socket programming (send_to/recv_from, connectionless model)
- Understand when to use UDP vs TCP (latency vs reliability trade-offs)
- Implement broadcast/multicast for service discovery
- Build reliable messaging on top of unreliable transport (sequence numbers, acks)
- Design retransmission algorithms (timeouts, exponential backoff)
- Create hybrid protocols (mix reliable and unreliable channels)
- Handle packet loss and out-of-order delivery gracefully

---

### Core Concepts

Before building a UDP game server, let's understand the fundamental concepts that make UDP ideal for real-time games:

#### 1. UDP vs TCP: The Fundamental Trade-off

**TCP (Transmission Control Protocol)**:
- **Connection-oriented**: Requires handshake before data transfer
- **Reliable**: Guarantees delivery, in-order, no duplicates
- **Automatic retransmission**: Lost packets are automatically resent
- **Flow control**: Slows down if network is congested
- **Head-of-line blocking**: One lost packet blocks all subsequent packets

**UDP (User Datagram Protocol)**:
- **Connectionless**: No handshake, just send packets
- **Unreliable**: Packets may be lost, duplicated, or arrive out-of-order
- **No automatic retransmission**: Application decides what to retransmit
- **No flow control**: Send as fast as you want
- **No head-of-line blocking**: Each packet is independent

**The Performance Difference**:
```
TCP latency for a single message:
  SYN → SYN-ACK → ACK (handshake) = 50ms
  Data → ACK = 25ms
  Total: 75ms minimum

UDP latency for a single message:
  Data = 10-20ms
  Total: 10-20ms (3-7x faster!)
```

**When to Use UDP**:
- **Real-time games**: Position updates arrive 30-120 times/sec, old data is useless
- **Voice/video chat**: Drop old audio frames, don't wait for retransmission
- **Physics simulation**: Latest state matters more than perfect history
- **High-frequency sensors**: Temperature, GPS updates

**When to Use TCP**:
- **File transfer**: Every byte must arrive correctly
- **Chat messages**: Can't lose "You won!" message
- **Database queries**: Results must be complete and correct
- **Web pages**: HTML must be perfect

#### 2. Connectionless Communication

**TCP Connection Model**:
```rust
// Server
let listener = TcpListener::bind("0.0.0.0:8080")?;
let (stream, addr) = listener.accept()?;  // Wait for client connection
stream.write(b"Hello")?;  // Send to this specific client
```

**UDP Connectionless Model**:
```rust
// Server
let socket = UdpSocket::bind("0.0.0.0:8080")?;
let (len, addr) = socket.recv_from(&mut buf)?;  // Receive from anyone
socket.send_to(b"Hello", addr)?;  // Send to whoever just sent
```

**Key Differences**:
- **No "connection"**: UDP doesn't maintain state between server and client
- **No accept()**: Any client can send to server anytime
- **No streams**: Each datagram is independent (no continuous byte stream)
- **Address on every send**: Must specify destination for each packet

**Implications for Games**:
```rust
// UDP game server handles 100 players without 100 connections
let mut players: HashMap<SocketAddr, PlayerState> = HashMap::new();

loop {
    let (len, player_addr) = socket.recv_from(&mut buf).await?;

    // First packet from new player? Add them
    players.entry(player_addr).or_insert(PlayerState::new());

    // Process packet
    update_player_state(player_addr, &buf[..len]);

    // Broadcast to all players
    for addr in players.keys() {
        socket.send_to(&game_state, addr).await?;
    }
}
```

#### 3. Packet Loss and Unreliability

**UDP's "Guarantees"** (or lack thereof):
- **May be lost**: Network congestion, router overflow → packet dropped
- **May be duplicated**: Network glitch → packet arrives twice
- **May arrive out-of-order**: Different routes → packet B before packet A
- **No notification**: You don't know if packet was delivered

**Real-World Loss Rates**:
- **Good network**: 0.1-1% packet loss
- **WiFi**: 1-5% packet loss
- **Mobile/LTE**: 2-10% packet loss
- **Poor conditions**: 10-20% packet loss

**Designing for Packet Loss**:

**Strategy 1: Accept Loss (Position Updates)**:
```rust
// Send position 30 times per second
loop {
    send_position(player.x, player.y, player.z);
    sleep(33ms);  // 30 Hz
}

// If one packet is lost, next one arrives in 33ms
// Old position data is worthless anyway
```

**Strategy 2: Add Reliability (Critical Events)**:
```rust
// Player joined - MUST be delivered
send_reliable(Message::PlayerJoined { name: "Alice" });

// Implementation:
// 1. Assign sequence number
// 2. Wait for ACK
// 3. Retransmit if no ACK after timeout
```

**Out-of-Order Example**:
```
Send: seq=1 (player moved), seq=2 (player jumped), seq=3 (player fired)
Arrive: seq=1, seq=3, seq=2 (jumped arrives last!)

Solution: Use sequence numbers to reorder or discard stale data
```

#### 4. Broadcast and Multicast

**Why Service Discovery?**
Players want to join games on their local network without typing IP addresses. "Find servers on LAN" button = broadcast/multicast.

**Broadcast (255.255.255.255)**:
Sends packet to all devices on the local network.

```rust
let socket = UdpSocket::bind("0.0.0.0:0").await?;

// Enable broadcast permission
socket.set_broadcast(true)?;

// Send to all devices on LAN
socket.send_to(b"DISCOVER_SERVER", "255.255.255.255:8080").await?;

// All devices on network receive the packet
```

**How It Works**:
```
Your Computer (192.168.1.5) → Broadcast (255.255.255.255:8080)
                                       ↓
              ┌─────────────────────────┼─────────────────────────┐
              ↓                         ↓                         ↓
    Game Server (192.168.1.10)    Laptop (192.168.1.20)    Phone (192.168.1.30)
         Responds!                    Ignores               Ignores
```

**Multicast (224.0.0.0 - 239.255.255.255)**:
Sends packet only to devices that have "subscribed" to a multicast group.

```rust
use std::net::Ipv4Addr;

// Server joins multicast group 224.0.0.1
let socket = UdpSocket::bind("0.0.0.0:8080").await?;
let multicast_addr: Ipv4Addr = "224.0.0.1".parse()?;
socket.join_multicast_v4(multicast_addr, Ipv4Addr::new(0,0,0,0))?;

// Client sends to multicast group
let client = UdpSocket::bind("0.0.0.0:0").await?;
client.send_to(b"DISCOVER", (multicast_addr, 8080)).await?;

// Only servers that joined 224.0.0.1 receive it
```

**Broadcast vs Multicast**:
- **Broadcast**: Simple, works everywhere, but spams entire LAN
- **Multicast**: Efficient, but requires router support (may not work on all networks)
- **Games**: Usually use broadcast for simplicity

#### 5. Reliable Messaging on Unreliable Transport

**The Challenge**: UDP is unreliable, but some messages (player joined, score changed) MUST be delivered.

**Solution: Layer Reliability on Top of UDP**

**Sequence Numbers**:
```rust
struct ReliableMessage {
    seq: u32,        // Unique message ID
    data: Vec<u8>,   // Actual game message
}

let mut next_seq = 0;

fn send_reliable(msg: &[u8]) {
    let reliable = ReliableMessage {
        seq: next_seq,
        data: msg.to_vec(),
    };
    next_seq += 1;

    // Send packet with sequence number
    socket.send_to(&serialize(reliable), server_addr)?;
}
```

**Acknowledgments (ACKs)**:
```rust
// Receiver gets message
let msg: ReliableMessage = deserialize(&packet);

// Send ACK back to sender
let ack = Ack { seq: msg.seq };
socket.send_to(&serialize(ack), sender_addr)?;

// Mark sequence number as received (detect duplicates)
received_seqs.insert(msg.seq);
```

**Flow**:
```
Client → Server: ReliableMsg { seq: 5, data: "PlayerJoined" }
Server → Client: Ack { seq: 5 }
[Client receives ACK, removes seq=5 from pending list]

If ACK is lost:
Client → Server: ReliableMsg { seq: 5, data: "PlayerJoined" } (retransmit)
Server: "Already received seq=5, send ACK again but don't process"
Server → Client: Ack { seq: 5 }
```

#### 6. Retransmission and Timeouts

**The Problem**: What if the packet OR its ACK is lost?

**Solution: Retransmission Timeout (RTO)**

```rust
struct PendingMessage {
    seq: u32,
    msg: Vec<u8>,
    send_time: Instant,
    rto: Duration,  // How long to wait before retransmit
}

let mut pending: HashMap<u32, PendingMessage> = HashMap::new();

// Send reliable message
fn send_reliable(seq: u32, msg: Vec<u8>) {
    socket.send_to(&msg, addr)?;

    pending.insert(seq, PendingMessage {
        seq,
        msg: msg.clone(),
        send_time: Instant::now(),
        rto: Duration::from_millis(500),  // Wait 500ms for ACK
    });
}

// Check for timeouts periodically
fn check_timeouts() {
    for (seq, pending_msg) in pending.iter_mut() {
        if pending_msg.send_time.elapsed() > pending_msg.rto {
            // Timeout! Retransmit
            socket.send_to(&pending_msg.msg, addr)?;
            pending_msg.send_time = Instant::now();
        }
    }
}

// Receive ACK
fn handle_ack(seq: u32) {
    pending.remove(&seq);  // Message delivered, stop retransmitting
}
```

**Exponential Backoff**:
```rust
// Avoid overwhelming network if it's congested
fn retransmit(pending_msg: &mut PendingMessage) {
    socket.send_to(&pending_msg.msg, addr)?;

    // Double the timeout each retry
    pending_msg.rto *= 2;  // 500ms → 1000ms → 2000ms → 4000ms
    pending_msg.send_time = Instant::now();
    pending_msg.retransmit_count += 1;

    // Give up after 3 retries
    if pending_msg.retransmit_count >= 3 {
        pending.remove(&pending_msg.seq);  // Accept loss
    }
}
```

**Why Exponential Backoff?**
- **Network congestion**: Constant retransmits make congestion worse
- **TCP uses it**: Proven strategy, backs off when network is struggling
- **Gives up gracefully**: After 3 retries, accept that the other side is unreachable

#### 7. Sequence Number Management

**The Problem**: How do we track which messages were received?

**Received Set (Simple but Memory-Heavy)**:
```rust
let mut received_seqs: HashSet<u32> = HashSet::new();

fn handle_message(seq: u32, data: &[u8]) -> Option<Vec<u8>> {
    if received_seqs.contains(&seq) {
        // Duplicate! Ignore
        return None;
    }

    received_seqs.insert(seq);
    Some(data.to_vec())  // Process message
}

// Problem: HashSet grows forever (memory leak!)
```

**Sliding Window (Better)**:
```rust
let mut expected_seq: u32 = 0;
let mut out_of_order: HashMap<u32, Vec<u8>> = HashMap::new();

fn handle_message(seq: u32, data: &[u8]) -> Option<Vec<u8>> {
    if seq == expected_seq {
        // In order! Process immediately
        expected_seq += 1;

        // Check if next messages are in out_of_order buffer
        while let Some(buffered) = out_of_order.remove(&expected_seq) {
            process_message(buffered);
            expected_seq += 1;
        }

        Some(data.to_vec())
    } else if seq > expected_seq {
        // Future message, buffer it
        out_of_order.insert(seq, data.to_vec());
        None
    } else {
        // Old message (seq < expected_seq), duplicate
        None
    }
}
```

**Real-World Approach (QUIC, RakNet)**:
- Track last N received sequences in a bitmap
- Allows detecting duplicates within window
- Old sequences are assumed received or lost (don't care)

#### 8. Hybrid Protocols: Best of Both Worlds

**The Insight**: Not all messages need the same guarantees.

**Message Classification**:
```rust
enum MessageType {
    // Send 30-120 times/sec, latest is most valuable
    PositionUpdate { x: f32, y: f32, z: f32 },       // Unreliable
    VelocityUpdate { vx: f32, vy: f32 },             // Unreliable
    WeaponFired { weapon_id: u32 },                   // Unreliable

    // Send rarely, must arrive exactly once
    PlayerJoined { name: String },                    // Reliable
    PlayerLeft { player_id: u32 },                    // Reliable
    ScoreChanged { player_id: u32, score: u32 },      // Reliable
    GameStateChange { new_state: GameState },         // Reliable
}

fn get_channel(msg: &MessageType) -> Channel {
    match msg {
        MessageType::PositionUpdate { .. } => Channel::Unreliable,
        MessageType::VelocityUpdate { .. } => Channel::Unreliable,
        MessageType::WeaponFired { .. } => Channel::Unreliable,
        _ => Channel::Reliable,
    }
}
```

**Protocol Structure**:
```rust
enum Packet {
    Unreliable { data: Vec<u8> },                    // Just data
    Reliable { seq: u32, data: Vec<u8> },            // Data + sequence
    Ack { seq: u32 },                                 // Acknowledgment
}

fn send(msg: MessageType) {
    match get_channel(&msg) {
        Channel::Unreliable => {
            let packet = Packet::Unreliable { data: serialize(msg) };
            socket.send_to(&packet, addr)?;
            // Done! No tracking
        }
        Channel::Reliable => {
            let seq = next_seq();
            let packet = Packet::Reliable { seq, data: serialize(msg) };
            socket.send_to(&packet, addr)?;
            track_for_ack(seq, packet);  // Track for retransmission
        }
    }
}
```

**Bandwidth Savings**:
```
All Reliable Protocol:
  - Position update (20 bytes) + seq (4 bytes) = 24 bytes
  - ACK packet (8 bytes)
  - 30 updates/sec = (24 + 8) × 30 = 960 bytes/sec per player

Hybrid Protocol:
  - Position update (20 bytes, unreliable) = 20 bytes
  - 30 updates/sec = 20 × 30 = 600 bytes/sec per player
  - Occasional reliable events: ~10 bytes/sec
  - Total: 610 bytes/sec (36% reduction!)

100 players: All reliable = 96 KB/sec, Hybrid = 61 KB/sec
```

**Real-World Examples**:
- **Overwatch**: Position/rotation unreliable, ability usage reliable
- **Valorant**: Movement unreliable, weapon fire unreliable, hit detection reliable
- **Minecraft**: Block placement reliable, player position unreliable
- **Rocket League**: Physics unreliable at 120Hz, goals reliable

### Connection to This Project

Now let's see how all these concepts come together in our UDP game server:

**1. Progressive Understanding of UDP**

This project takes you from basic UDP to production-ready game networking:

- **Milestone 1 (UDP Echo)**: Learn connectionless communication with `send_to`/`recv_from`. No state, no connections, just packets. Understand that UDP is stateless—server doesn't "know" about clients until they send.

- **Milestone 2 (Game Loop)**: Add state management. Track players in `HashMap<SocketAddr, PlayerState>`. Implement 30 Hz broadcast loop—the heartbeat of every real-time game. Experience continuous streaming vs request-response.

- **Milestone 3 (Service Discovery)**: Implement broadcast discovery so players can find servers on LAN without typing IPs. This is how Minecraft, Age of Empires, and StarCraft work.

**2. Building Reliability Layer from Scratch**

The most valuable learning comes from implementing your own reliable protocol:

**Milestone 4 (Sequence Numbers + ACKs)**:
- Assign unique ID to each reliable message
- Receiver sends ACK back
- Track pending messages in `HashMap<u32, PendingMessage>`
- Detect duplicates with `HashSet<u32>` of received sequences

**Milestone 5 (Retransmission)**:
- Check pending messages periodically for timeouts
- Resend if no ACK received within RTO
- Exponential backoff: 500ms → 1000ms → 2000ms
- Give up after 3 retries

This is essentially building TCP's reliability guarantees yourself! You'll deeply understand why TCP is reliable and what the cost is.

**3. Hybrid Protocol Design**

Milestone 6 demonstrates the key insight that makes modern games performant:

**Not all data needs reliability**:
- **Position updates (30-120 Hz)**: Latest position matters, old data is garbage. Send unreliable.
- **Player joined (1 time)**: Must arrive exactly once. Send reliable.
- **Weapon fired (frequent)**: Latest shot matters, miss a few? Next shot arrives soon. Unreliable.
- **Score changed (rare)**: Critical game state. Reliable.

**Architecture**:
```
┌─────────────────────────────────────────┐
│           UDP Socket (Single)            │
└───────────┬─────────────────────────────┘
            │
       ┌────┴────┐
       ↓         ↓
┌──────────┐ ┌──────────────┐
│Unreliable│ │   Reliable   │
│ Channel  │ │   Channel    │
├──────────┤ ├──────────────┤
│ Position │ │ Seq Numbers  │
│ Velocity │ │ ACKs         │
│ Shooting │ │ Retransmit   │
└──────────┘ └──────────────┘
```

Both channels multiplex over the same UDP socket, differentiated by packet type flag.

**4. Real-World Game Networking Patterns**

**Game Loop (30-120 Hz)**:
```rust
loop {
    tick_start = Instant::now();

    // 1. Receive player inputs (non-blocking)
    while let Ok((data, addr)) = socket.try_recv_from(&mut buf) {
        update_player_state(addr, data);
    }

    // 2. Simulate game world
    physics_update(dt);
    collision_detection();

    // 3. Broadcast state to all players
    for player in &players {
        broadcast_state_unreliable(player);
    }

    // 4. Wait for next tick
    sleep_until(tick_start + tick_duration);
}
```

**Retransmission Loop (Background Task)**:
```rust
loop {
    sleep(100ms);  // Check every 100ms

    // Find timed-out reliable messages
    for (seq, pending) in &reliable_channel.pending_acks {
        if pending.send_time.elapsed() > pending.rto {
            retransmit(seq, pending);
        }
    }
}
```

**5. Performance Characteristics**

By the end, you'll understand the exact performance trade-offs:

| Aspect | TCP | UDP (Unreliable) | UDP (Reliable Layer) |
|--------|-----|------------------|---------------------|
| Latency | 50-100ms | 10-30ms | 30-50ms |
| Overhead | High | None | Moderate |
| Loss Handling | Automatic | Accept loss | Manual retransmit |
| Order | Guaranteed | No guarantee | Optional |
| Use Case | Chat, files | Position updates | Critical events |

**Real Numbers from This Project**:
- **100 players**, **30 Hz** position updates
- **Unreliable only**: 100 × 30 × 20 bytes = 60 KB/sec
- **All reliable**: 100 × 30 × (20 + 4 seq + 8 ACK) = 96 KB/sec
- **Hybrid**: 60 KB/sec (positions) + 2 KB/sec (events) = 62 KB/sec

**6. Why This Matters**

Every modern multiplayer game uses these exact techniques:

**Fortnite (100 players, Battle Royale)**:
- Position/rotation: Unreliable at 30 Hz
- Building placement: Reliable
- Storm circle changes: Reliable
- Player elimination: Reliable

**Rocket League (Physics-heavy, 120 Hz)**:
- Ball position: Unreliable at 120 Hz
- Car physics: Unreliable at 120 Hz
- Goals scored: Reliable
- Match state: Reliable

**Valorant (Tactical FPS)**:
- Player movement: Unreliable at 60 Hz
- Weapon fire: Unreliable (client-side prediction)
- Hit detection: Reliable (server-authoritative)
- Round start/end: Reliable

**7. Design Decisions You'll Make**

This project forces you to answer real engineering questions:

**Q: How often should I broadcast positions?**
- 10 Hz: Too laggy (100ms between updates)
- 30 Hz: Standard for most games (33ms updates)
- 60 Hz: Smooth for fast-paced games (16ms updates)
- 120 Hz: Competitive games (8ms updates, high bandwidth)

**Q: When do I give up on retransmission?**
- Too aggressive: Spam network, make congestion worse
- Too patient: Stale data arrives too late to be useful
- Sweet spot: 3 retries with exponential backoff

**Q: How big should my receive buffer be?**
- Too small: Lose packets before processing
- Too large: Memory waste
- Rule of thumb: 1024-4096 bytes (handles fragmentation)

**8. From Learning to Production**

After completing this project, you'll be able to:
- Read game networking papers (e.g., Overwatch, Valorant GDC talks) and understand them
- Evaluate networking libraries (RakNet, Photon, Mirror) and know what they're doing under the hood
- Optimize network code: "We're using 200 KB/sec per player—why? Can we make some messages unreliable?"
- Debug networking issues: "Packet loss is 5%—how does that affect our reliable message delivery time?"

**You've built your own QUIC/RakNet/game networking stack from scratch!**

This is the foundation of:
- **QUIC protocol** (HTTP/3's transport layer)
- **RakNet** (Used in Minecraft, many Unity games)
- **Photon Engine** (Unity's multiplayer framework)
- Every game engine's networking system

---

## Milestone 1: Basic UDP Echo Server

### Introduction

**Starting Point**: Before building complex game logic, we need to understand UDP fundamentals. Unlike TCP's connection-oriented model, UDP is connectionless—each datagram is independent.

**What We're Building**: A UDP server that:
- Binds to a port and listens for datagrams
- Receives messages from clients (no "connection" concept)
- Echoes each datagram back to the sender
- Handles multiple clients simultaneously (no accept loop needed)

**Key Limitation**: This is just an echo—no game state, no player tracking, no position updates. It demonstrates UDP's connectionless nature: the server doesn't "know" about clients until they send a packet.

### Key Concepts

**Structs/Types**:
- `UdpSocket` - Tokio's async UDP socket
- `SocketAddr` - Client's IP address and port
- No connection state (unlike TCP)

**Functions and Their Roles**:
```rust
async fn run_echo_server(addr: &str) -> io::Result<()>
    // Bind UdpSocket to address
    // Loop: receive datagram, echo back to sender

async fn handle_datagram(socket: &UdpSocket, data: &[u8], addr: SocketAddr)
    // Process received data
    // Send response back to addr
```

**UDP vs TCP Key Differences**:
```
TCP:
  1. listener.accept() → get TcpStream
  2. stream.read() → receive data
  3. stream.write() → send data
  4. Connection state maintained

UDP:
  1. socket.recv_from() → get (data, sender_addr)
  2. socket.send_to(data, addr) → send to specific address
  3. No connection state - each packet is independent
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn test_udp_echo() {
        // Start server
        tokio::spawn(async {
            run_echo_server("127.0.0.1:9701").await.unwrap();
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create client socket
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Send message
        client.send_to(b"Hello UDP", "127.0.0.1:9701").await.unwrap();

        // Receive echo
        let mut buf = [0u8; 1024];
        let (len, addr) = client.recv_from(&mut buf).await.unwrap();

        assert_eq!(&buf[..len], b"Hello UDP");
        assert_eq!(addr.to_string(), "127.0.0.1:9701");
    }

    #[tokio::test]
    async fn test_multiple_clients() {
        tokio::spawn(async {
            run_echo_server("127.0.0.1:9702").await.unwrap();
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create 3 clients
        let client1 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client2 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client3 = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // All send simultaneously
        client1.send_to(b"Client1", "127.0.0.1:9702").await.unwrap();
        client2.send_to(b"Client2", "127.0.0.1:9702").await.unwrap();
        client3.send_to(b"Client3", "127.0.0.1:9702").await.unwrap();

        // All receive their echos
        let mut buf1 = [0u8; 1024];
        let mut buf2 = [0u8; 1024];
        let mut buf3 = [0u8; 1024];

        let (len1, _) = client1.recv_from(&mut buf1).await.unwrap();
        let (len2, _) = client2.recv_from(&mut buf2).await.unwrap();
        let (len3, _) = client3.recv_from(&mut buf3).await.unwrap();

        assert_eq!(&buf1[..len1], b"Client1");
        assert_eq!(&buf2[..len2], b"Client2");
        assert_eq!(&buf3[..len3], b"Client3");
    }

    #[tokio::test]
    async fn test_large_datagram() {
        tokio::spawn(async {
            run_echo_server("127.0.0.1:9703").await.unwrap();
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Send 1KB datagram
        let data = vec![0xAB; 1024];
        client.send_to(&data, "127.0.0.1:9703").await.unwrap();

        let mut buf = vec![0u8; 2048];
        let (len, _) = client.recv_from(&mut buf).await.unwrap();

        assert_eq!(len, 1024);
        assert_eq!(&buf[..len], &data[..]);
    }

    #[tokio::test]
    async fn test_packet_size_limit() {
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // UDP datagram max is ~65507 bytes
        // Trying to send more should fail or truncate
        let data = vec![0xFF; 70000];
        let result = client.send_to(&data, "127.0.0.1:9704").await;

        // Should either error or truncate
        assert!(result.is_err() || result.unwrap() < 70000);
    }
}
```

### Starter Code

```rust
use tokio::net::UdpSocket;
use std::io;

#[tokio::main]
async fn main() {
    if let Err(e) = run_echo_server("127.0.0.1:8080").await {
        eprintln!("Server error: {}", e);
    }
}

async fn run_echo_server(addr: &str) -> io::Result<()> {
    // TODO: Bind UDP socket to address
    let socket = todo!(); // UdpSocket::bind(addr).await?

    println!("UDP echo server listening on {}", addr);

    // Buffer for receiving datagrams
    let mut buf = vec![0u8; 1024];

    loop {
        // TODO: Receive datagram from any client
        // Returns (bytes_received, sender_address)
        let (len, addr) = todo!(); // socket.recv_from(&mut buf).await?

        println!("Received {} bytes from {}", len, addr);

        // TODO: Echo the datagram back to sender
        // socket.send_to(&buf[..len], addr).await?
        todo!();
    }
}
```

### Check Your Understanding

- **What's the difference between `recv_from` and TCP's `read`?** `recv_from` returns sender's address with data; TCP stream already knows peer.
- **Why no "accept" loop like TCP?** UDP is connectionless—no connection to accept, just receive from anyone.
- **Can multiple clients use the same server socket?** Yes! UDP is stateless; server receives from all clients on one socket.
- **What's the max UDP datagram size?** ~65,507 bytes (65,535 - IP header - UDP header).
- **What happens if client sends while server isn't listening?** Packet is lost (no buffering like TCP's accept queue).

---

## Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation: No Game State**
- Echo server has no concept of players or game world
- No position tracking, no continuous updates
- Can't broadcast player positions to all clients
- Not actually a game server yet

**What We're Adding**:
- **Player state**: Track position, rotation for each connected client
- **Continuous broadcasting**: Send position updates to all players at 30 Hz
- **Game loop**: Server tick that updates and broadcasts state
- **Player join/leave**: Detect new players, clean up disconnected ones

**Improvement**:
- **Functionality**: Echo → real-time game state sync
- **Update rate**: On-demand → 30 updates/sec (typical game rate)
- **State**: Stateless → tracks all players
- **Real-time**: Request-response → continuous stream

**Architecture**:
```
Game Loop (30 Hz tick)
  ↓
For each player:
  Update position
  Broadcast to all other players
```

---

## Milestone 2: Player Position Broadcasting (Real-Time Game Loop)

### Introduction

**The Problem**: Games need continuous position updates, not request-response.

**The Solution: Game Loop Pattern**
- Server maintains HashMap of players (keyed by SocketAddr)
- Each player has position (x, y, z) and rotation
- Every 33ms (30 Hz): broadcast all player positions to all clients
- Clients send position updates whenever they move

**Game Loop**:
```
loop {
    tick_start = now()

    // Receive player inputs
    while (now() - tick_start < 33ms) {
        if let Some((data, addr)) = socket.try_recv() {
            update_player(addr, data)
        }
    }

    // Broadcast state
    for player in players {
        broadcast_to_all(player.position)
    }

    sleep_until(tick_start + 33ms)
}
```

### Key Concepts

**Structs**:
```rust
#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy)]
struct PlayerState {
    position: Position,
    rotation: f32,
    last_seen: Instant,
}

struct GameServer {
    socket: UdpSocket,
    players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    tick_rate: u64, // Hz
}
```

**Messages** (binary protocol):
```rust
enum GameMessage {
    PlayerJoin { name: String },
    PositionUpdate { x: f32, y: f32, z: f32, rotation: f32 },
    StateSnapshot { players: Vec<(u32, Position, f32)> },
}
```

**Functions**:
```rust
impl GameServer {
    async fn run(&self)
        // Main game loop
        // Receive inputs, update state, broadcast

    async fn handle_player_input(&self, data: &[u8], addr: SocketAddr)
        // Parse message
        // Update player state

    async fn broadcast_state(&self)
        // Serialize all player positions
        // Send to each connected client

    async fn cleanup_stale_players(&self)
        // Remove players not seen in 5 seconds
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_player_join() {
        let server = GameServer::new("127.0.0.1:9801").await.unwrap();
        tokio::spawn(async move { server.run().await });

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Send join message
        let join_msg = GameMessage::PlayerJoin {
            name: "Alice".to_string(),
        };
        let data = serialize_message(&join_msg);
        client.send_to(&data, "127.0.0.1:9801").await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Server should have registered player
        // (check via server.players or receive broadcast)
    }

    #[tokio::test]
    async fn test_position_broadcast() {
        let server = GameServer::new("127.0.0.1:9802").await.unwrap();
        tokio::spawn(async move { server.run().await });

        // Two clients join
        let client1 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client2 = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Client1 joins
        let join = serialize_message(&GameMessage::PlayerJoin {
            name: "Player1".to_string(),
        });
        client1.send_to(&join, "127.0.0.1:9802").await.unwrap();

        // Client2 joins
        let join = serialize_message(&GameMessage::PlayerJoin {
            name: "Player2".to_string(),
        });
        client2.send_to(&join, "127.0.0.1:9802").await.unwrap();

        // Wait for broadcast
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Client1 should receive state with both players
        let mut buf = [0u8; 1024];
        let (len, _) = client1.recv_from(&mut buf).await.unwrap();
        let msg = deserialize_message(&buf[..len]).unwrap();

        if let GameMessage::StateSnapshot { players } = msg {
            assert_eq!(players.len(), 2);
        } else {
            panic!("Expected StateSnapshot");
        }
    }

    #[tokio::test]
    async fn test_position_update() {
        let server = GameServer::new("127.0.0.1:9803").await.unwrap();
        let players = server.players.clone();
        tokio::spawn(async move { server.run().await });

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client_addr = client.local_addr().unwrap();

        // Join
        client.send_to(
            &serialize_message(&GameMessage::PlayerJoin {
                name: "Mover".to_string(),
            }),
            "127.0.0.1:9803",
        ).await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Update position
        client.send_to(
            &serialize_message(&GameMessage::PositionUpdate {
                x: 10.0,
                y: 20.0,
                z: 30.0,
                rotation: 45.0,
            }),
            "127.0.0.1:9803",
        ).await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Check server state
        let players = players.read().await;
        let player = players.get(&client_addr).unwrap();
        assert_eq!(player.position.x, 10.0);
        assert_eq!(player.position.y, 20.0);
    }

    #[tokio::test]
    async fn test_update_rate() {
        let server = GameServer::new("127.0.0.1:9804").await.unwrap();
        tokio::spawn(async move { server.run().await });

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Join
        client.send_to(
            &serialize_message(&GameMessage::PlayerJoin {
                name: "Test".to_string(),
            }),
            "127.0.0.1:9804",
        ).await.unwrap();

        // Count broadcasts received in 1 second
        let mut count = 0;
        let start = Instant::now();

        while start.elapsed() < Duration::from_secs(1) {
            let mut buf = [0u8; 1024];
            if let Ok((len, _)) = tokio::time::timeout(
                Duration::from_millis(100),
                client.recv_from(&mut buf),
            ).await {
                if let Ok(_) = len {
                    count += 1;
                }
            }
        }

        // Should receive ~30 broadcasts (30 Hz)
        assert!(count >= 25 && count <= 35);
    }
}
```

### Starter Code

```rust
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::interval;

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy)]
struct PlayerState {
    position: Position,
    rotation: f32,
    last_seen: Instant,
}

struct GameServer {
    socket: UdpSocket,
    players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    tick_rate: u64,
}

#[derive(Debug)]
enum GameMessage {
    PlayerJoin { name: String },
    PositionUpdate { x: f32, y: f32, z: f32, rotation: f32 },
    StateSnapshot { players: Vec<(SocketAddr, Position, f32)> },
}

impl GameServer {
    async fn new(addr: &str, tick_rate: u64) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        println!("Game server listening on {}", addr);

        Ok(GameServer {
            socket,
            players: Arc::new(RwLock::new(HashMap::new())),
            tick_rate,
        })
    }

    async fn run(&self) -> io::Result<()> {
        let mut tick_interval = interval(Duration::from_millis(1000 / self.tick_rate));
        let mut buf = vec![0u8; 1024];

        loop {
            // TODO: Try to receive player inputs (non-blocking)
            loop {
                match self.socket.try_recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        self.handle_player_input(&buf[..len], addr).await;
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break; // No more packets
                    }
                    Err(e) => {
                        eprintln!("Receive error: {}", e);
                        break;
                    }
                }
            }

            // TODO: Wait for next tick
            tick_interval.tick().await;

            // TODO: Broadcast state to all players
            self.broadcast_state().await?;

            // TODO: Cleanup stale players
            self.cleanup_stale_players().await;
        }
    }

    async fn handle_player_input(&self, data: &[u8], addr: SocketAddr) {
        // TODO: Parse message
        if let Ok(msg) = deserialize_message(data) {
            match msg {
                GameMessage::PlayerJoin { name } => {
                    // TODO: Add player to players map
                    let mut players = self.players.write().await;
                    players.insert(addr, PlayerState {
                        position: Position { x: 0.0, y: 0.0, z: 0.0 },
                        rotation: 0.0,
                        last_seen: Instant::now(),
                    });
                    println!("Player {} joined from {}", name, addr);
                }
                GameMessage::PositionUpdate { x, y, z, rotation } => {
                    // TODO: Update player position
                    let mut players = self.players.write().await;
                    if let Some(player) = players.get_mut(&addr) {
                        player.position = Position { x, y, z };
                        player.rotation = rotation;
                        player.last_seen = Instant::now();
                    }
                }
                _ => {}
            }
        }
    }

    async fn broadcast_state(&self) -> io::Result<()> {
        // TODO: Get all player states
        let players = self.players.read().await;

        // TODO: Serialize state snapshot
        let player_list: Vec<(SocketAddr, Position, f32)> = players
            .iter()
            .map(|(addr, state)| (*addr, state.position, state.rotation))
            .collect();

        let msg = GameMessage::StateSnapshot {
            players: player_list,
        };
        let data = serialize_message(&msg);

        // TODO: Send to each player
        for addr in players.keys() {
            self.socket.send_to(&data, addr).await?;
        }

        Ok(())
    }

    async fn cleanup_stale_players(&self) {
        // TODO: Remove players not seen in 5 seconds
        let mut players = self.players.write().await;
        let stale_timeout = Duration::from_secs(5);

        players.retain(|addr, state| {
            let is_active = state.last_seen.elapsed() < stale_timeout;
            if !is_active {
                println!("Player {} disconnected (timeout)", addr);
            }
            is_active
        });
    }
}

// Simple serialization (in production use bincode or protobuf)
fn serialize_message(msg: &GameMessage) -> Vec<u8> {
    // TODO: Serialize to bytes (use bincode, serde_json, or custom)
    todo!()
}

fn deserialize_message(data: &[u8]) -> Result<GameMessage, String> {
    // TODO: Deserialize from bytes
    todo!()
}

#[tokio::main]
async fn main() {
    let server = GameServer::new("127.0.0.1:8080", 30).await.unwrap();
    server.run().await.unwrap();
}
```

### Check Your Understanding

- **Why 30 Hz tick rate?** Common game update rate (balance between responsiveness and bandwidth).
- **What's `try_recv_from`?** Non-blocking receive—returns immediately if no packet available.
- **Why track `last_seen`?** Detect disconnected clients (UDP has no "close" notification).
- **What happens if client doesn't receive a broadcast?** Packet is lost (UDP is unreliable), client shows stale data.
- **How much bandwidth per player?** ~20 bytes × 30 Hz = 600 bytes/sec (acceptable for games).

---

## Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Limitation: Manual Server Discovery**
- Clients must know server IP address beforehand
- Hard to find servers on local network
- No dynamic server list
- Can't auto-discover servers on LAN

**What We're Adding**:
- **Broadcast-based discovery**: Server announces itself via broadcast
- **Client discovery**: Clients send discovery request on local network
- **Server list**: Clients discover all available servers automatically
- **Multicast option**: Alternative to broadcast for discovery

**Improvement**:
- **Usability**: Manual IP entry → automatic discovery
- **LAN play**: Easy local multiplayer (no configuration)
- **Server list**: Discover all available game servers
- **Real-world**: How games like Minecraft, Age of Empires discover LAN servers

**Discovery Protocol**:
```
Client → 255.255.255.255:8080 (broadcast): "DISCOVER_SERVER"
Server → Client: "SERVER_INFO name=MyServer players=5/10"
```

---

## Milestone 3: Service Discovery (Broadcast/Multicast)

### Introduction

**The Problem**: Players can't find game servers on their local network.

**The Solution: Broadcast Discovery**
- Clients send discovery request to broadcast address (255.255.255.255)
- All servers on LAN receive the broadcast
- Servers respond with their info (name, player count, etc.)
- Client displays list of discovered servers

**Broadcast vs Multicast**:
- **Broadcast**: Reaches all hosts on local network (255.255.255.255)
- **Multicast**: Reaches only hosts subscribed to multicast group (e.g., 224.0.0.1)

### Key Concepts

**Structs**:
```rust
struct ServerInfo {
    name: String,
    address: SocketAddr,
    player_count: usize,
    max_players: usize,
}

struct DiscoveryServer {
    socket: UdpSocket,
    server_info: Arc<RwLock<ServerInfo>>,
}

struct DiscoveryClient {
    socket: UdpSocket,
}
```

**Functions**:
```rust
impl DiscoveryServer {
    async fn run(&self)
        // Listen for discovery requests
        // Respond with server info

    async fn handle_discovery_request(&self, addr: SocketAddr)
        // Send SERVER_INFO back to requester
}

impl DiscoveryClient {
    async fn discover_servers(&self, timeout: Duration) -> Vec<ServerInfo>
        // Enable broadcast on socket
        // Send DISCOVER_SERVER to broadcast address
        // Collect responses for timeout duration
        // Return list of discovered servers
}
```

**Protocol**:
- Client → Broadcast: `DISCOVER_SERVER\n`
- Server → Client: `SERVER_INFO name=MyServer players=5 max=10 port=8080\n`

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcast_enable() {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();

        // Enable broadcast
        socket.set_broadcast(true).unwrap();

        // Should be able to send to broadcast address
        let result = socket.send_to(b"test", "255.255.255.255:9999").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_server_responds_to_discovery() {
        // Start discovery server
        let server_info = ServerInfo {
            name: "TestServer".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            player_count: 3,
            max_players: 10,
        };
        let discovery = DiscoveryServer::new("127.0.0.1:9901", server_info).await.unwrap();
        tokio::spawn(async move { discovery.run().await });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Client sends discovery request
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(b"DISCOVER_SERVER", "127.0.0.1:9901").await.unwrap();

        // Should receive server info
        let mut buf = [0u8; 1024];
        let (len, _) = client.recv_from(&mut buf).await.unwrap();
        let response = String::from_utf8_lossy(&buf[..len]);

        assert!(response.contains("SERVER_INFO"));
        assert!(response.contains("TestServer"));
    }

    #[tokio::test]
    async fn test_client_discovers_server() {
        // Start server
        let server_info = ServerInfo {
            name: "DiscoverMe".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            player_count: 0,
            max_players: 16,
        };
        let discovery = DiscoveryServer::new("0.0.0.0:9902", server_info).await.unwrap();
        tokio::spawn(async move { discovery.run().await });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Client discovers
        let client = DiscoveryClient::new().await.unwrap();
        let servers = client.discover_servers(
            "127.0.0.1:9902",
            Duration::from_secs(1),
        ).await;

        assert!(servers.len() > 0);
        assert_eq!(servers[0].name, "DiscoverMe");
    }

    #[tokio::test]
    async fn test_multiple_servers_discovered() {
        // Start 3 servers on different ports
        for i in 0..3 {
            let port = 9903 + i;
            let server_info = ServerInfo {
                name: format!("Server{}", i),
                address: format!("127.0.0.1:{}", 8080 + i).parse().unwrap(),
                player_count: i as usize,
                max_players: 10,
            };
            let discovery = DiscoveryServer::new(
                &format!("0.0.0.0:{}", port),
                server_info,
            ).await.unwrap();
            tokio::spawn(async move { discovery.run().await });
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Client should discover all 3
        let client = DiscoveryClient::new().await.unwrap();

        let mut servers = Vec::new();
        for i in 0..3 {
            let port = 9903 + i;
            let discovered = client.discover_servers(
                &format!("127.0.0.1:{}", port),
                Duration::from_secs(1),
            ).await;
            servers.extend(discovered);
        }

        assert_eq!(servers.len(), 3);
    }

    #[tokio::test]
    async fn test_multicast_discovery() {
        use std::net::Ipv4Addr;

        // Server joins multicast group
        let server_socket = UdpSocket::bind("0.0.0.0:9906").await.unwrap();
        let multicast_addr: Ipv4Addr = "224.0.0.1".parse().unwrap();
        let interface = Ipv4Addr::new(0, 0, 0, 0);

        server_socket.join_multicast_v4(multicast_addr, interface).unwrap();

        // Server listens for discovery
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                if let Ok((len, addr)) = server_socket.recv_from(&mut buf).await {
                    if &buf[..len] == b"DISCOVER" {
                        server_socket.send_to(b"SERVER_HERE", addr).await.ok();
                    }
                }
            }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Client sends to multicast
        let client = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        client.send_to(b"DISCOVER", (multicast_addr, 9906)).await.unwrap();

        // Should receive response
        let mut buf = [0u8; 1024];
        let (len, _) = tokio::time::timeout(
            Duration::from_secs(1),
            client.recv_from(&mut buf),
        ).await.unwrap().unwrap();

        assert_eq!(&buf[..len], b"SERVER_HERE");
    }
}
```

### Starter Code

```rust
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
struct ServerInfo {
    name: String,
    address: SocketAddr,
    player_count: usize,
    max_players: usize,
}

struct DiscoveryServer {
    socket: UdpSocket,
    server_info: Arc<RwLock<ServerInfo>>,
}

impl DiscoveryServer {
    async fn new(listen_addr: &str, server_info: ServerInfo) -> io::Result<Self> {
        let socket = UdpSocket::bind(listen_addr).await?;

        // TODO: Enable broadcast reception
        // socket.set_broadcast(true)?;

        Ok(DiscoveryServer {
            socket,
            server_info: Arc::new(RwLock::new(server_info)),
        })
    }

    async fn run(&self) -> io::Result<()> {
        let mut buf = vec![0u8; 1024];

        loop {
            // TODO: Receive discovery requests
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            let request = String::from_utf8_lossy(&buf[..len]);

            // TODO: If it's a discovery request, respond
            if request.trim() == "DISCOVER_SERVER" {
                self.handle_discovery_request(addr).await?;
            }
        }
    }

    async fn handle_discovery_request(&self, addr: SocketAddr) -> io::Result<()> {
        // TODO: Get server info
        let info = self.server_info.read().await;

        // TODO: Format response
        let response = format!(
            "SERVER_INFO name={} players={} max={} port={}\n",
            info.name,
            info.player_count,
            info.max_players,
            info.address.port(),
        );

        // TODO: Send to requester
        // self.socket.send_to(response.as_bytes(), addr).await?;
        todo!();

        Ok(())
    }
}

struct DiscoveryClient {
    socket: UdpSocket,
}

impl DiscoveryClient {
    async fn new() -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        // TODO: Enable broadcast
        socket.set_broadcast(true)?;

        Ok(DiscoveryClient { socket })
    }

    async fn discover_servers(
        &self,
        broadcast_addr: &str,
        timeout_duration: Duration,
    ) -> Vec<ServerInfo> {
        let mut servers = Vec::new();

        // TODO: Send discovery request to broadcast address
        self.socket
            .send_to(b"DISCOVER_SERVER", broadcast_addr)
            .await
            .ok();

        // TODO: Collect responses for timeout duration
        let deadline = tokio::time::Instant::now() + timeout_duration;
        let mut buf = vec![0u8; 1024];

        while tokio::time::Instant::now() < deadline {
            let remaining = deadline - tokio::time::Instant::now();

            match timeout(remaining, self.socket.recv_from(&mut buf)).await {
                Ok(Ok((len, addr))) => {
                    // TODO: Parse server info
                    let response = String::from_utf8_lossy(&buf[..len]);
                    if let Some(server_info) = parse_server_info(&response, addr) {
                        servers.push(server_info);
                    }
                }
                _ => break,
            }
        }

        servers
    }
}

fn parse_server_info(response: &str, addr: SocketAddr) -> Option<ServerInfo> {
    // TODO: Parse "SERVER_INFO name=X players=Y max=Z port=P"
    if !response.starts_with("SERVER_INFO") {
        return None;
    }

    // Simple parsing (use regex or nom in production)
    let parts: HashMap<&str, &str> = response
        .split_whitespace()
        .skip(1) // Skip "SERVER_INFO"
        .filter_map(|part| {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() == 2 {
                Some((kv[0], kv[1]))
            } else {
                None
            }
        })
        .collect();

    Some(ServerInfo {
        name: parts.get("name")?.to_string(),
        player_count: parts.get("players")?.parse().ok()?,
        max_players: parts.get("max")?.parse().ok()?,
        address: SocketAddr::new(
            addr.ip(),
            parts.get("port")?.parse().ok()?,
        ),
    })
}
```

### Check Your Understanding

- **What is broadcast address 255.255.255.255?** Special address that sends to all hosts on local network.
- **Why enable broadcast on socket?** OS blocks broadcast by default for security; must explicitly enable.
- **What's the difference between broadcast and multicast?** Broadcast = everyone on LAN, multicast = only subscribers.
- **Why use multicast for discovery?** Reduces network traffic (only interested hosts receive).
- **What's the limitation of broadcast?** Only works on local network (doesn't cross routers).

---

## Why Milestone 3 Isn't Enough → Moving to Milestone 4

**Limitation: All Messages Are Unreliable**
- Position updates: OK to lose (next update arrives soon)
- Critical events: NOT OK to lose (player joined, score changed, game over)
- No way to ensure delivery of important messages
- No acknowledgment mechanism

**What We're Adding**:
- **Reliable message layer**: Guarantee delivery for critical events
- **Sequence numbers**: Track which messages have been sent
- **Acknowledgments**: Receiver confirms receipt
- **Separate channels**: Unreliable (positions) + Reliable (events)

**Improvement**:
- **Reliability**: All-or-nothing → selective reliability
- **Consistency**: Missing critical events → guaranteed delivery
- **Protocol design**: Learn to layer reliability on unreliable transport
- **Real-world**: How QUIC, RakNet, UNet work

**Reliable Protocol**:
```
Client → Server: MSG seq=5 type=PlayerJoined data=...
Server → Client: ACK seq=5
(if no ACK in 500ms, client retransmits)
```

---

## Milestone 4: Reliable Message Layer (Sequence Numbers + Acks)

### Introduction

**The Problem**: UDP loses packets. Critical game events (player joined, score) must be delivered.

**The Solution: Selective Reliability**
- Add sequence number to each reliable message
- Receiver sends ACK for each message
- Sender tracks unacknowledged messages
- Don't retransmit yet (Milestone 5), just track acks

**Sequence Numbers**:
```
seq=0: PlayerJoined → ACK
seq=1: ScoreUpdate → ACK
seq=2: PositionUpdate (unreliable, no seq)
seq=3: PlayerLeft → ACK
```

### Key Concepts

**Structs**:
```rust
#[derive(Debug, Clone)]
enum MessageType {
    Unreliable(GameMessage),
    Reliable { seq: u32, msg: GameMessage },
}

struct ReliableChannel {
    next_seq: u32,
    pending_acks: HashMap<u32, (Instant, GameMessage)>,
    received_seqs: HashSet<u32>,
}

struct GameServer {
    socket: UdpSocket,
    players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    reliable_channels: Arc<RwLock<HashMap<SocketAddr, ReliableChannel>>>,
}
```

**Functions**:
```rust
impl ReliableChannel {
    fn send_reliable(&mut self, msg: GameMessage) -> (u32, MessageType)
        // Assign sequence number
        // Track in pending_acks
        // Return (seq, wrapped message)

    fn handle_ack(&mut self, seq: u32)
        // Remove from pending_acks
        // Message delivered successfully

    fn handle_reliable_message(&mut self, seq: u32, msg: GameMessage) -> Option<GameMessage>
        // Check if already received (duplicate)
        // If new: add to received_seqs, return msg
        // If duplicate: return None

    fn send_ack(&self, seq: u32) -> MessageType
        // Create ACK message
}
```

**Protocol Messages**:
```rust
enum Protocol {
    ReliableMsg { seq: u32, data: Vec<u8> },
    Ack { seq: u32 },
    UnreliableMsg { data: Vec<u8> },
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_number_increment() {
        let mut channel = ReliableChannel::new();

        let (seq1, _) = channel.send_reliable(GameMessage::PlayerJoin {
            name: "Alice".to_string(),
        });
        let (seq2, _) = channel.send_reliable(GameMessage::ScoreUpdate { score: 100 });

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
    }

    #[test]
    fn test_ack_removes_pending() {
        let mut channel = ReliableChannel::new();

        let (seq, _) = channel.send_reliable(GameMessage::PlayerJoin {
            name: "Bob".to_string(),
        });

        assert_eq!(channel.pending_acks.len(), 1);

        channel.handle_ack(seq);

        assert_eq!(channel.pending_acks.len(), 0);
    }

    #[test]
    fn test_duplicate_detection() {
        let mut channel = ReliableChannel::new();

        let msg = GameMessage::PlayerJoin {
            name: "Test".to_string(),
        };

        // First receipt
        let result1 = channel.handle_reliable_message(5, msg.clone());
        assert!(result1.is_some());

        // Duplicate receipt
        let result2 = channel.handle_reliable_message(5, msg.clone());
        assert!(result2.is_none());
    }

    #[tokio::test]
    async fn test_reliable_message_acked() {
        let server = GameServer::new("127.0.0.1:9907").await.unwrap();
        tokio::spawn(async move { server.run().await });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Send reliable message
        let msg = Protocol::ReliableMsg {
            seq: 1,
            data: b"IMPORTANT".to_vec(),
        };
        client.send_to(&serialize_protocol(&msg), "127.0.0.1:9907")
            .await
            .unwrap();

        // Should receive ACK
        let mut buf = [0u8; 1024];
        let (len, _) = tokio::time::timeout(
            Duration::from_secs(1),
            client.recv_from(&mut buf),
        ).await.unwrap().unwrap();

        let response = deserialize_protocol(&buf[..len]).unwrap();
        assert!(matches!(response, Protocol::Ack { seq: 1 }));
    }

    #[tokio::test]
    async fn test_unreliable_no_ack() {
        let server = GameServer::new("127.0.0.1:9908").await.unwrap();
        tokio::spawn(async move { server.run().await });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Send unreliable message
        let msg = Protocol::UnreliableMsg {
            data: b"POSITION_UPDATE".to_vec(),
        };
        client.send_to(&serialize_protocol(&msg), "127.0.0.1:9908")
            .await
            .unwrap();

        // Should NOT receive ACK
        let mut buf = [0u8; 1024];
        let result = tokio::time::timeout(
            Duration::from_millis(500),
            client.recv_from(&mut buf),
        ).await;

        // Timeout expected (no ACK for unreliable)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_out_of_order_delivery() {
        let mut channel = ReliableChannel::new();

        // Receive seq 2 before seq 1
        let msg2 = channel.handle_reliable_message(2, GameMessage::ScoreUpdate { score: 50 });
        let msg1 = channel.handle_reliable_message(1, GameMessage::PlayerJoin {
            name: "Late".to_string(),
        });

        // Both should be accepted (no ordering requirement yet)
        assert!(msg2.is_some());
        assert!(msg1.is_some());
    }
}
```

### Starter Code

```rust
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone)]
struct ReliableChannel {
    next_seq: u32,
    pending_acks: HashMap<u32, (Instant, GameMessage)>,
    received_seqs: HashSet<u32>,
}

impl ReliableChannel {
    fn new() -> Self {
        ReliableChannel {
            next_seq: 0,
            pending_acks: HashMap::new(),
            received_seqs: HashSet::new(),
        }
    }

    fn send_reliable(&mut self, msg: GameMessage) -> (u32, MessageType) {
        // TODO: Assign sequence number
        let seq = self.next_seq;
        self.next_seq += 1;

        // TODO: Track in pending_acks
        self.pending_acks.insert(seq, (Instant::now(), msg.clone()));

        // TODO: Return sequence and wrapped message
        (seq, MessageType::Reliable { seq, msg })
    }

    fn handle_ack(&mut self, seq: u32) {
        // TODO: Remove from pending_acks
        if self.pending_acks.remove(&seq).is_some() {
            println!("ACK received for seq {}", seq);
        }
    }

    fn handle_reliable_message(&mut self, seq: u32, msg: GameMessage) -> Option<GameMessage> {
        // TODO: Check if already received
        if self.received_seqs.contains(&seq) {
            println!("Duplicate message seq {} ignored", seq);
            return None;
        }

        // TODO: Mark as received
        self.received_seqs.insert(seq);

        // TODO: Return message for processing
        Some(msg)
    }

    fn get_pending_acks(&self) -> Vec<(u32, Instant, GameMessage)> {
        self.pending_acks
            .iter()
            .map(|(seq, (time, msg))| (*seq, *time, msg.clone()))
            .collect()
    }
}

#[derive(Debug, Clone)]
enum Protocol {
    ReliableMsg { seq: u32, data: Vec<u8> },
    Ack { seq: u32 },
    UnreliableMsg { data: Vec<u8> },
}

impl GameServer {
    async fn handle_packet(&self, data: &[u8], addr: SocketAddr) -> io::Result<()> {
        // TODO: Deserialize protocol message
        let protocol = deserialize_protocol(data)?;

        match protocol {
            Protocol::ReliableMsg { seq, data } => {
                // TODO: Get or create reliable channel for this client
                let mut channels = self.reliable_channels.write().await;
                let channel = channels.entry(addr).or_insert_with(ReliableChannel::new);

                // TODO: Handle reliable message
                let game_msg = deserialize_game_message(&data)?;
                if let Some(msg) = channel.handle_reliable_message(seq, game_msg) {
                    // Process message
                    self.process_game_message(msg, addr).await;
                }
                drop(channels);

                // TODO: Send ACK
                let ack = Protocol::Ack { seq };
                self.socket.send_to(&serialize_protocol(&ack), addr).await?;
            }
            Protocol::Ack { seq } => {
                // TODO: Handle ACK
                let mut channels = self.reliable_channels.write().await;
                if let Some(channel) = channels.get_mut(&addr) {
                    channel.handle_ack(seq);
                }
            }
            Protocol::UnreliableMsg { data } => {
                // TODO: Process unreliable message (no ACK)
                let game_msg = deserialize_game_message(&data)?;
                self.process_game_message(game_msg, addr).await;
            }
        }

        Ok(())
    }

    async fn send_reliable(&self, addr: SocketAddr, msg: GameMessage) -> io::Result<()> {
        // TODO: Get reliable channel
        let mut channels = self.reliable_channels.write().await;
        let channel = channels.entry(addr).or_insert_with(ReliableChannel::new);

        // TODO: Send with sequence number
        let (seq, wrapped) = channel.send_reliable(msg);
        drop(channels);

        // TODO: Serialize and send
        let protocol = Protocol::ReliableMsg {
            seq,
            data: serialize_game_message(&wrapped),
        };
        self.socket.send_to(&serialize_protocol(&protocol), addr).await?;

        Ok(())
    }

    async fn send_unreliable(&self, addr: SocketAddr, msg: GameMessage) -> io::Result<()> {
        // TODO: Send without sequence number
        let protocol = Protocol::UnreliableMsg {
            data: serialize_game_message(&msg),
        };
        self.socket.send_to(&serialize_protocol(&protocol), addr).await?;

        Ok(())
    }
}

fn serialize_protocol(msg: &Protocol) -> Vec<u8> {
    // TODO: Serialize (use bincode or custom binary format)
    todo!()
}

fn deserialize_protocol(data: &[u8]) -> io::Result<Protocol> {
    // TODO: Deserialize
    todo!()
}
```

### Check Your Understanding

- **What is a sequence number?** Monotonically increasing counter to uniquely identify each message.
- **Why track pending_acks?** To know which messages haven't been acknowledged yet (for retransmission).
- **How do you detect duplicates?** Keep set of received sequence numbers, check before processing.
- **Why send ACK immediately?** Inform sender that message was received (allows sender to stop tracking it).
- **What's the overhead of reliable messages?** Sequence number (4 bytes) + ACK packet (adds latency).

---

## Why Milestone 4 Isn't Enough → Moving to Milestone 5

**Limitation: No Retransmission**
- Messages acknowledged, but not retransmitted if lost
- If ACK is lost, message sits in pending_acks forever
- No timeout mechanism to detect lost packets
- Unreliable reliability (ironic!)

**What We're Adding**:
- **Retransmission timeout (RTO)**: Resend after N milliseconds without ACK
- **Timeout detection**: Check pending_acks periodically
- **Exponential backoff**: Double timeout on each retry (avoid spam)
- **Max retries**: Give up after N attempts

**Improvement**:
- **Actual reliability**: Track acks → track + retransmit
- **Packet loss handling**: Lost packet → automatic retransmit
- **Robustness**: Works on lossy networks (real internet conditions)
- **Production-ready**: Matches TCP, QUIC, RakNet behavior

**Retransmission Logic**:
```
Send MSG seq=5 at t=0ms
  ↓ (no ACK)
Timeout at t=500ms → Retransmit MSG seq=5
  ↓ (no ACK)
Timeout at t=1500ms (exponential backoff) → Retransmit MSG seq=5
  ↓ (ACK received)
Remove seq=5 from pending_acks
```

---

## Milestone 5: Retransmission with Timeout

### Introduction

**The Problem**: If a packet or its ACK is lost, the message is never delivered.

**The Solution: Retransmission Timer**
- Track send time for each pending message
- Periodically check for timeouts (every 100ms)
- Resend messages that haven't been ACKed within RTO
- Use exponential backoff to avoid network congestion

**Timeout Calculation**:
```
Initial RTO = 500ms
After 1st retransmit: RTO = 1000ms
After 2nd retransmit: RTO = 2000ms
Max 3 retries, then give up
```

### Key Concepts

**Structs**:
```rust
struct PendingMessage {
    msg: GameMessage,
    send_time: Instant,
    retransmit_count: u8,
    rto: Duration, // Retransmission timeout
}

struct ReliableChannel {
    next_seq: u32,
    pending_acks: HashMap<u32, PendingMessage>,
    received_seqs: HashSet<u32>,
    base_rto: Duration,
    max_retries: u8,
}
```

**Functions**:
```rust
impl ReliableChannel {
    fn get_timed_out_messages(&self) -> Vec<(u32, GameMessage)>
        // Find messages past their RTO
        // Return list for retransmission

    fn retransmit(&mut self, seq: u32) -> Option<(GameMessage, Duration)>
        // Increment retransmit_count
        // Double RTO (exponential backoff)
        // Update send_time
        // Return message to resend

    fn should_give_up(&self, seq: u32) -> bool
        // Check if max_retries exceeded
}

impl GameServer {
    async fn retransmit_loop(&self)
        // Background task
        // Every 100ms: check for timeouts, retransmit
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_detection() {
        let mut channel = ReliableChannel::new();
        channel.base_rto = Duration::from_millis(100);

        // Send message
        let (seq, _) = channel.send_reliable(GameMessage::PlayerJoin {
            name: "Test".to_string(),
        });

        // No timeout yet
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(channel.get_timed_out_messages().len(), 0);

        // Timeout
        std::thread::sleep(Duration::from_millis(100));
        let timed_out = channel.get_timed_out_messages();
        assert_eq!(timed_out.len(), 1);
        assert_eq!(timed_out[0].0, seq);
    }

    #[test]
    fn test_exponential_backoff() {
        let mut channel = ReliableChannel::new();
        channel.base_rto = Duration::from_millis(100);

        let (seq, _) = channel.send_reliable(GameMessage::ScoreUpdate { score: 50 });

        // First retransmit: RTO doubles
        let (_, rto1) = channel.retransmit(seq).unwrap();
        assert_eq!(rto1, Duration::from_millis(200));

        // Second retransmit: RTO doubles again
        let (_, rto2) = channel.retransmit(seq).unwrap();
        assert_eq!(rto2, Duration::from_millis(400));
    }

    #[test]
    fn test_max_retries() {
        let mut channel = ReliableChannel::new();
        channel.base_rto = Duration::from_millis(100);
        channel.max_retries = 3;

        let (seq, _) = channel.send_reliable(GameMessage::PlayerLeft { name: "Test".to_string() });

        // Retry 3 times
        for _ in 0..3 {
            assert!(!channel.should_give_up(seq));
            channel.retransmit(seq);
        }

        // After 3 retries, should give up
        assert!(channel.should_give_up(seq));
    }

    #[tokio::test]
    async fn test_automatic_retransmit() {
        // Start server with retransmission
        let server = GameServer::new("127.0.0.1:9909").await.unwrap();
        tokio::spawn({
            let server = server.clone();
            async move { server.retransmit_loop().await }
        });
        tokio::spawn({
            let server = server.clone();
            async move { server.run().await }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Server sends reliable message to client
        server.send_reliable(
            client.local_addr().unwrap(),
            GameMessage::ScoreUpdate { score: 100 },
        ).await.unwrap();

        // Client IGNORES first message (simulate packet loss)
        let mut buf = [0u8; 1024];
        client.recv_from(&mut buf).await.unwrap(); // Receive and discard

        // Server should retransmit after timeout
        tokio::time::sleep(Duration::from_millis(600)).await;

        // Client receives retransmitted message
        let (len, _) = client.recv_from(&mut buf).await.unwrap();
        let protocol = deserialize_protocol(&buf[..len]).unwrap();

        assert!(matches!(protocol, Protocol::ReliableMsg { seq: 0, .. }));
    }

    #[tokio::test]
    async fn test_ack_stops_retransmission() {
        let mut channel = ReliableChannel::new();
        channel.base_rto = Duration::from_millis(100);

        let (seq, _) = channel.send_reliable(GameMessage::PlayerJoin {
            name: "Test".to_string(),
        });

        // ACK arrives
        channel.handle_ack(seq);

        // Wait past RTO
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should NOT timeout (ACK received)
        assert_eq!(channel.get_timed_out_messages().len(), 0);
    }
}
```

### Starter Code

```rust
use std::time::{Duration, Instant};

struct PendingMessage {
    msg: GameMessage,
    send_time: Instant,
    retransmit_count: u8,
    rto: Duration,
}

struct ReliableChannel {
    next_seq: u32,
    pending_acks: HashMap<u32, PendingMessage>,
    received_seqs: HashSet<u32>,
    base_rto: Duration,
    max_retries: u8,
}

impl ReliableChannel {
    fn new() -> Self {
        ReliableChannel {
            next_seq: 0,
            pending_acks: HashMap::new(),
            received_seqs: HashSet::new(),
            base_rto: Duration::from_millis(500),
            max_retries: 3,
        }
    }

    fn send_reliable(&mut self, msg: GameMessage) -> (u32, MessageType) {
        let seq = self.next_seq;
        self.next_seq += 1;

        // TODO: Track with timeout info
        self.pending_acks.insert(seq, PendingMessage {
            msg: msg.clone(),
            send_time: Instant::now(),
            retransmit_count: 0,
            rto: self.base_rto,
        });

        (seq, MessageType::Reliable { seq, msg })
    }

    fn get_timed_out_messages(&self) -> Vec<(u32, GameMessage)> {
        // TODO: Find messages past their RTO
        self.pending_acks
            .iter()
            .filter(|(_, pending)| {
                pending.send_time.elapsed() > pending.rto
            })
            .map(|(seq, pending)| (*seq, pending.msg.clone()))
            .collect()
    }

    fn retransmit(&mut self, seq: u32) -> Option<(GameMessage, Duration)> {
        // TODO: Get pending message
        let pending = self.pending_acks.get_mut(&seq)?;

        // TODO: Increment retransmit count
        pending.retransmit_count += 1;

        // TODO: Exponential backoff
        pending.rto = pending.rto * 2;

        // TODO: Update send time
        pending.send_time = Instant::now();

        Some((pending.msg.clone(), pending.rto))
    }

    fn should_give_up(&self, seq: u32) -> bool {
        // TODO: Check if exceeded max retries
        if let Some(pending) = self.pending_acks.get(&seq) {
            pending.retransmit_count >= self.max_retries
        } else {
            false
        }
    }
}

impl GameServer {
    async fn retransmit_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;

            // TODO: Check all clients for timeouts
            let channels = self.reliable_channels.read().await;

            for (addr, channel) in channels.iter() {
                // TODO: Get timed out messages
                let timed_out = channel.get_timed_out_messages();

                for (seq, msg) in timed_out {
                    // TODO: Check if should give up
                    if channel.should_give_up(seq) {
                        println!("Giving up on seq {} to {}", seq, addr);
                        // Remove from pending (accept loss)
                        continue;
                    }

                    println!("Retransmitting seq {} to {}", seq, addr);
                    // TODO: Retransmit
                    // self.send_reliable_with_seq(*addr, seq, msg).await;
                    todo!();
                }
            }
        }
    }

    async fn send_reliable_with_seq(
        &self,
        addr: SocketAddr,
        seq: u32,
        msg: GameMessage,
    ) -> io::Result<()> {
        // TODO: Send with existing sequence number (retransmit)
        let protocol = Protocol::ReliableMsg {
            seq,
            data: serialize_game_message(&msg),
        };
        self.socket.send_to(&serialize_protocol(&protocol), addr).await?;

        // TODO: Update retransmit info in channel
        let mut channels = self.reliable_channels.write().await;
        if let Some(channel) = channels.get_mut(&addr) {
            channel.retransmit(seq);
        }

        Ok(())
    }
}
```

### Check Your Understanding

- **What is RTO?** Retransmission Timeout—how long to wait for ACK before resending.
- **Why exponential backoff?** Avoid overwhelming network; gradual backoff in case of congestion.
- **What happens after max retries?** Give up, remove from pending, accept message loss.
- **Why check timeouts every 100ms?** Balance between responsiveness and CPU overhead.
- **How does this compare to TCP?** Similar! TCP also uses RTO and exponential backoff.

---

## Why Milestone 5 Isn't Enough → Moving to Milestone 6

**Limitation: Single Protocol for Everything**
- Position updates don't need reliability (waste of bandwidth for ACKs)
- Critical events need reliability (but we're ACKing positions too)
- Mixing concerns: fast unreliable + slow reliable in same channel
- Optimal: separate channels for different needs

**What We're Adding**:
- **Hybrid protocol**: Two channels (unreliable + reliable) on same socket
- **Message type flag**: Indicates which channel to use
- **Optimized bandwidth**: Positions unreliable (no ACKs), events reliable
- **Best of both worlds**: Low latency + guaranteed delivery where needed

**Improvement**:
- **Performance**: Stop ACKing position updates (30-50% bandwidth reduction)
- **Latency**: Position updates don't wait for ACKs
- **Reliability**: Critical events still guaranteed
- **Production pattern**: Exactly how modern games work (Overwatch, Valorant)

**Channel Selection**:
```
Position update → Unreliable channel (no seq, no ACK)
Player joined → Reliable channel (seq + ACK + retransmit)
Weapon fired → Unreliable (fast)
Score changed → Reliable (important)
```

---

## Milestone 6: Hybrid Protocol (Unreliable + Reliable Channels)

### Introduction

**The Problem**: Not all messages need the same reliability guarantees.

**The Solution: Channel-Based Design**
- Unreliable channel: Fast position/state updates (no ACKs)
- Reliable channel: Critical events (seq + ACK + retransmit)
- Application decides per-message which channel to use
- Both channels multiplex over same UDP socket

**Message Classification**:
```rust
match msg {
    GameMessage::PositionUpdate { .. } => send_unreliable(),
    GameMessage::WeaponFired { .. } => send_unreliable(),
    GameMessage::PlayerJoined { .. } => send_reliable(),
    GameMessage::ScoreUpdate { .. } => send_reliable(),
    GameMessage::GameOver { .. } => send_reliable(),
}
```

### Key Concepts

**Structs**:
```rust
enum Channel {
    Unreliable,
    Reliable,
}

struct GameServer {
    socket: UdpSocket,
    players: Arc<RwLock<HashMap<SocketAddr, PlayerState>>>,
    reliable_channels: Arc<RwLock<HashMap<SocketAddr, ReliableChannel>>>,
}

impl GameServer {
    async fn send(&self, addr: SocketAddr, msg: GameMessage, channel: Channel)
        // Route to appropriate channel based on type
}
```

**Message Type Selection**:
```rust
impl GameMessage {
    fn channel(&self) -> Channel {
        match self {
            GameMessage::PositionUpdate { .. } => Channel::Unreliable,
            GameMessage::PlayerJoined { .. } => Channel::Reliable,
            // ...
        }
    }
}
```

**Performance Comparison**:
```
All Reliable:
  30 position updates/sec × (20 bytes + 4 seq + 10 ACK) = 1020 bytes/sec

Hybrid:
  30 position updates/sec × 20 bytes = 600 bytes/sec
  1 critical event × (20 + 4 + 10) = 34 bytes/sec
  Total = 634 bytes/sec (38% reduction)
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_channel_selection() {
        let pos_update = GameMessage::PositionUpdate {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            rotation: 0.0,
        };
        assert_eq!(pos_update.channel(), Channel::Unreliable);

        let join = GameMessage::PlayerJoined {
            name: "Alice".to_string(),
        };
        assert_eq!(join.channel(), Channel::Reliable);
    }

    #[tokio::test]
    async fn test_hybrid_server() {
        let server = GameServer::new("127.0.0.1:9910").await.unwrap();
        tokio::spawn(async move { server.run().await });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Send unreliable position update
        server.send(
            client.local_addr().unwrap(),
            GameMessage::PositionUpdate {
                x: 10.0,
                y: 20.0,
                z: 30.0,
                rotation: 45.0,
            },
            Channel::Unreliable,
        ).await.unwrap();

        // Client receives (no ACK expected)
        let mut buf = [0u8; 1024];
        let (len, _) = client.recv_from(&mut buf).await.unwrap();
        let protocol = deserialize_protocol(&buf[..len]).unwrap();

        assert!(matches!(protocol, Protocol::UnreliableMsg { .. }));
    }

    #[tokio::test]
    async fn test_reliable_channel_gets_ack() {
        let server = GameServer::new("127.0.0.1:9911").await.unwrap();
        tokio::spawn(async move { server.run().await });

        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Send reliable message to server
        let msg = Protocol::ReliableMsg {
            seq: 0,
            data: serialize_game_message(&GameMessage::PlayerJoined {
                name: "Test".to_string(),
            }),
        };
        client.send_to(&serialize_protocol(&msg), "127.0.0.1:9911")
            .await
            .unwrap();

        // Should receive ACK
        let mut buf = [0u8; 1024];
        let (len, _) = client.recv_from(&mut buf).await.unwrap();
        let response = deserialize_protocol(&buf[..len]).unwrap();

        assert!(matches!(response, Protocol::Ack { seq: 0 }));
    }

    #[tokio::test]
    async fn test_bandwidth_comparison() {
        // Measure bytes sent with all-reliable vs hybrid

        let all_reliable_bytes = simulate_all_reliable(30).await;
        let hybrid_bytes = simulate_hybrid(30, 1).await;

        println!("All reliable: {} bytes/sec", all_reliable_bytes);
        println!("Hybrid: {} bytes/sec", hybrid_bytes);

        // Hybrid should use less bandwidth
        assert!(hybrid_bytes < all_reliable_bytes);
    }

    async fn simulate_all_reliable(position_updates_per_sec: usize) -> usize {
        // Each position update: 20 bytes data + 4 seq + ~10 ACK overhead
        position_updates_per_sec * (20 + 4 + 10)
    }

    async fn simulate_hybrid(
        position_updates_per_sec: usize,
        critical_events_per_sec: usize,
    ) -> usize {
        // Positions: unreliable (no seq, no ACK)
        let position_bytes = position_updates_per_sec * 20;

        // Critical events: reliable
        let event_bytes = critical_events_per_sec * (20 + 4 + 10);

        position_bytes + event_bytes
    }

    #[tokio::test]
    async fn test_full_game_scenario() {
        let server = GameServer::new("127.0.0.1:9912").await.unwrap();
        tokio::spawn({
            let server = server.clone();
            async move { server.run().await }
        });
        tokio::spawn({
            let server = server.clone();
            async move { server.retransmit_loop().await }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Two clients join
        let client1 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client2 = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Client1 joins (reliable)
        let join = GameMessage::PlayerJoined {
            name: "Player1".to_string(),
        };
        server.send(
            client1.local_addr().unwrap(),
            join,
            Channel::Reliable,
        ).await.unwrap();

        // Client1 sends position updates (unreliable)
        for i in 0..10 {
            let pos = GameMessage::PositionUpdate {
                x: i as f32,
                y: 0.0,
                z: 0.0,
                rotation: 0.0,
            };
            server.send(
                client1.local_addr().unwrap(),
                pos,
                Channel::Unreliable,
            ).await.unwrap();
            tokio::time::sleep(Duration::from_millis(33)).await; // 30 Hz
        }

        // Client1 scores (reliable)
        let score = GameMessage::ScoreUpdate { score: 100 };
        server.send(
            client1.local_addr().unwrap(),
            score,
            Channel::Reliable,
        ).await.unwrap();

        // Verify both channels work
        // (In real test, check received messages on client2)
    }
}
```

### Starter Code

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Channel {
    Unreliable,
    Reliable,
}

#[derive(Debug, Clone)]
enum GameMessage {
    PositionUpdate { x: f32, y: f32, z: f32, rotation: f32 },
    WeaponFired { weapon_id: u32, target_x: f32, target_y: f32 },
    PlayerJoined { name: String },
    PlayerLeft { name: String },
    ScoreUpdate { score: u32 },
    GameOver { winner: String },
}

impl GameMessage {
    fn channel(&self) -> Channel {
        // TODO: Classify messages by reliability needs
        match self {
            GameMessage::PositionUpdate { .. } => Channel::Unreliable,
            GameMessage::WeaponFired { .. } => Channel::Unreliable,
            GameMessage::PlayerJoined { .. } => Channel::Reliable,
            GameMessage::PlayerLeft { .. } => Channel::Reliable,
            GameMessage::ScoreUpdate { .. } => Channel::Reliable,
            GameMessage::GameOver { .. } => Channel::Reliable,
        }
    }
}

impl GameServer {
    async fn send(
        &self,
        addr: SocketAddr,
        msg: GameMessage,
        channel: Channel,
    ) -> io::Result<()> {
        // TODO: Route to appropriate channel
        match channel {
            Channel::Unreliable => self.send_unreliable(addr, msg).await,
            Channel::Reliable => self.send_reliable(addr, msg).await,
        }
    }

    async fn send_with_auto_channel(&self, addr: SocketAddr, msg: GameMessage) -> io::Result<()> {
        // TODO: Automatically select channel based on message type
        let channel = msg.channel();
        self.send(addr, msg, channel).await
    }

    async fn broadcast(&self, msg: GameMessage, channel: Channel) -> io::Result<()> {
        // TODO: Send to all connected players
        let players = self.players.read().await;

        for addr in players.keys() {
            self.send(*addr, msg.clone(), channel).await?;
        }

        Ok(())
    }

    async fn broadcast_except(
        &self,
        msg: GameMessage,
        channel: Channel,
        except: SocketAddr,
    ) -> io::Result<()> {
        // TODO: Broadcast to all except one player
        let players = self.players.read().await;

        for addr in players.keys() {
            if *addr != except {
                self.send(*addr, msg.clone(), channel).await?;
            }
        }

        Ok(())
    }
}

// Example game loop with hybrid channels
async fn game_loop(server: Arc<GameServer>) {
    let mut tick = tokio::time::interval(Duration::from_millis(33)); // 30 Hz

    loop {
        tick.tick().await;

        // TODO: Get all player states
        let players = server.players.read().await.clone();
        drop(players);

        // TODO: Broadcast positions (unreliable)
        for (addr, state) in players.iter() {
            let pos_msg = GameMessage::PositionUpdate {
                x: state.position.x,
                y: state.position.y,
                z: state.position.z,
                rotation: state.rotation,
            };

            server.broadcast_except(pos_msg, Channel::Unreliable, *addr)
                .await
                .ok();
        }

        // TODO: Process critical events (reliable)
        // Example: check for score milestones, game over, etc.
    }
}
```

### Check Your Understanding

- **Why use unreliable for position updates?** Frequent updates, latest data more valuable than old, ACK overhead wasteful.
- **When to use reliable channel?** Events that must be delivered exactly once (join, leave, score, game state changes).
- **How much bandwidth saved?** ~30-50% depending on ratio of reliable to unreliable messages.
- **Can both channels use same socket?** Yes! Multiplex via message type flag in protocol.
- **What's the trade-off?** Complexity (two channels to manage) vs performance (optimal bandwidth usage).

---

## Complete Working Example

Below is a simplified but functional UDP game server with reliable messaging:

```rust
// See full implementation by combining all milestones
// Key components:

#[tokio::main]
async fn main() {
    let server = Arc::new(GameServer::new("0.0.0.0:8080", 30).await.unwrap());

    // Start game loop (position broadcasting)
    tokio::spawn({
        let server = server.clone();
        async move { game_loop(server).await }
    });

    // Start retransmission loop (reliable messages)
    tokio::spawn({
        let server = server.clone();
        async move { server.retransmit_loop().await }
    });

    // Start discovery server
    let discovery = DiscoveryServer::new(
        "0.0.0.0:8081",
        ServerInfo {
            name: "MyGameServer".to_string(),
            address: "0.0.0.0:8080".parse().unwrap(),
            player_count: 0,
            max_players: 32,
        },
    ).await.unwrap();
    tokio::spawn(async move { discovery.run().await });

    // Main receive loop
    server.run().await.unwrap();
}
```

---

## Summary

**What You Built**: A production-ready UDP game server with service discovery, reliable messaging, and hybrid protocols optimized for real-time multiplayer.

**Key Concepts Mastered**:
- **UDP fundamentals**: Connectionless, send_to/recv_from, broadcast/multicast
- **Service discovery**: Broadcast-based LAN discovery (Minecraft pattern)
- **Reliable messaging**: Sequence numbers, acknowledgments, duplicate detection
- **Retransmission**: Timeouts, exponential backoff, max retries
- **Hybrid protocols**: Unreliable (fast) + Reliable (guaranteed) channels

**Performance Journey**:
- **Milestone 1**: Basic UDP echo (understand connectionless model)
- **Milestone 2**: 30 Hz game loop (real-time state sync)
- **Milestone 3**: Service discovery (easy LAN play)
- **Milestone 4**: Reliable layer (guarantee delivery)
- **Milestone 5**: Retransmission (handle packet loss)
- **Milestone 6**: Hybrid (30-50% bandwidth reduction)

**Real-World Applications**: This architecture powers Fortnite, Call of Duty, Rocket League, Minecraft, and every modern multiplayer game.
