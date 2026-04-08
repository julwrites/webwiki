---
title: Remove Insecure Default Username
status: completed
---

# Remove Insecure Default Username

## Overview
This task improves backend security by removing the insecure default fallback ("admin") for `WIKI_USERNAME`.

## Sub-tasks
- ✅ Modify `backend/src/auth.rs` to remove the fallback.
- ✅ Return an error if `WIKI_USERNAME` is missing.
- ✅ Ensure the fix matches the existing logic for `WIKI_PASSWORD`.

## Review Plan
- High-Level Summary: The default "admin" username is removed. `WIKI_USERNAME` is now mandatory, returning 500 Internal Server Error if missing, similar to `WIKI_PASSWORD`.
- Impact Assessment: Enhances security by failing fast if configuration is incomplete. No new dependencies.
