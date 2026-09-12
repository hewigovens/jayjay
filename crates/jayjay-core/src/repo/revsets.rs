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

pub const DEFAULT_REVSET_DEPTH: u32 = 20;
pub const DEFAULT_REVSET: &str = "present(@) | ancestors(immutable_heads().., 20) | trunk()";

const DEFAULT_REVSET_PREFIX: &str = "present(@) | ancestors(immutable_heads().., ";
const DEFAULT_REVSET_SUFFIX: &str = ") | trunk()";

pub fn build_default_revset(depth: u32) -> String {
    format!("{DEFAULT_REVSET_PREFIX}{depth}{DEFAULT_REVSET_SUFFIX}")
}

/// `None` when the user wrote their own revset; only the default shape can be paged.
pub fn default_revset_depth(revset: &str) -> Option<u32> {
    revset
        .strip_prefix(DEFAULT_REVSET_PREFIX)?
        .strip_suffix(DEFAULT_REVSET_SUFFIX)?
        .parse()
        .ok()
}

/// Selects the exact commit and all its ancestors, independent of bookmark names.
pub fn ancestors_revset(commit_id: &str) -> String {
    format!("::commit_id({commit_id})")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_revset_depth_round_trips_and_rejects_other_revsets() {
        assert_eq!(
            default_revset_depth(DEFAULT_REVSET),
            Some(DEFAULT_REVSET_DEPTH)
        );
        assert_eq!(default_revset_depth(&build_default_revset(140)), Some(140));
        assert_eq!(default_revset_depth("all()"), None);
        assert_eq!(
            default_revset_depth(&build_default_revset(20).replace("20", "x")),
            None
        );
    }
}
