use jayjay_primitives::{DiffHunk, HunkType};
use jj_diff::{ReviewFileSnapshot, canonical_review_snapshot, display_group_canonical_indices};

use crate::placeholder::is_editable_text;

/// Canonical review fingerprints for a loaded file diff, or empty when stable text groups are unavailable.
pub fn review_snapshot_from_hunk(hunk: &DiffHunk) -> ReviewFileSnapshot {
    let Some((old, new)) = review_text_pair(hunk) else {
        return ReviewFileSnapshot::empty();
    };
    canonical_review_snapshot(old, new)
}

/// The rendered diff's change groups mapped onto the canonical fingerprints, from the same text pair the snapshot hashes.
pub fn review_display_group_map_from_hunk(
    hunk: &DiffHunk,
    ignore_whitespace: bool,
) -> Vec<Vec<u32>> {
    match review_text_pair(hunk) {
        Some((old, new)) => display_group_canonical_indices(old, new, ignore_whitespace),
        None => Vec::new(),
    }
}

fn review_text_pair(hunk: &DiffHunk) -> Option<(&str, &str)> {
    if hunk.projection.is_some()
        || hunk.is_content_free_rename()
        || hunk.is_conflict_only_placeholder()
        || hunk.old.preview.is_some()
        || hunk.new.preview.is_some()
    {
        return None;
    }
    let pair = match hunk.hunk_type {
        HunkType::Added => ("", hunk.new.content.as_deref()?),
        HunkType::Removed => (hunk.old.content.as_deref()?, ""),
        HunkType::Modified | HunkType::Renamed => {
            (hunk.old.content.as_deref()?, hunk.new.content.as_deref()?)
        }
    };
    if is_editable_text(pair.0) && is_editable_text(pair.1) {
        Some(pair)
    } else {
        None
    }
}

impl super::Repo {
    pub fn review_file_snapshot(
        &self,
        rev: &str,
        path: &str,
        old_path: Option<&str>,
    ) -> crate::types::CoreResult<ReviewFileSnapshot> {
        let hunk = match old_path {
            Some(old) if old != path => self.show_file_rename(rev, old, path)?,
            _ => self.show_file(rev, path)?,
        };
        Ok(review_snapshot_from_hunk(&hunk))
    }
}

#[cfg(test)]
mod tests {
    use jayjay_primitives::{DiffContent, DiffHunk, DiffPreview, HunkType};

    use super::{review_display_group_map_from_hunk, review_snapshot_from_hunk, review_text_pair};

    fn text_hunk(old: &str, new: &str) -> DiffHunk {
        DiffHunk {
            path: "a.txt".into(),
            old_path: None,
            old: DiffContent::new(Some(old.into()), None),
            new: DiffContent::new(Some(new.into()), None),
            hunk_type: HunkType::Modified,
            supports_conflict_editor: false,
            supports_file_editor: true,
            review_identity: "id".into(),
            projection: None,
        }
    }

    #[test]
    fn text_hunks_produce_canonical_groups() {
        let snapshot = review_snapshot_from_hunk(&text_hunk(
            "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\n",
            "head-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\n",
        ));
        assert_eq!(snapshot.fingerprints.len(), 2);

        let mut added = text_hunk("", "new\n");
        added.hunk_type = HunkType::Added;
        added.old.content = Some("ignored old content".into());
        assert_eq!(review_snapshot_from_hunk(&added).fingerprints.len(), 1);

        let mut removed = text_hunk("old\n", "");
        removed.hunk_type = HunkType::Removed;
        removed.new.content = Some("ignored new content".into());
        assert_eq!(review_snapshot_from_hunk(&removed).fingerprints.len(), 1);
    }

    #[test]
    fn binary_and_placeholder_hunks_do_not_invent_groups() {
        let mut binary = text_hunk("<binary file (12 bytes)>", "<binary file (12 bytes)>");
        binary.old.content = Some("<binary file (12 bytes)>".into());
        assert!(review_snapshot_from_hunk(&binary).fingerprints.is_empty());
        assert!(review_display_group_map_from_hunk(&binary, true).is_empty());
        assert!(review_text_pair(&binary).is_none());

        let mut image = text_hunk("old", "new");
        image.old.preview = Some(DiffPreview::Image {
            path: "/tmp/a.png".into(),
        });
        assert!(review_snapshot_from_hunk(&image).fingerprints.is_empty());

        let rename = DiffHunk {
            path: "b.txt".into(),
            old_path: Some("a.txt".into()),
            old: DiffContent::default(),
            new: DiffContent::default(),
            hunk_type: HunkType::Renamed,
            supports_conflict_editor: false,
            supports_file_editor: false,
            review_identity: "id".into(),
            projection: None,
        };
        assert!(rename.is_content_free_rename());
        assert!(review_snapshot_from_hunk(&rename).fingerprints.is_empty());
    }
}
