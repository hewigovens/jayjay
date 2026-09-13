//! Mock builders for this crate's own unit tests; a helper crate linking jayjay-core would build a second copy of these types, so they live here and the `mock` feature lets other crates' tests reuse them.

use crate::diff_edit::DiffEditFile;
use crate::types::{
    ChangeInfo, CommitAuthor, EdgeType, FileDiffStats, GraphEdge, GraphEntry, HunkType,
    NewChangeEligibility, ShortId,
};

pub fn change_info(change_id: &str, commit_id: &str) -> ChangeInfo {
    ChangeInfo {
        change_id: ShortId::new(change_id.to_owned(), 1),
        commit_id: ShortId::new(commit_id.to_owned(), 1),
        description: String::new(),
        author: CommitAuthor::empty(0),
        parents: Vec::new(),
        bookmarks: Vec::new(),
        tags: Vec::new(),
        workspaces: Vec::new(),
        is_working_copy: false,
        has_conflict: false,
        is_empty: false,
        is_immutable: false,
        is_divergent: false,
        new_change: NewChangeEligibility {
            on_top: true,
            before: true,
            after: true,
        },
    }
}

pub fn graph_entry(commit_id: &str, parents: &[&str]) -> GraphEntry {
    let mut change = change_info(&format!("change-{commit_id}"), commit_id);
    change.parents = strings(parents);
    GraphEntry {
        change,
        edges: parents
            .iter()
            .map(|target| GraphEdge {
                target: (*target).to_owned(),
                edge_type: EdgeType::Direct,
            })
            .collect(),
    }
}

pub fn diff_edit_file(path: &str, changed_lines: &[u32]) -> DiffEditFile {
    DiffEditFile {
        path: path.to_owned(),
        old_path: None,
        hunk_type: HunkType::Modified,
        old_content: Some("old\n".to_owned()),
        new_content: Some("new\n".to_owned()),
        changed_lines: changed_lines.to_vec(),
    }
}

pub fn file_diff_stats(path: &str, insertions: u32) -> FileDiffStats {
    FileDiffStats {
        path: path.to_owned(),
        insertions,
        deletions: 0,
    }
}

pub fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
