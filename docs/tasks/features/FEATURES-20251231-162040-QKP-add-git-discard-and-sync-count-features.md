---
id: FEATURES-20251231-162040-QKP
status: pending
title: Add Git Discard and Sync Count Features
priority: medium
created: 2025-12-31 16:20:40
category: features
dependencies:
type: task
---

# Add Git Discard and Sync Count Features

## Context
The user requested two Git-related features:
1.  **Pending Commits Count**: Show how many commits are pending for sync (ahead of remote).
2.  **Remove Commits (Discard Changes)**: Allow the user to discard uncommitted changes or undo commits. Given the context "remove commits", it likely refers to discarding uncommitted changes shown in the list.

## Goals
1.  Display the number of commits ahead of origin in the UI.
2.  Add a mechanism to discard uncommitted changes (restore files to HEAD state).

## Implementation Plan
1.  **Backend (`backend/src/git.rs`)**:
    *   Add logic to `get_status` or a new endpoint to count commits ahead of the tracked remote branch (e.g., `HEAD..origin/main`).
    *   Add an endpoint `/api/git/restore` (or similar) to discard changes for specific files or all files.
2.  **Frontend (`frontend/src/commit_modal.rs` / `frontend/src/components/git_status.rs`)**:
    *   Fetch and display the "Pending Sync" count.
    *   Add a "Discard" button in the Commit Modal for each file or globally.
3.  **Verification**:
    *   Make a commit, check if "Pending Sync" count increases.
    *   Modify a file, then click "Discard", and verify the file reverts to the original state.
