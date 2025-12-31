---
id: TESTING-20251228-REF
title: Refactor Backend Tests
status: completed
priority: medium
dependencies: []
type: task
created: 2025-12-28T00:00:00Z
updated: 2025-12-28T00:00:00Z
---

# Refactor Backend Tests

## Description
Refactor backend code to split `main.rs` into `lib.rs` and `main.rs` to support integration testing. Move inline tests to `tests/` directory.

## Subtasks
- [x] Create `backend/src/lib.rs` and move core logic there.
- [x] Update `backend/src/main.rs` to use `lib.rs`.
- [x] Move tests to `backend/tests/`.
- [x] Verify all tests pass.
