// Pattern 4: Concurrency Patterns - Thread Pool, Producer-Consumer, Fork-Join, Actor, Async/Await
// Demonstrates patterns for parallel and asynchronous execution.

use rayon::prelude::*;
use std::sync::mpsc::{self, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio;

// ============================================================================
// Example: Thread Pool Pattern
// ============================================================================

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    #[allow(dead_code)]
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(job)).unwrap();
    }
}

struct Worker {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Worker {} executing job", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} shutting down", id);
                    break;
                }
            }
        });

        Worker { id, thread }
    }
}

fn thread_pool_example() {
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        pool.execute(move || {
            println!("  Task {} running", i);
            thread::sleep(Duration::from_millis(50));
        });
    }

    thread::sleep(Duration::from_millis(500));
}

// ============================================================================
// Example: Rayon for Data Parallelism
// ============================================================================

fn rayon_example() {
    // Parallel iterator (much simpler than manual thread pool)
    let numbers: Vec<i32> = (0..1000).collect();
    let sum: i32 = numbers.par_iter().map(|&x| x * 2).sum();
    println!("Rayon parallel sum: {}", sum);

    // Parallel sort
    let mut data: Vec<i32> = (0..100).rev().collect();
    data.par_sort();
    println!("Rayon parallel sort: first 5 = {:?}", &data[..5]);
}

// ============================================================================
// Example: Producer-Consumer Pattern
// ============================================================================

fn producer(tx: mpsc::Sender<i32>) {
    for i in 0..5 {
        println!("Producing {}", i);
        tx.send(i).unwrap();
        thread::sleep(Duration::from_millis(50));
    }
}

fn consumer(rx: mpsc::Receiver<i32>) {
    for item in rx {
        println!("  Consuming {}", item);
        thread::sleep(Duration::from_millis(100));
    }
}

fn producer_consumer_example() {
    let (tx, rx) = mpsc::channel();

    let producer_handle = thread::spawn(move || producer(tx));
    let consumer_handle = thread::spawn(move || consumer(rx));

    producer_handle.join().unwrap();
    consumer_handle.join().unwrap();
}

// ============================================================================
// Example: Multiple Producers, Single Consumer
// ============================================================================

fn multiple_producers_example() {
    let (tx, rx) = mpsc::channel();

    for id in 0..3 {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            for i in 0..3 {
                tx_clone.send((id, i)).unwrap();
                thread::sleep(Duration::from_millis(30));
            }
        });
    }
    drop(tx); // Drop original to allow channel to close

    let consumer = thread::spawn(move || {
        for (id, item) in rx {
            println!("Producer {} sent {}", id, item);
        }
    });

    consumer.join().unwrap();
}

// ============================================================================
// Example: Bounded Channel for Backpressure
// ============================================================================

fn bounded_channel_example() {
    let (tx, rx) = sync_channel(3); // Buffer size of 3

    let producer = thread::spawn(move || {
        for i in 0..6 {
            println!("Sending {}", i);
            tx.send(i).unwrap(); // Blocks if buffer full
        }
    });

    let consumer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100)); // Delay consumer
        for item in rx {
            println!("  Received {}", item);
            thread::sleep(Duration::from_millis(50));
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

// ============================================================================
// Example: Fork-Join Pattern
// ============================================================================

fn parallel_sum(data: &[i32]) -> i32 {
    const THRESHOLD: usize = 100;

    if data.len() <= THRESHOLD {
        // Base case: sequential sum
        data.iter().sum()
    } else {
        // Fork: split into two halves
        let mid = data.len() / 2;
        let (left, right) = data.split_at(mid);

        let left_data = left.to_vec();
        let handle = thread::spawn(move || parallel_sum(&left_data));

        let right_sum = parallel_sum(right);
        let left_sum = handle.join().unwrap();

        // Join: combine results
        left_sum + right_sum
    }
}

fn fork_join_example() {
    let data: Vec<i32> = (1..=1000).collect();
    let total = parallel_sum(&data);
    println!("Fork-join sum: {}", total);
}

// ============================================================================
// Example: Rayon Fork-Join
// ============================================================================

fn rayon_sum(data: &[i32]) -> i32 {
    data.par_iter().sum() // Automatically parallelizes
}

fn quicksort<T: Ord + Send + Sync + Clone>(data: Vec<T>) -> Vec<T> {
    if data.len() <= 1 {
        return data;
    }

    let pivot = data[0].clone();
    let rest: Vec<T> = data.into_iter().skip(1).collect();

    let (less, greater): (Vec<_>, Vec<_>) = rest.into_par_iter().partition(|x| x < &pivot);

    // Parallel recursive calls
    let (sorted_less, sorted_greater) = rayon::join(|| quicksort(less), || quicksort(greater));

    let mut result = sorted_less;
    result.push(pivot);
    result.extend(sorted_greater);
    result
}

fn rayon_fork_join_example() {
    let data: Vec<i32> = (1..=100).collect();
    let sum = rayon_sum(&data);
    println!("Rayon fork-join sum: {}", sum);

    let unsorted = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
    let sorted = quicksort(unsorted);
    println!("Rayon quicksort: {:?}", sorted);
}

// ============================================================================
// Example: Actor Pattern
// ============================================================================

enum AccountMessage {
    Deposit(u64),
    Withdraw(u64),
    GetBalance(mpsc::Sender<u64>),
    Shutdown,
}

struct BankAccount {
    balance: u64,
    receiver: mpsc::Receiver<AccountMessage>,
}

impl BankAccount {
    fn new(initial: u64) -> (mpsc::Sender<AccountMessage>, thread::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let account = BankAccount {
            balance: initial,
            receiver: rx,
        };

        let handle = thread::spawn(move || account.run());
        (tx, handle)
    }

    fn run(mut self) {
        for msg in self.receiver {
            match msg {
                AccountMessage::Deposit(amount) => {
                    self.balance += amount;
                    println!("Deposited {}. Balance: {}", amount, self.balance);
                }
                AccountMessage::Withdraw(amount) => {
                    if self.balance >= amount {
                        self.balance -= amount;
                        println!("Withdrew {}. Balance: {}", amount, self.balance);
                    } else {
                        println!("Insufficient funds");
                    }
                }
                AccountMessage::GetBalance(reply) => {
                    reply.send(self.balance).ok();
                }
                AccountMessage::Shutdown => {
                    println!("Shutting down account");
                    break;
                }
            }
        }
    }
}

fn actor_example() {
    let (handle, join_handle) = BankAccount::new(100);

    handle.send(AccountMessage::Deposit(50)).unwrap();
    handle.send(AccountMessage::Withdraw(30)).unwrap();

    let (tx, rx) = mpsc::channel();
    handle.send(AccountMessage::GetBalance(tx)).unwrap();
    let balance = rx.recv().unwrap();
    println!("Final balance: {}", balance);

    handle.send(AccountMessage::Shutdown).unwrap();
    join_handle.join().unwrap();
}

// ============================================================================
// Example: Async/Await Pattern
// ============================================================================

async fn fetch_user(id: u64) -> String {
    // Simulated async operation
    tokio::time::sleep(Duration::from_millis(50)).await;
    format!("User {}", id)
}

async fn fetch_posts(user: &str) -> Vec<String> {
    tokio::time::sleep(Duration::from_millis(50)).await;
    vec![format!("{}'s post 1", user), format!("{}'s post 2", user)]
}

async fn display_user_data(id: u64) {
    // Sequential async operations
    let user = fetch_user(id).await;
    println!("Fetched: {}", user);

    let posts = fetch_posts(&user).await;
    for post in posts {
        println!("  - {}", post);
    }
}

// ============================================================================
// Example: Parallel Async with join!
// ============================================================================

async fn parallel_fetch() {
    // Wait for all to complete
    let (user1, user2, user3) = tokio::join!(fetch_user(1), fetch_user(2), fetch_user(3),);

    println!("Parallel fetch: {}, {}, {}", user1, user2, user3);
}

// ============================================================================
// Example: Select Pattern
// ============================================================================

async fn timeout_example() {
    let fetch = fetch_user(1);
    let timeout = tokio::time::sleep(Duration::from_millis(100));

    tokio::select! {
        user = fetch => println!("Got user: {}", user),
        _ = timeout => println!("Timeout!"),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(Mutex::new(0));

        for _ in 0..4 {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                let mut num = counter.lock().unwrap();
                *num += 1;
            });
        }

        thread::sleep(Duration::from_millis(200));
        assert_eq!(*counter.lock().unwrap(), 4);
    }

    #[test]
    fn test_rayon_sum() {
        let data: Vec<i32> = (1..=100).collect();
        let sum: i32 = data.par_iter().sum();
        assert_eq!(sum, 5050);
    }

    #[test]
    fn test_rayon_parallel_map() {
        let data: Vec<i32> = vec![1, 2, 3, 4, 5];
        let doubled: Vec<i32> = data.par_iter().map(|&x| x * 2).collect();
        assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_producer_consumer_channel() {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            for i in 0..5 {
                tx.send(i).unwrap();
            }
        });

        let received: Vec<i32> = rx.iter().collect();
        assert_eq!(received, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_bounded_channel() {
        let (tx, rx) = sync_channel(2);

        tx.send(1).unwrap();
        tx.send(2).unwrap();
        // tx.send(3) would block

        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
    }

    #[test]
    fn test_parallel_sum() {
        let data: Vec<i32> = (1..=1000).collect();
        let expected: i32 = data.iter().sum();
        let result = parallel_sum(&data);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_quicksort() {
        let data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        let sorted = quicksort(data.clone());
        let mut expected = data;
        expected.sort();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_actor_deposit() {
        let (handle, join_handle) = BankAccount::new(100);

        handle.send(AccountMessage::Deposit(50)).unwrap();

        let (tx, rx) = mpsc::channel();
        handle.send(AccountMessage::GetBalance(tx)).unwrap();
        let balance = rx.recv().unwrap();

        assert_eq!(balance, 150);

        handle.send(AccountMessage::Shutdown).unwrap();
        join_handle.join().unwrap();
    }

    #[test]
    fn test_actor_withdraw() {
        let (handle, join_handle) = BankAccount::new(100);

        handle.send(AccountMessage::Withdraw(30)).unwrap();

        let (tx, rx) = mpsc::channel();
        handle.send(AccountMessage::GetBalance(tx)).unwrap();
        let balance = rx.recv().unwrap();

        assert_eq!(balance, 70);

        handle.send(AccountMessage::Shutdown).unwrap();
        join_handle.join().unwrap();
    }

    #[tokio::test]
    async fn test_async_fetch_user() {
        let user = fetch_user(1).await;
        assert_eq!(user, "User 1");
    }

    #[tokio::test]
    async fn test_async_fetch_posts() {
        let posts = fetch_posts("User 1").await;
        assert_eq!(posts.len(), 2);
        assert!(posts[0].contains("User 1"));
    }

    #[tokio::test]
    async fn test_parallel_fetch() {
        let (u1, u2) = tokio::join!(fetch_user(1), fetch_user(2));
        assert_eq!(u1, "User 1");
        assert_eq!(u2, "User 2");
    }
}

#[tokio::main]
async fn main() {
    println!("Pattern 4: Concurrency Patterns");
    println!("================================\n");

    println!("=== Thread Pool Pattern ===");
    thread_pool_example();
    println!();

    println!("=== Rayon Data Parallelism ===");
    rayon_example();
    println!();

    println!("=== Producer-Consumer Pattern ===");
    producer_consumer_example();
    println!();

    println!("=== Multiple Producers ===");
    multiple_producers_example();
    println!();

    println!("=== Bounded Channel (Backpressure) ===");
    bounded_channel_example();
    println!();

    println!("=== Fork-Join Pattern ===");
    fork_join_example();
    println!();

    println!("=== Rayon Fork-Join ===");
    rayon_fork_join_example();
    println!();

    println!("=== Actor Pattern ===");
    actor_example();
    println!();

    println!("=== Async/Await Pattern ===");
    display_user_data(1).await;
    println!();

    println!("=== Parallel Async (join!) ===");
    parallel_fetch().await;
    println!();

    println!("=== Select Pattern (Timeout) ===");
    timeout_example().await;
}
