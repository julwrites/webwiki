## 2023-10-27 - [Add Async Feedback to Commit/Discard Buttons]
**Learning:** Found that `commit_modal.rs` was not providing feedback during git commit or discard operations. Users might double-click buttons if requests take time. Adding an `is_committing` state provides immediate visual feedback ("Committing..."), disables the button temporarily, and uses `aria-busy` for screen readers.
**Action:** When working on async UI actions in Yew, always check if the submit/action button changes state to prevent double submission and provide feedback.
