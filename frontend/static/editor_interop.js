// All packages share the same @codemirror/state and @codemirror/view instance
// via esm.sh's ?deps= deduplication. Without this, @replit/codemirror-vim
// bundles its own @codemirror/state, causing the "multiple instances"
// instanceof error that breaks EditorState.create().
import { EditorState, EditorSelection } from "https://esm.sh/@codemirror/state@6.6.0";
import { EditorView, keymap, lineNumbers } from "https://esm.sh/@codemirror/view@6.43.1?deps=@codemirror/state@6.6.0";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "https://esm.sh/@codemirror/commands@6.5.0?deps=@codemirror/state@6.6.0,@codemirror/view@6.43.1";
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

    // If original line lacked leading pipe, adjust targetCell index
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

// Half-page navigation commands
Vim.mapCommand("<C-d>", "action", "scrollHalfPageDown", {}, { context: "normal" });
Vim.mapCommand("<C-d>", "action", "scrollHalfPageDown", {}, { context: "visual" });
Vim.mapCommand("<C-u>", "action", "scrollHalfPageUp", {}, { context: "normal" });
Vim.mapCommand("<C-u>", "action", "scrollHalfPageUp", {}, { context: "visual" });

// DateTime shortcut chord: <C-t><C-s>
Vim.mapCommand("<C-t><C-s>", "action", "insertDateTime", {}, { context: "normal" });
Vim.mapCommand("<C-t><C-s>", "action", "insertDateTime", {}, { context: "insert" });
Vim.mapCommand("<C-t><C-s>", "action", "insertDateTime", {}, { context: "visual" });

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

    let extensions = [
        lineNumbers(),
        history(),
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
                backgroundColor: "var(--editor-bg)",
                color: "var(--editor-fg)"
            },
            ".cm-scroller": { overflow: "auto" },
            ".cm-gutters": {
                backgroundColor: "var(--color-canvas-subtle)",
                color: "var(--color-fg-muted)",
                borderRight: "1px solid var(--color-border-default)"
            },
            ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--color-fg-default)" }
        }),
        EditorView.updateListener.of((v) => {
            if (v.docChanged) {
                textArea.value = v.state.doc.toString();
                if (window._cmChangeCallback && typeof window._cmChangeCallback === "function") {
                    window._cmChangeCallback();
                }
            }
            if (vimMode && !isMobile) {
                const cm = (typeof getCM === "function" ? getCM(v.view) : v.view.cm);
                const inInsertMode = !!(cm && cm.state && cm.state.vim && cm.state.vim.insertMode);
                if (wasInInsertMode && !inInsertMode) {
                    formatActiveTable(v.view);
                }
                wasInInsertMode = inInsertMode;
            }
        })
    ];

    if (vimMode && !isMobile) {
        // Vim plugin must precede standard keymaps
        extensions.push(vim());
    }

    // Filter out Ctrl-d (deleteCharForward) and Ctrl-t (transposeChars) across both key and mac properties
    const filteredDefaultKeymap = defaultKeymap.filter(b => 
        b.key !== "Ctrl-d" && b.mac !== "Ctrl-d" &&
        b.key !== "Ctrl-t" && b.mac !== "Ctrl-t"
    );

    extensions.push(keymap.of([
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
        },
        {
            key: "Ctrl-t Ctrl-s",
            run: (view) => {
                window.insertDateTime(view);
                return true;
            }
        },
        {
            key: "Ctrl-d",
            run: (view) => {
                scrollHalfPage(view, true);
                return true;
            }
        },
        {
            key: "Ctrl-u",
            run: (view) => {
                scrollHalfPage(view, false);
                return true;
            }
        },
        ...filteredDefaultKeymap,
        ...historyKeymap,
        indentWithTab
    ]));

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
                if (!active || active === document.body || views[elementId].dom.contains(active)) {
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
