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
        let is_md = path.extension().map_or(false, |ext| ext == "md");

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
