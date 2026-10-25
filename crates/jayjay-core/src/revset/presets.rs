use super::default::DEFAULT_REVSET;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevsetPreset {
    pub id: String,
    pub label: String,
    pub revset: String,
}

pub fn revset_presets() -> Vec<RevsetPreset> {
    vec![
        RevsetPreset {
            id: "all".to_owned(),
            label: jayjay_i18n::repo::preset_all(),
            revset: "all()".to_owned(),
        },
        RevsetPreset {
            id: "mine".to_owned(),
            label: jayjay_i18n::repo::preset_mine(),
            revset: "mine()".to_owned(),
        },
        RevsetPreset {
            id: "bookmarks".to_owned(),
            label: jayjay_i18n::repo::bookmarks(),
            revset: "bookmarks()".to_owned(),
        },
        RevsetPreset {
            id: "tags".to_owned(),
            label: jayjay_i18n::repo::preset_tags(),
            revset: "tags()".to_owned(),
        },
        RevsetPreset {
            id: "trunk".to_owned(),
            label: jayjay_i18n::repo::preset_trunk(),
            revset: "trunk()".to_owned(),
        },
        RevsetPreset {
            id: "conflicts".to_owned(),
            label: jayjay_i18n::repo::preset_conflicts(),
            revset: "conflicts()".to_owned(),
        },
        RevsetPreset {
            id: "heads".to_owned(),
            label: jayjay_i18n::repo::preset_heads(),
            revset: "heads(all())".to_owned(),
        },
    ]
}

pub fn default_revset_preset() -> RevsetPreset {
    RevsetPreset {
        id: "default".to_owned(),
        label: jayjay_i18n::repo::preset_default(),
        revset: DEFAULT_REVSET.to_owned(),
    }
}
