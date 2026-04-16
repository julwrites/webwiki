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
