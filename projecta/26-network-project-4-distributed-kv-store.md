# Chapter 26: Network Programming

## Project 4: Distributed Key-Value Store with Replication

### Problem Statement

Build a distributed key-value database that evolves from a simple in-memory HashMap to a production-ready replicated system with persistence, leader election, and consistency guarantees. You'll start with basic TCP commands (GET/SET/DELETE), add write-ahead logging for durability, implement async replication to followers, upgrade to synchronous quorum writes for strong consistency, add automatic leader election on failures, and finish with client-side connection pooling and smart routing.

### Why It Matters

**Real-World Impact**: Distributed key-value stores are the foundation of modern infrastructure:
- **Redis**: 10M+ deployments, powers caching for Twitter, GitHub, StackOverflow (100K+ ops/sec per instance)
- **etcd**: Kubernetes control plane storage, manages cluster state for millions of containers
- **Consul**: Service discovery and configuration for HashiCorp Vault, Netflix microservices
- **DynamoDB**: AWS's managed KV store, handles 10+ trillion requests/day
- **Riak**: Distributed database for chat systems (WhatsApp used it for 900M users)

**Performance Numbers**:
- **Single-node**: 100K reads/sec, 50K writes/sec (memory-bound)
- **Async replication**: 30K writes/sec (3x slower than no replication, but durable)
- **Quorum writes (N=3, W=2)**: 15K writes/sec (consistency cost), but survives 1 node failure
- **Read scaling**: 3 replicas = 300K reads/sec (linear scaling with replicas)
- **Failover time**: Manual = minutes, automatic leader election = 2-5 seconds

**Rust-Specific Challenge**: Distributed systems require careful handling of concurrent mutable state, network failures, and partial failures. Rust's ownership system prevents many classes of bugs (use-after-free, data races) that plague distributed systems in other languages. This project teaches you to use Arc<RwLock<T>>, async networking, and message passing to build reliable distributed systems that handle failures gracefully.

### Use Cases

**When you need this pattern**:
1. **Caching layer** - Speed up database queries, API responses (Redis pattern)
2. **Session storage** - Distributed web sessions across servers (sticky sessions without stickiness)
3. **Configuration management** - Distribute config to microservices (etcd/Consul pattern)
4. **Service discovery** - Track which services are available at which addresses
5. **Distributed locks** - Coordinate exclusive access across servers (leader election, cron job deduplication)
6. **Feature flags** - Toggle features dynamically across fleet (LaunchDarkly pattern)
7. **Metadata storage** - Store file locations in distributed filesystem (HDFS NameNode pattern)

**Real Examples**:
- **Redis Cluster**: Sharded KV store with automatic failover, 1000 nodes max
- **etcd**: Raft consensus for strong consistency, used by Kubernetes, CloudFoundry
- **Cassandra**: Eventually consistent KV store, Netflix uses it for 2.5 trillion ops/day
- **Memcached**: Simple KV cache, Facebook uses 800+ servers with 28 TB of RAM

### Learning Goals

- Master TCP client-server patterns with custom protocols
- Understand write-ahead logging (WAL) for durability
- Learn async vs sync replication trade-offs
- Practice quorum-based consistency (CAP theorem in action)
- Implement leader election (simplified Raft/Paxos)
- Build connection pooling and client-side routing
- Experience distributed systems failure modes

---

## Milestone 1: In-Memory KV Store (TCP Protocol)

### Introduction

**Starting Point**: Before building distribution and replication, we need a functional single-node key-value store. This is the foundation we'll extend.

**What We're Building**: A TCP server that:
- Stores key-value pairs in a HashMap
- Implements a simple text protocol: `GET key`, `SET key value`, `DELETE key`
- Handles multiple concurrent clients
- Returns responses: `OK`, `VALUE data`, `NOT_FOUND`

**Key Limitation**: This is an in-memory store with no persistence. If the server crashes, all data is lost. Also, it's a single point of failure—if the server goes down, the entire system is unavailable.

### Key Concepts

**Structs/Types**:
- `KvStore` - Wraps HashMap with thread-safe access
- `Command` - Enum representing GET/SET/DELETE operations
- `Response` - Enum for OK/VALUE/NOT_FOUND/ERROR

**Functions and Their Roles**:
```rust
struct KvStore {
    data: Arc<RwLock<HashMap<String, String>>>,
}

enum Command {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
}

enum Response {
    Ok,
    Value { data: String },
    NotFound,
    Error { msg: String },
}

impl KvStore {
    fn new() -> Self
        // Initialize with empty HashMap

    async fn get(&self, key: &str) -> Option<String>
        // Read lock, lookup key, return value

    async fn set(&self, key: String, value: String)
        // Write lock, insert key-value

    async fn delete(&self, key: &str) -> bool
        // Write lock, remove key, return true if existed
}

fn parse_command(line: &str) -> Result<Command, String>
    // Parse "GET key" or "SET key value" etc.

async fn handle_client(stream: TcpStream, store: Arc<KvStore>)
    // Read commands, execute, send responses
```

**Protocol**:
- Client → Server: `GET mykey\n`
- Server → Client: `VALUE myvalue\n` or `NOT_FOUND\n`
- Client → Server: `SET mykey myvalue\n`
- Server → Client: `OK\n`

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_and_get() {
        let store = KvStore::new();

        store.set("name".to_string(), "Alice".to_string()).await;
        let value = store.get("name").await;

        assert_eq!(value, Some("Alice".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let store = KvStore::new();
        let value = store.get("missing").await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_delete() {
        let store = KvStore::new();

        store.set("temp".to_string(), "value".to_string()).await;
        assert!(store.delete("temp").await);
        assert_eq!(store.get("temp").await, None);
    }

    #[tokio::test]
    async fn test_overwrite() {
        let store = KvStore::new();

        store.set("key".to_string(), "v1".to_string()).await;
        store.set("key".to_string(), "v2".to_string()).await;

        assert_eq!(store.get("key").await, Some("v2".to_string()));
    }

    #[test]
    fn test_parse_get() {
        let cmd = parse_command("GET mykey").unwrap();
        assert!(matches!(cmd, Command::Get { key } if key == "mykey"));
    }

    #[test]
    fn test_parse_set() {
        let cmd = parse_command("SET mykey myvalue").unwrap();
        assert!(matches!(cmd, Command::Set { key, value }
            if key == "mykey" && value == "myvalue"));
    }

    #[test]
    fn test_parse_set_with_spaces() {
        let cmd = parse_command("SET mykey hello world").unwrap();
        assert!(matches!(cmd, Command::Set { key, value }
            if key == "mykey" && value == "hello world"));
    }

    #[tokio::test]
    async fn test_concurrent_clients() {
        tokio::spawn(async {
            run_kv_server("127.0.0.1:9301").await.unwrap();
        });
        sleep(Duration::from_millis(100)).await;

        // Connect multiple clients
        let client1 = TcpStream::connect("127.0.0.1:9301").await.unwrap();
        let client2 = TcpStream::connect("127.0.0.1:9301").await.unwrap();

        let mut writer1 = client1;
        let mut writer2 = client2;

        // Both clients set different keys
        writer1.write_all(b"SET key1 value1\n").await.unwrap();
        writer2.write_all(b"SET key2 value2\n").await.unwrap();

        // Both should succeed
        let mut buf1 = [0u8; 1024];
        let mut buf2 = [0u8; 1024];

        let n1 = writer1.read(&mut buf1).await.unwrap();
        let n2 = writer2.read(&mut buf2).await.unwrap();

        assert!(String::from_utf8_lossy(&buf1[..n1]).contains("OK"));
        assert!(String::from_utf8_lossy(&buf2[..n2]).contains("OK"));
    }
}
```

### Starter Code

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

struct KvStore {
    data: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Debug)]
enum Command {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
}

enum Response {
    Ok,
    Value { data: String },
    NotFound,
    Error { msg: String },
}

impl KvStore {
    fn new() -> Self {
        KvStore {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get(&self, key: &str) -> Option<String> {
        // TODO: Acquire read lock and get value
        let data = todo!(); // self.data.read().await
        data.get(key).cloned()
    }

    async fn set(&self, key: String, value: String) {
        // TODO: Acquire write lock and insert
        let mut data = todo!(); // self.data.write().await
        data.insert(key, value);
    }

    async fn delete(&self, key: &str) -> bool {
        // TODO: Acquire write lock and remove
        let mut data = todo!();
        data.remove(key).is_some()
    }
}

impl Response {
    fn to_string(&self) -> String {
        match self {
            Response::Ok => "OK\n".to_string(),
            Response::Value { data } => format!("VALUE {}\n", data),
            Response::NotFound => "NOT_FOUND\n".to_string(),
            Response::Error { msg } => format!("ERROR {}\n", msg),
        }
    }
}

fn parse_command(line: &str) -> Result<Command, String> {
    let parts: Vec<&str> = line.trim().splitn(3, ' ').collect();

    match parts.as_slice() {
        ["GET", key] => Ok(Command::Get {
            key: key.to_string(),
        }),
        ["SET", key, value] => Ok(Command::Set {
            key: key.to_string(),
            value: value.to_string(),
        }),
        ["DELETE", key] => Ok(Command::Delete {
            key: key.to_string(),
        }),
        _ => Err("Invalid command".to_string()),
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_kv_server("127.0.0.1:6379").await {
        eprintln!("Server error: {}", e);
    }
}

async fn run_kv_server(addr: &str) -> tokio::io::Result<()> {
    let store = Arc::new(KvStore::new());
    let listener = TcpListener::bind(addr).await?;

    println!("KV store listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        let store = store.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, store).await {
                eprintln!("Client {} error: {}", addr, e);
            }
        });
    }
}

async fn handle_client(stream: TcpStream, store: Arc<KvStore>) -> tokio::io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        // TODO: Read command from client
        let bytes_read = todo!(); // reader.read_line(&mut line).await?

        if bytes_read == 0 {
            break; // EOF
        }

        // TODO: Parse command
        let command = match parse_command(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                // Send error response
                writer.write_all(Response::Error { msg: e }.to_string().as_bytes()).await?;
                continue;
            }
        };

        // TODO: Execute command
        let response = match command {
            Command::Get { key } => {
                // store.get(&key).await
                todo!();
            }
            Command::Set { key, value } => {
                // store.set(key, value).await
                todo!();
            }
            Command::Delete { key } => {
                // store.delete(&key).await
                todo!();
            }
        };

        // TODO: Send response
        // writer.write_all(response.to_string().as_bytes()).await?;
        todo!();
    }

    Ok(())
}
```

### Check Your Understanding

- **Why use `Arc<RwLock<HashMap>>`?** Arc for shared ownership across tasks, RwLock for concurrent read/write access.
- **What's the advantage of RwLock over Mutex?** Multiple concurrent readers (GET operations) don't block each other.
- **Why parse commands as an enum?** Type-safe representation, pattern matching for execution.
- **What happens if the server crashes?** All data is lost (no persistence yet).
- **How many concurrent readers can access the store?** Unlimited (RwLock allows multiple readers).

---

## Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation: No Durability**
- All data is in RAM only
- Server crash = total data loss
- Unacceptable for production databases
- Restart = empty database

**What We're Adding**:
- **Write-Ahead Log (WAL)**: Append-only log of all writes
- **Durability**: Writes persisted to disk before acknowledging
- **Recovery**: Replay WAL on startup to restore state
- **Crash resistance**: Can recover from power failures

**Improvement**:
- **Durability**: Volatile → persistent (survives crashes)
- **Recovery**: Empty on restart → full state restored
- **Reliability**: Data loss risk eliminated (at cost of ~2x slower writes)
- **Production-ready**: Foundation for real databases

**Performance Impact**:
- **Write latency**: 0.01ms (memory) → 1-5ms (with fsync to disk)
- **Throughput**: 50K writes/sec → 10K writes/sec (disk I/O bound)
- **Trade-off**: Speed vs durability (can batch writes for better throughput)

---

## Milestone 2: Persistence with Write-Ahead Log (WAL)

### Introduction

**The Problem**: In-memory data is volatile. Crash = data loss.

**The Solution: Write-Ahead Logging**
1. Before modifying in-memory state, append operation to log file
2. Sync log to disk (fsync)
3. Then modify in-memory HashMap
4. On restart: replay log to rebuild state

**WAL Pattern** (used by PostgreSQL, Redis, etcd):
```
Time 0: SET key1 value1  → Write to log, fsync, update HashMap
Time 1: SET key2 value2  → Write to log, fsync, update HashMap
Time 2: DELETE key1      → Write to log, fsync, update HashMap
--- CRASH ---
Time 3: Restart → Replay log: SET key1, SET key2, DELETE key1
        → State: {key2: value2}
```

### Key Concepts

**Structs**:
```rust
struct WalEntry {
    command: Command,
    timestamp: u64,
}

struct KvStore {
    data: Arc<RwLock<HashMap<String, String>>>,
    wal: Arc<RwLock<WriteAheadLog>>,
}

struct WriteAheadLog {
    file: File,
    path: PathBuf,
}
```

**Functions**:
```rust
impl WriteAheadLog {
    async fn new(path: PathBuf) -> io::Result<Self>
        // Open or create WAL file in append mode

    async fn append(&mut self, entry: &WalEntry) -> io::Result<()>
        // Serialize entry to bytes
        // Write to file
        // fsync to ensure durability

    async fn replay(&self) -> io::Result<Vec<WalEntry>>
        // Read entire file
        // Deserialize all entries
        // Return for playback
}

impl KvStore {
    async fn new_with_wal(wal_path: PathBuf) -> io::Result<Self>
        // Create or open WAL
        // Replay WAL to rebuild state
        // Return initialized store

    async fn set_durable(&self, key: String, value: String) -> io::Result<()>
        // 1. Append to WAL
        // 2. Sync to disk
        // 3. Update in-memory HashMap
}
```

**Serialization Format** (simple text format):
```
SET key1 value1
SET key2 value2
DELETE key1
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_wal_append_and_replay() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("test.wal");

        // Create WAL and append entries
        {
            let mut wal = WriteAheadLog::new(wal_path.clone()).await.unwrap();

            wal.append(&WalEntry {
                command: Command::Set {
                    key: "key1".to_string(),
                    value: "value1".to_string(),
                },
                timestamp: 1,
            }).await.unwrap();

            wal.append(&WalEntry {
                command: Command::Set {
                    key: "key2".to_string(),
                    value: "value2".to_string(),
                },
                timestamp: 2,
            }).await.unwrap();
        }

        // Replay WAL
        let wal = WriteAheadLog::new(wal_path).await.unwrap();
        let entries = wal.replay().await.unwrap();

        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_persistence_across_restart() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("store.wal");

        // First run: set some data
        {
            let store = KvStore::new_with_wal(wal_path.clone()).await.unwrap();
            store.set_durable("name".to_string(), "Alice".to_string()).await.unwrap();
            store.set_durable("age".to_string(), "30".to_string()).await.unwrap();
        }

        // Second run: reload from WAL
        {
            let store = KvStore::new_with_wal(wal_path).await.unwrap();
            assert_eq!(store.get("name").await, Some("Alice".to_string()));
            assert_eq!(store.get("age").await, Some("30".to_string()));
        }
    }

    #[tokio::test]
    async fn test_delete_persistence() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("delete.wal");

        {
            let store = KvStore::new_with_wal(wal_path.clone()).await.unwrap();
            store.set_durable("temp".to_string(), "value".to_string()).await.unwrap();
            store.delete_durable("temp").await.unwrap();
        }

        {
            let store = KvStore::new_with_wal(wal_path).await.unwrap();
            assert_eq!(store.get("temp").await, None);
        }
    }

    #[tokio::test]
    async fn test_wal_file_size_grows() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("grow.wal");

        let store = KvStore::new_with_wal(wal_path.clone()).await.unwrap();

        let initial_size = tokio::fs::metadata(&wal_path).await.unwrap().len();

        for i in 0..10 {
            store.set_durable(format!("key{}", i), format!("value{}", i))
                .await
                .unwrap();
        }

        let final_size = tokio::fs::metadata(&wal_path).await.unwrap().len();
        assert!(final_size > initial_size);
    }
}
```

### Starter Code

```rust
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone)]
struct WalEntry {
    command: Command,
    timestamp: u64,
}

struct WriteAheadLog {
    file: File,
    path: PathBuf,
}

impl WriteAheadLog {
    async fn new(path: PathBuf) -> io::Result<Self> {
        // TODO: Open file in append mode, create if doesn't exist
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)
            .await?;

        Ok(WriteAheadLog { file, path })
    }

    async fn append(&mut self, entry: &WalEntry) -> io::Result<()> {
        // TODO: Serialize command to text format
        let line = match &entry.command {
            Command::Set { key, value } => format!("SET {} {}\n", key, value),
            Command::Delete { key } => format!("DELETE {}\n", key),
            Command::Get { .. } => return Ok(()), // Don't log reads
        };

        // TODO: Write to file
        // self.file.write_all(line.as_bytes()).await?;
        todo!();

        // TODO: Sync to disk (ensure durability)
        // self.file.sync_all().await?;
        todo!();

        Ok(())
    }

    async fn replay(&self) -> io::Result<Vec<WalEntry>> {
        // TODO: Read entire file
        let mut file = File::open(&self.path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        // TODO: Parse each line into WalEntry
        let mut entries = Vec::new();
        for (idx, line) in contents.lines().enumerate() {
            if let Ok(command) = parse_command(line) {
                entries.push(WalEntry {
                    command,
                    timestamp: idx as u64,
                });
            }
        }

        Ok(entries)
    }
}

impl KvStore {
    async fn new_with_wal(wal_path: PathBuf) -> io::Result<Self> {
        let wal = WriteAheadLog::new(wal_path).await?;

        // TODO: Replay WAL to rebuild state
        let entries = wal.replay().await?;

        let data = HashMap::new();
        // TODO: Apply each entry to rebuild state
        // for entry in entries {
        //     match entry.command {
        //         Command::Set { key, value } => data.insert(key, value),
        //         Command::Delete { key } => data.remove(&key),
        //         _ => {}
        //     }
        // }
        todo!();

        Ok(KvStore {
            data: Arc::new(RwLock::new(data)),
            wal: Arc::new(RwLock::new(wal)),
        })
    }

    async fn set_durable(&self, key: String, value: String) -> io::Result<()> {
        // TODO: 1. Append to WAL
        let mut wal = self.wal.write().await;
        wal.append(&WalEntry {
            command: Command::Set {
                key: key.clone(),
                value: value.clone(),
            },
            timestamp: 0, // Use current time in production
        }).await?;
        drop(wal);

        // TODO: 2. Update in-memory HashMap
        // self.data.write().await.insert(key, value);
        todo!();

        Ok(())
    }

    async fn delete_durable(&self, key: &str) -> io::Result<bool> {
        // TODO: Similar to set_durable but for delete
        todo!();
    }
}
```

### Check Your Understanding

- **What is a Write-Ahead Log?** Append-only log of operations written before applying them to in-memory state.
- **Why write to WAL before updating HashMap?** Ensures we can recover operations even if we crash before updating memory.
- **What does `fsync` do?** Forces OS to flush data to physical disk (ensures durability).
- **How do we recover from a crash?** Replay entire WAL on startup to rebuild HashMap.
- **What's the performance cost of fsync?** ~1-5ms per write (vs 0.01ms in-memory), limits to ~1K writes/sec.

---

## Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Limitation: Single Point of Failure**
- Only one server holds the data
- Server crash = system unavailable until restart
- No redundancy if disk fails
- Cannot scale reads

**What We're Adding**:
- **Replication**: Copy data to multiple servers (master + replicas)
- **Async replication**: Master sends writes to replicas without waiting
- **Fault tolerance**: System stays available if 1 replica fails
- **Read scaling**: Distribute reads across replicas

**Improvement**:
- **Availability**: Single failure point → N-1 fault tolerance
- **Durability**: 1 copy → N copies (survive disk failures)
- **Read throughput**: 100K reads/sec → 300K reads/sec (3 replicas)
- **Write latency**: Unchanged (async replication doesn't wait)

**Architecture**:
```
         Master (read/write)
           /    \
          /      \
    Replica1   Replica2
   (read-only) (read-only)
```

---

## Milestone 3: Async Replication (Master-Replica)

### Introduction

**The Problem**: Single server = single point of failure and limited read capacity.

**The Solution: Master-Replica Replication**
- One master accepts writes
- Multiple replicas receive replicated writes asynchronously
- Reads can go to any replica (eventual consistency)
- Writes only to master

**Replication Flow**:
```
Client → SET key value → Master
                           ↓ (async)
                        Replica1, Replica2, Replica3
                           ↓ (eventually)
                        All replicas have key=value
```

### Key Concepts

**Structs**:
```rust
struct ReplicaInfo {
    address: String,
    client: TcpStream,
}

struct KvStore {
    data: Arc<RwLock<HashMap<String, String>>>,
    wal: Arc<RwLock<WriteAheadLog>>,
    replicas: Arc<RwLock<Vec<ReplicaInfo>>>,
    is_master: bool,
}
```

**Functions**:
```rust
impl KvStore {
    async fn add_replica(&self, address: String) -> io::Result<()>
        // Connect to replica
        // Add to replicas list
        // Send current snapshot

    async fn replicate_to_all(&self, command: &Command)
        // For each replica: send command (don't wait for ack)

    async fn set_with_replication(&self, key: String, value: String) -> io::Result<()>
        // 1. Append to WAL
        // 2. Update HashMap
        // 3. Replicate to followers (async, fire-and-forget)
}

// Replica server
async fn run_replica(master_addr: &str, listen_addr: &str)
    // Connect to master
    // Receive replicated commands
    // Apply to local store
```

**Replication Protocol**:
- Master → Replica: `REPLICATE SET key value\n`
- Replica → Master: (no ack in async mode)

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_replica() {
        // Start replica server
        tokio::spawn(async {
            run_replica_server("127.0.0.1:9401").await.unwrap();
        });
        sleep(Duration::from_millis(100)).await;

        let store = KvStore::new();
        store.add_replica("127.0.0.1:9401".to_string()).await.unwrap();

        assert_eq!(store.replicas.read().await.len(), 1);
    }

    #[tokio::test]
    async fn test_replication_propagates() {
        // Start master
        let master = Arc::new(KvStore::new());
        tokio::spawn({
            let master = master.clone();
            async move {
                run_master_server("127.0.0.1:9402", master).await.unwrap();
            }
        });

        // Start replica
        let replica = Arc::new(KvStore::new());
        tokio::spawn({
            let replica = replica.clone();
            async move {
                run_replica_server_with_store("127.0.0.1:9403", replica).await.unwrap();
            }
        });

        sleep(Duration::from_millis(100)).await;

        // Connect master to replica
        master.add_replica("127.0.0.1:9403".to_string()).await.unwrap();

        // Write to master
        master.set_with_replication("key1".to_string(), "value1".to_string())
            .await
            .unwrap();

        // Wait for async replication
        sleep(Duration::from_millis(100)).await;

        // Read from replica
        let replica_value = replica.get("key1").await;
        assert_eq!(replica_value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_replicas() {
        let master = Arc::new(KvStore::new());

        // Start 3 replicas
        let replica1 = Arc::new(KvStore::new());
        let replica2 = Arc::new(KvStore::new());
        let replica3 = Arc::new(KvStore::new());

        // ... (start servers and connect)

        master.add_replica("127.0.0.1:9404".to_string()).await.unwrap();
        master.add_replica("127.0.0.1:9405".to_string()).await.unwrap();
        master.add_replica("127.0.0.1:9406".to_string()).await.unwrap();

        master.set_with_replication("shared".to_string(), "data".to_string())
            .await
            .unwrap();

        sleep(Duration::from_millis(100)).await;

        // All replicas should have the data
        assert_eq!(replica1.get("shared").await, Some("data".to_string()));
        assert_eq!(replica2.get("shared").await, Some("data".to_string()));
        assert_eq!(replica3.get("shared").await, Some("data".to_string()));
    }

    #[tokio::test]
    async fn test_replica_failure_doesnt_block_master() {
        let master = Arc::new(KvStore::new());

        // Add a replica that will fail
        master.add_replica("127.0.0.1:9999".to_string()).await.ok(); // Nonexistent

        // Master should still accept writes
        let result = master.set_with_replication("key".to_string(), "value".to_string()).await;
        assert!(result.is_ok());
    }
}
```

### Starter Code

```rust
use tokio::net::TcpStream;

struct ReplicaInfo {
    address: String,
    stream: TcpStream,
}

impl KvStore {
    async fn add_replica(&self, address: String) -> io::Result<()> {
        // TODO: Connect to replica
        let stream = TcpStream::connect(&address).await?;

        // TODO: Add to replicas list
        let mut replicas = self.replicas.write().await;
        replicas.push(ReplicaInfo {
            address,
            stream,
        });

        Ok(())
    }

    async fn replicate_to_all(&self, command: &Command) {
        // TODO: For each replica, send command
        let replicas = self.replicas.read().await;

        for replica in replicas.iter() {
            // Serialize command
            let msg = match command {
                Command::Set { key, value } => format!("REPLICATE SET {} {}\n", key, value),
                Command::Delete { key } => format!("REPLICATE DELETE {}\n", key),
                _ => continue,
            };

            // TODO: Send to replica (ignore errors - fire and forget)
            // replica.stream.write_all(msg.as_bytes()).await.ok();
            todo!();
        }
    }

    async fn set_with_replication(&self, key: String, value: String) -> io::Result<()> {
        // TODO: 1. Append to WAL (if enabled)
        if let Some(wal) = &self.wal {
            // wal.write().await.append(...).await?;
            todo!();
        }

        // TODO: 2. Update HashMap
        self.data.write().await.insert(key.clone(), value.clone());

        // TODO: 3. Replicate to followers (async, spawn task)
        let command = Command::Set { key, value };
        let store = self.clone();
        tokio::spawn(async move {
            store.replicate_to_all(&command).await;
        });

        Ok(())
    }
}

async fn run_replica_server(listen_addr: &str) -> io::Result<()> {
    let store = Arc::new(KvStore::new());
    let listener = TcpListener::bind(listen_addr).await?;

    println!("Replica listening on {}", listen_addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let store = store.clone();

        tokio::spawn(async move {
            handle_replica_client(stream, store).await.ok();
        });
    }
}

async fn handle_replica_client(stream: TcpStream, store: Arc<KvStore>) -> io::Result<()> {
    let (reader, _writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        // TODO: Parse REPLICATE commands
        if let Some(cmd_str) = line.strip_prefix("REPLICATE ") {
            if let Ok(command) = parse_command(cmd_str) {
                // TODO: Apply to local store
                match command {
                    Command::Set { key, value } => {
                        store.set(key, value).await;
                    }
                    Command::Delete { key } => {
                        store.delete(&key).await;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

### Check Your Understanding

- **What is async replication?** Master sends writes to replicas but doesn't wait for acknowledgment.
- **Why is async replication faster than sync?** Master doesn't wait for replicas, so write latency is just local write time.
- **What's the downside of async replication?** Data loss if master crashes before replicas receive the write.
- **Can replicas serve reads?** Yes, but data may be slightly stale (eventual consistency).
- **What happens if a replica is down?** Master continues operating (fire-and-forget pattern).

---

## Why Milestone 3 Isn't Enough → Moving to Milestone 4

**Limitation: Data Loss Window**
- Async replication = master can crash before replicas receive write
- Example: Master receives write, crashes before replicating → data lost
- Eventual consistency = replicas lag behind master
- No guarantee writes are durable

**What We're Adding**:
- **Synchronous replication**: Wait for quorum before acknowledging
- **Quorum writes**: W=2 out of N=3 replicas must acknowledge
- **Strong consistency**: Guaranteed durability (data on ≥2 nodes)
- **Configurable consistency**: Trade latency for durability

**Improvement**:
- **Durability**: Async (data loss possible) → Sync quorum (guaranteed durability)
- **Consistency**: Eventual → Strong (reads see committed writes)
- **Fault tolerance**: Survive F=W-1 failures (W=2 → survive 1 failure)
- **Latency cost**: Write time increases (wait for slowest replica in quorum)

**Quorum Example** (N=3, W=2):
```
Client → SET key value → Master
                           ↓ (wait for 2 acks)
                    Replica1 ✓, Replica2 ✓, Replica3 ✗
                           ↓
Client ← OK (write durable on 2 nodes)
```

---

## Milestone 4: Synchronous Replication with Quorum Writes

### Introduction

**The Problem**: Async replication can lose data on master crash.

**The Solution: Quorum Writes**
- Configure N (total replicas) and W (write quorum)
- Master waits for W replicas to acknowledge before returning OK
- Common: N=3, W=2 (majority quorum, survive 1 failure)
- Trade-off: Higher latency for guaranteed durability

**Consistency Guarantee**:
```
If W + R > N (where R = read quorum), reads see committed writes
Example: N=3, W=2, R=2 → 2+2 > 3 → strong consistency
```

### Key Concepts

**Structs**:
```rust
struct ReplicationConfig {
    total_replicas: usize,  // N
    write_quorum: usize,     // W
    read_quorum: usize,      // R
}

struct ReplicaInfo {
    address: String,
    stream: TcpStream,
    healthy: bool,
}

struct WriteAck {
    replica_id: usize,
    success: bool,
}
```

**Functions**:
```rust
impl KvStore {
    async fn set_with_quorum(&self, key: String, value: String) -> io::Result<()>
        // 1. Write to WAL locally
        // 2. Send to all replicas
        // 3. Wait for W acknowledgments (with timeout)
        // 4. If quorum reached: commit, return OK
        // 5. If quorum failed: rollback, return error

    async fn wait_for_quorum(&self, write_id: u64) -> Result<(), QuorumError>
        // Wait for W replicas to acknowledge
        // Timeout after 5 seconds
        // Return Ok if quorum reached, Err otherwise
}

// Replica acknowledges writes
async fn handle_replica_sync_write(stream: TcpStream, store: Arc<KvStore>)
    // Receive REPLICATE_SYNC command
    // Apply to local store
    // Send ACK back to master
```

**Protocol**:
- Master → Replica: `REPLICATE_SYNC <write_id> SET key value\n`
- Replica → Master: `ACK <write_id>\n` or `NACK <write_id> <error>\n`

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quorum_write_success() {
        let config = ReplicationConfig {
            total_replicas: 3,
            write_quorum: 2,
            read_quorum: 2,
        };

        let master = Arc::new(KvStore::new_with_config(config));

        // Start 3 replicas
        let replicas = start_replicas(3).await;

        // Connect master to replicas
        for (i, addr) in replicas.iter().enumerate() {
            master.add_replica(addr.clone()).await.unwrap();
        }

        // Write with quorum (should succeed with 2/3 acks)
        let result = master.set_with_quorum("key".to_string(), "value".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quorum_write_failure() {
        let config = ReplicationConfig {
            total_replicas: 3,
            write_quorum: 3, // Require all 3
            read_quorum: 2,
        };

        let master = Arc::new(KvStore::new_with_config(config));

        // Only connect 2 replicas (1 is down)
        master.add_replica("127.0.0.1:9501".to_string()).await.unwrap();
        master.add_replica("127.0.0.1:9502".to_string()).await.unwrap();

        // Write should fail (need 3, have 2)
        let result = master.set_with_quorum("key".to_string(), "value".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quorum_with_slow_replica() {
        let config = ReplicationConfig {
            total_replicas: 3,
            write_quorum: 2,
            read_quorum: 2,
        };

        let master = Arc::new(KvStore::new_with_config(config));

        // 2 fast replicas, 1 slow replica
        master.add_replica("127.0.0.1:9503".to_string()).await.unwrap();
        master.add_replica("127.0.0.1:9504".to_string()).await.unwrap();
        master.add_replica("127.0.0.1:9999".to_string()).await.ok(); // Slow/dead

        // Should succeed (2 fast replicas = quorum)
        let start = Instant::now();
        let result = master.set_with_quorum("key".to_string(), "value".to_string()).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed < Duration::from_secs(1)); // Doesn't wait for slow replica
    }

    #[tokio::test]
    async fn test_majority_quorum() {
        // N=5, W=3 (majority)
        let config = ReplicationConfig {
            total_replicas: 5,
            write_quorum: 3,
            read_quorum: 3,
        };

        let master = Arc::new(KvStore::new_with_config(config));

        // Connect 5 replicas
        for i in 0..5 {
            let addr = format!("127.0.0.1:{}", 9505 + i);
            start_replica_on(&addr).await;
            master.add_replica(addr).await.unwrap();
        }

        // Should succeed with 3/5 acks
        let result = master.set_with_quorum("data".to_string(), "value".to_string()).await;
        assert!(result.is_ok());
    }
}
```

### Starter Code

```rust
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

struct ReplicationConfig {
    total_replicas: usize,
    write_quorum: usize,
    read_quorum: usize,
}

struct WriteAck {
    replica_id: usize,
    success: bool,
}

impl KvStore {
    async fn set_with_quorum(&self, key: String, value: String) -> io::Result<()> {
        // Generate unique write ID
        let write_id = generate_write_id();

        // TODO: 1. Write to local WAL
        if let Some(wal) = &self.wal {
            // wal.write().await.append(...).await?;
            todo!();
        }

        // TODO: 2. Send to all replicas
        let replicas = self.replicas.read().await;
        let msg = format!("REPLICATE_SYNC {} SET {} {}\n", write_id, key, value);

        let (ack_tx, mut ack_rx) = tokio::sync::mpsc::channel(replicas.len());

        for (replica_id, replica) in replicas.iter().enumerate() {
            let msg = msg.clone();
            let ack_tx = ack_tx.clone();
            let mut stream = replica.stream.clone();

            tokio::spawn(async move {
                // Send command
                if stream.write_all(msg.as_bytes()).await.is_err() {
                    ack_tx.send(WriteAck {
                        replica_id,
                        success: false,
                    }).await.ok();
                    return;
                }

                // Wait for ACK with timeout
                let mut buf = [0u8; 1024];
                match timeout(Duration::from_secs(5), stream.read(&mut buf)).await {
                    Ok(Ok(n)) if n > 0 => {
                        let response = String::from_utf8_lossy(&buf[..n]);
                        let success = response.starts_with("ACK");
                        ack_tx.send(WriteAck { replica_id, success }).await.ok();
                    }
                    _ => {
                        ack_tx.send(WriteAck {
                            replica_id,
                            success: false,
                        }).await.ok();
                    }
                }
            });
        }
        drop(ack_tx);
        drop(replicas);

        // TODO: 3. Wait for quorum acknowledgments
        let mut acks = 1; // Master counts as 1
        while let Some(ack) = ack_rx.recv().await {
            if ack.success {
                acks += 1;
            }

            if acks >= self.config.write_quorum {
                break;
            }
        }

        // TODO: 4. Check if quorum reached
        if acks >= self.config.write_quorum {
            // Quorum reached: commit to local store
            self.data.write().await.insert(key, value);
            Ok(())
        } else {
            // Quorum failed: return error
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to reach write quorum",
            ))
        }
    }
}

async fn handle_replica_sync_write(
    stream: TcpStream,
    store: Arc<KvStore>,
) -> io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        // TODO: Parse REPLICATE_SYNC command
        // Format: REPLICATE_SYNC <write_id> SET key value
        if let Some(cmd_str) = line.strip_prefix("REPLICATE_SYNC ") {
            let parts: Vec<&str> = cmd_str.splitn(2, ' ').collect();
            if parts.len() != 2 {
                continue;
            }

            let write_id = parts[0];
            let command_str = parts[1];

            // TODO: Parse and apply command
            if let Ok(command) = parse_command(command_str) {
                match command {
                    Command::Set { key, value } => {
                        store.set(key, value).await;

                        // TODO: Send ACK
                        writer.write_all(format!("ACK {}\n", write_id).as_bytes()).await?;
                    }
                    Command::Delete { key } => {
                        store.delete(&key).await;
                        writer.write_all(format!("ACK {}\n", write_id).as_bytes()).await?;
                    }
                    _ => {}
                }
            } else {
                // Send NACK on parse error
                writer.write_all(format!("NACK {} parse_error\n", write_id).as_bytes()).await?;
            }
        }
    }

    Ok(())
}

fn generate_write_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
```

### Check Your Understanding

- **What is a write quorum?** Minimum number of replicas that must acknowledge a write.
- **Why use quorum writes?** Guarantee durability—data is on multiple nodes before acknowledging.
- **What's the trade-off?** Higher write latency (wait for W replicas) vs stronger consistency.
- **With N=3, W=2, how many failures can we tolerate?** 1 failure (2 nodes still form quorum).
- **What if quorum is not reached?** Write fails, client receives error, should retry.

---

## Why Milestone 4 Isn't Enough → Moving to Milestone 5

**Limitation: Manual Failover**
- Master crashes → system unavailable until manual intervention
- Need operator to promote replica to master
- Downtime = minutes to hours (human in the loop)
- No automatic recovery

**What We're Adding**:
- **Leader election**: Replicas automatically elect new master on failure
- **Heartbeats**: Detect master failure quickly (2-5 seconds)
- **Automatic promotion**: Replica becomes master without human intervention
- **Simplified Raft**: Voting-based consensus for leader election

**Improvement**:
- **Availability**: Manual failover (minutes) → automatic (seconds)
- **Recovery**: Human required → fully automated
- **Downtime**: Minutes → 2-5 seconds
- **Production-ready**: Can deploy without 24/7 on-call

**Leader Election Algorithm** (simplified Raft):
1. Nodes send heartbeats to leader
2. If no heartbeat for N seconds → start election
3. Candidate increments term, votes for self
4. Requests votes from other nodes
5. Node grants vote if: term is newer, haven't voted this term
6. Candidate with majority becomes leader

---

## Milestone 5: Leader Election (Simplified Raft)

### Introduction

**The Problem**: Master failure requires manual intervention (downtime).

**The Solution: Automated Leader Election**
- All nodes monitor leader via heartbeats
- On timeout: start election
- Majority vote determines new leader
- New leader starts replicating to followers

**Simplified Raft Election**:
```
Time 0: Master sends heartbeat every 1s
Time 5: Master crashes (no heartbeat)
Time 7: Replica timeout → starts election (term 2, votes for self)
Time 7.1: Requests votes from other replicas
Time 7.2: Receives majority votes → becomes master
Time 7.3: Sends heartbeat to establish leadership
```

### Key Concepts

**Structs**:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeRole {
    Leader,
    Follower,
    Candidate,
}

struct NodeState {
    role: NodeRole,
    current_term: u64,
    voted_for: Option<String>, // Node ID
    leader_id: Option<String>,
}

struct KvStore {
    // ... existing fields ...
    node_id: String,
    state: Arc<RwLock<NodeState>>,
    peers: Arc<RwLock<Vec<String>>>, // Other node addresses
}
```

**Functions**:
```rust
impl KvStore {
    async fn start_election(&self)
        // Increment term
        // Change role to Candidate
        // Vote for self
        // Request votes from all peers
        // If majority: become Leader

    async fn send_heartbeat(&self)
        // Send heartbeat to all peers
        // Maintain leadership

    async fn handle_vote_request(&self, term: u64, candidate_id: String) -> bool
        // Grant vote if:
        //   1. Term is greater than current term
        //   2. Haven't voted for anyone else this term

    async fn handle_heartbeat(&self, term: u64, leader_id: String)
        // Reset election timeout
        // Update leader_id

    async fn run_election_timeout(&self)
        // Background task
        // If no heartbeat for N seconds: start election
}
```

**Messages**:
- `HEARTBEAT term=5 leader=node1`
- `VOTE_REQUEST term=6 candidate=node2`
- `VOTE_RESPONSE term=6 granted=true`

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_leader_sends_heartbeats() {
        let node = KvStore::new_with_role(NodeRole::Leader, "node1".to_string());

        // Start heartbeat task
        let node_clone = node.clone();
        tokio::spawn(async move {
            node_clone.run_heartbeat_loop().await;
        });

        sleep(Duration::from_secs(2)).await;

        // Verify heartbeats were sent (check logs or mock peers)
    }

    #[tokio::test]
    async fn test_follower_starts_election_on_timeout() {
        let node = KvStore::new_with_role(NodeRole::Follower, "node1".to_string());

        // Start election timeout task
        let node_clone = node.clone();
        tokio::spawn(async move {
            node_clone.run_election_timeout().await;
        });

        sleep(Duration::from_secs(6)).await; // Timeout is 5s

        // Should have started election
        let state = node.state.read().await;
        assert_eq!(state.role, NodeRole::Candidate);
    }

    #[tokio::test]
    async fn test_vote_granting() {
        let node = KvStore::new_with_role(NodeRole::Follower, "node1".to_string());

        // First vote request (term 2)
        let granted = node.handle_vote_request(2, "candidate1".to_string()).await;
        assert!(granted);

        // Second vote request same term (should reject)
        let granted = node.handle_vote_request(2, "candidate2".to_string()).await;
        assert!(!granted);

        // Higher term (should grant)
        let granted = node.handle_vote_request(3, "candidate2".to_string()).await;
        assert!(granted);
    }

    #[tokio::test]
    async fn test_majority_election() {
        // Create 3-node cluster
        let node1 = Arc::new(KvStore::new_with_id("node1".to_string()));
        let node2 = Arc::new(KvStore::new_with_id("node2".to_string()));
        let node3 = Arc::new(KvStore::new_with_id("node3".to_string()));

        // node1 starts election
        node1.start_election().await;

        // node2 and node3 grant votes
        let vote2 = node2.handle_vote_request(1, "node1".to_string()).await;
        let vote3 = node3.handle_vote_request(1, "node1".to_string()).await;

        assert!(vote2);
        assert!(vote3);

        // node1 should become leader (has majority: 3/3)
        let state = node1.state.read().await;
        assert_eq!(state.role, NodeRole::Leader);
    }

    #[tokio::test]
    async fn test_split_vote_retry() {
        // Create 4-node cluster
        let nodes = vec![
            Arc::new(KvStore::new_with_id("node1".to_string())),
            Arc::new(KvStore::new_with_id("node2".to_string())),
            Arc::new(KvStore::new_with_id("node3".to_string())),
            Arc::new(KvStore::new_with_id("node4".to_string())),
        ];

        // node1 and node2 both start election simultaneously
        tokio::join!(
            nodes[0].start_election(),
            nodes[1].start_election(),
        );

        // Split vote: each gets 2 votes (self + 1 other)
        // No majority (need 3/4)

        // Should timeout and retry with higher term
        sleep(Duration::from_secs(6)).await;

        // Eventually one should become leader
    }
}
```

### Starter Code

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeRole {
    Leader,
    Follower,
    Candidate,
}

struct NodeState {
    role: NodeRole,
    current_term: u64,
    voted_for: Option<String>,
    leader_id: Option<String>,
    last_heartbeat: Instant,
}

impl KvStore {
    async fn start_election(&self) {
        println!("[{}] Starting election", self.node_id);

        // TODO: Increment term and vote for self
        let mut state = self.state.write().await;
        state.current_term += 1;
        state.role = NodeRole::Candidate;
        state.voted_for = Some(self.node_id.clone());
        let term = state.current_term;
        drop(state);

        // TODO: Request votes from all peers
        let peers = self.peers.read().await.clone();
        let mut votes = 1; // Vote for self

        for peer in peers.iter() {
            // TODO: Send VOTE_REQUEST to peer
            if let Ok(granted) = send_vote_request(peer, term, &self.node_id).await {
                if granted {
                    votes += 1;
                }
            }
        }

        // TODO: Check if we have majority
        let total_nodes = peers.len() + 1;
        let majority = total_nodes / 2 + 1;

        if votes >= majority {
            // TODO: Become leader
            let mut state = self.state.write().await;
            state.role = NodeRole::Leader;
            state.leader_id = Some(self.node_id.clone());
            println!("[{}] Became leader (term {})", self.node_id, term);
        } else {
            // TODO: Revert to follower
            let mut state = self.state.write().await;
            state.role = NodeRole::Follower;
            state.voted_for = None;
        }
    }

    async fn run_heartbeat_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let state = self.state.read().await;
            if state.role != NodeRole::Leader {
                break; // Stop if no longer leader
            }
            let term = state.current_term;
            drop(state);

            // TODO: Send heartbeat to all peers
            let peers = self.peers.read().await;
            for peer in peers.iter() {
                send_heartbeat(peer, term, &self.node_id).await.ok();
            }
        }
    }

    async fn run_election_timeout(&self) {
        let timeout_duration = Duration::from_secs(5);

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let state = self.state.read().await;
            if state.role == NodeRole::Leader {
                continue; // Leaders don't timeout
            }

            let elapsed = state.last_heartbeat.elapsed();
            drop(state);

            // TODO: If timeout, start election
            if elapsed > timeout_duration {
                self.start_election().await;
            }
        }
    }

    async fn handle_vote_request(&self, term: u64, candidate_id: String) -> bool {
        let mut state = self.state.write().await;

        // TODO: Grant vote if term is newer and haven't voted
        if term > state.current_term {
            state.current_term = term;
            state.voted_for = Some(candidate_id.clone());
            state.role = NodeRole::Follower;
            return true;
        }

        if term == state.current_term && state.voted_for.is_none() {
            state.voted_for = Some(candidate_id);
            return true;
        }

        false
    }

    async fn handle_heartbeat(&self, term: u64, leader_id: String) {
        let mut state = self.state.write().await;

        // TODO: Update term and reset timeout
        if term >= state.current_term {
            state.current_term = term;
            state.role = NodeRole::Follower;
            state.leader_id = Some(leader_id);
            state.last_heartbeat = Instant::now();
        }
    }
}

async fn send_vote_request(peer: &str, term: u64, candidate_id: &str) -> io::Result<bool> {
    // TODO: Connect to peer and send VOTE_REQUEST
    // Format: VOTE_REQUEST term=X candidate=Y
    todo!();
}

async fn send_heartbeat(peer: &str, term: u64, leader_id: &str) -> io::Result<()> {
    // TODO: Connect to peer and send HEARTBEAT
    // Format: HEARTBEAT term=X leader=Y
    todo!();
}
```

### Check Your Understanding

- **What triggers a leader election?** Follower doesn't receive heartbeat within timeout period.
- **How does a node decide who to vote for?** Grants vote if term is newer and hasn't voted this term yet.
- **What is split brain?** Two nodes think they're leader (prevented by majority quorum).
- **Why send heartbeats?** Maintain leadership and prevent unnecessary elections.
- **What happens if no majority?** Election times out, nodes retry with higher term.

---

## Why Milestone 5 Isn't Enough → Moving to Milestone 6

**Limitation: Client Complexity**
- Clients must manually track which node is leader
- Need to retry on different node if leader changes
- Connection overhead (new TCP connection per request)
- No load balancing across replicas for reads

**What We're Adding**:
- **Client connection pool**: Reuse TCP connections (avoid handshake overhead)
- **Smart routing**: Automatically send writes to leader, reads to any node
- **Automatic failover**: Retry on different node if leader changes
- **Read load balancing**: Distribute reads across all replicas

**Improvement**:
- **Performance**: New connection (3-way handshake) → pooled connection (instant)
- **Throughput**: 1K req/sec → 10K req/sec (connection reuse)
- **Availability**: Manual retry → automatic failover
- **Read scaling**: All reads to leader → distributed across N replicas

**Client Architecture**:
```
Client → Pool[Leader, Replica1, Replica2]
           ├─ GET key → Replica2 (load balanced)
           └─ SET key → Leader (routed)
```

---

## Milestone 6: Client Connection Pool and Smart Routing

### Introduction

**The Problem**: Creating new TCP connections is expensive (3-way handshake = 1-3ms).

**The Solution: Connection Pooling**
- Maintain pool of open connections to each node
- Reuse connections for multiple requests
- Route writes to leader, reads to any replica
- Automatically detect leader changes and re-route

**Connection Pool Benefits**:
- **Latency**: 3ms (new connection) → 0.1ms (pooled)
- **Throughput**: 300 req/sec → 10K req/sec per client
- **Efficiency**: No handshake overhead, TCP window already tuned

### Key Concepts

**Structs**:
```rust
struct KvClient {
    pools: HashMap<String, ConnectionPool>,
    leader_addr: Arc<RwLock<Option<String>>>,
    replica_addrs: Vec<String>,
}

struct ConnectionPool {
    address: String,
    available: Arc<Mutex<VecDeque<TcpStream>>>,
    max_size: usize,
}

struct PooledConnection {
    stream: Option<TcpStream>,
    pool: Arc<Mutex<VecDeque<TcpStream>>>,
}
```

**Functions**:
```rust
impl ConnectionPool {
    async fn acquire(&self) -> io::Result<PooledConnection>
        // Try to reuse connection from pool
        // If none available: create new connection
        // Return PooledConnection (returns to pool on drop)

    async fn release(&self, stream: TcpStream)
        // Return connection to pool
}

impl KvClient {
    async fn get(&self, key: &str) -> io::Result<Option<String>>
        // Pick random replica (load balancing)
        // Acquire connection from pool
        // Send GET command
        // Return value

    async fn set(&self, key: String, value: String) -> io::Result<()>
        // Get leader address
        // Acquire connection to leader
        // Send SET command
        // Handle NOT_LEADER error (retry on new leader)

    async fn discover_leader(&self) -> io::Result<String>
        // Ask any node who the leader is
        // Update cached leader address
}
```

**Protocol Extensions**:
- `WHO_IS_LEADER` → `LEADER node1.example.com:6379`
- `SET key value` → `NOT_LEADER leader=node2:6379` (redirect)

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_reuse() {
        let pool = ConnectionPool::new("127.0.0.1:9601".to_string(), 10);

        // Acquire connection
        let conn1 = pool.acquire().await.unwrap();
        let stream_addr = format!("{:p}", &conn1.stream);
        drop(conn1); // Return to pool

        // Acquire again - should be same connection
        let conn2 = pool.acquire().await.unwrap();
        let stream_addr2 = format!("{:p}", &conn2.stream);

        assert_eq!(stream_addr, stream_addr2); // Same connection reused
    }

    #[tokio::test]
    async fn test_pool_max_size() {
        let pool = ConnectionPool::new("127.0.0.1:9602".to_string(), 2);

        let _conn1 = pool.acquire().await.unwrap();
        let _conn2 = pool.acquire().await.unwrap();

        // Pool is at max size, should create new connection (not pool it on return)
        let _conn3 = pool.acquire().await.unwrap();
    }

    #[tokio::test]
    async fn test_client_get_with_pool() {
        // Start server
        tokio::spawn(async {
            run_kv_server("127.0.0.1:9603").await.unwrap();
        });
        sleep(Duration::from_millis(100)).await;

        let client = KvClient::new(vec!["127.0.0.1:9603".to_string()]);

        // First GET (creates connection)
        let start = Instant::now();
        client.get("key1").await.unwrap();
        let first_duration = start.elapsed();

        // Second GET (reuses connection)
        let start = Instant::now();
        client.get("key2").await.unwrap();
        let second_duration = start.elapsed();

        // Second should be faster (no handshake)
        println!("First: {:?}, Second: {:?}", first_duration, second_duration);
    }

    #[tokio::test]
    async fn test_smart_routing_to_leader() {
        // Start 3-node cluster
        let leader = start_node_as_leader("127.0.0.1:9604").await;
        let replica1 = start_node_as_replica("127.0.0.1:9605").await;
        let replica2 = start_node_as_replica("127.0.0.1:9606").await;

        let client = KvClient::new(vec![
            "127.0.0.1:9604".to_string(),
            "127.0.0.1:9605".to_string(),
            "127.0.0.1:9606".to_string(),
        ]);

        // SET should go to leader
        client.set("key".to_string(), "value".to_string()).await.unwrap();

        // Verify write reached leader
        assert_eq!(leader.get("key").await, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_read_load_balancing() {
        let client = KvClient::new(vec![
            "127.0.0.1:9607".to_string(),
            "127.0.0.1:9608".to_string(),
            "127.0.0.1:9609".to_string(),
        ]);

        // Track which replicas were used
        let mut replica_usage = HashMap::new();

        for _ in 0..30 {
            let replica = client.pick_read_replica().await;
            *replica_usage.entry(replica).or_insert(0) += 1;
        }

        // Should have distributed reads across multiple replicas
        assert!(replica_usage.len() > 1);
    }

    #[tokio::test]
    async fn test_automatic_leader_failover() {
        // Start 3-node cluster
        let leader = start_node_as_leader("127.0.0.1:9610").await;
        let replica1 = start_node_as_replica("127.0.0.1:9611").await;

        let client = KvClient::new(vec![
            "127.0.0.1:9610".to_string(),
            "127.0.0.1:9611".to_string(),
        ]);

        // Write succeeds to leader
        client.set("key1".to_string(), "value1".to_string()).await.unwrap();

        // Simulate leader crash
        drop(leader);

        // replica1 should become new leader
        sleep(Duration::from_secs(6)).await; // Election timeout

        // Client should discover new leader and succeed
        client.set("key2".to_string(), "value2".to_string()).await.unwrap();
    }
}
```

### Starter Code

```rust
use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;
use rand::Rng;

struct ConnectionPool {
    address: String,
    available: Arc<Mutex<VecDeque<TcpStream>>>,
    max_size: usize,
}

impl ConnectionPool {
    fn new(address: String, max_size: usize) -> Self {
        ConnectionPool {
            address,
            available: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
        }
    }

    async fn acquire(&self) -> io::Result<PooledConnection> {
        // TODO: Try to get connection from pool
        let mut pool = self.available.lock().await;

        if let Some(stream) = pool.pop_front() {
            return Ok(PooledConnection {
                stream: Some(stream),
                pool: self.available.clone(),
            });
        }
        drop(pool);

        // TODO: No available connection - create new one
        let stream = TcpStream::connect(&self.address).await?;

        Ok(PooledConnection {
            stream: Some(stream),
            pool: self.available.clone(),
        })
    }
}

struct PooledConnection {
    stream: Option<TcpStream>,
    pool: Arc<Mutex<VecDeque<TcpStream>>>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        // TODO: Return connection to pool
        if let Some(stream) = self.stream.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let mut pool = pool.lock().await;
                pool.push_back(stream);
            });
        }
    }
}

impl std::ops::Deref for PooledConnection {
    type Target = TcpStream;
    fn deref(&self) -> &Self::Target {
        self.stream.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stream.as_mut().unwrap()
    }
}

struct KvClient {
    pools: HashMap<String, ConnectionPool>,
    leader_addr: Arc<RwLock<Option<String>>>,
    replica_addrs: Vec<String>,
}

impl KvClient {
    fn new(addrs: Vec<String>) -> Self {
        let mut pools = HashMap::new();
        for addr in &addrs {
            pools.insert(addr.clone(), ConnectionPool::new(addr.clone(), 10));
        }

        KvClient {
            pools,
            leader_addr: Arc::new(RwLock::new(None)),
            replica_addrs: addrs,
        }
    }

    async fn get(&self, key: &str) -> io::Result<Option<String>> {
        // TODO: Pick random replica for read load balancing
        let replica = self.pick_read_replica().await;

        // TODO: Acquire connection from pool
        let pool = self.pools.get(&replica).unwrap();
        let mut conn = pool.acquire().await?;

        // TODO: Send GET command
        conn.write_all(format!("GET {}\n", key).as_bytes()).await?;

        // TODO: Read response
        let mut buf = [0u8; 4096];
        let n = conn.read(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..n]);

        // TODO: Parse response
        if let Some(value) = response.strip_prefix("VALUE ") {
            Ok(Some(value.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: String, value: String) -> io::Result<()> {
        // TODO: Discover leader if not known
        let leader = match self.leader_addr.read().await.clone() {
            Some(addr) => addr,
            None => self.discover_leader().await?,
        };

        // TODO: Acquire connection to leader
        let pool = self.pools.get(&leader).unwrap();
        let mut conn = pool.acquire().await?;

        // TODO: Send SET command
        conn.write_all(format!("SET {} {}\n", key, value).as_bytes()).await?;

        // TODO: Read response
        let mut buf = [0u8; 1024];
        let n = conn.read(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..n]);

        // TODO: Handle NOT_LEADER redirect
        if response.contains("NOT_LEADER") {
            // Extract new leader address and retry
            // self.leader_addr.write().await = Some(new_leader);
            // return self.set(key, value).await;
            todo!();
        }

        if response.starts_with("OK") {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "SET failed"))
        }
    }

    async fn discover_leader(&self) -> io::Result<String> {
        // TODO: Ask any replica who the leader is
        for addr in &self.replica_addrs {
            if let Ok(leader) = self.query_leader(addr).await {
                *self.leader_addr.write().await = Some(leader.clone());
                return Ok(leader);
            }
        }

        Err(io::Error::new(io::ErrorKind::Other, "No leader found"))
    }

    async fn query_leader(&self, addr: &str) -> io::Result<String> {
        // TODO: Send WHO_IS_LEADER command
        let pool = self.pools.get(addr).unwrap();
        let mut conn = pool.acquire().await?;

        conn.write_all(b"WHO_IS_LEADER\n").await?;

        let mut buf = [0u8; 1024];
        let n = conn.read(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..n]);

        if let Some(leader) = response.strip_prefix("LEADER ") {
            Ok(leader.trim().to_string())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Unknown leader"))
        }
    }

    async fn pick_read_replica(&self) -> String {
        // TODO: Random load balancing
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.replica_addrs.len());
        self.replica_addrs[idx].clone()
    }
}
```

### Check Your Understanding

- **What is connection pooling?** Reusing TCP connections across multiple requests instead of creating new ones.
- **Why is pooling faster?** Avoids TCP handshake (SYN, SYN-ACK, ACK) which takes 1-3ms.
- **How does smart routing work?** Writes go to leader, reads go to any replica (load balanced).
- **What happens if leader changes?** Client receives NOT_LEADER redirect, updates cached leader address, retries.
- **How much faster is pooling?** ~10-30x for small requests (handshake overhead eliminated).

---

## Complete Working Example

Below is a simplified but functional distributed key-value store with replication and leader election:

```rust
// See full implementation in previous milestones combined
// This example demonstrates the key components:

#[tokio::main]
async fn main() {
    // Start 3-node cluster
    let node1 = Arc::new(KvStore::new_with_id("node1".to_string()));
    let node2 = Arc::new(KvStore::new_with_id("node2".to_string()));
    let node3 = Arc::new(KvStore::new_with_id("node3".to_string()));

    // Configure as cluster
    node1.add_peer("127.0.0.1:6380".to_string()).await;
    node1.add_peer("127.0.0.1:6381".to_string()).await;

    // Start servers
    tokio::spawn(run_server("127.0.0.1:6379", node1.clone()));
    tokio::spawn(run_server("127.0.0.1:6380", node2.clone()));
    tokio::spawn(run_server("127.0.0.1:6381", node3.clone()));

    // Start leader election
    tokio::spawn(node1.run_election_timeout());
    tokio::spawn(node2.run_election_timeout());
    tokio::spawn(node3.run_election_timeout());

    // Client usage
    let client = KvClient::new(vec![
        "127.0.0.1:6379".to_string(),
        "127.0.0.1:6380".to_string(),
        "127.0.0.1:6381".to_string(),
    ]);

    // Writes go to leader, reads distributed
    client.set("user:1".to_string(), "Alice".to_string()).await.unwrap();
    let value = client.get("user:1").await.unwrap();
    println!("Value: {:?}", value);
}
```

---

## Summary

**What You Built**: A production-grade distributed key-value store with persistence, replication, consistency, and automatic failover.

**Key Concepts Mastered**:
- **TCP client-server patterns**: Custom protocols, async networking
- **Write-Ahead Logging**: Durability and crash recovery
- **Replication**: Async (performance) vs Sync quorum (consistency)
- **Distributed consensus**: Leader election (simplified Raft)
- **Client patterns**: Connection pooling, smart routing, automatic failover

**Performance Journey**:
- **Milestone 1**: 50K writes/sec (memory-only)
- **Milestone 2**: 10K writes/sec (WAL durability cost)
- **Milestone 3**: 30K writes/sec (async replication)
- **Milestone 4**: 15K writes/sec (quorum writes for consistency)
- **Milestone 6**: 10K req/sec per client (connection pooling)

**Real-World Applications**: This architecture is the foundation of Redis, etcd, Consul, and every distributed database.
