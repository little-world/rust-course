// Pattern 4: Connection Pooling
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

// Simulated connection
struct Connection {
    id: usize,
    created_at: Instant,
}

impl Connection {
    async fn new(id: usize) -> io::Result<Self> {
        // Simulate connection setup (DNS, TCP handshake, authentication)
        println!("  Creating connection {}...", id);
        sleep(Duration::from_millis(100)).await;
        Ok(Connection {
            id,
            created_at: Instant::now(),
        })
    }

    async fn execute(&self, query: &str) -> io::Result<String> {
        println!("  Connection {} executing: {}", self.id, query);
        sleep(Duration::from_millis(50)).await;
        Ok(format!("Result from connection {} (age: {:?})", self.id, self.created_at.elapsed()))
    }

    fn id(&self) -> usize {
        self.id
    }
}

// Simple connection pool
struct SimplePool {
    connections: Arc<Mutex<Vec<Connection>>>,
    max_size: usize,
    created_count: Arc<Mutex<usize>>,
}

impl SimplePool {
    async fn new(size: usize) -> io::Result<Self> {
        let mut connections = Vec::new();

        println!("Initializing pool with {} connections", size);
        // Pre-create all connections
        for i in 0..size {
            connections.push(Connection::new(i).await?);
        }
        println!("Pool initialized\n");

        Ok(SimplePool {
            connections: Arc::new(Mutex::new(connections)),
            max_size: size,
            created_count: Arc::new(Mutex::new(size)),
        })
    }

    async fn acquire(&self) -> io::Result<PooledConnection> {
        loop {
            let mut pool = self.connections.lock().await;

            if let Some(conn) = pool.pop() {
                // Got a connection from the pool
                return Ok(PooledConnection {
                    conn: Some(conn),
                    pool: self.connections.clone(),
                });
            }

            // No connections available—wait and retry
            drop(pool);
            println!("  No connections available, waiting...");
            sleep(Duration::from_millis(10)).await;
        }
    }

    fn clone(&self) -> Self {
        SimplePool {
            connections: self.connections.clone(),
            max_size: self.max_size,
            created_count: self.created_count.clone(),
        }
    }
}

// RAII wrapper: returns connection to pool on drop
struct PooledConnection {
    conn: Option<Connection>,
    pool: Arc<Mutex<Vec<Connection>>>,
}

impl PooledConnection {
    async fn execute(&self, query: &str) -> io::Result<String> {
        self.conn.as_ref().unwrap().execute(query).await
    }

    fn id(&self) -> usize {
        self.conn.as_ref().unwrap().id()
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            // Return connection to pool
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.lock().await.push(conn);
            });
        }
    }
}

// Demonstrate pool usage
async fn pool_demo() -> io::Result<()> {
    println!("=== Connection Pool Demo ===\n");

    let pool = SimplePool::new(3).await?;
    let start = Instant::now();

    let mut handles = vec![];

    // Spawn 10 tasks, but only 3 connections exist
    // Tasks will wait for connections to become available
    println!("Spawning 10 tasks with only 3 pooled connections\n");

    for i in 0..10 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let conn = pool.acquire().await.unwrap();
            let conn_id = conn.id();
            let result = conn.execute(&format!("Query {}", i)).await.unwrap();
            println!("Task {}: using connection {}, got: {}", i, conn_id, result);
            // Connection returned to pool when `conn` is dropped

            // Simulate some work after getting result
            sleep(Duration::from_millis(30)).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("\nTotal time: {:?}", start.elapsed());
    println!("10 tasks shared 3 connections efficiently");

    Ok(())
}

// Compare pooled vs non-pooled performance
async fn compare_pooled_vs_non_pooled() -> io::Result<()> {
    println!("\n=== Pooled vs Non-Pooled Comparison ===\n");

    // Non-pooled: create new connection for each request
    println!("Non-pooled (new connection per request):");
    let start = Instant::now();

    for i in 0..5 {
        let conn = Connection::new(i).await?;
        let _ = conn.execute(&format!("Query {}", i)).await?;
    }

    let non_pooled_time = start.elapsed();
    println!("Non-pooled time: {:?}", non_pooled_time);

    // Pooled: reuse connections
    println!("\nPooled (reusing connections):");
    let pool = SimplePool::new(2).await?;
    let start = Instant::now();

    for i in 0..5 {
        let conn = pool.acquire().await?;
        let _ = conn.execute(&format!("Query {}", i)).await?;
        // Connection automatically returned to pool
    }

    let pooled_time = start.elapsed();
    println!("Pooled time: {:?}", pooled_time);

    println!("\nSpeedup: {:.1}x faster with pooling",
             non_pooled_time.as_secs_f64() / pooled_time.as_secs_f64());

    Ok(())
}

// Demonstrate concurrent access to pool
async fn concurrent_pool_demo() -> io::Result<()> {
    println!("\n=== Concurrent Pool Access Demo ===\n");

    let pool = Arc::new(SimplePool::new(2).await?);
    let start = Instant::now();

    // All tasks start at once, competing for 2 connections
    let handles: Vec<_> = (0..6).map(|i| {
        let pool = pool.clone();
        tokio::spawn(async move {
            println!("Task {} requesting connection at {:?}", i, start.elapsed());
            let conn = pool.acquire().await.unwrap();
            println!("Task {} got connection {} at {:?}", i, conn.id(), start.elapsed());

            // Hold the connection for a bit
            sleep(Duration::from_millis(100)).await;

            println!("Task {} releasing connection {} at {:?}", i, conn.id(), start.elapsed());
            // Connection released on drop
        })
    }).collect();

    for handle in handles {
        handle.await.unwrap();
    }

    println!("\nTotal time: {:?}", start.elapsed());
    println!("6 tasks competed for 2 connections");

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    pool_demo().await?;
    compare_pooled_vs_non_pooled().await?;
    concurrent_pool_demo().await?;

    println!("\nConnection pool demo completed");
    Ok(())
}
