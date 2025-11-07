//! A simple thread pool implementation for learning purposes.
//!
//! This crate provides a `ThreadPool` that can be used to execute tasks
//! concurrently.

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle},
};
use uuid::Uuid;
pub mod server;

/// Represents a pool of threads that can execute jobs.
///
/// The pool has a fixed number of worker threads. When a `ThreadPool` is dropped,
/// it signals all workers to shut down and waits for them to finish.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
    jobs: Arc<Mutex<HashMap<Uuid, JobMetadata>>>,
}

#[derive(Debug)]
pub enum PoolCreateError {
    NonValueZeroAllowed,
}

type JobPayload = Box<dyn FnOnce() -> Result<String, String> + Send + 'static>;

#[derive(Debug, Clone)]

enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct JobMetadata {
    state: JobStatus,
    result: Option<String>,
}

struct Job {
    id: Uuid,
    payload: JobPayload,
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Job")
            .field("id", &self.id)
            .field("payload", &"FnOnce(...)") // No podemos imprimir el closure
            .finish()
    }
}

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
    /// # use harbor::ThreadPool;
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
        let jobs: Arc<Mutex<HashMap<Uuid, JobMetadata>>> = Arc::new(Mutex::new(HashMap::new()));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver_clone),
                Arc::clone(&jobs),
            ));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sendx),
            jobs,
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
    pub fn execute<F>(&self, f: F) -> Uuid
    where
        F: FnOnce() -> Result<String, String> + Send + 'static,
    {
        let job = Job {
            id: Uuid::new_v4(),
            payload: Box::new(f),
        };

        let metadata = JobMetadata {
            result: None,
            state: JobStatus::Pending,
        };

        let job_id = job.id;
        {
            let mut jobs_map = self.jobs.lock().unwrap();
            jobs_map.insert(job_id, metadata);
        }

        if let Some(sender) = self.sender.as_ref() {
            if let Err(err) = sender.send(job) {
                eprintln!("No one worker active: {}", err);
            }
        }

        job_id
    }

    pub fn get_job_metadata(&self, job_id: Uuid) -> Option<JobMetadata> {
        let job = self.jobs.lock().unwrap();
        job.get(&job_id).cloned()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                match thread.join() {
                    Ok(thread) => thread,
                    Err(err) => {
                        eprintln!("The new thread could not be joined {:#?}", err);
                        continue;
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<Receiver<Job>>>,
        jobs: Arc<Mutex<HashMap<Uuid, JobMetadata>>>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = match receiver.lock() {
                    Ok(lock) => lock.recv(),
                    Err(err) => {
                        eprintln!("Avoid Mutex poisoning. {}", err);
                        return;
                    }
                };
                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        let job_id = job.id;
                        {
                            let mut jobs_map = jobs.lock().unwrap();
                            if let Some(metadata) = jobs_map.get_mut(&job_id) {
                                metadata.state = JobStatus::Processing;
                            }
                        }

                        let result = (job.payload)();

                        {
                            let mut jobs_map = jobs.lock().unwrap();
                            if let Some(metadata) = jobs_map.get_mut(&job_id) {
                                match result {
                                    Ok(res_str) => {
                                        metadata.state = JobStatus::Completed;
                                        metadata.result = Some(res_str.clone());
                                        println!(
                                            "Worker {} finished job '{}' successfully with result: {}",
                                            id, job.id, res_str
                                        );
                                    }
                                    Err(err_str) => {
                                        metadata.state = JobStatus::Failed(err_str.clone());
                                        metadata.result = Some(err_str.clone());
                                        println!(
                                            "Worker {} failed job '{}': {}",
                                            id, job.id, err_str
                                        );
                                    }
                                }
                            }
                        }
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn build_should_create_pool_with_valid_size() {
        let pool_result = ThreadPool::build(6);

        assert!(pool_result.is_ok())
    }

    #[test]
    fn build_should_return_error_when_size_is_zero() {
        let pool_result = ThreadPool::build(0);

        assert!(pool_result.is_err());

        assert!(matches!(
            pool_result.unwrap_err(),
            PoolCreateError::NonValueZeroAllowed
        ));
    }

    #[test]
    fn execute_should_run_job_in_pool() {
        let pool = ThreadPool::build(1);
        let (sender, receiver) = channel();
        let _ = pool.unwrap().execute(move || {
            sender.send("Job Executed").unwrap();
            Ok("Job Executed".to_string())
        });

        let receive_a_message = receiver.recv().unwrap();
        assert_eq!(receive_a_message, "Job Executed")
    }

    #[test]
    fn get_job_metadata_should_track_job_status() {
        let pool = ThreadPool::build(2).unwrap();
        let (tx, rx) = channel();

        let job_id = pool.execute(move || {
            thread::sleep(Duration::from_secs(10));
            tx.send(()).unwrap();
            Ok("Job Done".to_string())
        });

        let initial_metadata = pool.get_job_metadata(job_id).unwrap();
        assert!(
            matches!(initial_metadata.state, JobStatus::Pending)
                || matches!(initial_metadata.state, JobStatus::Processing)
        );
        assert_eq!(initial_metadata.result, None);

        rx.recv().unwrap();
        thread::sleep(Duration::from_secs(5));

        let final_metadata = pool.get_job_metadata(job_id).unwrap();
        assert!(matches!(final_metadata.state, JobStatus::Completed));
        assert_eq!(final_metadata.result, Some("Job Done".to_string()));

        let failed_job_id = pool.execute(move || Err("Job Failed".to_string()));

        thread::sleep(Duration::from_secs(5));

        let failed_metadata = pool.get_job_metadata(failed_job_id).unwrap();
        assert!(matches!(failed_metadata.state, JobStatus::Failed(_)));
        assert_eq!(failed_metadata.result, Some("Job Failed".to_string()));
    }
}
