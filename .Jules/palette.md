## 2025-04-13 - Add ARIA Labels and Disabled State with Title
**Learning:** Icon-only buttons often lack accessible names, making them difficult to use with screen readers. Providing a disabled state alongside a helpful tooltip for critical actions like committing code helps users understand why an action is unavailable. Adding placeholders gives context without occupying space.
**Action:** Use `aria-label` for icon buttons and always provide a disabled state and tooltip for inactive primary actions.
## 2026-04-14 - Added aria-labels to bottom bar\n**Learning:** Icon-only buttons in this Yew application require  attributes to be accessible to screen readers, especially in components like `bottom_bar.rs`.\n**Action:** Ensure all future icon-only buttons include an `aria-label` matching their `title`.
## 2026-04-14 - Added aria-labels to bottom bar
**Learning:** Icon-only buttons in this Yew application require aria-label attributes to be accessible to screen readers, especially in components like bottom_bar.rs.
**Action:** Ensure all future icon-only buttons include an aria-label matching their title.
