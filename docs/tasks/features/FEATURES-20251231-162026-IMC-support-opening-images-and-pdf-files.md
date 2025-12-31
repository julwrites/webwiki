---
id: FEATURES-20251231-162026-IMC
status: pending
title: Support Opening Images and PDF Files
priority: medium
created: 2025-12-31 16:20:26
category: features
dependencies:
type: task
---

# Support Opening Images and PDF Files

## Context
The user reports they cannot open images or PDF files. The current implementation of `/api/wiki/{path}` blindly tries to read any file as a string and return it as a JSON `WikiPage` object. This fails for binary files like images and PDFs.

## Goals
1.  Enable the backend to serve raw files (images, PDFs) correctly.
2.  Update the frontend to display images (via `<img>`) and PDFs (via `<embed>` or `<iframe>`) when navigating to them.
3.  Ensure assets stored in `assets/` and relative links work correctly.

## Implementation Plan
1.  **Backend Changes**:
    *   Modify `read_page` in `backend/src/lib.rs` to detect file types.
    *   If the file is not Markdown (e.g., png, jpg, pdf), serve it as a raw file with the correct MIME type instead of a JSON object.
    *   Alternatively, rely on the `ServeDir` service for these files if the routing allows it.
2.  **Frontend Changes**:
    *   Update the viewer component to handle non-Markdown content.
    *   If the backend returns a raw file, the frontend router might need to handle it differently (e.g., by checking headers or extension before trying to parse JSON).
    *   Or, if the backend serves raw files, the frontend might just need to link to them directly for "Open" actions.
3.  **Verification**:
    *   Test opening a `.png` file.
    *   Test opening a `.pdf` file.
