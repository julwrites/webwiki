---
id: FEATURES-20251227-135925-NMV
title: Backend Core (File Ops)
status: verified
priority: high
dependencies: [FOUNDATION-20251227-135911-DRJ]
type: task
created: 2025-12-27T13:59:25Z
updated: 2025-12-27T13:59:25Z
---

# Backend Core (File Ops)

## Description
Implement the core file system operations in the Axum backend to support reading and writing markdown files.

## Subtasks
- [ ] Implement `common` crate for shared types (e.g., `WikiPage`).
- [ ] Implement `GET /api/wiki/*path` to read files.
- [ ] Implement `PUT /api/wiki/*path` to write files.
- [ ] Implement `GET /api/tree` to return directory structure.
- [ ] Ensure path safety (prevent directory traversal).
- [ ] Add basic Markdown parsing (if needed server-side) or pass raw content.
