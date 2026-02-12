---
id: PRESENTATION-20260212-103214-VWB
status: completed
title: Refactor UI for Mobile-First/Vim-like Experience
priority: medium
created: 2026-02-12 10:32:14
category: presentation
dependencies:
type: task
---

# Refactor UI for Mobile-First/Vim-like Experience

## Objective
Refactor the application's look and feel to be more "Vim-like" and mobile-friendly, with a focus on keyboard efficiency and touch targets. Move interactable elements to a persistent bottom bar and hide the file tree by default.

## Tasks
- [x] **Update Keybindings**:
    - Implement Leader key sequences (`<Leader>gl`, `<Leader>gp`, `<Leader>gc`).
    - Implement search chord (`<Ctrl+f><Ctrl+f>`).
    - Make keybindings configurable via `KeyBindings` struct.
- [x] **Refactor Bottom Bar**:
    - Add Home, New File, and Theme Toggle buttons.
    - Ensure Search is the dominant element.
    - Ensure Git controls are grouped.
    - Persist on desktop.
- [x] **Update Layout**:
    - Wire up `on_home` and `on_new_file` actions in `Layout`.
    - Pass theme state and toggle logic.
- [x] **Add Icons**:
    - Add `IconHome` to `frontend/src/components/icons.rs`.
- [x] **Type Safety**:
    - Fix `CommandPalette` props to accept `Callback<()>`.

## Technical Details
- **Key Handler**: `use_key_handler` updated to use a buffer for key sequences. Default leader is Space.
- **Bottom Bar**: `BottomBar` component updated with new buttons and props.
- **State Management**: `Layout` handles the callbacks for navigation and file creation.

## Verification
- Code compiles (`cargo check -p frontend`).
- Unit tests pass (`cargo test -p frontend`).
