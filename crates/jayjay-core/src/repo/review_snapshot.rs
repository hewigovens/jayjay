use jayjay_primitives::DiffHunk;
use jj_diff::{ReviewFileSnapshot, canonical_review_snapshot};

use crate::placeholder::is_editable_text;

/// Canonical review fingerprints for a loaded file diff, or empty when stable text groups are unavailable.
pub fn review_snapshot_from_hunk(hunk: &DiffHunk) -> ReviewFileSnapshot {
    if !hunk_supports_group_fingerprints(hunk) {
        return ReviewFileSnapshot::empty();
    }
    canonical_review_snapshot(
        hunk.old.content.as_deref().unwrap_or(""),
        hunk.new.content.as_deref().unwrap_or(""),
    )
}

fn hunk_supports_group_fingerprints(hunk: &DiffHunk) -> bool {
    if hunk.projection.is_some()
        || hunk.is_content_free_rename()
        || hunk.is_conflict_only_placeholder()
        || hunk.old.preview.is_some()
        || hunk.new.preview.is_some()
    {
        return false;
    }
    match (hunk.old.content.as_deref(), hunk.new.content.as_deref()) {
        (Some(old), Some(new)) => is_editable_text(old) && is_editable_text(new),
        _ => false,
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

    use super::{hunk_supports_group_fingerprints, review_snapshot_from_hunk};

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
    }

    #[test]
    fn binary_and_placeholder_hunks_do_not_invent_groups() {
        let mut binary = text_hunk("<binary file (12 bytes)>", "<binary file (12 bytes)>");
        binary.old.content = Some("<binary file (12 bytes)>".into());
        assert!(review_snapshot_from_hunk(&binary).fingerprints.is_empty());
        assert!(!hunk_supports_group_fingerprints(&binary));

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
