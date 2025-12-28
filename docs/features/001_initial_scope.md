# Feature Scope: Initial Release

## Core Features

### 1. Wiki Viewing
*   **Markdown Rendering:** Render GFM (GitHub Flavored Markdown).
*   **WikiLinks:** Support `[[Link]]` and `[[Link|Description]]` syntax.
*   **Navigation:** Clickable links navigate to other wiki pages. Red links (non-existent pages) lead to a creation page.
*   **Assets:** Display images referenced via `![alt](path)` or `[[path|alt]]`.

### 2. Editing (Vim Mode)
*   **Editor:** Raw Markdown editing.
*   **Keybindings:** Vim keybindings (Normal, Insert, Visual modes).
*   **Live Preview:** Side-by-side or togglable preview of the rendered content.
*   **Save:** Manual save button (persists to disk, does not commit).

### 3. File Management
*   **File Tree:** Visual representation of the wiki directory structure.
*   **Operations:**
    *   **Create:** New files and directories.
    *   **Rename:** Rename files and directories (auto-update links is a nice-to-have, but out of scope for v1 unless specified).
    *   **Delete:** Move files to trash or permanently delete.
    *   **Upload:** Drag-and-drop or file picker to upload images to `assets/images`.

### 4. Git Workflow
*   **Status Indicator:** Visual indication if the current file has uncommitted changes.
*   **Draft Mode:** Changes are saved to the filesystem immediately (or on save), acting as a "Draft".
*   **Commit Workflow:**
    *   **Trigger:** "Commit" button available when changes are detected.
    *   **Selection:** User is presented with a checklist of all modified/staged files.
    *   **Action:** User selects files to include, enters a Commit Message, and confirms.
    *   **Backend:** Stages selected files and creates a commit with the configured author.
*   **History:** View simple history/log for the current page.

### 5. Search
*   **Full Text Search:** Search bar to find content across all markdown files.
*   **Fuzzy Filename Search:** Quick jump to file (Ctrl-P style).

### 6. Authentication
*   **Identity:** Configuration for `user.name` and `user.email` for Git commits.
*   **Session:** Simple session to persist this identity.

## UX/UI Requirements
*   **Responsive:** Usable on desktop (primary) and mobile (viewing).
*   **Theme:** Clean, readable, potentially dark mode support (Vim users usually prefer dark).
