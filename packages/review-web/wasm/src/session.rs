use jayjay_primitives::avatar::AvatarSource;
use jayjay_primitives::{NoteSide, ReviewFileDiff, ReviewFileRollup, ReviewGroupState};
use jayjay_review::ReviewStore;
use jj_diff::ReviewFileSnapshot;
use serde::Deserialize;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use crate::bundle::{ReviewBundle, ReviewFile};

#[wasm_bindgen]
pub struct ReviewSession {
    bundle: ReviewBundle,
    snapshots: Vec<ReviewFileSnapshot>,
    stats: Vec<FileStats>,
    filtered_labels: Vec<String>,
}

struct FileStats {
    added: u32,
    removed: u32,
    kind: &'static str,
}

impl FileStats {
    fn new(file: &ReviewFile) -> Self {
        let (added, removed) = jj_diff::count_changed_lines(&file.before, &file.after, false);
        let kind = match (
            file.before.is_empty(),
            file.after.is_empty(),
            file.old_path.is_some(),
        ) {
            (_, _, true) => "renamed",
            (true, false, _) => "added",
            (false, true, _) => "removed",
            _ => "modified",
        };
        Self {
            added,
            removed,
            kind,
        }
    }
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum ReviewAction {
    SetFile {
        file: usize,
        reviewed: bool,
    },
    SetGroup {
        file: usize,
        group: u32,
        reviewed: bool,
        ignore_whitespace: bool,
    },
    AddNote {
        file: usize,
        side: NoteSide,
        line: u32,
        body: String,
        ignore_whitespace: bool,
    },
    ResolveNote {
        id: String,
    },
}

#[wasm_bindgen]
impl ReviewSession {
    #[wasm_bindgen(constructor)]
    pub fn new(bundle_json: &str) -> Result<ReviewSession, JsError> {
        Self::parse(bundle_json).map_err(|error| JsError::new(&error))
    }

    pub fn overview(&self, state: &str) -> Result<String, JsError> {
        self.describe(state)
            .map(|value| value.to_string())
            .map_err(|error| JsError::new(&error))
    }

    pub fn apply(&self, action: &str, state: &str) -> Result<String, JsError> {
        self.mutate(action, state)
            .map_err(|error| JsError::new(&error))
    }

    pub fn file_review(
        &self,
        index: usize,
        state: &str,
        ignore_whitespace: bool,
    ) -> Result<String, JsError> {
        self.file_state(index, state, ignore_whitespace)
            .map(|value| value.to_string())
            .map_err(|error| JsError::new(&error))
    }
}

/// The same operations for native hosts, which cannot create `JsError`s.
impl ReviewSession {
    pub fn parse(bundle_json: &str) -> Result<Self, String> {
        let bundle = ReviewBundle::parse(bundle_json)?;
        let snapshots = bundle
            .files
            .iter()
            .map(|file| jj_diff::canonical_review_snapshot(&file.before, &file.after))
            .collect();
        let stats = bundle.files.iter().map(FileStats::new).collect();
        // The file filter counts in the viewer, so every shown-of-total label is formatted once with the catalog's plural rules.
        let total = bundle.files.len();
        let filtered_labels = (0..=total)
            .map(|shown| jayjay_i18n::repo::files_filtered(shown, total))
            .collect();
        Ok(Self {
            bundle,
            snapshots,
            stats,
            filtered_labels,
        })
    }

    fn store(state: &str) -> Result<ReviewStore, String> {
        ReviewStore::in_memory_from_json(state).map_err(|error| error.to_string())
    }

    fn file(&self, index: usize) -> Result<(usize, &ReviewFile), String> {
        let file = self.bundle.files.get(index).ok_or("Unknown review file")?;
        Ok((index, file))
    }

    pub fn describe(&self, state: &str) -> Result<Value, String> {
        let store = Self::store(state)?;
        let notes = store
            .reconcile(&self.bundle, &self.bundle.change_id, true)
            .map_err(|error| error.to_string())?;
        let mut reviewed = 0;
        let files: Vec<Value> = self.bundle.files.iter().enumerate().map(|(i, file)| {
            let FileStats { added, removed, kind } = self.stats[i];
            let rollup = store.file_review_state(&self.bundle.change_id, &file.path, &file.review_identity, Some(&self.snapshots[i])).rollup();
            reviewed += usize::from(rollup == ReviewFileRollup::Reviewed);
            json!({ "path": file.path, "oldPath": file.old_path, "kind": kind, "added": added, "removed": removed,
                "review": rollup, "noteCount": notes.iter().filter(|note| note.note.path == file.path && !note.note.resolved).count() })
        }).collect();
        let total = files.len();
        let counts = json!({ "files": jayjay_i18n::repo::file_count(total), "reviewed": jayjay_i18n::diff::reviewed_of_total(reviewed, total),
            "filtered": self.filtered_labels });
        let tree: Vec<Value> = jayjay_primitives::file_tree::build_file_tree(&self.bundle.files.iter().map(|file| file.path.clone()).collect::<Vec<_>>())
            .into_iter().map(|entry| json!({"name": entry.name, "path": entry.path, "depth": entry.depth, "file": entry.hunk_index})).collect();
        Ok(
            json!({ "repositoryId": self.bundle.repository_id, "changeId": self.bundle.change_id, "title": self.bundle.title,
            "description": self.bundle.description, "author": self.bundle.author,
            "authorAvatar": AvatarSource::for_email(&self.bundle.author_email, 96), "files": files, "counts": counts, "tree": tree, "notes": notes }),
        )
    }

    pub fn file_state(
        &self,
        index: usize,
        state: &str,
        ignore_whitespace: bool,
    ) -> Result<Value, String> {
        let file = self.bundle.files.get(index).ok_or("Unknown review file")?;
        let store = Self::store(state)?;
        let mapping =
            jj_diff::display_group_canonical_indices(&file.before, &file.after, ignore_whitespace);
        let groups = store.display_hunk_states(
            &self.bundle.change_id,
            &file.path,
            &file.review_identity,
            &self.snapshots[index],
            &mapping,
        );
        Ok(json!({"groupStates": groups}))
    }

    pub fn mutate(&self, action: &str, state: &str) -> Result<String, String> {
        let action: ReviewAction =
            serde_json::from_str(action).map_err(|error| error.to_string())?;
        let mut store = Self::store(state)?;
        let change_id = &self.bundle.change_id;
        match action {
            ReviewAction::SetFile { file, reviewed } => {
                let (index, file) = self.file(file)?;
                if reviewed {
                    store.mark_reviewed_snapshot(
                        change_id,
                        &file.path,
                        &file.review_identity,
                        Some(&self.snapshots[index]),
                    );
                } else {
                    store.mark_unreviewed(change_id, &file.path);
                }
            }
            ReviewAction::SetGroup {
                file,
                group,
                reviewed,
                ignore_whitespace,
            } => {
                let (index, file) = self.file(file)?;
                let mapping = jj_diff::display_group_canonical_indices(
                    &file.before,
                    &file.after,
                    ignore_whitespace,
                );
                if group as usize >= mapping.len() {
                    return Err("Unknown review group".into());
                }
                let states = store.display_hunk_states(
                    change_id,
                    &file.path,
                    &file.review_identity,
                    &self.snapshots[index],
                    &mapping,
                );
                if (states.get(group as usize) == Some(&ReviewGroupState::Reviewed)) != reviewed {
                    store.toggle_display_group_snapshot(
                        change_id,
                        &file.path,
                        &file.review_identity,
                        &self.snapshots[index],
                        &mapping,
                        group,
                    );
                }
            }
            ReviewAction::AddNote {
                file,
                side,
                line,
                body,
                ignore_whitespace,
            } => {
                let (_, file) = self.file(file)?;
                let body =
                    jayjay_review::checked_note_body(&body).map_err(|error| error.to_string())?;
                let file_diff = ReviewFileDiff {
                    old_content: Some(file.before.clone()),
                    new_content: Some(file.after.clone()),
                };
                let anchor = jayjay_review::note_anchor(
                    change_id,
                    &file.path,
                    &file.review_identity,
                    &file_diff,
                    side,
                    line,
                    ignore_whitespace,
                )
                .ok_or("Notes must anchor to a changed line")?;
                store.add_note(anchor, body);
            }
            ReviewAction::ResolveNote { id } => {
                if !store
                    .list_notes(change_id, true)
                    .iter()
                    .any(|note| note.id == id)
                {
                    return Err("Unknown review note".into());
                }
                store.resolve_note(&id);
            }
        }
        store.snapshot_json().map_err(|error| error.to_string())
    }
}
