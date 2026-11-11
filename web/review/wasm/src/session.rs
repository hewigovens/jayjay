use jayjay_primitives::avatar::AvatarSource;
use jayjay_primitives::seti::SetiGlyph;
use jayjay_primitives::{
    NoteSide, NoteStatus, ReviewFileDiff, ReviewFileRollup, ReviewGroupState, ReviewNoteStatus,
};
use jayjay_review::ReviewStore;
use jj_diff::ReviewFileSnapshot;
use serde::Deserialize;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use crate::bundle::{ReviewBundle, ReviewFile};

#[wasm_bindgen]
pub struct ReviewSession {
    bundle: ReviewBundle,
    /// `None` until the file's content is loaded.
    snapshots: Vec<Option<ReviewFileSnapshot>>,
    stats: Vec<(u32, u32)>,
    filtered_labels: Vec<String>,
    description_html: String,
}

/// Counts the host supplied, else counts from loaded content, else zero until it loads.
fn line_counts(file: &ReviewFile) -> (u32, u32) {
    match (file.added, file.removed, file.content()) {
        (Some(added), Some(removed), _) => (added, removed),
        (_, _, Some((before, after))) => jj_diff::count_changed_lines(before, after, false),
        _ => (0, 0),
    }
}

fn snapshot(file: &ReviewFile) -> Option<ReviewFileSnapshot> {
    file.content()
        .map(|(before, after)| jj_diff::canonical_review_snapshot(before, after))
}

fn file_icon(path: &str) -> Value {
    let glyph = SetiGlyph::for_path(path);
    let metrics = glyph.metrics();
    let (light, dark) = glyph.colors();
    json!({ "light": light, "dark": dark, "glyph": glyph.0.to_string(), "scale": metrics.scale, "inkLeft": metrics.ink_left, "inkWidth": metrics.ink_width, "inkCenter": metrics.ink_center })
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
    UpdateNote {
        id: String,
        body: String,
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

    pub fn load_file(&mut self, index: usize, before: &str, after: &str) -> Result<(), JsError> {
        self.load(index, before, after)
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
        let snapshots = bundle.files.iter().map(snapshot).collect();
        let stats = bundle.files.iter().map(line_counts).collect();
        // The file filter counts in the viewer, so every shown-of-total label is formatted once with the catalog's plural rules.
        let total = bundle.files.len();
        let filtered_labels = (0..=total)
            .map(|shown| jayjay_i18n::repo::files_filtered(shown, total))
            .collect();
        let description_html = jayjay_markdown::render_markdown_fragment(
            &bundle.description,
            jayjay_markdown::MarkdownImages::Remote,
        );
        Ok(Self {
            bundle,
            snapshots,
            stats,
            filtered_labels,
            description_html,
        })
    }

    pub fn load(&mut self, index: usize, before: &str, after: &str) -> Result<(), String> {
        let file = self
            .bundle
            .files
            .get_mut(index)
            .ok_or("Unknown review file")?;
        file.load(before, after)?;
        self.snapshots[index] = snapshot(file);
        self.stats[index] = line_counts(file);
        Ok(())
    }

    /// A note on a file whose content has not loaded yet cannot be placed; its identity still decides whether it is stale.
    fn pending_note(&self, status: ReviewNoteStatus) -> ReviewNoteStatus {
        let unloaded = self.bundle.files.iter().any(|file| {
            file.path == status.note.path
                && file.content().is_none()
                && file.review_identity == status.note.identity
        });
        if unloaded && status.status == NoteStatus::Stale {
            ReviewNoteStatus::new(&status.note, NoteStatus::Current, None)
        } else {
            status
        }
    }

    fn store(state: &str) -> Result<ReviewStore, String> {
        ReviewStore::in_memory_from_json(state).map_err(|error| error.to_string())
    }

    fn loaded(&self, index: usize) -> Result<Loaded<'_>, String> {
        let file = self.bundle.files.get(index).ok_or("Unknown review file")?;
        let (before, after) = file.content().ok_or("Review file is not loaded")?;
        let snapshot = self.snapshots[index]
            .as_ref()
            .ok_or("Review file is not loaded")?;
        Ok(Loaded {
            file,
            before,
            after,
            snapshot,
        })
    }

    pub fn describe(&self, state: &str) -> Result<Value, String> {
        let store = Self::store(state)?;
        let notes: Vec<ReviewNoteStatus> = store
            .reconcile(&self.bundle, &self.bundle.change_id, true)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|status| self.pending_note(status))
            .collect();
        let mut reviewed = 0;
        let files: Vec<Value> = self.bundle.files.iter().enumerate().map(|(i, file)| {
            let (added, removed) = self.stats[i];
            let kind = file.kind().as_str();
            let rollup = store.file_review_state(&self.bundle.change_id, &file.path, &file.review_identity, self.snapshots[i].as_ref()).rollup();
            reviewed += usize::from(rollup == ReviewFileRollup::Reviewed);
            json!({ "path": file.path, "oldPath": file.old_path, "kind": kind, "icon": file_icon(&file.path), "added": added, "removed": removed,
                "notice": file.notice(), "review": rollup, "noteCount": notes.iter().filter(|note| note.note.path == file.path && !note.note.resolved).count() })
        }).collect();
        let total = files.len();
        let counts = json!({ "files": jayjay_i18n::repo::file_count(total), "reviewed": jayjay_i18n::diff::reviewed_of_total(reviewed, total),
            "filtered": self.filtered_labels });
        let tree: Vec<Value> = jayjay_primitives::file_tree::build_file_tree(&self.bundle.files.iter().map(|file| file.path.clone()).collect::<Vec<_>>())
            .into_iter().map(|entry| json!({"name": entry.name, "path": entry.path, "depth": entry.depth, "file": entry.hunk_index})).collect();
        Ok(
            json!({ "repositoryId": self.bundle.repository_id, "changeId": self.bundle.change_id, "changeIdPrefix": self.bundle.change_id_prefix, "title": self.bundle.title, "timestampMillis": self.bundle.timestamp_millis,
            "description": self.bundle.description, "descriptionHtml": self.description_html, "author": self.bundle.author,
            "authorAvatar": AvatarSource::for_email(&self.bundle.author_email, 96), "files": files, "counts": counts, "tree": tree, "notes": notes }),
        )
    }

    pub fn file_state(
        &self,
        index: usize,
        state: &str,
        ignore_whitespace: bool,
    ) -> Result<Value, String> {
        let loaded = self.loaded(index)?;
        let store = Self::store(state)?;
        let mapping = jj_diff::display_group_canonical_indices(
            loaded.before,
            loaded.after,
            ignore_whitespace,
        );
        let groups = store.display_hunk_states(
            &self.bundle.change_id,
            &loaded.file.path,
            &loaded.file.review_identity,
            loaded.snapshot,
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
                let Loaded { file, snapshot, .. } = self.loaded(file)?;
                if reviewed {
                    store.mark_reviewed_snapshot(
                        change_id,
                        &file.path,
                        &file.review_identity,
                        Some(snapshot),
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
                let Loaded {
                    file,
                    before,
                    after,
                    snapshot,
                } = self.loaded(file)?;
                let mapping =
                    jj_diff::display_group_canonical_indices(before, after, ignore_whitespace);
                if group as usize >= mapping.len() {
                    return Err("Unknown review group".into());
                }
                let states = store.display_hunk_states(
                    change_id,
                    &file.path,
                    &file.review_identity,
                    snapshot,
                    &mapping,
                );
                if (states.get(group as usize) == Some(&ReviewGroupState::Reviewed)) != reviewed {
                    store.toggle_display_group_snapshot(
                        change_id,
                        &file.path,
                        &file.review_identity,
                        snapshot,
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
                let Loaded {
                    file,
                    before,
                    after,
                    ..
                } = self.loaded(file)?;
                let body =
                    jayjay_review::checked_note_body(&body).map_err(|error| error.to_string())?;
                let file_diff = ReviewFileDiff {
                    old_content: Some(before.to_owned()),
                    new_content: Some(after.to_owned()),
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
            ReviewAction::UpdateNote { id, body } => {
                if !store
                    .list_notes(change_id, true)
                    .iter()
                    .any(|note| note.id == id)
                {
                    return Err("Unknown review note".into());
                }
                let body =
                    jayjay_review::checked_note_body(&body).map_err(|error| error.to_string())?;
                store.update_note(&id, body);
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

struct Loaded<'a> {
    file: &'a ReviewFile,
    before: &'a str,
    after: &'a str,
    snapshot: &'a ReviewFileSnapshot,
}
