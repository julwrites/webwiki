---
id: INFRASTRUCTURE-20260401-065809-HTD
status: in_progress
title: Optimize synchronous I/O in backend handlers
priority: medium
created: 2026-04-01 06:58:09
category: infrastructure
dependencies:
type: task
---

# Optimize synchronous I/O in backend handlers

## 💡 What
The optimization involves replacing synchronous `std::fs` operations with asynchronous `tokio::fs` calls in the Axum request handlers: `serve_wiki_asset`, `read_page`, and `write_page`. Additionally, the recursive `build_file_tree` function, which performs synchronous directory traversal, will be offloaded to a background thread using `tokio::task::spawn_blocking`.

## 🎯 Why
Axum handlers run on a multi-threaded Tokio runtime. Performing synchronous I/O directly in these handlers blocks the executor thread. If multiple requests perform blocking I/O simultaneously, it can lead to "thread starvation," where the runtime is unable to schedule other tasks, significantly increasing latency and reducing throughput. Using `tokio::fs` allows the thread to yield to other tasks while waiting for I/O, improving concurrency. For complex, recursive I/O like directory listing, `spawn_blocking` is the preferred way to handle synchronous code without stalling the async executor.

## 📊 Measured Improvement
Establishing a formal baseline is currently impractical because the development environment is offline and lacks several cached dependencies (e.g., `async-trait`), preventing a successful build and execution of a benchmarking suite.

However, the theoretical benefit is high:
- **Reduced Latency:** Threads are not blocked, allowing the runtime to handle other concurrent requests.
- **Improved Throughput:** The system can handle more simultaneous I/O-bound requests.
- **Better Stability:** Prevents potential deadlocks or severe slowdowns under heavy I/O load.
