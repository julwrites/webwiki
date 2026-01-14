---
id: FOUNDATION-20260110-041317-DTT
status: completed
title: Technical Debt: Memory Leaks and Git Concurrency
priority: medium
created: 2026-01-10 04:13:17
category: foundation
dependencies:
type: task
---

# Technical Debt: Memory Leaks and Git Concurrency

## Context
During code review, two technical issues were identified:
1.  **Memory Leak in Frontend:** The `Editor` component creates a closure for the `setupEditor` callback and calls `closure.forget()`. This leaks memory every time the editor is mounted/unmounted.
2.  **Git Concurrency:** The `GitState` is shared across threads/requests. While `libgit2` is generally thread-safe, concurrent write operations (commits) or checking status while writing might lead to race conditions or index locking errors if not synchronized.

## Objectives
1.  Fix the memory leak in the Editor component.
2.  Ensure thread safety for Git operations.

## Requirements
*   **Editor:**
    *   Properly manage the lifetime of the WASM closure.
    *   Store the closure in the component state or a `ref` and drop it when the component unmounts.
*   **Git:**
    *   Wrap `git2::Repository` access or the critical sections in a `Mutex` or `RwLock` if necessary.
    *   Alternatively, use a job queue for write operations if high contention is expected (likely overkill, Mutex is sufficient for now).

## Implementation Plan
1.  **Frontend (`frontend/src/lib.rs` - `Editor`):**
    *   Use `use_state` or `use_mut_ref` to store the `Closure`.
    *   In the `use_effect_with` cleanup function, ensure the closure is dropped.
    *   *Note:* The `setupEditor` JS function might need adjustment to allow unregistering the callback if it attaches global event listeners.
2.  **Backend (`backend/src/git.rs`):**
    *   Review `GitState`. If it just holds `PathBuf`, the repo is opened per request. `git2::Repository::open` is safe, but operations on the same working directory might conflict.
    *   Introduce a `tokio::sync::Mutex<()>` in `GitState` to serialize operations that modify the index/HEAD (commit, restore, sync).

## Verification
*   **Memory:** Monitor memory usage in browser dev tools while toggling the editor multiple times.
*   **Git:** Run a stress test script that fires multiple `/api/git/commit` requests simultaneously.

## Verification Details (2026-01-13)
*   **Frontend Memory Leak:** Verified via static analysis of `frontend/src/lib.rs`. The `Editor` component uses `use_mut_ref` to store the closure and explicit `*closure_ref.borrow_mut() = None` in the cleanup function, which prevents the memory leak associated with `closure.forget()`.
*   **Git Concurrency:** Verified via a new integration test `test_git_concurrency` in `backend/tests/integration_tests.rs`. The test spawns 25 concurrent tasks (20 readers, 5 writers) against the backend. The test passed, confirming that the `tokio::sync::Mutex` in `GitState` correctly serializes access to the underlying `git2::Repository`, preventing race conditions and potential panics/errors from libgit2.
