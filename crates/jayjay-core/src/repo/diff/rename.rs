use std::collections::HashMap;

use jj_lib::copies::CopyRecords;

use crate::types::*;
use jayjay_primitives::hex_sha256;

/// Folds each removed/added pair that the backend's copy records describe as a rename into one renamed hunk; copies (source still present) stay additions.
pub(super) fn pair_renames(hunks: &mut Vec<DiffHunk>, copies: &CopyRecords) {
    let index_by_path = |hunk_type: HunkType| -> HashMap<String, usize> {
        hunks
            .iter()
            .enumerate()
            .filter(|(_, hunk)| hunk.hunk_type == hunk_type)
            .map(|(index, hunk)| (hunk.path.clone(), index))
            .collect()
    };
    let removed_by_path = index_by_path(HunkType::Removed);
    let added_by_path = index_by_path(HunkType::Added);

    let mut matched_removed = Vec::new();
    for record in copies.iter() {
        // Ambiguous sources or targets resolve to no record, matching how `jj diff` drops them.
        if copies.for_source(&record.source).is_none()
            || copies.for_target(&record.target).is_none()
        {
            continue;
        }
        let (Some(&removed_index), Some(&added_index)) = (
            removed_by_path.get(record.source.as_internal_file_string()),
            added_by_path.get(record.target.as_internal_file_string()),
        ) else {
            continue;
        };

        let old_path = hunks[removed_index].path.clone();
        let removed_preview = hunks[removed_index].old.preview.clone();
        // Combine both sides so removed-side changes also invalidate the mark.
        let combined_identity = hex_sha256(
            format!(
                "rename|{}|{}",
                hunks[removed_index].review_identity, hunks[added_index].review_identity
            )
            .as_bytes(),
        );
        let byte_equal = hunks[removed_index].old.content == hunks[added_index].new.content;
        let removed_content = hunks[removed_index].old.content.clone();

        let added = &mut hunks[added_index];
        added.old_path = Some(old_path);
        added.hunk_type = HunkType::Renamed;
        added.old.preview = removed_preview;
        added.review_identity = combined_identity;
        if byte_equal {
            added.old.content = None;
            added.new.content = None;
        } else {
            added.old.content = removed_content;
        }
        matched_removed.push(removed_index);
    }

    matched_removed.sort_unstable();
    for &index in matched_removed.iter().rev() {
        hunks.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use jj_lib::backend::{CommitId, CopyRecord, FileId};
    use jj_lib::repo_path::RepoPathBuf;

    use super::*;

    fn hunk(path: &str, hunk_type: HunkType, old: Option<&str>, new: Option<&str>) -> DiffHunk {
        hunk_with_identity(path, hunk_type, old, new, "")
    }

    fn hunk_with_identity(
        path: &str,
        hunk_type: HunkType,
        old: Option<&str>,
        new: Option<&str>,
        review_identity: &str,
    ) -> DiffHunk {
        DiffHunk {
            path: path.to_owned(),
            old_path: None,
            old: DiffContent::new(old.map(str::to_owned), None),
            new: DiffContent::new(new.map(str::to_owned), None),
            hunk_type,
            supports_conflict_editor: false,
            supports_file_editor: false,
            review_identity: review_identity.to_owned(),
            projection: None,
        }
    }

    fn copies(pairs: &[(&str, &str)]) -> CopyRecords {
        let mut records = CopyRecords::default();
        records.add_records(pairs.iter().map(|(source, target)| CopyRecord {
            target: RepoPathBuf::from_internal_string(*target).unwrap(),
            target_commit: CommitId::new(vec![1]),
            source: RepoPathBuf::from_internal_string(*source).unwrap(),
            source_file: FileId::new(vec![0]),
            source_commit: CommitId::new(vec![0]),
        }));
        records
    }

    #[test]
    fn rename_with_changed_content_keeps_both_sides() {
        let mut hunks = vec![
            hunk("a/x.rs", HunkType::Removed, Some("a\nb\nc\n"), None),
            hunk("b/x.rs", HunkType::Added, None, Some("c\nb\na\n")),
        ];
        pair_renames(&mut hunks, &copies(&[("a/x.rs", "b/x.rs")]));
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert_eq!(hunks[0].old_path.as_deref(), Some("a/x.rs"));
        assert_eq!(hunks[0].old.content.as_deref(), Some("a\nb\nc\n"));
        assert_eq!(hunks[0].new.content.as_deref(), Some("c\nb\na\n"));
    }

    #[test]
    fn rename_review_identity_combines_both_sides() {
        let renamed_identity = |old_identity: &str| {
            let mut hunks = vec![
                hunk_with_identity(
                    "old.rs",
                    HunkType::Removed,
                    Some("body"),
                    None,
                    old_identity,
                ),
                hunk_with_identity("new.rs", HunkType::Added, None, Some("body"), "id-new"),
            ];
            pair_renames(&mut hunks, &copies(&[("old.rs", "new.rs")]));
            assert_eq!(hunks.len(), 1);
            hunks.remove(0).review_identity
        };
        let v1 = renamed_identity("id-old-v1");
        assert_ne!(v1, renamed_identity("id-old-v2"));
        assert_ne!(v1, "id-new");
    }

    #[test]
    fn copies_and_unrecorded_pairs_stay_separate() {
        let mut hunks = vec![
            hunk("kept.rs", HunkType::Modified, Some("a"), Some("b")),
            hunk("copy.rs", HunkType::Added, None, Some("a")),
            hunk("dir/AGENTS.md", HunkType::Removed, Some("old"), None),
            hunk("AGENTS.md", HunkType::Added, None, Some("new")),
        ];
        pair_renames(&mut hunks, &copies(&[("kept.rs", "copy.rs")]));
        let types: Vec<_> = hunks.iter().map(|hunk| hunk.hunk_type).collect();
        assert_eq!(
            types,
            [
                HunkType::Modified,
                HunkType::Added,
                HunkType::Removed,
                HunkType::Added
            ]
        );
    }

    #[test]
    fn rename_carries_old_preview_from_removed_hunk() {
        let mut removed = hunk(
            "old/icon.png",
            HunkType::Removed,
            Some("<image (100 bytes)>"),
            None,
        );
        removed.old.preview = Some(DiffPreview::Image {
            path: "/tmp/jayjay-images/abc123.png".to_owned(),
        });
        let added = hunk(
            "new/icon.png",
            HunkType::Added,
            None,
            Some("<image (100 bytes)>"),
        );

        let mut hunks = vec![removed, added];
        pair_renames(&mut hunks, &copies(&[("old/icon.png", "new/icon.png")]));

        assert_eq!(hunks.len(), 1);
        assert!(hunks[0].old.content.is_none());
        assert!(hunks[0].new.content.is_none());
        let Some(DiffPreview::Image { path }) = hunks[0].old.preview.as_ref() else {
            panic!("missing removed-side preview: {:?}", hunks[0].old.preview);
        };
        assert_eq!(path, "/tmp/jayjay-images/abc123.png");
    }
}
