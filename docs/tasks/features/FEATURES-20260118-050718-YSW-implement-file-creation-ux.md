---
id: FEATURES-20260118-050718-YSW
status: completed
title: Implement File Creation UX
priority: medium
created: 2026-01-18 05:07:18
category: features
dependencies:
type: task
---

# Implement File Creation UX

## Goal
Provide a user-friendly way to create new files in the Wiki, as currently users must manually manipulate the URL to create a new page.

## Implementation Plan

### 1. Sidebar "New File" Button
*   **File**: `frontend/src/lib.rs`
*   **Component**: `Layout` (or `Sidebar`)
*   **Action**: Add a button in the sidebar (e.g., in `.sidebar-controls` or `.sidebar-header`).
*   **Behavior**:
    *   On click, prompt the user for a file path (using `gloo_dialogs::prompt`).
    *   Validate the input (not empty).
    *   Navigate to the new path using `navigator.push`.

### 2. Command Palette "Create File"
*   **File**: `frontend/src/components/command_palette.rs`
*   **Action**: Add a static command "Create New File".
*   **Behavior**: Same as the sidebar button (prompt and navigate).

## Verification
*   User can click "New File" in the sidebar.
*   User enters "my_new_page.md".
*   Browser navigates to `/wiki/{volume}/my_new_page.md`.
*   User sees "Page Not Found" and can click "Edit" to create the content.
