use super::default::default_revset_depth;
use super::expressions::bookmark_filter_name;
use super::presets::{default_revset_preset, revset_presets};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevsetFilterKind {
    Default,
    Preset,
    Bookmark,
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
            return Self::named(RevsetFilterKind::Default, &default_revset_preset().label);
        }
        if let Some(preset) = revset_presets()
            .iter()
            .find(|preset| preset.revset == revset)
        {
            return Self::named(RevsetFilterKind::Preset, &preset.label);
        }
        if let Some(name) = bookmark_filter_name(revset) {
            return Self::named(RevsetFilterKind::Bookmark, &name);
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
    use crate::revset::{DEFAULT_REVSET, bookmark_filter_revset, build_default_revset};

    #[test]
    fn filters_name_the_default_presets_and_bookmarks_they_came_from() {
        for (revset, kind, label) in [
            (
                DEFAULT_REVSET.to_owned(),
                RevsetFilterKind::Default,
                "Default",
            ),
            (
                build_default_revset(80),
                RevsetFilterKind::Default,
                "Default",
            ),
            ("mine()".to_owned(), RevsetFilterKind::Preset, "Mine"),
            (
                bookmark_filter_revset("work", Some("upstream")),
                RevsetFilterKind::Bookmark,
                "work@upstream",
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
