---
id: FEATURES-20251227-135926-BQW
title: Git Integration
status: completed
priority: medium
dependencies: [FEATURES-20251227-135925-BXW]
type: task
created: 2025-12-27T13:59:26Z
updated: 2025-12-27T13:59:26Z
---

# Git Integration

## Description
Implement the Git workflow for version control.

## Subtasks
- [ ] Add `git2` crate to backend.
- [ ] Implement `GET /api/git/status` to check file status.
- [ ] Implement `POST /api/git/commit` to stage and commit.
- [ ] Frontend: Add "Commit" button/modal.
- [ ] Frontend: File selection checklist in Commit Modal.
- [ ] Backend: Handle `user.name` and `user.email` configuration.
