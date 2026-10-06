use std::sync::LazyLock;

use super::default::DEFAULT_REVSET;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevsetPreset {
    pub id: String,
    pub label: String,
    pub revset: String,
}

static REVSET_PRESETS: LazyLock<[RevsetPreset; 7]> = LazyLock::new(|| {
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
            id: "tags".to_owned(),
            label: "Tags".to_owned(),
            revset: "tags()".to_owned(),
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

pub fn default_revset_preset() -> RevsetPreset {
    RevsetPreset {
        id: "default".to_owned(),
        label: "Default".to_owned(),
        revset: DEFAULT_REVSET.to_owned(),
    }
}
