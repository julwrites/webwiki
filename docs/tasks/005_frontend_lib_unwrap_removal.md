---
title: Remove unwrap calls in frontend lib.rs
status: review_requested
---

# Remove unwrap calls in frontend lib.rs

## Overview
This task improves frontend stability by removing instances of `unwrap()` from `frontend/src/lib.rs` which can cause the WebAssembly application to panic.

## Sub-tasks
- ✅ Fix `unwrap()` on `serde_json::to_string` and `Request::body()` in `lib.rs`.
- ✅ Properly handle errors using `match` or `if let`.

## Review Plan
- High-Level Summary: Removed `unwrap()` from request building in `frontend/src/lib.rs`.
- Impact Assessment: Enhances frontend robustness.
