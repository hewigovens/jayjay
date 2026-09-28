//! Mock builders for this crate's own unit tests; a helper crate linking jayjay-core would build a second copy of these types, so they live here and the `mock` feature lets other crates' tests reuse them.

use crate::diff_edit::DiffEditFile;
use crate::types::{
    BookmarkInfo, ChangeInfo, CommitAuthor, DiffContent, DiffHunk, DiffProjection,
    DiffProjectionMode, DiffRenderKind, EdgeType, FileDiffStats, GraphEdge, GraphEntry, HunkType,
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

/// A local bookmark with no remotes; tests set the remote fields they exercise.
pub fn bookmark_info(name: &str) -> BookmarkInfo {
    BookmarkInfo {
        name: name.to_owned(),
        change_id: ShortId::new("abcdefghijkl".to_owned(), 3),
        description: String::new(),
        is_tracking_remote: false,
        is_deleted: false,
        is_conflicted: false,
        tracked_remotes: Vec::new(),
        available_remotes: Vec::new(),
        has_local_target: true,
        remote_targets: Vec::new(),
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

/// A modified file with empty contents; tests set the contents, identity and flags they exercise.
pub fn diff_hunk(path: &str) -> DiffHunk {
    DiffHunk {
        path: path.to_owned(),
        old_path: None,
        old: DiffContent::default(),
        new: DiffContent::default(),
        hunk_type: HunkType::Modified,
        supports_conflict_editor: false,
        supports_file_editor: false,
        review_identity: String::new(),
        projection: None,
    }
}

pub fn diff_projection(plugin_id: &str, mode: DiffProjectionMode) -> DiffProjection {
    DiffProjection {
        plugin_id: plugin_id.to_owned(),
        plugin_label: "Notebook".to_owned(),
        plugin_version: 1,
        mode,
        render_kind: DiffRenderKind::Markdown,
        virtual_path: "analysis.ipynb.md".to_owned(),
        diagnostics: Vec::new(),
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
