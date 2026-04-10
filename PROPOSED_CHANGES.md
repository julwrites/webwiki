# Proposed Changes

## Logic Check
**Discovery:**
I reviewed the system architecture and investigated technical debt indicators across the repository. I discovered performance inefficiencies in the backend search functionality (`backend/src/search/mod.rs`).

Specifically, `is_fuzzy_match` requires callers to pass pre-lowercased strings, forcing an unnecessary string allocation (`.to_lowercase()`) for every file processed by the `search_wiki` function. Additionally, the `WalkDir` traversal checks if an entry is a directory by calling `entry.path().is_dir()`, which issues an expensive system call (`fs::metadata`) for every entry.

**Why this improvement is needed:**
Replacing `entry.path().is_dir()` with `entry.file_type().is_dir()` significantly minimizes syscalls during filesystem traversal, as `WalkDir` caches the file type on most operating systems. Modifying `is_fuzzy_match` to perform case-insensitive comparisons on-the-fly (with an ASCII fast path) avoids allocating a new `String` for every file scanned in the repository. As the wiki scales to thousands of files, these two optimizations combined will heavily reduce CPU and memory overhead during user searches, enhancing overall performance and system stability.

## Execution
I will optimize the `backend/src/search/mod.rs` component by:
1. Updating `is_fuzzy_match` to take the original casing for the `text` parameter, checking equality using an ASCII-optimized on-the-fly lowercase conversion.
2. Replacing `path.is_dir()` with `entry.file_type().is_dir()` within the `WalkDir` loop in `search_wiki` to eliminate redundant metadata syscalls.
