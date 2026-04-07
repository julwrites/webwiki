---
status: completed
category: tech-debt
---
# Remove unwraps from git.rs

## Description
There are a few `unwrap()` calls in `backend/src/git.rs` that can cause the server to panic if git operations fail or encounter unexpected states.
They should be replaced with proper error handling.

## Tasks
- [x] Remove `unwrap()` in `pull` method on `repo.index().unwrap().has_conflicts()`.
- [x] Remove `unwrap()` in `commit` method when resolving `repo.head().target().unwrap()` and `repo.find_commit(target).unwrap()`.
- [x] Ensure all errors are bubbled up or returned as `StatusCode::INTERNAL_SERVER_ERROR`.
