# Feature Specification: Editor Autofocus & Neovim-Style Table Auto-Formatting

## Overview

This specification details two editor enhancements for WebWiki:
1. **Immediate Input Focus on Note Edit**: Eliminating the need to click into the editor after entering edit mode.
2. **Neovim-Style Markdown Table Auto-Formatting**: Dynamically re-aligning and padding table columns to match content length when exiting edit mode (both when exiting Vim Insert mode via `<Esc>` and when saving/exiting the note editor).

---

## 1. Feature 1: Immediate Input Focus on Note Edit

### 1.1 Problem Statement
When switching from viewing mode to editing mode (`Ctrl+E` or clicking "Edit"), the CodeMirror 6 editor is initialized and rendered with a cursor displayed. However, browser keyboard focus is not placed on `.cm-content`. As a result:
- Keystrokes are not captured by CodeMirror.
- In Vim mode, normal mode commands (such as `i`, `a`, `h`, `j`, `k`, `l`, `:w`) do not work until the editor area is clicked.
- In non-Vim mode, typing immediately does nothing until clicked.

### 1.2 Root Cause
In [frontend/static/editor_interop.js](file:///Users/julianteh/julwrites/webwiki/frontend/static/editor_interop.js), `window.setupEditor` constructs the `EditorView`:
```javascript
let view = new EditorView({
    state,
    parent: cmContainer
});

view._onSaveCallback = onSaveCallback;
views[elementId] = view;
```
It mounts the DOM nodes, but never calls `view.focus()`.

### 1.3 Solution & Implementation
Inside `window.setupEditor` in `frontend/static/editor_interop.js`:
1. Call `view.focus()` immediately after storing `views[elementId] = view;`.
2. Schedule a secondary `requestAnimationFrame` call to ensure focus is applied even if the Yew render cycle or DOM layout performs layout shifts:
```javascript
view.focus();
requestAnimationFrame(() => {
    if (views[elementId]) {
        views[elementId].focus();
    }
});
```

---

## 2. Feature 2: Neovim-Style Markdown Table Auto-Formatting

### 2.1 Problem Statement
When editing Markdown tables (adding/editing cells), column lengths become jagged:
```markdown
| Name | Description | Status |
| --- | --- | --- |
| Task 1 | Short | Done |
| Task 2 with a much longer name | Detailed description of the task | In Progress |
```
In Neovim (specifically with plugins like `vim-table-mode` / `tabular` / native table tools), exiting insert mode automatically recalculates column widths and expands/shrinks delimiters and cell padding so the table is cleanly aligned.

### 2.2 Desired Behavior & Trigger Points
1. **Vim Mode — Exiting Insert Mode (`<Esc>`)**:
   - When the user transitions from Insert mode to Normal mode, check if the cursor is within a Markdown table.
   - If yes, reformat **that specific table** in place.
   - Preserve cursor position mapped to the reformatted table cell.
2. **Note Save / Exit**:
   - On note save (`:w`, `Ctrl+S`, or clicking Save) or exiting edit mode, reformat **all Markdown tables** in the document so that saved files are always cleanly formatted.

### 2.3 Table Formatting Specification

#### Table Detection
A block of lines constitutes a GFM table if:
1. It contains at least two lines.
2. Each line contains at least one pipe delimiter `|`.
3. The second line (or first non-header line) is a delimiter row:
   Pattern: `^\s*\|?(\s*:?-+:?\s*\|)+\s*:?-+:?\s*\|?\s*$`

#### Parsing & Cell Extraction
For each line in the table block:
1. Strip leading and trailing whitespace.
2. Remove leading `|` and trailing `|` if present.
3. Split remaining text on `|` that are not escaped (`\|`):
   Regex split: `/(?<!\\)\|/`
4. Trim whitespace from each extracted cell content.

#### Column Width & Alignment Rules
1. Calculate the maximum number of columns across all rows in the block.
2. For each column $j$:
   - Delimiter alignment:
     - Centered: starts and ends with `:` (e.g. `:---:`)
     - Right: ends with `:` (e.g. `---:`)
     - Left / default: starts with `:` or has no colons (e.g. `:---` or `---`)
   - Max width $W_j = \max(\text{length of trimmed cell in column } j)$.
   - Delimiter line minimum width: $\max(W_j, 3)$ (GFM requires at least 3 dashes if no colons, or 1 dash + 2 colons).
3. Delimiter generation for column $j$ with width $W$:
   - Centered: `:${'-'.repeat(W - 2)}:`
   - Right: `${'-'.repeat(W - 1)}:`
   - Left: `:${'-'.repeat(W - 1)}`
   - Default: `${'-'.repeat(W)}`
4. Row reconstruction:
   - Format: `| ${padded_cell_0} | ${padded_cell_1} | ... |`
   - Data cells: left-aligned (padded with trailing spaces) or aligned to column alignment specifier.
   - Ensure a single space on either side of the cell text and delimiter.

#### Cursor Preservation
- Compute the cursor line and column offset within the table before formatting.
- Find the cell containing the cursor.
- After formatting, place the cursor at the corresponding relative offset inside the reformatted cell to prevent cursor jumping.

### 2.4 CodeMirror 6 Integration Details

#### Exiting Insert Mode Detection
`@replit/codemirror-vim` wraps CodeMirror 6 and provides `getCM(view)` to access the CM5-compatible Vim instance:
```javascript
import { vim, Vim, getCM } from "https://esm.sh/@replit/codemirror-vim@6.2.1?deps=@codemirror/state@6.6.0,@codemirror/view@6.43.1";
```
Inside the `EditorView.updateListener` or a dedicated `ViewPlugin`:
```javascript
let wasInInsertMode = false;

// Within updateListener:
if (vimMode) {
    const cm = getCM(view);
    const inInsertMode = !!(cm && cm.state && cm.state.vim && cm.state.vim.insertMode);
    
    if (wasInInsertMode && !inInsertMode) {
        // Exited insert mode!
        formatActiveTable(view);
    }
    wasInInsertMode = inInsertMode;
}
```

#### Formatting Function Implementation
```javascript
function formatActiveTable(view) {
    const state = view.state;
    const cursorPos = state.selection.main.head;
    const line = state.doc.lineAt(cursorPos);
    
    // Check if current line contains a pipe
    if (!line.text.includes('|')) return;
    
    // Find contiguous table line bounds [startLine, endLine]
    let startLine = line.number;
    while (startLine > 1) {
        const prev = state.doc.line(startLine - 1);
        if (prev.text.includes('|') && prev.text.trim().length > 0) {
            startLine--;
        } else {
            break;
        }
    }
    
    let endLine = line.number;
    while (endLine < state.doc.lines) {
        const next = state.doc.line(endLine + 1);
        if (next.text.includes('|') && next.text.trim().length > 0) {
            endLine++;
        } else {
            break;
        }
    }
    
    const lines = [];
    for (let l = startLine; l <= endLine; l++) {
        lines.push(state.doc.line(l).text);
    }
    
    // Check if block has a delimiter row (usually row index 1)
    if (lines.length < 2) return;
    const isDelimiter = (str) => /^\s*\|?(\s*:?-+:?\s*\|)+\s*:?-+:?\s*\|?\s*$/.test(str);
    if (!isDelimiter(lines[1])) return;
    
    const formatted = formatTableLines(lines);
    if (formatted.join('\n') === lines.join('\n')) return; // No change needed
    
    const from = state.doc.line(startLine).from;
    const to = state.doc.line(endLine).to;
    
    view.dispatch({
        changes: { from, to, insert: formatted.join('\n') }
    });
}
```

#### Format on Save
In `triggerSave` and `onSaveCallback`:
```javascript
function formatAllTables(docText) {
    // Scan docText for table blocks and format them sequentially
    // ...
}
```

---

## 3. Files Impacted

| File | Change |
| --- | --- |
| `frontend/static/editor_interop.js` | Add `view.focus()`, import `getCM`, add table formatting utility functions, attach insert mode listener and on-save table formatter. |
| `frontend/src/lib.rs` | Ensure `on_save` and `quit_closure` trigger formatting if applicable (or rely on `editor_interop.js`). |
| `docs/features/README.md` | Register this feature spec in documentation. |

---

## 4. Verification & Test Plan

1. **Docker Container Testing**:
   - Run `./scripts/docker-dev.sh`.
   - Access `http://localhost:3000`.
2. **Editor Focus Verification**:
   - Press `Ctrl+E` to enter edit mode.
   - Verify cursor is active and typing `iHello` (in Vim mode) or `Hello` (in normal mode) immediately enters text without any mouse click.
3. **Table Auto-Formatting Verification**:
   - Type an unaligned table:
     ```markdown
     | A | B |
     |---|---|
     | Short | A very long column entry |
     ```
   - Press `<Esc>` to exit Insert mode.
   - Verify the table automatically realigns with aligned pipes and padded columns.
   - Verify delimiter dashes match column widths.
   - Test tables with alignments (`:---`, `:---:`, `---:`).
4. **Save Verification**:
   - Save via `:w` or `Ctrl+S`.
   - Verify the note displays the reformatted table correctly in view mode.
