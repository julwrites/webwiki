use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub matches: Vec<String>,
    pub volume: Option<String>,
}

pub fn search_wiki(root: &PathBuf, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        // Check extension
        let is_md = path
            .extension()
            .is_none_or(|ext| ext == "md" || ext == "markdown");

        // Check filename match
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        let filename_match = file_name.contains(&query_lower);

        if !is_md && !filename_match {
            continue;
        }

        let mut file_matches = Vec::new();

        if filename_match {
            file_matches.push(format!("Filename match: {}", file_name));
        }

        if is_md {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    if line.to_lowercase().contains(&query_lower) {
                        file_matches.push(line.trim().to_string());
                        if file_matches.len() >= 3 {
                            break;
                        }
                    }
                }
            }
        }

        if !file_matches.is_empty() {
            let relative_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            results.push(SearchResult {
                path: relative_path,
                matches: file_matches,
                volume: None,
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_search_filename_match() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("find_me.txt");
        fs::write(file_path, "some content").unwrap();

        let results = search_wiki(&root, "find_me");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "find_me.txt");
        assert!(results[0].matches[0].contains("Filename match"));
    }

    #[test]
    fn test_search_content_match() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("note.md");
        fs::write(file_path, "This is a secret note about Rust.").unwrap();

        let results = search_wiki(&root, "secret");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "note.md");
        assert!(results[0].matches.contains(&"This is a secret note about Rust.".to_string()));
    }

    #[test]
    fn test_search_no_extension() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("README");
        fs::write(file_path, "This is a README file with search_term.").unwrap();

        // Match by filename
        let results = search_wiki(&root, "README");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "README");

        // Match by content (files without extension are treated as markdown)
        let results = search_wiki(&root, "search_term");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "README");
        assert!(results[0].matches.iter().any(|m| m.contains("search_term")));
    }

    #[test]
    fn test_search_case_insensitivity() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("Note.md");
        fs::write(file_path, "RUST IS GREAT.").unwrap();

        let results = search_wiki(&root, "rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "Note.md");
        assert!(results[0].matches.iter().any(|m| m.contains("RUST IS GREAT")));
    }

    #[test]
    fn test_search_match_limit() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("multi.md");
        fs::write(
            file_path,
            "line 1: match\nline 2: match\nline 3: match\nline 4: match",
        )
        .unwrap();

        let results = search_wiki(&root, "match");
        assert_eq!(results.len(), 1);
        // Filename "multi.md" doesn't match "match", so it should only have content matches.
        // It should be limited to 3 matches.
        assert_eq!(results[0].matches.len(), 3);
    }

    #[test]
    fn test_search_empty_results() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let file_path = root.join("nothing.txt");
        fs::write(file_path, "empty").unwrap();

        let results = search_wiki(&root, "nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_nested_directories() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let sub_dir = root.join("nested");
        fs::create_dir(&sub_dir).unwrap();
        let file_path = sub_dir.join("deep.md");
        fs::write(file_path, "deep search").unwrap();

        let results = search_wiki(&root, "deep");
        assert_eq!(results.len(), 1);
        // WalkDir uses the path separator of the OS, but in Linux it is /
        // search_wiki uses strip_prefix which should keep the relative path
        assert!(results[0].path.contains("nested"));
        assert!(results[0].path.contains("deep.md"));
    }
}
