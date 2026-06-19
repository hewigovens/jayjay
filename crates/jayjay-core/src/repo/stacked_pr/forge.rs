use crate::types::{StackLayerOutcome, SubmittedLayer};

/// One PR/MR target — the subset of a stack layer the forge clients use.
pub(super) struct ForgeTarget {
    pub bookmark: String,
    pub base: String,
    pub title: String,
    pub body: String,
}

/// `value` if non-empty, else `fallback` — for title/description fallbacks.
pub(super) fn non_empty_or(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

/// The numeric PR/MR id from a `.../<n>` URL.
pub(super) fn number_from_url(url: &str) -> Option<u32> {
    url.trim_end_matches('/').rsplit('/').next()?.parse().ok()
}

pub(super) fn failed(target: &ForgeTarget, detail: String) -> SubmittedLayer {
    SubmittedLayer {
        bookmark: target.bookmark.clone(),
        base: target.base.clone(),
        title: target.title.clone(),
        outcome: StackLayerOutcome::Failed,
        pr_number: 0,
        pr_url: String::new(),
        detail: detail.trim().to_owned(),
    }
}
