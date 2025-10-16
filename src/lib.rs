use std::thread::JoinHandle;

pub struct ThreadPool {
   pub threads: Vec<JoinHandle<()>>
}

#[derive(Debug)]
pub enum PoolCreateError {
    NonValueZeroAllowed
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreateError> {

        if size == 0 {
            return Err(PoolCreateError::NonValueZeroAllowed);
        }

        let mut threads = Vec::with_capacity(size);

        for _ in 0..size {
            
        }

        Ok(ThreadPool {
            threads
        })
    }
}

impl ThreadPool {
    pub fn execute<F>(&self, f: F) 
    where 
        F: FnOnce() + Send + 'static
     {

    }
}
