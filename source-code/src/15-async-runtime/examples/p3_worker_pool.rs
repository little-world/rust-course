//! Pattern 3: Async/Await Patterns
//! Worker pool pattern
//!
//! Run with: cargo run --example p3_worker_pool

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

type Task = Box<dyn FnOnce() + Send + 'static>;

struct WorkerPool {
    sender: mpsc::Sender<Task>,
}

impl WorkerPool {
    fn new(num_workers: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Task>(100);
        let rx = Arc::new(tokio::sync::Mutex::new(rx));

        for i in 0..num_workers {
            let rx = Arc::clone(&rx);
            tokio::spawn(async move {
                loop {
                    let task = {
                        let mut guard = rx.lock().await;
                        guard.recv().await
                    };
                    match task {
                        Some(task) => {
                            println!("Worker {} executing task", i);
                            task();
                        }
                        None => break, // Channel closed
                    }
                }
            });
        }

        Self { sender: tx }
    }

    async fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(task)).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let pool = WorkerPool::new(4);
    for i in 0..10 {
        pool.submit(move || println!("Task {} executed", i)).await;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
