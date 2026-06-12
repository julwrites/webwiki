## 2024-05-24 - Dynamic elements and aria-labels
**Learning:** Adding static `aria-label` attributes to elements with dynamically updating contents, such as buttons displaying badge numbers, creates an accessibility anti-pattern. Screen readers may announce the static label instead of the newly updated visible content or context.
**Action:** When working with dynamically updated UI components (like uncommitted file counters), either dynamically format the `aria-label` to include the updated values, or remove the `aria-label` so the screen reader correctly reads the element’s text content directly.
## 2024-05-27 - Empty States
**Learning:** Rendering empty `<ul>` tags when a data structure (like a file tree) is empty lacks feedback. Explicitly rendering a clear "No items found" empty state message provides significantly better UX.
**Action:** Always check if a collection is empty and provide a descriptive empty state fallback in the UI, especially in context menus and side drawers.
## 2024-05-28 - Dynamic Context for Action Buttons
**Learning:** Using generic, static ARIA labels (e.g., "Rename Page") for repeating action buttons in a list (like a file tree) creates an accessibility anti-pattern. When screen reader users navigate through the list or review the page's buttons out of context, they will hear "Rename Page" multiple times without knowing *which* page the action applies to.
**Action:** Always provide contextually specific `aria-label`s and `title`s (e.g., `aria-label="Rename my_file.md"`) for repeating actions in lists or grids to ensure screen reader users have the full context of what the button will affect.
