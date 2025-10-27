//! A simple thread pool implementation for learning purposes.
//! 
//! This crate provides a `ThreadPool` that can be used to execute tasks
//! concurrently.

use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender, channel},
};
use std::thread::{self, JoinHandle};


/// Represents a pool of threads that can execute jobs.
///
/// The pool has a fixed number of worker threads. When a `ThreadPool` is dropped,
/// it signals all workers to shut down and waits for them to finish.
#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

#[derive(Debug)]
pub enum PoolCreateError {
    NonValueZeroAllowed,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a New ThreadPool.
    ///
    /// The `size` parameter is the number of threads in the pool
    /// 
    /// # Arguments
    /// 
    /// * `size` - The number of worker threads to create. Must be greater than 0.
    /// 
    /// # Returns
    /// 
    /// A `Result` which is `OK` containing the new `ThreadPool` if the size is valid,
    /// or `Err` with a `PoolCreateError` if any error occurs when creating the threads.
    /// 
    /// # Example
    /// 
    /// ```
    /// # use hello::ThreadPool;
    /// 
    /// // Create a pool with 4 threads
    /// let pool = ThreadPool::build(4).unwrap();
    /// 
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreateError> {
        if size == 0 {
            return Err(PoolCreateError::NonValueZeroAllowed);
        }

        let (sendx, recx) = channel::<Job>();
        let receiver_clone = Arc::new(Mutex::new(recx));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver_clone)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sendx),
        })
    }
    /// Executes a new job in the thread pool.
    /// 
    /// The provided clousure will be send to an available worker thread
    /// and executed.
    /// 
    /// # Arguments
    /// 
    /// `f` - A clousure that will be executed by a thread. It must be `Sent`
    /// and have a `'Static` lifetime.
    /// 
    /// # Panics
    /// 
    /// This method will panic if the channel for sending jobs has been closed,
    /// which should not happen in normal operation.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");

                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
