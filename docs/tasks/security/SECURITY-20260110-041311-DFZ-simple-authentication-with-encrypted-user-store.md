---
id: SECURITY-20260110-041311-DFZ
status: completed
title: Simple Authentication with Encrypted User Store
priority: high
created: 2026-01-10 04:13:11
category: security
dependencies:
type: task
---

# Simple Authentication with Encrypted User Store

## Context
Currently, the application has no backend authentication. Access is open to anyone who can reach the server. The user requested a simple authentication mechanism using an encrypted `users.json` file, with the decryption key provided via an environment variable.

## Objectives
1.  Secure the application against unauthorized access.
2.  Support multiple users with distinct credentials.

## Requirements
*   **User Store (`users.json`):**
    *   A JSON file containing user records.
    *   Structure: `[ { "username": "alice", "password_hash": "...", "salt": "..." } ]`.
    *   The content of the file (or sensitive fields) should be encrypted at rest if required, OR the user meant the *loading* of users is protected. *Clarification from prompt:* "encrypted users.json; the decryption key can be injected as an environment variable". This implies the file on disk is encrypted (e.g., AES-GCM) and the app decrypts it on startup using the key.
*   **Environment Variable:**
    *   `AUTH_SECRET`: Used as the decryption key for `users.json`.
*   **Authentication Flow:**
    *   Login Page (`/login`).
    *   POST `/api/login` endpoint.
    *   Session management (Cookies or JWT).
*   **Route Protection:**
    *   Middleware to block access to `/api/wiki/*`, `/api/git/*`, and `/api/tree` for unauthenticated requests.
    *   Redirect unauthenticated browser requests to `/login`.

## Implementation Plan
1.  **Crypto Utils (`common` or `backend`):**
    *   Add `aes-gcm` or similar crate.
    *   Implement `load_users(path: PathBuf, key: &str) -> Vec<User>`.
2.  **Backend Auth (`backend/src/auth.rs`):**
    *   Implement Login handler.
    *   Implement Session Middleware (using `tower-sessions` or `axum-extra` with signed cookies).
3.  **Frontend:**
    *   Create `Login` component.
    *   Handle 401 Unauthorized responses by redirecting to Login.
4.  **CLI Tool (Optional but recommended):**
    *   A script or cargo binary to help the admin create/update the encrypted `users.json` file (since they can't manually edit an encrypted blob).

## Verification
*   Start server with `AUTH_SECRET` and a valid `users.json`.
*   Verify accessing `/` redirects to `/login`.
*   Verify logging in with correct credentials works.
*   Verify logging in with incorrect credentials fails.
