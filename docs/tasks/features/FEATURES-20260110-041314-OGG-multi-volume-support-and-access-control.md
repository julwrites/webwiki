---
id: FEATURES-20260110-041314-OGG
status: completed
title: Multi-Volume Support and Access Control
priority: medium
created: 2026-01-10 04:13:14
category: features
dependencies: SECURITY-20260110-041311-DFZ
type: task
---

# Multi-Volume Support and Access Control

## Context
The application currently serves a single root directory defined by `WIKI_PATH`. The user wants to support multiple independent volumes and control access to them on a per-user basis.

## Objectives
1.  Allow mounting multiple directories as distinct "Volumes" (e.g., "Personal", "Work", "Public").
2.  Restrict access to specific volumes based on user identity.

## Requirements
*   **Volume Configuration:**
    *   Define volumes in a config file or environment variables.
    *   Example: `VOLUMES={"personal": "/data/personal", "work": "/data/work"}`.
*   **Access Control List (ACL):**
    *   Map Users to Volumes + Permissions (Read, Write).
    *   Could be part of the `users.json` structure from the Authentication task.
    *   Example User: `{ "username": "alice", "permissions": { "personal": "rw", "work": "r" } }`.
*   **API Changes:**
    *   Paths must include the volume identifier: `/api/wiki/{volume_id}/{path}`.
    *   `/api/tree` should return a list of available volumes or accept a volume query param.

## Implementation Plan
1.  **Config Update:**
    *   Update `AppState` to hold a `HashMap<String, PathBuf>` for volumes instead of a single `wiki_path`.
2.  **Routing Update (`backend/src/lib.rs`):**
    *   Change route patterns to capture `volume_id`.
    *   Middleware or Handler logic to check `user.permissions.get(volume_id)`.
3.  **Frontend Update:**
    *   Add a "Volume Switcher" in the UI (Sidebar).
    *   Update `FileTree` to fetch the tree for the current volume.
    *   Update URLs to include volume prefix (e.g., `/wiki/{volume}/{path}`).

## Verification
*   Configure 2 volumes.
*   Create User A (Access to Vol 1) and User B (Access to Vol 2).
*   Verify User A cannot access Vol 2.
