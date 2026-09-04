use std::sync::Arc;

use jayjay_core::diff::{FileDiff, compute_file_diff};
use jayjay_core::{
    CoreResult, DiffHunk, DiffPreview, DiffProjection, DiffProjectionMode, HunkType, Repo,
    review_display_group_map_from_hunk, review_snapshot_from_hunk,
};
use jayjay_markdown::MarkdownDocument;

use super::super::{LoadedReviewSnapshot, SvgPreviewContent};
use crate::diff::projection;

pub(super) struct ComputedDiff {
    pub(super) file_diff: FileDiff,
    /// Retained rather than re-read at action time: "Abandon Selected Lines" maps a selection back to these exact strings, and a fresh working-copy snapshot could silently mis-target the range.
    pub(super) old_content: Arc<str>,
    pub(super) new_content: Arc<str>,
    pub(super) old_preview: Option<DiffPreview>,
    pub(super) new_preview: Option<DiffPreview>,
    pub(super) supports_file_editor: bool,
    pub(super) projection: Option<DiffProjection>,
    pub(super) svg_preview: Option<SvgPreviewContent>,
    pub(super) markdown_preview: Option<MarkdownDocument>,
    pub(super) review: Option<Arc<LoadedReviewSnapshot>>,
}

impl Default for ComputedDiff {
    fn default() -> Self {
        Self {
            file_diff: compute_file_diff("", "", "", false),
            old_content: Arc::from(""),
            new_content: Arc::from(""),
            old_preview: None,
            new_preview: None,
            supports_file_editor: false,
            projection: None,
            svg_preview: None,
            markdown_preview: None,
            review: None,
        }
    }
}

pub(super) fn compute_diff_blocking(
    repo: &Repo,
    rev: &str,
    hunk: &DiffHunk,
    compare_from_rev: Option<&str>,
    projection_mode: Option<DiffProjectionMode>,
    ignore_whitespace: bool,
    reviewable: bool,
) -> CoreResult<ComputedDiff> {
    let path = hunk.path.clone();
    if hunk.is_content_free_rename() {
        return Ok(ComputedDiff {
            file_diff: compute_file_diff(&path, "", "", ignore_whitespace),
            ..Default::default()
        });
    }
    if hunk.is_conflict_only_placeholder() {
        return Ok(ComputedDiff {
            file_diff: compute_file_diff(&path, "", "", ignore_whitespace),
            ..Default::default()
        });
    }
    let mut old_preview = hunk.old.preview.clone();
    let mut new_preview = hunk.new.preview.clone();
    let mut projection = hunk.projection.clone();
    let mut supports_file_editor = hunk.supports_file_editor;
    let requested_raw = projection_mode
        .or_else(|| hunk.projection.as_ref().map(|projection| projection.mode))
        == Some(DiffProjectionMode::Raw);
    let projection_mode_changed = projection_mode.is_some_and(|mode| {
        hunk.projection
            .as_ref()
            .is_some_and(|projection| projection.mode != mode)
    });
    let (old, new): (Arc<str>, Arc<str>) =
        match (hunk.old.content.clone(), hunk.new.content.clone()) {
            (Some(o), Some(n)) if !(projection_mode_changed || o.is_empty() && n.is_empty()) => {
                (Arc::from(o), Arc::from(n))
            }
            _ => {
                let h = load_hunk(repo, rev, hunk, compare_from_rev, requested_raw)?;
                old_preview = h.old.preview.clone();
                new_preview = h.new.preview.clone();
                projection = h.projection.clone();
                supports_file_editor = h.supports_file_editor;
                (
                    Arc::from(h.old.content.unwrap_or_default()),
                    Arc::from(h.new.content.unwrap_or_default()),
                )
            }
        };
    let diff_path = projection
        .as_ref()
        .map(|projection| projection.virtual_path.as_str())
        .unwrap_or(&path);
    let svg_preview = projection::is_svg_path(&path).then(|| SvgPreviewContent {
        old: (!old.is_empty()).then(|| old.to_string()),
        new: (!new.is_empty()).then(|| new.to_string()),
    });
    let markdown_preview = (projection::renders_as_markdown(&path, projection.as_ref())
        && !new.is_empty())
    .then(|| MarkdownDocument::parse(new.to_string()));
    let review = if reviewable && compare_from_rev.is_none() {
        let mut hydrated = hunk.clone();
        hydrated.old.content = Some(old.to_string());
        hydrated.new.content = Some(new.to_string());
        hydrated.old.preview = old_preview.clone();
        hydrated.new.preview = new_preview.clone();
        hydrated.projection = projection.clone();
        let snapshot = review_snapshot_from_hunk(&hydrated);
        (!snapshot.fingerprints.is_empty()).then(|| {
            Arc::new(LoadedReviewSnapshot {
                display_groups: review_display_group_map_from_hunk(&hydrated, ignore_whitespace),
                snapshot,
            })
        })
    } else {
        None
    };
    Ok(ComputedDiff {
        file_diff: compute_file_diff(diff_path, &old, &new, ignore_whitespace),
        old_content: old,
        new_content: new,
        old_preview,
        new_preview,
        supports_file_editor,
        projection,
        svg_preview,
        markdown_preview,
        review,
    })
}

pub(super) fn diff_cache_key(
    compare_from_rev: Option<&str>,
    rev: &str,
    hunk: &DiffHunk,
    projection_mode: Option<DiffProjectionMode>,
    ignore_whitespace: bool,
) -> String {
    format!(
        "{}\0{}\0{}\0{}\0{}\0{}",
        compare_from_rev.unwrap_or(""),
        rev,
        hunk.path,
        hunk.review_identity,
        projection::cache_identity(hunk.projection.as_ref(), projection_mode),
        ignore_whitespace
    )
}

fn load_hunk(
    repo: &Repo,
    rev: &str,
    hunk: &DiffHunk,
    compare_from_rev: Option<&str>,
    raw: bool,
) -> CoreResult<DiffHunk> {
    let path = hunk.path.as_str();
    if let Some(from_rev) = compare_from_rev {
        if raw {
            return repo.interdiff_file_raw(from_rev, rev, path);
        }
        return repo.interdiff_file(from_rev, rev, path);
    }
    if hunk.hunk_type == HunkType::Renamed
        && let Some(old_path) = hunk.old_path.as_deref()
    {
        if raw {
            return repo.show_file_rename_raw(rev, old_path, path);
        }
        return repo.show_file_rename(rev, old_path, path);
    }
    if raw {
        repo.show_file_raw(rev, path)
    } else {
        repo.show_file(rev, path)
    }
}
