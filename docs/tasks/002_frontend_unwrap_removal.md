---
title: Remove unwrap calls in frontend
status: review_requested
---

# Remove unwrap calls in frontend

## Overview
This task improves frontend stability by removing instances of `unwrap()` from `frontend/src/commit_modal.rs` which can cause the WebAssembly application to panic.

## Sub-tasks
- ✅ Fix `unwrap()` on `serde_json::to_string` and `Request::post().body()` during commit in `commit_modal.rs`.
- ✅ Fix `unwrap()` on `serde_json::to_string` and `Request::post().body()` during discard in `commit_modal.rs`.
- ✅ Properly handle errors and propagate them to the UI error state.

## Review Plan
- High-Level Summary: The `unwrap()` calls on `serde_json::to_string(&req).unwrap()` and `Request::post().body().unwrap()` in `frontend/src/commit_modal.rs` have been removed. Errors are now caught using `match` statements and set in the UI using the component's `error` state.
- Impact Assessment: Enhances frontend robustness. A failed serialization or request build will no longer crash the entire frontend, but correctly show an error inside the commit modal instead. No dependencies added.
