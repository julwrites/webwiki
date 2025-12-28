---
id: FEATURES-20251227-135925-SGY
title: Frontend Core (Viewer & Router)
status: verified
priority: high
dependencies: [FEATURES-20251227-135925-NMV]
type: task
created: 2025-12-27T13:59:25Z
updated: 2025-12-27T13:59:25Z
---

# Frontend Core (Viewer & Router)

## Description
Build the basic Yew application structure to view wiki pages.

## Subtasks
- [ ] Implement Yew Router for `/wiki/*` paths.
- [ ] Create `WikiViewer` component.
- [ ] Fetch content from Backend API.
- [ ] Implement Markdown rendering (using a Rust crate compiled to WASM or JS interop).
- [ ] Handle WikiLinks (intercept clicks and route internally).
- [ ] Implement Sidebar File Tree.
