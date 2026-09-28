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

    fn flatten(&self, depth: u32, parent_path: &str, results: &mut Vec<FileTreeEntry>) {
        // Sort: directories first, then files
        let mut sorted: Vec<&(String, TreeNode)> = self.children.iter().collect();
        sorted.sort_by_key(|(_, n)| n.hunk_index.is_some());

        for (key, child) in sorted {
            if let Some(idx) = child.hunk_index {
                let path = if parent_path.is_empty() {
                    key.clone()
                } else {
                    format!("{parent_path}/{key}")
                };
                results.push(FileTreeEntry {
                    name: key.clone(),
                    path,
                    depth,
                    hunk_index: Some(idx),
                });
            } else {
                let dir_segment = child.name.clone();
                let dir_path = if parent_path.is_empty() {
                    dir_segment.clone()
                } else {
                    format!("{parent_path}/{dir_segment}")
                };
                results.push(FileTreeEntry {
                    name: dir_segment,
                    path: dir_path.clone(),
                    depth,
                    hunk_index: None,
                });
                child.flatten(depth + 1, &dir_path, results);
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
    let root_prefix = root.name.clone();
    root.flatten(0, &root_prefix, &mut results);

    // File entries point at the original on-disk path.
    for entry in &mut results {
        if let Some(idx) = entry.hunk_index
            && let Some(p) = paths.get(idx as usize)
        {
            entry.path = p.clone();
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Every entry — including directories — has a non-empty, unique `path`.
    #[test]
    fn test_directory_paths_are_unique_and_populated() {
        let tree = build_file_tree(&[
            "src/diff/line.rs".to_string(),
            "src/diff/mod.rs".to_string(),
            "src/repo/log.rs".to_string(),
            "src/repo/mod.rs".to_string(),
        ]);

        for entry in &tree {
            assert!(
                !entry.path.is_empty(),
                "entry {:?} has empty path",
                entry.name
            );
        }

        let paths: std::collections::HashSet<&str> = tree.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            paths.len(),
            tree.len(),
            "duplicate paths found in tree: {:?}",
            tree.iter().map(|e| &e.path).collect::<Vec<_>>()
        );

        let dirs: Vec<&FileTreeEntry> = tree.iter().filter(|e| e.hunk_index.is_none()).collect();
        let dir_paths: Vec<&str> = dirs.iter().map(|e| e.path.as_str()).collect();
        // Directory paths must be the full accumulated path (including any
        // prefix collapsed into root) so they prefix-match their file children.
        assert!(
            dir_paths.contains(&"src/diff"),
            "missing 'src/diff' dir, got {dir_paths:?}"
        );
        assert!(
            dir_paths.contains(&"src/repo"),
            "missing 'src/repo' dir, got {dir_paths:?}"
        );
        let dir_path: &str = "src/diff";
        assert!(
            tree.iter()
                .any(|e| e.hunk_index.is_some() && e.path.starts_with(&format!("{dir_path}/"))),
            "files under {dir_path} should have it as a prefix"
        );
    }
}
