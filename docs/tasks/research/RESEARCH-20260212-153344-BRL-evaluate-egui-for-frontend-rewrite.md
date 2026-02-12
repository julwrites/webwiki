---
id: RESEARCH-20260212-153344-BRL
status: review_requested
title: Evaluate egui for frontend rewrite
priority: medium
created: 2026-02-12 15:33:44
category: research
dependencies:
type: task
estimate: 2
---

# Evaluate egui for frontend rewrite

## Task Information
- **Objective**: Determine if `egui` (and `eframe`) is a viable replacement for the current Yew + HTML/CSS frontend stack, or suitable for isolated components.
- **Context**: The user is exploring alternatives to the current "canvas handler" (perceived) or general rendering approach, interested in Immediate Mode GUI vs. Retained Mode (DOM).

## Executive Summary
**Recommendation: Do NOT rewrite the core application in `egui`.**

The current application relies heavily on rich text editing (Vim mode, syntax highlighting), complex document rendering (Markdown, Mermaid, Graphviz), and browser-native features (accessibility, text selection). `egui` is an excellent library for tools and game UIs, but it is ill-suited for a document-centric application like a Wiki.

However, `egui` could be used for specific, isolated *tools* within the app (e.g., a visual node graph explorer), but integration complexity must be weighed against the benefits.

## Detailed Analysis

### 1. Text Editing Capabilities (The Core Feature)

| Feature | Current Stack (CodeMirror via Yew) | `egui` Implementation |
| :--- | :--- | :--- |
| **Vim Mode** | ✅ Native support, mature, widely used. | ❌ Non-existent. Must be implemented from scratch. |
| **Syntax Highlighting** | ✅ robust, supports hundreds of languages. | ⚠️ Basic support exists, but performance on large files can be an issue. |
| **Text Selection** | ✅ Native browser selection, copy/paste works perfectly. | ⚠️ Custom implementation. Browser integration for copy/paste can be finicky. |
| **Mobile Input** | ✅ Native virtual keyboard support. | ⚠️ often struggles with mobile keyboards and autocorrect. |

**Impact**: Replicating the current Vim editing experience in `egui` would require months of dedicated effort to build a custom editor widget.

### 2. Rich Content Rendering

The wiki displays Markdown, Images, PDFs, and Diagrams (Mermaid, Graphviz, Draw.io).

*   **Current (DOM)**:
    *   Markdown -> HTML (via `pulldown-cmark`).
    *   Diagrams -> SVG/DOM (via JS libraries like `mermaid.js`).
    *   PDFs -> `<embed>` or `<iframe>`.
    *   Images -> `<img>`.

*   **`egui` (Canvas/WebGL)**:
    *   **Markdown**: `egui` has a rich text layout engine, but it is not a full HTML/Markdown renderer. You would need to parse Markdown and map it to `egui` primitives manually. Tables, complex lists, and embedded HTML would be difficult.
    *   **Diagrams**: Libraries like Mermaid render to the DOM (SVG). To show them in `egui`, you would need to:
        *   Render to an image (loss of text selection/links).
        *   Overlay an HTML element on top of the canvas (complex positioning/z-index issues).
    *   **PDFs**: Would require an HTML overlay (iframe).

### 3. Architecture & Performance

*   **DOM (Yew)**:
    *   Pros: Accessible by default, SEO-friendly (if server-rendered/static), integrates with all browser APIs, leverages browser text layout engine.
    *   Cons: heavy DOM manipulation *can* be slow, but Yew's VDOM is efficient.
*   **Immediate Mode (`egui`)**:
    *   Pros: Extremely fast for dynamic UIs (60fps), simple code structure (no state sync issues), great for "tools" (sliders, knobs, graphs).
    *   Cons: Re-renders everything every frame (energy intensive on mobile), poor accessibility (screen readers), non-native look and feel.

### 4. Integration of Isolated Components

If we wanted to use `egui` for a specific feature (e.g., a "Knowledge Graph" visualizer):

1.  **Wasm-in-Wasm?**: No, we would likely compile the `egui` code as part of the main Wasm binary.
2.  **Canvas Integration**: We would mount a `<canvas>` element in the Yew tree and let `egui` take control of it.
3.  **Interop**: Passing data between Yew (state) and `egui` (immediate mode loop) would require careful synchronization.

## Conclusion

For a text-heavy, editor-focused application like a Wiki:
*   **HTML/DOM is the correct technology.** It handles text layout, selection, and accessibility natively.
*   **`egui` is the wrong tool.** It is designed for game UIs, debug tools, and heavy interactive visualizations, not for document editing.

**Verdict**: Stick with Yew. If a complex visualizer is needed later (e.g., a node graph), evaluate `egui` specifically for that component, but do not rewrite the editor or page viewer.
