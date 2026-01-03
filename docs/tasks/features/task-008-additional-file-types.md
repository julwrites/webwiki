---
id: task-008-additional-file-types
category: features
status: proposed
title: Support for Additional File Types
description: |
  Add support for displaying Diagrams (Graphviz, Mermaid, Draw.io) and Code files (JSON, TOML, YAML, OPML).
  Unrecognized small files should be treated as plaintext.
dependencies: []
---

# Support for Additional File Types

## Goal
Enable the wiki to render diagrams and display code files natively.

## Implementation Details

### Backend
- Update `read_page` to treat the following extensions as `WikiPage` (JSON wrapper):
  - Code: `.json`, `.toml`, `.yaml`, `.yml`, `.opml`
  - Diagrams: `.dot`, `.mermaid`, `.mmd`, `.drawio`, `.dio`
- Implement fallback for unknown small files (< 2MB) to be treated as plaintext/WikiPage.

### Frontend
- Update `WikiViewer` component:
  - Detect file extension from path.
  - Render code files in `<pre><code>`.
  - Render Mermaid diagrams using `mermaid.js`.
  - Render Graphviz diagrams using `viz.js`.
  - Render Draw.io diagrams using viewer (iframe or library).
- Add necessary CDN links to `frontend/index.html`.
- Add interop functions in `editor_interop.js`.

## Verification
- Create test files for each new type.
- Verify rendering in browser.
