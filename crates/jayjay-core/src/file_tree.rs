use crate::types::FileTreeEntry;

struct TreeNode {
    name: String,
    children: Vec<(String, TreeNode)>,
    hunk_index: Option<u32>,
}

impl TreeNode {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            children: Vec::new(),
            hunk_index: None,
        }
    }

    fn insert(&mut self, components: &[&str], hunk_index: u32) {
        let Some(first) = components.first() else {
            return;
        };
        if components.len() == 1 {
            let mut leaf = TreeNode::new(first);
            leaf.hunk_index = Some(hunk_index);
            self.children.push((first.to_string(), leaf));
        } else if let Some(pos) = self.children.iter().position(|(k, _)| k == first) {
            self.children[pos].1.insert(&components[1..], hunk_index);
        } else {
            let mut child = TreeNode::new(first);
            child.insert(&components[1..], hunk_index);
            self.children.push((first.to_string(), child));
        }
    }

    fn collapse(&mut self) {
        for (_, child) in &mut self.children {
            child.collapse();
        }
        if self.hunk_index.is_none()
            && self.children.len() == 1
            && self.children[0].1.hunk_index.is_none()
        {
            let (key, child) = self.children.remove(0);
            self.name = if self.name.is_empty() {
                key
            } else {
                format!("{}/{}", self.name, key)
            };
            self.children = child.children;
        }
    }

    fn flatten(&self, depth: u32, results: &mut Vec<FileTreeEntry>) {
        // Sort: directories first, then files
        let mut sorted: Vec<&(String, TreeNode)> = self.children.iter().collect();
        sorted.sort_by_key(|(_, n)| n.hunk_index.is_some());

        for (key, child) in sorted {
            if let Some(idx) = child.hunk_index {
                results.push(FileTreeEntry {
                    name: key.clone(),
                    path: String::new(), // filled by caller
                    depth,
                    hunk_index: Some(idx),
                });
            } else {
                let dir_name = if child.name.is_empty() {
                    key.clone()
                } else {
                    child.name.clone()
                };
                results.push(FileTreeEntry {
                    name: dir_name,
                    path: String::new(),
                    depth,
                    hunk_index: None,
                });
                child.flatten(depth + 1, results);
            }
        }
    }
}

/// Build a flattened file tree from a list of file paths.
/// Each path is split by `/`, built into a trie, collapsed (single-child dirs merged),
/// then flattened with depth info. `hunk_index` corresponds to the index in the input `paths` vec.
pub fn build_file_tree(paths: &[String]) -> Vec<FileTreeEntry> {
    let mut root = TreeNode::new("");
    for (i, path) in paths.iter().enumerate() {
        let components: Vec<&str> = path.split('/').collect();
        root.insert(&components, i as u32);
    }
    root.collapse();
    let mut results = Vec::new();
    root.flatten(0, &mut results);

    // Fill in paths for file entries
    for entry in &mut results {
        if let Some(idx) = entry.hunk_index {
            if let Some(p) = paths.get(idx as usize) {
                entry.path = p.clone();
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tree() {
        let paths = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "README.md".to_string(),
        ];
        let entries = build_file_tree(&paths);
        // Should have: dir "src", then files main.rs and lib.rs, then README.md
        assert!(!entries.is_empty());
        // First entry should be the "src" directory
        assert_eq!(entries[0].name, "src");
        assert!(entries[0].hunk_index.is_none());
    }

    #[test]
    fn test_collapse_single_child_dirs() {
        let paths = vec!["a/b/c/file.rs".to_string()];
        let entries = build_file_tree(&paths);
        // Single file: entire directory prefix collapses into root (not emitted),
        // only the file entry appears at depth 0.
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file.rs");
        assert_eq!(entries[0].path, "a/b/c/file.rs");
        assert!(entries[0].hunk_index.is_some());
    }

    #[test]
    fn test_collapse_with_multiple_files() {
        let paths = vec!["a/b/c/file1.rs".to_string(), "a/b/d/file2.rs".to_string()];
        let entries = build_file_tree(&paths);
        // a/b is collapsed into one dir, then c and d are separate dirs
        // Expected: dir "c" (depth 0), file1.rs (depth 1), dir "d" (depth 0), file2.rs (depth 1)
        // But root collapses a/b, then children are c and d.
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].name, "c");
        assert!(entries[0].hunk_index.is_none());
        assert_eq!(entries[0].depth, 0);
        assert_eq!(entries[1].name, "file1.rs");
        assert_eq!(entries[1].depth, 1);
        assert_eq!(entries[2].name, "d");
        assert!(entries[2].hunk_index.is_none());
        assert_eq!(entries[2].depth, 0);
        assert_eq!(entries[3].name, "file2.rs");
        assert_eq!(entries[3].depth, 1);
    }

    #[test]
    fn test_empty_paths() {
        let paths: Vec<String> = vec![];
        let entries = build_file_tree(&paths);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_single_root_file() {
        let tree = build_file_tree(&["README.md".to_string()]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "README.md");
        assert_eq!(tree[0].hunk_index, Some(0));
    }

    #[test]
    fn test_deep_collapse_preserves_path() {
        // a/b/c/file.txt should collapse the directory chain and preserve the full path
        let tree = build_file_tree(&["a/b/c/file.txt".to_string()]);
        // With full collapse, only the leaf file remains at depth 0
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "file.txt");
        assert_eq!(tree[0].path, "a/b/c/file.txt");
        assert_eq!(tree[0].hunk_index, Some(0));
    }

    #[test]
    fn test_mixed_depth_files() {
        // Files at different depths should build correctly
        let tree = build_file_tree(&[
            "Cargo.toml".to_string(),
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
        ]);
        // Should have: dir "src", two files inside it, and "Cargo.toml" at root
        // Directories sort before files at the same depth
        assert!(!tree.is_empty());
        // The root-level directory "src" should come before root-level file "Cargo.toml"
        let dir_entries: Vec<_> = tree.iter().filter(|e| e.hunk_index.is_none()).collect();
        let file_entries: Vec<_> = tree.iter().filter(|e| e.hunk_index.is_some()).collect();
        assert_eq!(dir_entries.len(), 1, "should have 1 directory");
        assert_eq!(file_entries.len(), 3, "should have 3 files");
    }
}
