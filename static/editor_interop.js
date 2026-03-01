window.setupEditor = function(elementId, initialContent, onSaveCallback, vimMode, onQuitCallback) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;

    // Check if CodeMirror is already attached
    var existingEditor = textArea.nextSibling && textArea.nextSibling.CodeMirror;

    if (vimMode) {
        // --- VIM MODE (CodeMirror) ---
        if (existingEditor) {
            // Already have CodeMirror, just ensure Vim mode
            existingEditor.setOption("keyMap", "vim");
            existingEditor._saveCallback = onSaveCallback;
            existingEditor._quitCallback = onQuitCallback;
            return;
        }

        // Create CodeMirror
        // If textarea has value (from user edits in raw mode), use it.
        // Otherwise use initialContent.
        if (textArea.value === "") {
             textArea.value = initialContent;
        }

        var editor = CodeMirror.fromTextArea(textArea, {
            mode: "markdown",
            keyMap: "vim",
            lineNumbers: true,
            theme: "default",
            lineWrapping: true
        });

        // Save command (:w) - Global for Vim mode
        CodeMirror.Vim.defineEx("write", "w", function(cm) {
            if (cm._saveCallback) {
                cm._saveCallback(cm.getValue());
            }
        });

        // Quit command (:q)
        CodeMirror.Vim.defineEx("quit", "q", function(cm) {
            if (cm._quitCallback) {
                cm._quitCallback();
            }
        });

        // Store the callback on the instance
        editor._saveCallback = onSaveCallback;
        editor._quitCallback = onQuitCallback;

        // Save with Ctrl+S
        editor.setOption("extraKeys", {
            "Ctrl-S": function(cm) {
                var content = cm.getValue();
                onSaveCallback(content);
            }
        });

    } else {
        // --- STANDARD MODE (Raw Textarea) ---
        if (existingEditor) {
            // Destroy CodeMirror to revert to textarea
            existingEditor.save(); // Updates textArea.value with current content
            existingEditor.toTextArea();
        } else {
             // Just ensure content is set if it's the first load
             if (textArea.value === "") {
                 textArea.value = initialContent;
             }
        }

        // Setup save handler on textarea
        // Remove old if exists (to avoid duplicates if called multiple times)
        if (textArea._saveHandler) {
            textArea.removeEventListener("keydown", textArea._saveHandler);
        }

        textArea._saveHandler = function(e) {
            if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                e.preventDefault();
                onSaveCallback(textArea.value);
            }
        };

        textArea.addEventListener("keydown", textArea._saveHandler);
    }
};

window.wrapSelection = function(elementId, prefix, suffix) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;
    var cm = textArea.nextSibling && textArea.nextSibling.CodeMirror;

    if (cm) {
        var selection = cm.getSelection();
        if (selection) {
            cm.replaceSelection(prefix + selection + suffix);
        } else {
            // No selection, insert prefix + suffix and place cursor in middle
            var cursor = cm.getCursor();
            cm.replaceSelection(prefix + suffix);
            cm.setCursor({line: cursor.line, ch: cursor.ch + prefix.length});
        }
        cm.focus();
    } else {
        // Raw Textarea
        var start = textArea.selectionStart;
        var end = textArea.selectionEnd;
        var text = textArea.value;
        var selectedText = text.substring(start, end);
        var replacement = prefix + selectedText + suffix;

        textArea.setRangeText(replacement, start, end, 'select');

        // If it was just a cursor (no selection), move cursor between tags
        if (start === end) {
             textArea.selectionStart = start + prefix.length;
             textArea.selectionEnd = start + prefix.length;
        }
        textArea.focus();
    }
};

window.insertTextAtCursor = function(elementId, text) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;
    var cm = textArea.nextSibling && textArea.nextSibling.CodeMirror;

    if (cm) {
        cm.replaceSelection(text);
        cm.focus();
    } else {
        // Raw Textarea
        var start = textArea.selectionStart;
        var end = textArea.selectionEnd;
        textArea.setRangeText(text, start, end, 'end');
        textArea.focus();
    }
};

window.toggleHeader = function(elementId, level) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;
    var cm = textArea.nextSibling && textArea.nextSibling.CodeMirror;

    var hashes = "#".repeat(level) + " ";

    if (cm) {
        var cursor = cm.getCursor();
        var lineContent = cm.getLine(cursor.line);

        var match = lineContent.match(/^(#+ )/);
        if (match) {
            if (match[1] === hashes) {
                // Remove header
                cm.replaceRange("", {line: cursor.line, ch: 0}, {line: cursor.line, ch: match[1].length});
            } else {
                // Change header level
                cm.replaceRange(hashes, {line: cursor.line, ch: 0}, {line: cursor.line, ch: match[1].length});
            }
        } else {
            // Add header
            cm.replaceRange(hashes, {line: cursor.line, ch: 0});
        }
        cm.focus();
    } else {
        // Raw Textarea
        var start = textArea.selectionStart;
        var text = textArea.value;

        // Find line start
        var lineStart = text.lastIndexOf('\n', start - 1) + 1;
        var lineEnd = text.indexOf('\n', start);
        if (lineEnd === -1) lineEnd = text.length;

        var lineContent = text.substring(lineStart, lineEnd);
        var match = lineContent.match(/^(#+ )/);

        if (match) {
            if (match[1] === hashes) {
                // Remove header
                textArea.setRangeText("", lineStart, lineStart + match[1].length, 'preserve');
            } else {
                // Change header level
                textArea.setRangeText(hashes, lineStart, lineStart + match[1].length, 'preserve');
            }
        } else {
            // Add header
            textArea.setRangeText(hashes, lineStart, lineStart, 'preserve');
        }
        textArea.focus();
    }
};

window.getEditorContent = function(elementId) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return "";
    var cm = textArea.nextSibling && textArea.nextSibling.CodeMirror;
    if (cm) {
        return cm.getValue();
    } else {
        return textArea.value;
    }
};

window.renderMermaid = function() {
    if (window.mermaid) {
        // mermaid.run() is available in v10+
        if (mermaid.run) {
             mermaid.run({
                nodes: document.querySelectorAll(".mermaid")
            });
        } else if (mermaid.init) {
             // Fallback for older versions if something changes
             mermaid.init(undefined, document.querySelectorAll(".mermaid"));
        }
    }
};

window.renderGraphviz = function(elementId, dotContent) {
    if (window.Viz) {
        var viz = new Viz();
        viz.renderSVGElement(dotContent)
        .then(function(element) {
            var container = document.getElementById(elementId);
            if (container) {
                container.innerHTML = "";
                container.appendChild(element);
            }
        })
        .catch(function(error) {
            console.error(error);
            var container = document.getElementById(elementId);
            if (container) {
                container.innerText = "Error rendering Graphviz: " + error;
            }
        });
    }
};

window.renderDrawio = function(elementId, xmlContent) {
    // Draw.io viewer automatically handles .mxgraph-viewer divs if loaded.
    // However, if we are dynamically inserting, we might need to tell GraphViewer to process.
    // The viewer library usually exposes GraphViewer.
    if (window.GraphViewer) {
        var container = document.getElementById(elementId);
        if (container) {
            container.innerHTML = "";
            var div = document.createElement("div");
            div.className = "mxgraph-viewer";
            div.setAttribute("data-mxgraph", JSON.stringify({
                xml: xmlContent,
                resize: true,
                center: true,
                nav: true
            }));
            container.appendChild(div);
            GraphViewer.processElements();
        }
    }
};
