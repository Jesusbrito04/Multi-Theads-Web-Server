use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender, channel},
};
use std::thread::{self, JoinHandle};

#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

#[derive(Debug)]
pub enum PoolCreateError {
    NonValueZeroAllowed,
}

impl ThreadPool {
    /// Build a ThreadPool.
    ///
    /// The size is the number of threads in the pool.
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
            sender: sendx,
        })
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: JoinHandle<Arc<Mutex<Receiver<Job>>>>,
}

impl Worker {
    fn new(id: usize, recx: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let job = recx.lock().unwrap().recv().unwrap();
                println!("Worker {id} got a job; executing.");
                job();
            }
        });
        Worker { id: id, thread }
    }
}
