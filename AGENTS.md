# AI Agent Instructions

You are an expert Software Engineer working on this project. Your primary responsibility is to implement features and fixes.

## Workflow
1.  **Plan**:
    *   **Security Check**: Ask the user about specific security considerations for this work.
2.  **Implement**: Write code, run tests.
3.  **Review & Verify**:
    *   Ask a human or another agent to review the code.
4.  **Finalize**:
    *   Ensure all acceptance criteria are met.

## Documentation Reference
*   **Architecture**: Refer to `docs/architecture/` for system design.
*   **Features**: Refer to `docs/features/` for feature specifications.
*   **Security**: Refer to `docs/security/` for risk assessments and mitigations.

## Code Style & Standards
*   Follow the existing patterns in the codebase.
*   Ensure all new code is covered by tests (if testing infrastructure exists).

## PR Review Methodology
When performing a PR review, follow this "Human-in-the-loop" process to ensure depth and efficiency.

### 1. Preparation
1.  **Fetch Details**: Use `gh` to get the PR context.
    *   `gh pr view <N>`
    *   `gh pr diff <N>`

### 2. Analysis & Planning (The "Review Plan")
**Do not review line-by-line yet.** Instead, analyze the changes and document a **Review Plan** (or present it for approval).

Your plan must include:
*   **High-Level Summary**: Purpose, new APIs, breaking changes.
*   **Dependency Check**: New libraries, maintenance status, security.
*   **Impact Assessment**: Effect on existing code/docs.
*   **Focus Areas**: Prioritized list of files/modules to check.
*   **Suggested Comments**: Draft comments for specific lines.
    *   Format: `File: <path> | Line: <N> | Comment: <suggestion>`
    *   Tone: Friendly, suggestion-based ("Consider...", "Nit: ...").

### 3. Execution
Once the human approves the plan and comments:
1.  **Pending Review**: Create a pending review using `gh`.
    *   `COMMIT_SHA=$(gh pr view <N> --json headRefOid -q .headRefOid)`
    *   `gh api repos/{owner}/{repo}/pulls/{N}/reviews -f commit_id="$COMMIT_SHA"`
2.  **Batch Comments**: Add comments to the pending review.
    *   `gh api repos/{owner}/{repo}/pulls/{N}/comments -f body="..." -f path="..." -f commit_id="$COMMIT_SHA" -F line=<L> -f side="RIGHT"`
3.  **Submit**:
    *   `gh pr review <N> --approve --body "Summary..."` (or `--request-changes`).

## Agent Interoperability
- **Tool Definitions**: `docs/interop/tool_definitions.json`

## Local Testing

**Always test using the Docker container, not the native dev server.** The Docker build is the production environment; bugs that only appear in Docker (e.g. CodeMirror 6 vs 5, missing CSS) will be missed if you test against a native backend.

### Static Assets — Single Source of Truth

`frontend/static/` is the **only** static asset directory. It contains:
- All CSS, JS, icons, manifest (hand-authored)
- WASM build artifacts (`wasm.js`, `wasm_bg.wasm`, etc.) — output of `wasm-pack`

`backend/static` is a symlink → `../frontend/static`.  
The root `static/` directory is **deprecated** — do not edit files there.

### Starting the Docker Dev Server

```bash
# Full rebuild (required after any Rust or WASM source change):
./scripts/docker-dev.sh

# Skip rebuild (only if image already exists and you want to re-run it):
./scripts/docker-dev.sh --no-rebuild
```

- Runs on `http://localhost:3000`
- `DEV_BYPASS_AUTH=true` — no login required
- Mounts `./wiki_data/` as the wiki volume

### Verifying Fixes with Chrome DevTools MCP

After the container is up, use the Chrome DevTools MCP to verify:

1. Navigate to `http://localhost:3000`
2. `take_screenshot` — verify layout, styling, side panel
3. Open the editor → verify CodeMirror loads without console errors
4. Press `Escape` in the editor → must go to vim Normal mode, NOT dismiss the editor
5. Save with `:w` or `Ctrl+S` → editor should close and show the rendered page
6. Open Settings modal → verify layout is correct
7. Open Side Panel → verify `.file-tree-item` hover actions appear
8. Check console via `evaluate_script` for any JS errors

### Starting the Native Dev Server (for rapid Rust iteration only)

```bash
./scripts/dev.sh
```

Note: `dev.sh` also serves from `frontend/static/` and uses `./wiki_data/` — it mirrors Docker. Use it only when you need fast Rust recompile cycles.
