## 2024-05-24 - Dynamic elements and aria-labels
**Learning:** Adding static `aria-label` attributes to elements with dynamically updating contents, such as buttons displaying badge numbers, creates an accessibility anti-pattern. Screen readers may announce the static label instead of the newly updated visible content or context.
**Action:** When working with dynamically updated UI components (like uncommitted file counters), either dynamically format the `aria-label` to include the updated values, or remove the `aria-label` so the screen reader correctly reads the element’s text content directly.
## 2024-05-27 - Empty States
**Learning:** Rendering empty `<ul>` tags when a data structure (like a file tree) is empty lacks feedback. Explicitly rendering a clear "No items found" empty state message provides significantly better UX.
**Action:** Always check if a collection is empty and provide a descriptive empty state fallback in the UI, especially in context menus and side drawers.
