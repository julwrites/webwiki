---
id: FOUNDATION-20260225-083157-KBU
status: review_requested
title: Fix Git Pull Detached HEAD
priority: medium
created: 2026-02-25 08:31:57
category: foundation
dependencies:
type: bug
---

# Fix Git Pull Detached HEAD

## Description
Users reported that `git pull` operations were updating the working directory but not the branch reference, resulting in a detached HEAD state. This caused subsequent push operations to fail or behave unexpectedly.

## Implementation Plan
- [x] Analyze `backend/src/git.rs` to identify the cause.
  - The issue was traced to `repo.find_reference("HEAD")` returning the symbolic reference "HEAD" instead of the resolved branch reference. `set_target` on the symbolic HEAD reference caused it to detach.
- [x] Create a reproduction test case.
  - Created `backend/tests/repro_pull.rs` which confirmed the detached HEAD behavior.
- [x] Implement the fix.
  - Replaced `repo.find_reference("HEAD")` with `repo.head()` which correctly resolves the current branch reference.
  - Removed redundant `repo.set_head` call.
- [x] Verify the fix.
  - Verified that the reproduction test now passes with the fix.

## Verification
- Run `cargo test --test repro_pull` (test file was created temporarily and then deleted).
- Verified that `git pull` correctly updates the local branch reference (e.g., `refs/heads/master`) and checks out the new commit.

## Learnings
- `git2::Repository::find_reference("HEAD")` returns the symbolic HEAD reference.
- `git2::Repository::head()` returns the resolved reference if HEAD is symbolic.
- calling `set_target` on a symbolic reference with an OID detaches it (or fails if not allowed). To update the branch, one must operate on the branch reference itself.
