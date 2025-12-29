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
