use crate::types::{StackLayerOutcome, SubmittedLayer};

pub(super) struct ForgeTarget {
    pub bookmark: String,
    pub base: String,
    pub title: String,
    pub body: String,
}
pub(super) fn non_empty_or(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
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

pub(super) fn created_with_number(
    target: &ForgeTarget,
    title: String,
    number: u32,
    url: String,
) -> SubmittedLayer {
    SubmittedLayer {
        bookmark: target.bookmark.clone(),
        base: target.base.clone(),
        title,
        outcome: StackLayerOutcome::Created,
        pr_number: number,
        pr_url: url,
        detail: format!("Created onto {}", target.base),
    }
}

pub(super) fn updated(
    target: &ForgeTarget,
    title: String,
    number: u32,
    url: String,
    label: String,
) -> SubmittedLayer {
    SubmittedLayer {
        bookmark: target.bookmark.clone(),
        base: target.base.clone(),
        title,
        outcome: StackLayerOutcome::Updated,
        pr_number: number,
        pr_url: url,
        detail: format!("{label} → base {}", target.base),
    }
}
