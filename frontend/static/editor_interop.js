// All packages share the same @codemirror/state and @codemirror/view instance
// via esm.sh's ?deps= deduplication. Without this, @replit/codemirror-vim
// bundles its own @codemirror/state, causing the "multiple instances"
// instanceof error that breaks EditorState.create().
import { EditorState, EditorSelection, Compartment } from "https://esm.sh/@codemirror/state@6.6.0";
import { EditorView, keymap, lineNumbers, drawSelection, dropCursor } from "https://esm.sh/@codemirror/view@6.43.1?deps=@codemirror/state@6.6.0";
import { defaultKeymap, history, historyKeymap, indentWithTab, undo, redo } from "https://esm.sh/@codemirror/commands@6.5.0?deps=@codemirror/state@6.6.0,@codemirror/view@6.43.1";
import { markdown } from "https://esm.sh/@codemirror/lang-markdown@6.2.5?deps=@codemirror/state@6.6.0,@codemirror/view@6.43.1";
import { vim, Vim, getCM } from "https://esm.sh/@replit/codemirror-vim@6.2.1?deps=@codemirror/state@6.6.0,@codemirror/view@6.43.1";
import { syntaxHighlighting, HighlightStyle } from "https://esm.sh/@codemirror/language@6.10.1?deps=@codemirror/state@6.6.0,@codemirror/view@6.43.1";
import { tags as t } from "https://esm.sh/@lezer/highlight@1.2.0?deps=@codemirror/state@6.6.0";


const customHighlightStyle = HighlightStyle.define([
    { tag: t.heading, color: "var(--cm-header)", fontWeight: "bold" },
    { tag: t.quote, color: "var(--cm-comment)", fontStyle: "italic" },
    { tag: t.strong, color: "var(--cm-strong)", fontWeight: "bold" },
    { tag: t.emphasis, color: "var(--cm-em)", fontStyle: "italic" },
    { tag: t.link, color: "var(--cm-link)", textDecoration: "underline" },
    { tag: t.url, color: "var(--cm-url)" },
    { tag: t.list, color: "var(--cm-list)" },
    { tag: t.comment, color: "var(--cm-comment)" },
    { tag: t.string, color: "var(--cm-url)" },
]);

// Store view instances keyed by elementId
const views = {};
window.views = views;

// Compartment for dynamically reconfigurable editor keymap
const keymapCompartment = new Compartment();

// Default shortcuts fallback matching Rust Shortcuts::default()
const DEFAULT_SHORTCUTS = {
    edit: "Ctrl+E",
    save: "Ctrl+S",
    cancel: "Escape",
    search: "<Ctrl+p><Ctrl+f>",
    toggle_drawer: "Ctrl+B",
    copy_link: "Ctrl+Shift+C",
    undo: "u",
    redo: "r, <Ctrl+r>",
    scroll_down: "<Ctrl+d>",
    scroll_up: "<Ctrl+u>",
    insert_datetime: "<Ctrl+t><Ctrl+s>"
};

function loadConfiguredShortcuts() {
    try {
        const raw = localStorage.getItem("shortcuts");
        if (!raw) return { ...DEFAULT_SHORTCUTS };
        const parsed = JSON.parse(raw);
        const merged = { ...DEFAULT_SHORTCUTS, ...parsed };
        if (merged.search && merged.search.trim() === "Ctrl+K, Ctrl+P") {
            merged.search = "<Ctrl+p><Ctrl+f>";
            localStorage.setItem("shortcuts", JSON.stringify(merged));
        }
        return merged;
    } catch (_) {
        return { ...DEFAULT_SHORTCUTS };
    }
}

// --- Keybinding Normalization Utilities (Vim & CodeMirror 6) ---

function splitChordStrokes(alt) {
    const strokes = [];
    let current = "";
    let inAngle = false;
    for (let i = 0; i < alt.length; i++) {
        const ch = alt[i];
        if (ch === "<") {
            if (!inAngle && current.trim()) {
                strokes.push(current.trim());
                current = "";
            }
            inAngle = true;
            current += ch;
        } else if (ch === ">") {
            current += ch;
            inAngle = false;
            if (current.trim()) {
                strokes.push(current.trim());
            }
            current = "";
        } else if ((ch === " " || ch === "\t") && !inAngle) {
            if (current.trim()) {
                strokes.push(current.trim());
                current = "";
            }
        } else {
            current += ch;
        }
    }
    if (current.trim()) {
        strokes.push(current.trim());
    }
    return strokes;
}

function parseSingleStroke(stroke) {
    let trimmed = (stroke || "").trim();
    if (!trimmed) return null;
    if (trimmed.startsWith("<") && trimmed.endsWith(">")) {
        trimmed = trimmed.slice(1, -1).trim();
    }
    if (!trimmed) return null;

    if (trimmed.length === 1) {
        const isUpper = trimmed >= "A" && trimmed <= "Z";
        return {
            key: trimmed.toLowerCase(),
            rawChar: trimmed,
            ctrl: false,
            shift: isUpper,
            alt: false
        };
    }

    const parts = trimmed.split(/[+-]/).filter(Boolean);
    if (parts.length === 0) return null;

    let ctrl = false;
    let shift = false;
    let alt = false;

    for (let i = 0; i < parts.length - 1; i++) {
        const m = parts[i].trim().toLowerCase();
        if (m === "ctrl" || m === "c" || m === "cmd" || m === "meta" || m === "m" || m === "control") {
            ctrl = true;
        } else if (m === "shift" || m === "s") {
            shift = true;
        } else if (m === "alt" || m === "a" || m === "opt" || m === "option") {
            alt = true;
        }
    }

    const lastRaw = parts[parts.length - 1].trim();
    if (!lastRaw) return null;

    if (parts.length === 1 && lastRaw.length === 1 && lastRaw >= "A" && lastRaw <= "Z") {
        shift = true;
    }

    const lower = lastRaw.toLowerCase();
    let key = lower;
    if (lower === "esc" || lower === "escape") key = "escape";
    else if (lower === "return" || lower === "enter") key = "enter";
    else if (lower === "space") key = " ";
    else if (lower === "bs" || lower === "backspace") key = "backspace";
    else if (lower === "del" || lower === "delete") key = "delete";

    return {
        key,
        rawChar: lastRaw.length === 1 ? lastRaw : key,
        ctrl,
        shift,
        alt
    };
}

function strokeToVimNotation(step) {
    if (!step) return "";
    if (!step.ctrl && !step.alt) {
        if (step.key.length === 1) {
            return step.shift ? step.key.toUpperCase() : step.key.toLowerCase();
        }
        if (step.key === "escape") return "<Esc>";
        if (step.key === "enter") return "<CR>";
        if (step.key === "backspace") return "<BS>";
        if (step.key === "delete") return "<Del>";
        if (step.key === "tab") return step.shift ? "<S-Tab>" : "<Tab>";
        if (step.key === " ") return "<Space>";
    }

    let mods = "";
    if (step.ctrl) mods += "C-";
    if (step.alt) mods += "A-";
    if (step.shift) mods += "S-";

    let keyPart = step.key;
    if (keyPart === "escape") keyPart = "Esc";
    else if (keyPart === "enter") keyPart = "CR";
    else if (keyPart === "backspace") keyPart = "BS";
    else if (keyPart === "delete") keyPart = "Del";
    else if (keyPart === "tab") keyPart = "Tab";
    else if (keyPart === " ") keyPart = "Space";

    return `<${mods}${keyPart}>`;
}

function strokeToCM6Notation(step) {
    if (!step) return "";
    let mods = "";
    if (step.ctrl) mods += "Ctrl-";
    if (step.alt) mods += "Alt-";
    if (step.shift) mods += "Shift-";

    let keyPart = step.key;
    if (keyPart === "escape") keyPart = "Escape";
    else if (keyPart === "enter") keyPart = "Enter";
    else if (keyPart === "backspace") keyPart = "Backspace";
    else if (keyPart === "delete") keyPart = "Delete";
    else if (keyPart === "tab") keyPart = "Tab";
    else if (keyPart === " ") keyPart = "Space";

    return `${mods}${keyPart}`;
}

function parseShortcutAlternatives(shortcutDef) {
    if (!shortcutDef || !shortcutDef.trim()) return [];
    const results = [];
    const parts = shortcutDef.split(",");
    for (const part of parts) {
        const trimmed = part.trim();
        if (!trimmed) continue;
        const strokeTokens = splitChordStrokes(trimmed);
        const steps = strokeTokens.map(parseSingleStroke).filter(Boolean);
        if (steps.length > 0) {
            results.push({
                steps,
                vimKey: steps.map(strokeToVimNotation).join(""),
                cmKey: steps.map(strokeToCM6Notation).join(" ")
            });
        }
    }
    return results;
}

// --- Markdown Table Formatting Utilities ---

function isDelimiterRow(line) {
    if (!line || !line.includes("-")) return false;
    let trimmed = line.trim();
    if (trimmed.startsWith("|")) trimmed = trimmed.slice(1);
    if (trimmed.endsWith("|")) trimmed = trimmed.slice(0, -1);
    const cells = trimmed.split(/(?<!\\)\|/);
    if (cells.length === 0) return false;
    return cells.every(c => /^\s*:?-+:?\s*$/.test(c));
}

function splitRow(line) {
    let trimmed = line.trim();
    if (trimmed.startsWith("|")) trimmed = trimmed.slice(1);
    if (trimmed.endsWith("|")) trimmed = trimmed.slice(0, -1);
    return trimmed.split(/(?<!\\)\|/).map(c => c.trim());
}

function parseAlignment(cell) {
    const trimmed = cell.trim();
    const leftColon = trimmed.startsWith(":");
    const rightColon = trimmed.endsWith(":");
    if (leftColon && rightColon) return "center";
    if (rightColon) return "right";
    if (leftColon) return "left";
    return "default";
}

function formatTableLines(lines) {
    if (lines.length < 2) return lines;
    if (!isDelimiterRow(lines[1])) return lines;

    const rows = lines.map(splitRow);
    const delimIndex = 1;
    const numCols = Math.max(...rows.map(r => r.length));
    if (numCols === 0) return lines;

    for (const row of rows) {
        while (row.length < numCols) row.push("");
    }

    const alignments = rows[delimIndex].map(parseAlignment);
    const colWidths = new Array(numCols).fill(3);
    for (let c = 0; c < numCols; c++) {
        for (let r = 0; r < rows.length; r++) {
            if (r === delimIndex) continue;
            colWidths[c] = Math.max(colWidths[c], rows[r][c].length);
        }
        if (alignments[c] === "center") {
            colWidths[c] = Math.max(colWidths[c], 3);
        }
    }

    const formattedLines = [];
    for (let r = 0; r < rows.length; r++) {
        const row = rows[r];
        if (r === delimIndex) {
            const delimCells = row.map((_, c) => {
                const w = colWidths[c];
                const align = alignments[c];
                if (align === "center") return ":" + "-".repeat(Math.max(1, w - 2)) + ":";
                if (align === "right") return "-".repeat(Math.max(1, w - 1)) + ":";
                if (align === "left") return ":" + "-".repeat(Math.max(1, w - 1));
                return "-".repeat(Math.max(3, w));
            });
            formattedLines.push("| " + delimCells.join(" | ") + " |");
        } else {
            const dataCells = row.map((cell, c) => {
                const w = colWidths[c];
                const align = alignments[c];
                if (align === "right") {
                    return cell.padStart(w);
                } else if (align === "center") {
                    const totalPad = w - cell.length;
                    const leftPad = Math.floor(totalPad / 2);
                    const rightPad = totalPad - leftPad;
                    return " ".repeat(leftPad) + cell + " ".repeat(rightPad);
                } else {
                    return cell.padEnd(w);
                }
            });
            formattedLines.push("| " + dataCells.join(" | ") + " |");
        }
    }
    return formattedLines;
}

function formatActiveTable(view) {
    if (!view) return;
    const state = view.state;
    const cursorPos = state.selection.main.head;
    const line = state.doc.lineAt(cursorPos);

    if (!line.text.includes('|')) return;

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

    if (lines.length < 2 || !isDelimiterRow(lines[1])) return;

    const formatted = formatTableLines(lines);
    if (formatted.join('\n') === lines.join('\n')) return;

    const lineIndexInTable = line.number - startLine;
    const colInLine = cursorPos - line.from;

    const beforeParts = [];
    const pipeRegex = /(?<!\\)\|/g;
    let m;
    let last = 0;
    while ((m = pipeRegex.exec(line.text)) !== null) {
        beforeParts.push({ from: last, to: m.index });
        last = m.index + 1;
    }
    beforeParts.push({ from: last, to: line.text.length });

    let targetCell = 0;
    let offsetInCell = 0;
    for (let idx = 0; idx < beforeParts.length; idx++) {
        const p = beforeParts[idx];
        if (colInLine >= p.from && colInLine <= p.to) {
            targetCell = idx;
            const cellSub = line.text.substring(p.from, p.to);
            const firstNonWs = cellSub.search(/\S/);
            const startOfContent = firstNonWs === -1 ? p.from : p.from + firstNonWs;
            offsetInCell = Math.max(0, colInLine - startOfContent);
            break;
        }
    }

    const from = state.doc.line(startLine).from;
    const to = state.doc.line(endLine).to;

    const reformattedLine = formatted[lineIndexInTable] || "";
    const afterParts = [];
    pipeRegex.lastIndex = 0;
    last = 0;
    while ((m = pipeRegex.exec(reformattedLine)) !== null) {
        afterParts.push({ from: last, to: m.index });
        last = m.index + 1;
    }
    afterParts.push({ from: last, to: reformattedLine.length });

    let adjustedTargetCell = targetCell;
    if (!line.text.trim().startsWith("|") && reformattedLine.trim().startsWith("|")) {
        adjustedTargetCell = targetCell + 1;
    }

    let newCol = colInLine;
    if (adjustedTargetCell < afterParts.length) {
        const p = afterParts[adjustedTargetCell];
        const cellSub = reformattedLine.substring(p.from, p.to);
        const firstNonWs = cellSub.search(/\S/);
        const startOfContent = firstNonWs === -1 ? p.from : p.from + firstNonWs;
        const trimmedLen = cellSub.trim().length;
        newCol = startOfContent + Math.min(offsetInCell, trimmedLen);
    }

    let newHead = from;
    for (let l = 0; l < lineIndexInTable; l++) {
        newHead += formatted[l].length + 1;
    }
    newHead += Math.min(newCol, reformattedLine.length);

    view.dispatch({
        changes: { from, to, insert: formatted.join('\n') },
        selection: EditorSelection.cursor(Math.min(newHead, from + formatted.join('\n').length))
    });
}

function formatAllTables(docText) {
    if (!docText) return docText;
    const lines = docText.split("\n");
    const result = [];
    let inCodeBlock = false;
    let i = 0;

    while (i < lines.length) {
        const line = lines[i];
        if (line.trim().startsWith("```")) {
            inCodeBlock = !inCodeBlock;
            result.push(line);
            i++;
            continue;
        }

        if (!inCodeBlock && line.includes("|") && i + 1 < lines.length && isDelimiterRow(lines[i + 1])) {
            const tableLines = [line, lines[i + 1]];
            let j = i + 2;
            while (j < lines.length && lines[j].includes("|") && lines[j].trim().length > 0) {
                tableLines.push(lines[j]);
                j++;
            }
            const formatted = formatTableLines(tableLines);
            result.push(...formatted);
            i = j;
        } else {
            result.push(line);
            i++;
        }
    }
    return result.join("\n");
}

// --- Viewport Half-Page Scrolling ---

function scrollHalfPage(view, forward) {
    if (!view) return;
    const scrollEl = view.scrollDOM;
    const lineHeight = view.defaultLineHeight || 20;
    const clientHeight = scrollEl && scrollEl.clientHeight > 0 ? scrollEl.clientHeight : (window.innerHeight || 600);
    const distance = Math.max(5 * lineHeight, Math.floor(clientHeight / 2));

    const mainRange = view.state.selection.main;
    const targetRange = view.moveVertically(mainRange, forward, distance);
    const newSelection = view.state.selection.replaceRange(targetRange);

    if (scrollEl) {
        scrollEl.scrollTop += (forward ? distance : -distance);
    }

    view.dispatch({
        selection: newSelection,
        effects: EditorView.scrollIntoView(newSelection.main.head, { y: "nearest" })
    });
}

// --- Vim Actions & Ex Commands (Registered Once at Module Scope) ---

function getActiveView(cm) {
    if (cm && cm.cm6) return cm.cm6;
    if (cm && cm.view) return cm.view;
    for (const id in views) {
        if (views[id] && views[id].hasFocus) return views[id];
    }
    const allViews = Object.values(views);
    return allViews.length > 0 ? allViews[0] : null;
}

function isInVimNormalMode(view) {
    if (!view) return false;
    const cm = (typeof getCM === "function" ? getCM(view) : view.cm);
    if (!cm || !cm.state || !cm.state.vim) return true;
    return !cm.state.vim.insertMode && !cm.state.vim.visualMode;
}

Vim.defineEx("write", "w", function(cm) {
    const view = getActiveView(cm);
    if (!view) return;
    let currentText = view.state.doc.toString();
    let formatted = formatAllTables(currentText);
    if (formatted !== currentText) {
        view.dispatch({
            changes: { from: 0, to: view.state.doc.length, insert: formatted }
        });
        currentText = formatted;
    }
    if (view._onSaveCallback) {
        view._onSaveCallback(currentText);
    }
});

Vim.defineEx("quit", "q", function(cm) {
    const view = getActiveView(cm);
    if (view && view._onQuitCallback) {
        view._onQuitCallback();
    }
});

Vim.defineAction("cmUndo", function(cm, actionArgs) {
    const view = getActiveView(cm);
    if (!view) return;
    const repeat = (actionArgs && actionArgs.repeat) || 1;
    for (let i = 0; i < repeat; i++) {
        undo(view);
    }
});

Vim.defineAction("cmRedo", function(cm, actionArgs) {
    const view = getActiveView(cm);
    if (!view) return;
    const repeat = (actionArgs && actionArgs.repeat) || 1;
    for (let i = 0; i < repeat; i++) {
        redo(view);
    }
});

Vim.defineAction("openCommandPalette", function(_cm) {
    if (typeof window._onOpenSearchCallback === "function") {
        window._onOpenSearchCallback();
    }
});

Vim.defineAction("scrollHalfPageDown", function(cm) {
    const view = getActiveView(cm);
    if (view) scrollHalfPage(view, true);
});

Vim.defineAction("scrollHalfPageUp", function(cm) {
    const view = getActiveView(cm);
    if (view) scrollHalfPage(view, false);
});

Vim.defineAction("insertDateTime", function(cm) {
    const view = getActiveView(cm);
    if (view) window.insertDateTime(view);
});

// Track dynamically registered Vim mappings so we can cleanly unmap them on live reload
let activeCustomVimMappings = [];

function registerVimMapping(vimKey, actionName, context) {
    if (!vimKey) return;
    try {
        Vim.unmap(vimKey, context);
    } catch (_) {}
    Vim.mapCommand(vimKey, "action", actionName, {}, { context });
    activeCustomVimMappings.push({ keys: vimKey, context });
}

function buildEditorKeymapExtension(shortcuts) {
    const saveAlts = parseShortcutAlternatives(shortcuts.save);
    const dtAlts = parseShortcutAlternatives(shortcuts.insert_datetime);
    const downAlts = parseShortcutAlternatives(shortcuts.scroll_down);
    const upAlts = parseShortcutAlternatives(shortcuts.scroll_up);
    const searchAlts = parseShortcutAlternatives(shortcuts.search);
    const undoAlts = parseShortcutAlternatives(shortcuts.undo);
    const redoAlts = parseShortcutAlternatives(shortcuts.redo);

    // Collect first-stroke keys of any multi-stroke chords so we filter them out of defaultKeymap
    const chordFirstKeys = new Set(["Ctrl-d", "Ctrl-t", "Ctrl-p", "Ctrl-f", "Ctrl-u"]);
    for (const list of [saveAlts, dtAlts, downAlts, upAlts, searchAlts, undoAlts, redoAlts]) {
        for (const alt of list) {
            if (alt.steps.length > 1) {
                chordFirstKeys.add(strokeToCM6Notation(alt.steps[0]));
            }
        }
    }

    const filteredDefaultKeymap = defaultKeymap.filter(b =>
        !chordFirstKeys.has(b.key) && !chordFirstKeys.has(b.mac)
    );

    const customBindings = [
        {
            key: "Mod-s",
            preventDefault: true,
            run: (view) => {
                let currentText = view.state.doc.toString();
                let formatted = formatAllTables(currentText);
                if (formatted !== currentText) {
                    view.dispatch({
                        changes: { from: 0, to: view.state.doc.length, insert: formatted }
                    });
                    currentText = formatted;
                }
                if (view._onSaveCallback) view._onSaveCallback(currentText);
                return true;
            }
        }
    ];

    for (const alt of saveAlts) {
        if (alt.cmKey && alt.cmKey !== "Ctrl-s") {
            customBindings.push({
                key: alt.cmKey,
                preventDefault: true,
                run: (view) => {
                    let currentText = view.state.doc.toString();
                    let formatted = formatAllTables(currentText);
                    if (formatted !== currentText) {
                        view.dispatch({
                            changes: { from: 0, to: view.state.doc.length, insert: formatted }
                        });
                        currentText = formatted;
                    }
                    if (view._onSaveCallback) view._onSaveCallback(currentText);
                    return true;
                }
            });
        }
    }

    // Bind search / open document chord in CM6 keymap (Normal mode only when Vim is enabled)
    for (const alt of searchAlts) {
        if (alt.cmKey) {
            customBindings.push({
                key: alt.cmKey,
                preventDefault: true,
                run: (view) => {
                    if (!isInVimNormalMode(view)) return false;
                    if (typeof window._onOpenSearchCallback === "function") {
                        window._onOpenSearchCallback();
                    }
                    return true;
                }
            });
        }
    }

    for (const alt of dtAlts) {
        if (alt.cmKey) {
            customBindings.push({
                key: alt.cmKey,
                preventDefault: true,
                run: (view) => {
                    window.insertDateTime(view);
                    return true;
                }
            });
        }
    }

    for (const alt of downAlts) {
        if (alt.cmKey) {
            customBindings.push({
                key: alt.cmKey,
                preventDefault: true,
                run: (view) => {
                    scrollHalfPage(view, true);
                    return true;
                }
            });
        }
    }

    for (const alt of upAlts) {
        if (alt.cmKey) {
            customBindings.push({
                key: alt.cmKey,
                preventDefault: true,
                run: (view) => {
                    scrollHalfPage(view, false);
                    return true;
                }
            });
        }
    }

    return keymap.of([
        ...customBindings,
        ...filteredDefaultKeymap,
        ...historyKeymap,
        indentWithTab
    ]);
}

window.applyEditorShortcuts = function() {
    const shortcuts = loadConfiguredShortcuts();

    // 1. Unmap previously registered custom Vim bindings
    for (const m of activeCustomVimMappings) {
        try {
            Vim.unmap(m.keys, m.context);
        } catch (_) {}
    }
    activeCustomVimMappings = [];

    // Restore standard Vim 'r' replace action and '<C-p>' k motion first before applying custom overrides
    try {
        Vim.mapCommand("r", "action", "replace", {}, { context: "normal" });
        Vim.mapCommand("<C-p>", "motion", "moveByLines", { forward: false, linewise: true }, { context: "normal" });
    } catch (_) {}

    // 2. Register Undo in Vim Normal mode
    for (const alt of parseShortcutAlternatives(shortcuts.undo)) {
        registerVimMapping(alt.vimKey, "cmUndo", "normal");
    }

    // 3. Register Redo in Vim Normal mode (e.g. 'r' and '<C-r>')
    for (const alt of parseShortcutAlternatives(shortcuts.redo)) {
        registerVimMapping(alt.vimKey, "cmRedo", "normal");
    }

    // 4. Register Search / Open Document chord in Vim Normal mode ONLY (e.g. '<C-p><C-f>')
    for (const alt of parseShortcutAlternatives(shortcuts.search)) {
        if (alt.steps.length > 1) {
            const firstStrokeVim = strokeToVimNotation(alt.steps[0]);
            try {
                Vim.unmap(firstStrokeVim, "normal");
            } catch (_) {}
        }
        registerVimMapping(alt.vimKey, "openCommandPalette", "normal");
    }

    // 5. Register Half-page scrolling in Normal & Visual modes
    for (const alt of parseShortcutAlternatives(shortcuts.scroll_down)) {
        registerVimMapping(alt.vimKey, "scrollHalfPageDown", "normal");
        registerVimMapping(alt.vimKey, "scrollHalfPageDown", "visual");
    }

    for (const alt of parseShortcutAlternatives(shortcuts.scroll_up)) {
        registerVimMapping(alt.vimKey, "scrollHalfPageUp", "normal");
        registerVimMapping(alt.vimKey, "scrollHalfPageUp", "visual");
    }

    // 6. Register Insert DateTime in Normal, Insert, and Visual modes
    for (const alt of parseShortcutAlternatives(shortcuts.insert_datetime)) {
        registerVimMapping(alt.vimKey, "insertDateTime", "normal");
        registerVimMapping(alt.vimKey, "insertDateTime", "insert");
        registerVimMapping(alt.vimKey, "insertDateTime", "visual");
    }

    // 7. Reconfigure any active EditorViews with the updated keymap compartment
    const newKeymapExt = buildEditorKeymapExtension(shortcuts);
    for (const id in views) {
        if (views[id]) {
            views[id].dispatch({
                effects: keymapCompartment.reconfigure(newKeymapExt)
            });
        }
    }
};

// Apply initial shortcuts at module load
window.applyEditorShortcuts();

window.registerOpenSearchCallback = function(cb) {
    window._onOpenSearchCallback = cb;
};

window.triggerUndo = function(elementId) {
    const view = views[elementId];
    if (view) {
        undo(view);
        view.focus();
    }
};

window.triggerRedo = function(elementId) {
    const view = views[elementId];
    if (view) {
        redo(view);
        view.focus();
    }
};

// --- Editor Lifecycle ---

window.destroyEditor = function(elementId) {
    if (views[elementId]) {
        let parent = views[elementId].dom.parentNode;
        views[elementId].destroy();
        delete views[elementId];
        if (parent && parent.classList.contains('cm-container-wrapper')) {
            parent.remove();
        }
    }
};

window.focusEditor = function(elementId) {
    let view = views[elementId];
    if (view && view.contentDOM) {
        view.focus();
    }
};

window.setupEditor = function(elementId, initialContent, onSaveCallback, vimMode, onQuitCallback) {
    let textArea = document.getElementById(elementId);
    if (!textArea) return;

    if (views[elementId]) {
        views[elementId].destroy();
        delete views[elementId];
    }

    textArea.style.display = 'none';

    let parent = textArea.parentNode;
    let cmContainer = parent.querySelector('.cm-container-wrapper');
    if (!cmContainer) {
        cmContainer = document.createElement('div');
        cmContainer.className = 'cm-container-wrapper';
        cmContainer.style.flex = "1";
        cmContainer.style.position = "relative";
        cmContainer.style.minHeight = "0";
        cmContainer.style.display = "flex";
        cmContainer.style.flexDirection = "column";
        parent.insertBefore(cmContainer, textArea.nextSibling);
    } else {
        cmContainer.innerHTML = ''; 
    }

    const isMobile = (('ontouchstart' in window) || (navigator.maxTouchPoints > 0) || (navigator.msMaxTouchPoints > 0)) &&
        (/Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent) ||
         (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1) ||
         (window.matchMedia && (window.matchMedia('(max-width: 768px)').matches || window.matchMedia('(pointer: coarse)').matches)));

    let wasInInsertMode = false;

    // Ensure latest shortcuts from LocalStorage are applied
    window.applyEditorShortcuts();
    const shortcuts = loadConfiguredShortcuts();

    let extensions = [
        lineNumbers(),
        history(),
        drawSelection(),
        dropCursor(),
        markdown(),
        syntaxHighlighting(customHighlightStyle),
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({
            spellcheck: "false",
            autocorrect: "off",
            autocapitalize: "off",
            "data-enable-grammarly": "false",
            "data-gramm": "false"
        }),
        EditorView.theme({
            "&": { 
                height: "100%",
                flex: "1",
                fontFamily: "'Fira Code', ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace",
                fontSize: "15px",
                lineHeight: "1.6",
                backgroundColor: "var(--color-canvas-default)",
                color: "var(--color-fg-default)"
            },
            ".cm-scroller": {
                overflow: "auto",
                fontFamily: "'Fira Code', ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace"
            },
            ".cm-content": {
                caretColor: "var(--color-accent-fg)"
            },
            ".cm-gutters": {
                backgroundColor: "var(--color-canvas-subtle)",
                color: "var(--color-fg-muted)",
                borderRight: "1px solid var(--color-border-default)"
            },
            ".cm-cursor, .cm-dropCursor": {
                borderLeftColor: "var(--color-accent-fg) !important",
                borderLeftWidth: "2.5px !important"
            },
            ".cm-vimCursorLayer": {
                position: "absolute !important",
                top: "0 !important",
                left: "0 !important",
                width: "100%",
                height: "100%",
                pointerEvents: "none",
                zIndex: "200"
            },
            ".cm-fat-cursor": {
                background: "var(--color-accent-fg) !important",
                color: "var(--color-canvas-default) !important",
                minWidth: "0.55em",
                zIndex: "200"
            },
            "&:not(.cm-focused) .cm-fat-cursor": {
                background: "rgba(88, 166, 255, 0.25) !important",
                outline: "2px solid var(--color-accent-fg) !important",
                color: "var(--color-fg-default) !important"
            }
        }),
        EditorView.updateListener.of((v) => {
            if (v.docChanged) {
                textArea.value = v.state.doc.toString();
                if (window._cmChangeCallback && typeof window._cmChangeCallback === "function") {
                    window._cmChangeCallback();
                }
            }
            if (vimMode) {
                const cm = (typeof getCM === "function" ? getCM(v.view) : v.view.cm);
                const inInsertMode = !!(cm && cm.state && cm.state.vim && cm.state.vim.insertMode);
                if (wasInInsertMode && !inInsertMode) {
                    formatActiveTable(v.view);
                }
                wasInInsertMode = inInsertMode;
            }
        })
    ];

    if (vimMode) {
        // Vim plugin must precede standard keymaps
        extensions.push(vim());
    }

    extensions.push(keymapCompartment.of(buildEditorKeymapExtension(shortcuts)));

    let state = EditorState.create({
        doc: initialContent,
        extensions: extensions
    });

    let view = new EditorView({
        state,
        parent: cmContainer
    });

    view._onSaveCallback = onSaveCallback;
    view._onQuitCallback = onQuitCallback;
    views[elementId] = view;

    // Autofocus editor on desktop / non-touch devices to ensure immediate keyboard input
    if (!isMobile) {
        view.focus();
        const safeFocus = () => {
            if (views[elementId] && views[elementId].contentDOM) {
                const active = document.activeElement;
                const isOtherInputFocused = active &&
                    !views[elementId].dom.contains(active) &&
                    (active.tagName === "INPUT" || active.tagName === "TEXTAREA" || active.isContentEditable);
                if (!isOtherInputFocused) {
                    views[elementId].focus();
                }
            }
        };
        requestAnimationFrame(safeFocus);
        setTimeout(safeFocus, 50);
    }
};

window.wrapSelection = function(elementId, prefix, suffix) {
    let view = views[elementId];
    if (!view) return;
    
    let state = view.state;
    let changes;
    
    if (state.selection.main.empty) {
        changes = state.changeByRange(range => {
            return {
                changes: {from: range.from, insert: prefix + suffix},
                range: EditorSelection.cursor(range.from + prefix.length)
            };
        });
    } else {
        changes = state.changeByRange(range => {
            let text = state.sliceDoc(range.from, range.to);
            return {
                changes: {from: range.from, to: range.to, insert: prefix + text + suffix},
                range: EditorSelection.cursor(range.from + prefix.length + text.length)
            };
        });
    }

    view.dispatch(changes);
    view.focus();
};

window.insertTextAtCursor = function(elementIdOrView, text) {
    let view = (typeof elementIdOrView === 'string') ? views[elementIdOrView] : elementIdOrView;
    if (!view) return;
    
    let changes = view.state.changeByRange(range => {
        return {
            changes: {from: range.from, to: range.to, insert: text},
            range: EditorSelection.cursor(range.from + text.length)
        };
    });
    view.dispatch(changes);
    view.focus();
};

window.toggleHeader = function(elementId, level) {
    let view = views[elementId];
    if (!view) return;
    
    let state = view.state;
    let line = state.doc.lineAt(state.selection.main.head);
    let hashes = "#".repeat(level) + " ";
    
    let match = line.text.match(/^(#+ )/);
    let changes;
    
    if (match) {
        if (match[1] === hashes) {
            changes = {from: line.from, to: line.from + match[1].length, insert: ""};
        } else {
            changes = {from: line.from, to: line.from + match[1].length, insert: hashes};
        }
    } else {
        changes = {from: line.from, insert: hashes};
    }
    
    view.dispatch({changes});
    view.focus();
};

window.triggerSave = function(elementId) {
    let view = views[elementId];
    if (view && view._onSaveCallback) {
        let currentText = view.state.doc.toString();
        let formatted = formatAllTables(currentText);
        if (formatted !== currentText) {
            view.dispatch({
                changes: { from: 0, to: view.state.doc.length, insert: formatted }
            });
            currentText = formatted;
        }
        view._onSaveCallback(currentText);
    }
};

window.insertDateTime = function(elementIdOrView) {
    let view = (typeof elementIdOrView === 'string') ? views[elementIdOrView] : elementIdOrView;
    if (!view) return;

    var now = new Date();
    var days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    var year = now.getFullYear();
    var month = String(now.getMonth() + 1).padStart(2, '0');
    var date = String(now.getDate()).padStart(2, '0');
    var day = days[now.getDay()];
    var hours = now.getHours();
    var minutes = String(now.getMinutes()).padStart(2, '0');
    var ampm = hours >= 12 ? 'PM' : 'AM';
    hours = hours % 12;
    hours = hours ? hours : 12; 
    hours = String(hours).padStart(2, '0');
    
    var formattedString = year + '-' + month + '-' + date + ' ' + day + ' ' + hours + ':' + minutes + ' ' + ampm;
    window.insertTextAtCursor(view, formattedString);
};

window.getEditorContent = function(elementId) {
    let view = views[elementId];
    if (view) {
        return view.state.doc.toString();
    }
    let textArea = document.getElementById(elementId);
    return textArea ? textArea.value : "";
};

window.onEditorChange = function(elementId, callback) {
    // Store globally so the view update listener can trigger it
    window._cmChangeCallback = callback;
};

// Keep existing diagram renderers unchanged
window.renderMermaid = function() {
    if (window.mermaid) {
        if (mermaid.run) {
             mermaid.run({ nodes: document.querySelectorAll(".mermaid") });
        } else if (mermaid.init) {
             mermaid.init(undefined, document.querySelectorAll(".mermaid"));
        }
    }
};

window.renderGraphviz = function(elementId, dotContent) {
    if (window.Viz) {
        var viz = new Viz();
        viz.renderSVGElement(dotContent).then(function(element) {
            var container = document.getElementById(elementId);
            if (container) { container.innerHTML = ""; container.appendChild(element); }
        }).catch(function(error) {
            var container = document.getElementById(elementId);
            if (container) { container.innerText = "Error rendering Graphviz: " + error; }
        });
    }
};

window.renderDrawio = function(elementId, xmlContent) {
    if (window.GraphViewer) {
        var container = document.getElementById(elementId);
        if (container) {
            container.innerHTML = "";
            var div = document.createElement("div");
            div.className = "mxgraph-viewer";
            div.setAttribute("data-mxgraph", JSON.stringify({ xml: xmlContent, resize: true, center: true, nav: true }));
            container.appendChild(div);
            GraphViewer.processElements();
        }
    }
};

if (Array.isArray(window._pendingSetupEditor)) {
    const args = window._pendingSetupEditor;
    window._pendingSetupEditor = null;
    window.setupEditor(...args);
}
