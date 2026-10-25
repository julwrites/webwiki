# Feature Specification: Vim Navigation, Undo/Redo, `<Ctrl+p><Ctrl+f>` Document Opener, & Configurable Shortcuts

## Overview

This specification details the keyboard navigation and shortcut architecture for WebWiki:
1. **Vim Undo & Redo (`u`, `r`, `<Ctrl+r>`)**: Supporting undo and redo in Vim Normal mode and via the editor toolbar, bridged directly to CodeMirror 6's history commands.
2. **Document Opener / Search Chord (`<Ctrl+p><Ctrl+f>`)**: Opening the Command Palette / document search via the two-stroke chord `<Ctrl+p><Ctrl+f>` both globally (View Mode) and in Vim Normal mode inside the editor.
3. **Vim Half-Page Navigation (`<Ctrl+d>` and `<Ctrl+u>`)**: Smooth half-page scrolling in Normal, Visual, and non-Vim modes.
4. **DateTime Insertion Chord (`<Ctrl+t><Ctrl+s>`)**: Multi-stroke chord to insert the current timestamp directly at the cursor location.
5. **Comprehensive Remapping & Live Reload**: Allowing all 11 global and editor shortcuts to be customized in the Settings Modal using single keys, modifier combinations, or multi-stroke chords, taking effect immediately without requiring a page reload.

---

## 1. Vim Undo & Redo (`u` and `r, <Ctrl+r>`)

- **Default Bindings**:
  - **Undo**: `u` (in Vim Normal mode)
  - **Redo**: `r, <Ctrl+r>` (in Vim Normal mode)
- **Implementation**:
  - `cmUndo` and `cmRedo` custom Vim actions invoke `@codemirror/commands`'s `undo(view)` and `redo(view)` (supporting count repeats via `actionArgs.repeat`).
  - Toolbar **Undo** and **Redo** buttons invoke `window.triggerUndo("code-editor")` and `window.triggerRedo("code-editor")`.

---

## 2. Document Opener / Command Palette Chord (`<Ctrl+p><Ctrl+f>`)

- **Default Binding**: `<Ctrl+p><Ctrl+f>` (replaces legacy `Ctrl+K, Ctrl+P`).
- **Behavior**:
  - **View Mode**: Handled by the Rust `ChordState` state machine in `frontend/src/hooks.rs`. Pressing `<Ctrl+p>` matches as a chord prefix and calls `e.prevent_default()` (blocking the browser Print dialog), and pressing `<Ctrl+f>` within 1500ms opens the Command Palette.
  - **Editor (Vim Normal Mode Only)**: Mapped via `Vim.mapCommand("<C-p><C-f>", "action", "openCommandPalette", {}, { context: "normal" })`. Does not fire in Vim Insert or Visual mode.
  - **Focus Restoration**: Closing the Command Palette (or Settings / History / Commit modals) while editing automatically restores focus to `#code-editor`.

---

## 3. Remapping & Live Configuration

All 11 shortcuts are stored in `LocalStorage` under `"shortcuts"` and configurable via the **Settings** modal:
- **Global & Navigation**: Open Document / Search, Edit Page, Save Page, Cancel / Exit Edit, Toggle Side Panel, Copy Link.
- **Editor & Vim Mode**: Undo (Vim Normal), Redo (Vim Normal), Scroll Half-Page Down, Scroll Half-Page Up, Insert Date/Time.
- **Live Reload**: Saving Settings validates all shortcut syntax and conflict rules, updates `Layout` state in Yew, and calls `window.applyEditorShortcuts()` to unmap previous custom Vim bindings and reconfigure active CodeMirror 6 instances via `Compartment`.
