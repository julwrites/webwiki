# Proposed Changes

* Feature/Enhancement: Add "Rename Page" and "Delete Page" commands to the Command Palette to improve the Vim-like keyboard-centric workflow. Also, ensuring these actions are properly accessible without having to click buttons in the UI.

* User Impact: Users can quickly rename or delete the current file they are viewing without leaving the keyboard or having to scroll the page to find the actions. This strongly enhances the "Vim-like" experience.

* Technical Approach:
  - Add `RenamePage` and `DeletePage` variants to `CommandType` or handle them as `Action` callbacks in `command_palette.rs`.
  - Pass down `on_rename` and `on_delete` callbacks to `CommandPalette` from `WikiApp` (in `frontend/src/lib.rs`).
  - Add static commands for renaming and deleting the current page.
