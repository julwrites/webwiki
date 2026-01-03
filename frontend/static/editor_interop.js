window.setupEditor = function(elementId, initialContent, onSaveCallback) {
    var textArea = document.getElementById(elementId);
    if (!textArea) return;

    var editor = CodeMirror.fromTextArea(textArea, {
        mode: "markdown",
        keyMap: "vim",
        lineNumbers: true,
        theme: "default",
        lineWrapping: true
    });

    editor.setValue(initialContent);

    // Save command (:w)
    CodeMirror.Vim.defineEx("write", "w", function() {
        var content = editor.getValue();
        onSaveCallback(content);
    });

    // Save with Ctrl+S
    editor.setOption("extraKeys", {
        "Ctrl-S": function(cm) {
            var content = cm.getValue();
            onSaveCallback(content);
        }
    });

    // Track changes? Not needed for explicit save, but maybe for draft.
    // For now, explicit save via callback.
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
