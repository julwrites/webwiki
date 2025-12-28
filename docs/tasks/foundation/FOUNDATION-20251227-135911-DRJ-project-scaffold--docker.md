---
id: FOUNDATION-20251227-135911-DRJ
title: Project Scaffold & Docker
status: verified
priority: high
dependencies: []
type: story
created: 2025-12-27T13:59:11Z
updated: 2025-12-27T13:59:11Z
---

# Project Scaffold & Docker

## Description
Set up the initial Rust workspace, Docker environment, and "Hello World" endpoints for both Backend (Axum) and Frontend (Yew).

## Subtasks
- [ ] Initialize Rust Workspace (`backend`, `frontend`, `common`).
- [ ] Setup `backend` with Axum and a health check endpoint.
- [ ] Setup `frontend` with Yew and a basic landing page.
- [ ] Create `Dockerfile` for multi-stage build (build frontend -> serve with backend).
- [ ] Create `docker-compose.yml` mounting a local volume for the wiki.
- [ ] Verify local deployment with `docker compose up`.
