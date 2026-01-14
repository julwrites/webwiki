window.setupEditor = function(elementId, initialContent, onSaveCallback, vimMode) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;

    // Check if CodeMirror is already attached to this element
    // CodeMirror hides the textarea and adds a sibling .CodeMirror element.
    // We can check if the textarea has a 'nextSibling' that is the editor.
    // Or better, we can store the instance on the textarea element itself if we wanted,
    // but standard CodeMirror way is often to look for the wrapper.

    // However, since we might re-render the parent component, the DOM might be fresh.
    // But if Yew preserves the textarea, we need to handle it.

    // Actually, CodeMirror.fromTextArea sets textArea.style.display = 'none'.
    // If it's already 'none', it might be already initialized.

    var existingEditor = textArea.nextSibling && textArea.nextSibling.CodeMirror;

    if (existingEditor) {
        existingEditor.setOption("keyMap", vimMode ? "vim" : "default");

        // Update the callback reference
        existingEditor._saveCallback = onSaveCallback;

        existingEditor.setOption("extraKeys", {
            "Ctrl-S": function(cm) {
                var content = cm.getValue();
                onSaveCallback(content);
            }
        });

        return;
    }

    var editor = CodeMirror.fromTextArea(textArea, {
        mode: "markdown",
        keyMap: vimMode ? "vim" : "default",
        lineNumbers: true,
        theme: "default",
        lineWrapping: true
    });

    editor.setValue(initialContent);

    // Save command (:w) - Global for Vim mode
    // We define this once, but it relies on _saveCallback attached to the instance
    CodeMirror.Vim.defineEx("write", "w", function(cm) {
        if (cm._saveCallback) {
            cm._saveCallback(cm.getValue());
        }
    });

    // Store the callback on the instance
    editor._saveCallback = onSaveCallback;

    // Save with Ctrl+S
    editor.setOption("extraKeys", {
        "Ctrl-S": function(cm) {
            var content = cm.getValue();
            onSaveCallback(content);
        }
    });
};

window.wrapSelection = function(elementId, prefix, suffix) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;
    var cm = textArea.nextSibling && textArea.nextSibling.CodeMirror;
    if (!cm) return;

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
};

window.insertTextAtCursor = function(elementId, text) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;
    var cm = textArea.nextSibling && textArea.nextSibling.CodeMirror;
    if (!cm) return;

    cm.replaceSelection(text);
    cm.focus();
};

window.toggleHeader = function(elementId, level) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;
    var cm = textArea.nextSibling && textArea.nextSibling.CodeMirror;
    if (!cm) return;

    var cursor = cm.getCursor();
    var lineContent = cm.getLine(cursor.line);
    var hashes = "#".repeat(level) + " ";

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
