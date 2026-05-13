import { EditorState, EditorSelection } from "https://esm.sh/@codemirror/state@6.4.1";
import { EditorView, keymap, lineNumbers } from "https://esm.sh/@codemirror/view@6.26.3";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "https://esm.sh/@codemirror/commands@6.5.0";
import { markdown } from "https://esm.sh/@codemirror/lang-markdown@6.2.5";
import { vim, Vim } from "https://esm.sh/@replit/codemirror-vim@6.2.1";
import { syntaxHighlighting, HighlightStyle } from "https://esm.sh/@codemirror/language@6.10.1";
import { tags as t } from "https://esm.sh/@lezer/highlight@1.2.0";

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
        parent.insertBefore(cmContainer, textArea.nextSibling);
    } else {
        cmContainer.innerHTML = ''; 
    }

    let extensions = [
        lineNumbers(),
        history(),
        keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
        markdown(),
        syntaxHighlighting(customHighlightStyle),
        EditorView.lineWrapping,
        EditorView.theme({
            "&": { 
                position: "absolute",
                top: "0",
                left: "0",
                right: "0",
                bottom: "0",
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
        })
    ];

    if (vimMode) {
        extensions.push(vim());
        Vim.defineEx("write", "w", function() {
            if (onSaveCallback) onSaveCallback(views[elementId].state.doc.toString());
        });
        Vim.defineEx("quit", "q", function() {
            if (onQuitCallback) onQuitCallback();
        });
    }

    // Ctrl+S
    extensions.push(keymap.of([{
        key: "Mod-s",
        preventDefault: true,
        run: (view) => {
            if (onSaveCallback) onSaveCallback(view.state.doc.toString());
            return true;
        }
    }]));

    let state = EditorState.create({
        doc: initialContent,
        extensions: extensions
    });

    let view = new EditorView({
        state,
        parent: cmContainer
    });

    view._onSaveCallback = onSaveCallback;
    views[elementId] = view;
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

window.insertTextAtCursor = function(elementId, text) {
    let view = views[elementId];
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
        view._onSaveCallback(view.state.doc.toString());
    }
};

window.insertDateTime = function(elementId) {
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
    insertTextAtCursor(elementId, formattedString);
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
