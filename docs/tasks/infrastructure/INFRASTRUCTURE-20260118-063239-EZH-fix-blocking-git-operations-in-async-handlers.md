---
id: INFRASTRUCTURE-20260118-063239-EZH
status: completed
title: Fix Blocking Git Operations in Async Handlers
priority: high
created: 2026-01-18 06:32:39
category: infrastructure
dependencies:
type: task
---

# Fix Blocking Git Operations in Async Handlers

The current implementation of Git operations in backend/src/git.rs uses synchronous git2 calls directly within async handlers. This blocks the Tokio runtime threads. All git2 operations should be wrapped in tokio::task::spawn_blocking.
