use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub matches: Vec<String>,
}

pub fn search_wiki(root: &PathBuf, query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        if let Some(ext) = path.extension() {
            if ext != "md" {
                continue;
            }
        } else {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            let mut file_matches = Vec::new();
            for line in content.lines() {
                if line.to_lowercase().contains(&query_lower) {
                    file_matches.push(line.trim().to_string());
                    if file_matches.len() >= 3 {
                        break;
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
                });
            }
        }
    }

    results
}
