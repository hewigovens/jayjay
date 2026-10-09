use std::collections::HashMap;

use jj_lib::copies::CopyRecords;
use jj_lib::repo_path::RepoPath;

use crate::types::*;
use jayjay_primitives::hex_sha256;

pub(super) trait SourceLookup {
    fn source_exists(&self, source: &RepoPath) -> JayResult<bool>;
    fn matches_source(&self, source: &RepoPath, target: &RepoPath) -> JayResult<bool>;
}

/// Folds recorded renames into one hunk and marks targets whose source still exists as copies.
pub(super) fn apply_copy_records(
    hunks: &mut Vec<DiffHunk>,
    copies: &CopyRecords,
    lookup: &impl SourceLookup,
) -> JayResult<()> {
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
        // `jj diff` drops ambiguous targets too.
        if copies.for_target(&record.target).is_none() {
            continue;
        }
        let Some(&added_index) = added_by_path.get(record.target.as_internal_file_string()) else {
            continue;
        };
        let source = record.source.as_internal_file_string();
        let Some(&removed_index) = removed_by_path.get(source) else {
            // A merge's record can name a source its auto-merged parent tree lacks; there is nothing to diff against.
            if !lookup.source_exists(&record.source)? {
                continue;
            }
            // The identity stays the target's own, so editing the source elsewhere in the change never invalidates the copy's mark.
            let added = &mut hunks[added_index];
            added.matches_source = lookup.matches_source(&record.source, &record.target)?;
            added.old_path = Some(source.to_owned());
            added.hunk_type = HunkType::Copied;
            continue;
        };
        // A removed source claimed by several targets has no single rename.
        if copies.for_source(&record.source).is_none() {
            continue;
        }

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
        let matches_source = lookup.matches_source(&record.source, &record.target)?;
        let removed_content = hunks[removed_index].old.content.clone();

        let added = &mut hunks[added_index];
        added.old_path = Some(old_path);
        added.hunk_type = HunkType::Renamed;
        added.old.preview = removed_preview;
        added.review_identity = combined_identity;
        added.matches_source = matches_source;
        if matches_source {
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
    Ok(())
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
            matches_source: false,
        }
    }

    #[derive(Default)]
    struct Lookup {
        matching_targets: &'static [&'static str],
        missing_sources: &'static [&'static str],
    }

    impl SourceLookup for Lookup {
        fn source_exists(&self, source: &RepoPath) -> JayResult<bool> {
            Ok(!self
                .missing_sources
                .contains(&source.as_internal_file_string()))
        }

        fn matches_source(&self, _source: &RepoPath, target: &RepoPath) -> JayResult<bool> {
            Ok(self
                .matching_targets
                .contains(&target.as_internal_file_string()))
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

    fn apply(hunks: &mut Vec<DiffHunk>, copies: &CopyRecords, lookup: Lookup) {
        apply_copy_records(hunks, copies, &lookup).unwrap();
    }

    #[test]
    fn rename_with_changed_content_keeps_both_sides() {
        let mut hunks = vec![
            hunk("a/x.rs", HunkType::Removed, Some("a\nb\nc\n"), None),
            hunk("b/x.rs", HunkType::Added, None, Some("c\nb\na\n")),
        ];
        apply(
            &mut hunks,
            &copies(&[("a/x.rs", "b/x.rs")]),
            Lookup::default(),
        );
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert!(!hunks[0].is_content_free_rename_or_copy());
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
            apply(
                &mut hunks,
                &copies(&[("old.rs", "new.rs")]),
                Lookup::default(),
            );
            assert_eq!(hunks.len(), 1);
            hunks.remove(0).review_identity
        };
        let v1 = renamed_identity("id-old-v1");
        assert_ne!(v1, renamed_identity("id-old-v2"));
        assert_ne!(v1, "id-new");
    }

    #[test]
    fn copies_keep_their_identity_and_unusable_records_change_nothing() {
        let mut hunks = vec![
            hunk("kept.rs", HunkType::Modified, Some("a"), Some("b")),
            hunk_with_identity("copy.rs", HunkType::Added, None, Some("a"), "id-copy"),
            hunk("dir/AGENTS.md", HunkType::Removed, Some("old"), None),
            hunk("AGENTS.md", HunkType::Added, None, Some("new")),
            hunk("orphan.rs", HunkType::Added, None, Some("a")),
        ];
        let lookup = Lookup {
            missing_sources: &["gone.rs"],
            ..Lookup::default()
        };
        apply(
            &mut hunks,
            &copies(&[("kept.rs", "copy.rs"), ("gone.rs", "orphan.rs")]),
            lookup,
        );
        let cards: Vec<_> = hunks
            .iter()
            .map(|hunk| (hunk.path.as_str(), hunk.old_path.as_deref(), hunk.hunk_type))
            .collect();
        assert_eq!(
            cards,
            [
                ("kept.rs", None, HunkType::Modified),
                ("copy.rs", Some("kept.rs"), HunkType::Copied),
                ("dir/AGENTS.md", None, HunkType::Removed),
                ("AGENTS.md", None, HunkType::Added),
                ("orphan.rs", None, HunkType::Added),
            ]
        );
        assert_eq!(
            hunks[1].review_identity, "id-copy",
            "a copy keeps its own identity"
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
        apply(
            &mut hunks,
            &copies(&[("old/icon.png", "new/icon.png")]),
            Lookup {
                matching_targets: &["new/icon.png"],
                ..Lookup::default()
            },
        );

        assert_eq!(hunks.len(), 1);
        assert!(hunks[0].is_content_free_rename_or_copy());
        assert!(hunks[0].old.content.is_none());
        assert!(hunks[0].new.content.is_none());
        let Some(DiffPreview::Image { path }) = hunks[0].old.preview.as_ref() else {
            panic!("missing removed-side preview: {:?}", hunks[0].old.preview);
        };
        assert_eq!(path, "/tmp/jayjay-images/abc123.png");
    }
}
