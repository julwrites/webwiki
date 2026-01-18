---
id: FEATURES-20260118-042924-VUL
status: completed
title: File Deletion Support
priority: medium
created: 2026-01-18 04:29:24
category: features
dependencies: []
type: task
---

# File Deletion Support

## Description
Implement the ability to delete files from the wiki. This involves a new backend API endpoint and a UI element in the frontend to trigger the deletion.

## Goals
- Allow users to delete the currently viewed file.
- Persist deletion to disk (Git integration will handle the "staging" of the deletion automatically via the existing Git workflow or will need to be verified).
- Restrict deletion to users with `rw` permissions.

## Implementation Plan

### Backend
- [ ] Add `DELETE` route to `/api/wiki/{volume}/{*path}`.
- [ ] Implement `delete_page` handler in `backend/src/lib.rs`.
    - Check auth and permissions (`rw`).
    - Validate path (prevent traversal).
    - Use `std::fs::remove_file` (and potentially `remove_dir_all` if path is a directory).
- [ ] Add integration tests for deletion.

### Frontend
- [ ] Add "Delete" button to `WikiViewer` toolbar in `frontend/src/lib.rs`.
- [ ] Implement confirmation dialog (e.g., `gloo_dialogs::confirm`).
- [ ] Call `DELETE` API.
- [ ] On success:
    - Alert user.
    - Redirect to Home (`/`).
    - Trigger File Tree refresh (if possible, or rely on auto-refresh/re-render).

## Notes
- "Delete" means removing the file from the filesystem.
- The existing Git workflow (`/api/git/status`) should detect the deleted file as `Deleted`, allowing the user to commit the removal later.
