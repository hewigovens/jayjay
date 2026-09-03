use std::sync::LazyLock;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevsetPreset {
    pub id: String,
    pub label: String,
    pub revset: String,
}

static REVSET_PRESETS: LazyLock<[RevsetPreset; 6]> = LazyLock::new(|| {
    [
        RevsetPreset {
            id: "all".to_owned(),
            label: "All".to_owned(),
            revset: "all()".to_owned(),
        },
        RevsetPreset {
            id: "mine".to_owned(),
            label: "Mine".to_owned(),
            revset: "mine()".to_owned(),
        },
        RevsetPreset {
            id: "bookmarks".to_owned(),
            label: "Bookmarks".to_owned(),
            revset: "bookmarks()".to_owned(),
        },
        RevsetPreset {
            id: "trunk".to_owned(),
            label: "Trunk".to_owned(),
            revset: "trunk()".to_owned(),
        },
        RevsetPreset {
            id: "conflicts".to_owned(),
            label: "Conflicts".to_owned(),
            revset: "conflicts()".to_owned(),
        },
        RevsetPreset {
            id: "heads".to_owned(),
            label: "Heads".to_owned(),
            revset: "heads(all())".to_owned(),
        },
    ]
});

pub fn revset_presets() -> &'static [RevsetPreset] {
    REVSET_PRESETS.as_slice()
}

pub fn build_default_revset(depth: u32) -> String {
    format!("present(@) | ancestors(immutable_heads().., {depth}) | trunk()")
}

/// Context depth for the pinned `builtin_log()` fallback used by [`LogQuery::Default`] — matches the pinned `jj` release's own `revsets.log` default, independent of how many rows a page requests.
pub const DEFAULT_LOG_CONTEXT_DEPTH: u32 = 2;

/// Real change rows one page load (or one **Load More**) returns.
pub const LOG_PAGE_SIZE: u32 = 50;

/// Hard ceiling on rows one graph view holds; beyond this the UI must ask the user to narrow the revset.
pub const MAX_LOADED_LOG_ROWS: u32 = 500;

/// Selects which revset a log/graph load evaluates: `Default` resolves the repository's `revsets.log` setting (falling back to the pinned `builtin_log()` expression), `Explicit` is used as-is, and both stay subject to the caller's row limit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogQuery {
    Default,
    Explicit(String),
}

pub fn combined_diff_revsets(revisions: &[String]) -> Option<(String, String)> {
    let mut unique_revisions = Vec::with_capacity(revisions.len());
    for revision in revisions.iter().map(|revision| revision.trim()) {
        if !revision.is_empty() && !unique_revisions.contains(&revision) {
            unique_revisions.push(revision);
        }
    }
    if unique_revisions.len() < 2 {
        return None;
    }

    let selection = unique_revisions
        .into_iter()
        .map(|revision| format!("({revision})"))
        .collect::<Vec<_>>()
        .join(" | ");
    Some((
        format!("roots({selection})-"),
        format!("heads({selection})"),
    ))
}
