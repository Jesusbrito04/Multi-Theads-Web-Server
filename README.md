# Harbor: An Educational ThreadPool in Rust

`Harbor` is a thread pool project written from scratch in Rust, following the principles of Chapter 20 of The Rust Programming Language book. The main goal of this project was to gain a deeper understanding of concurrency, safe API design, and shared state management in Rust.

This project was developed as a learning tool and serves as a practical demonstration of these concepts.

## Implemented Features

- **Fixed-Size Thread Pool:** Creates a predefined number of worker threads to process tasks concurrently.
- **Closure-Based Job Execution:** Allows users to safely and easily submit code to be executed on a worker thread.
- **Job Tracking System:**
    - Each submitted job is assigned a unique `UUID`.
    - Keeps a record of each job's status: `Pending`, `Processing`, `Completed`, or `Failed`.
    - The result (both success and error) of each job is stored.
- **Status Query API:** Includes a public method (`get_job_metadata`) to query the status and result of a job at any time using its `UUID`.
- **Example Web Server:** Includes a binary that runs a multi-threaded web server using the `ThreadPool` to handle incoming connections.

## How to Run

To run the examples, first clone the repository.

### Main Server with `Harbor`

This command will compile and run the main web server, which uses our custom `harbor` ThreadPool.

```bash
cargo run
```
The server will be listening at `http://127.0.0.1:7878`.

### Comparison Server

This project also includes a second binary to compare the `harbor` implementation with the popular `threadpool` crate.

```bash
cargo run --bin external_server
```
This server will be listening at `http://127.0.0.1:7879`.

## Lessons Learned: `Harbor` vs. the `threadpool` Crate

A key part of this project was comparing our implementation with a production-grade library like `threadpool`.

- **Creation API:** `harbor` returns a `Result` on creation to enforce error handling (e.g., a size of 0), whereas `threadpool` simplifies its API and panics in that case. This is a trade-off between explicit robustness and API simplicity.

- **Job Tracking:** `harbor` was designed with tracking as a core feature, returning a `UUID` for every job. This allows for granular control (querying, and potentially canceling or prioritizing). `threadpool` offers a "fire-and-forget" API (`execute`), optimized for simplicity and performance when tracking is not needed.

- **Job Cancellation:** We theoretically discussed the complexity of implementing job cancellation, concluding that **cooperative cancellation** (where the job periodically checks a flag to cancel itself) is the safest and most robust approach, although it impacts the design of the execution API.

This project demonstrates the value of building from scratch to learn, but also the importance of analyzing existing solutions to understand different design philosophies and their trade-offs.
