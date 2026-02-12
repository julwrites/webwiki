---
id: PRESENTATION-20260212-PWA-service-worker
status: in_progress
title: Implement PWA Service Worker
priority: medium
created: 2026-02-12 11:30:00
category: presentation
dependencies:
type: task
---

# Implement PWA Service Worker

## Context
The application is intended to be a Progressive Web App (PWA) that can be installed on devices. While a `manifest.json` and icon exist, a Service Worker is missing. A Service Worker is required for the "Add to Home Screen" prompt to appear reliably and for basic offline capabilities.

## Objectives
1.  Create a Service Worker (`sw.js`) to cache static assets.
2.  Register the Service Worker in `index.html`.
3.  Ensure the app is recognized as installable.

## Implementation Plan
1.  **Service Worker (`frontend/static/sw.js`):**
    *   Implement `install` event to cache core assets (`/`, `index.html`, `style.css`, `wasm.js`, `wasm_bg.wasm`, `manifest.json`, `icon.svg`).
    *   Implement `fetch` event to serve from cache first, falling back to network.
    *   For API requests (`/api/`), use network-first or network-only strategy.
2.  **Registration (`frontend/index.html`):**
    *   Add script to register `sw.js` on load.

## Verification
*   Verify `sw.js` exists and is served.
*   Verify registration script is present in `index.html`.
