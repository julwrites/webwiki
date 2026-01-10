use serde::{Deserialize, Serialize};

pub mod auth;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WikiPage {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_wiki_page_serialization() {
        let page = WikiPage {
            path: "foo/bar.md".to_string(),
            content: "# Hello".to_string(),
        };

        let serialized = serde_json::to_value(&page).unwrap();
        assert_eq!(
            serialized,
            json!({
                "path": "foo/bar.md",
                "content": "# Hello"
            })
        );
    }

    #[test]
    fn test_wiki_page_deserialization() {
        let json = json!({
            "path": "foo/bar.md",
            "content": "# Hello"
        });

        let page: WikiPage = serde_json::from_value(json).unwrap();
        assert_eq!(page.path, "foo/bar.md");
        assert_eq!(page.content, "# Hello");
    }

    #[test]
    fn test_file_node_serialization() {
        let node = FileNode {
            name: "foo".to_string(),
            path: "foo".to_string(),
            is_dir: true,
            children: Some(vec![FileNode {
                name: "bar.md".to_string(),
                path: "foo/bar.md".to_string(),
                is_dir: false,
                children: None,
            }]),
        };

        let serialized = serde_json::to_value(&node).unwrap();
        assert_eq!(serialized["name"], "foo");
        assert_eq!(serialized["is_dir"], true);
        assert_eq!(serialized["children"][0]["name"], "bar.md");
    }
}
