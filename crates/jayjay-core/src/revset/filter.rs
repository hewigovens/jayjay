use super::default::default_revset_depth;
use super::expressions::{bookmark_filter_name, workspace_filter_name};
use super::presets::revset_presets;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevsetFilterKind {
    Default,
    Preset,
    Bookmark,
    Workspace,
    Custom,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevsetFilter {
    pub kind: RevsetFilterKind,
    pub label: String,
}

impl RevsetFilter {
    pub fn of(revset: &str) -> Self {
        if default_revset_depth(revset).is_some() {
            return Self::named(RevsetFilterKind::Default, &jayjay_i18n::repo::revset());
        }
        if let Some(preset) = revset_presets()
            .into_iter()
            .find(|preset| preset.revset == revset)
        {
            return Self::named(RevsetFilterKind::Preset, &preset.label);
        }
        if let Some(name) = bookmark_filter_name(revset) {
            return Self::named(RevsetFilterKind::Bookmark, &name);
        }
        if let Some(name) = workspace_filter_name(revset) {
            return Self::named(RevsetFilterKind::Workspace, &format!("{name}@"));
        }
        Self::named(RevsetFilterKind::Custom, "")
    }

    fn named(kind: RevsetFilterKind, label: &str) -> Self {
        Self {
            kind,
            label: label.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revset::{
        DEFAULT_REVSET, bookmark_filter_revset, build_default_revset, workspace_filter_revset,
    };

    #[test]
    fn filters_name_the_default_presets_bookmarks_and_workspaces_they_came_from() {
        for (revset, kind, label) in [
            (
                DEFAULT_REVSET.to_owned(),
                RevsetFilterKind::Default,
                "Revset",
            ),
            (
                build_default_revset(80),
                RevsetFilterKind::Default,
                "Revset",
            ),
            ("mine()".to_owned(), RevsetFilterKind::Preset, "Mine"),
            (
                bookmark_filter_revset("work", Some("upstream")),
                RevsetFilterKind::Bookmark,
                "work@upstream",
            ),
            (
                workspace_filter_revset("my-font"),
                RevsetFilterKind::Workspace,
                "my-font@",
            ),
            ("::main".to_owned(), RevsetFilterKind::Custom, ""),
        ] {
            assert_eq!(
                RevsetFilter::of(&revset),
                RevsetFilter::named(kind, label),
                "{revset}"
            );
        }
    }
}
