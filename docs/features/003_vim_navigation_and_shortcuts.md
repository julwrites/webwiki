# Feature Specification: Vim Navigation Commands & DateTime Insertion Shortcut

## Overview

This specification details two editor enhancements for WebWiki:
1. **Vim Half-Page Navigation (`Ctrl+D` and `Ctrl+U`)**: Enabling smooth half-page scrolling in both Normal and Visual modes, as well as in standard non-Vim editor mode.
2. **DateTime Insertion Chord (`<C-t><C-s>`)**: Providing a convenient multi-stroke chord to insert the current timestamp directly at the cursor location.

---

## 1. Feature 1: Vim Movement Commands (`Ctrl+D`, `Ctrl+U`)

### 1.1 Problem Statement
When pressing `Ctrl+D` in the WebWiki editor, characters ahead of the cursor were deleted instead of scrolling down. Additionally, `Ctrl+U` did not reliably navigate up half a page.

### 1.2 Root Cause
In CodeMirror 6, `defaultKeymap` contains macOS Emacs-compatible keybindings mapping `Ctrl-d` to `deleteCharForward`. Because `defaultKeymap` was registered before Vim in the editor extension list, `Ctrl-d` was intercepted before reaching Vim mode. Furthermore, the default `moveByScroll` implementation in `@replit/codemirror-vim` relies on DOM coordinate measurements that return `null` for unrendered lines outside the active viewport.

### 1.3 Implementation
1. Reordered extensions so that `vim()` is loaded before other keymaps.
2. Filtered out `Ctrl-d` (`deleteCharForward`) and `Ctrl-t` (`transposeChars`) from `defaultKeymap` (accounting for both `key` and `mac` platform properties).
3. Implemented `scrollHalfPage(view, forward)` using CodeMirror's native `view.moveVertically` and `EditorView.scrollIntoView`.
4. Registered custom Vim actions `scrollHalfPageDown` and `scrollHalfPageUp` and mapped them to `<C-d>` and `<C-u>` in both Normal and Visual modes.
5. Added fallback bindings for `Ctrl-d` and `Ctrl-u` to the standard editor keymap.

---

## 2. Feature 2: DateTime Insertion Chord (`<C-t><C-s>`)

### 2.1 Desired Behavior
Pressing the chord `Ctrl-t` followed by `Ctrl-s` (`<C-t><C-s>`) inserts the current formatted date and time (`YYYY-MM-DD Day HH:MM AM/PM`) at the current cursor position.

### 2.2 Implementation
1. In `editor_interop.js`, updated `window.insertDateTime` to accept either an element ID or an `EditorView` instance.
2. Defined a custom Vim action `"insertDateTime"` invoking `window.insertDateTime`.
3. Mapped `<C-t><C-s>` in Normal, Insert, and Visual modes.
4. Added a multi-stroke `"Ctrl-t Ctrl-s"` keybinding to CodeMirror's keymap for non-Vim mode.

---

## 3. Files Impacted

| File | Change |
| --- | --- |
| `frontend/static/editor_interop.js` | Implement half-page scrolling, filter conflicting keymap bindings, register `<C-d>`, `<C-u>`, and `<C-t><C-s>` in Vim mode and editor keymap. |
| `docs/features/003_vim_navigation_and_shortcuts.md` | Feature specification and documentation. |
| `docs/features/README.md` | Feature index registration. |
