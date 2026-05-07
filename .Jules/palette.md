## 2025-04-13 - Add ARIA Labels and Disabled State with Title
**Learning:** Icon-only buttons often lack accessible names, making them difficult to use with screen readers. Providing a disabled state alongside a helpful tooltip for critical actions like committing code helps users understand why an action is unavailable. Adding placeholders gives context without occupying space.
**Action:** Use `aria-label` for icon buttons and always provide a disabled state and tooltip for inactive primary actions.
## 2026-04-14 - Added aria-labels to bottom bar\n**Learning:** Icon-only buttons in this Yew application require  attributes to be accessible to screen readers, especially in components like `bottom_bar.rs`.\n**Action:** Ensure all future icon-only buttons include an `aria-label` matching their `title`.
## 2026-04-14 - Added aria-labels to bottom bar
**Learning:** Icon-only buttons in this Yew application require aria-label attributes to be accessible to screen readers, especially in components like bottom_bar.rs.
**Action:** Ensure all future icon-only buttons include an aria-label matching their title.
## 2026-04-14 - Selective ARIA labels
**Learning:** Adding `aria-label` attributes that match visible text (e.g., `aria-label="Commit"` on a button containing the text `"Commit"`) is a known accessibility anti-pattern. They should only be used to clarify ambiguous or abbreviated visible text, like "B" for "Bold". Additionally, avoid adding static `aria-label`s to elements whose text updates dynamically (like a "Login" button changing to "Logging in..."), as the static label overrides the dynamic state for screen readers.
**Action:** When adding `aria-label`s, only target elements with abbreviated or missing visible context. Avoid using them on elements with clear, static text or dynamically updating text.
## 2024-05-24 - Form Field Accessibility
**Learning:** Input fields inside functional components (like the `SettingsModal` in Yew) must be explicitly associated with their respective `<label>` elements using matching `id` and `for` attributes. This provides context to screen readers and expands the clickable area to focus the input.
**Action:** When adding new form elements or reviewing existing ones, ensure every `<input>`, `<textarea>`, and `<select>` is accompanied by a `<label>` and correctly associated via `for` and `id` attributes, rather than relying on proximity alone.

## 2024-04-18 - Dynamically Formatted ARIA Labels in Loops
**Learning:** When adding `aria-label`s inside loops or maps (like the commit modal's file list in Yew), using a variable that was previously moved or borrowing from an iterated item directly inside a macro format string can cause compilation failures.
**Action:** Always create a locally scoped clone of the variable (e.g., `let display_path = f.path.clone();`) explicitly intended for use in the `aria-label` format string within the `html!` block.

## 2026-04-20 - Ensure Aria-Labels on Elements Hidden on Mobile
**Learning:** In responsive designs where elements contain both text and icons (like `<span>{"Search files"}</span>` + `<IconSearch />`), the text is often hidden using `display: none` on smaller screens. This makes the element act as an "icon-only" button visually on mobile, which fails accessibility checks if `aria-label` and `title` attributes aren't explicitly provided, since screen readers might lose context when the text node is unrendered.
**Action:** When adding or reviewing buttons that have text and an icon, check if the CSS uses media queries to hide the text on mobile (e.g., `.bottom-bar-search-trigger span { display: none; }`). If so, ensure the parent button has a robust `aria-label` and `title` attribute so it remains fully accessible across all screen sizes.
## 2024-05-24 - Form Field Accessibility
**Learning:** Input fields inside functional components (like the `SettingsModal` in Yew) must be explicitly associated with their respective `<label>` elements using matching `id` and `for` attributes. This provides context to screen readers and expands the clickable area to focus the input.
**Action:** When adding new form elements or reviewing existing ones, ensure every `<input>`, `<textarea>`, and `<select>` is accompanied by a `<label>` and correctly associated via `for` and `id` attributes, rather than relying on proximity alone.

## 2026-05-18 - Keyboard Shortcut Discoverability
**Learning:** Powerful keyboard shortcuts (like Ctrl+K for search) are often hidden from new users unless explicitly documented. Users might not know they exist, relying on slower mouse interactions.
**Action:** Expose common keyboard shortcuts directly in the UI, such as in button text or tooltips (e.g., "Search files... (Ctrl+K)"), to gently teach users the faster keyboard-centric workflows without requiring them to read documentation.

## 2024-05-18 - Keyboard Navigation Enhancements
**Learning:** Default browser focus rings are often stripped out or poorly visible against custom dark/light themes, making keyboard navigation difficult for screen reader and keyboard-only users.
**Action:** Always add explicit `:focus-visible` styles to interactive elements (`button`, `a`, `input`, etc.) using the theme's accent color (e.g., `outline: 2px solid var(--color-accent-fg) !important;`) in the global stylesheet (`frontend/static/style.css`).
## 2025-05-13 - Add ARIA Labels and Disabled State with Title
**Learning:** Adding aria-labels that are identical to the element's text content is an accessibility anti-pattern. However, providing aria-labels with more specific context (e.g. "Cancel commit" instead of just "Cancel" or "Save settings" instead of "Save") significantly improves screen reader clarity, particularly when multiple buttons with the same name exist on the page. Similarly, for keyboard accessibility of custom UI elements (like a clickable div functioning as a file tree toggle), they must include `role="button"`, `tabindex="0"`, and respond to `Enter` and `Space` keypresses via an `onkeydown` handler, along with appropriate `aria-expanded` and `aria-label` attributes for state representation.
**Action:** Enhance ambiguous visible text (like "Cancel", "Save", "Edit") with descriptive aria-labels (like "Cancel settings changes", "Save settings", "Edit page {filename}"). Make custom interactive elements fully keyboard and screen-reader accessible by adding role, tabindex, aria attributes, and keyboard event handlers.
