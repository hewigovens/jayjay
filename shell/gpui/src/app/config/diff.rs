use jayjay_core::DiffPalette;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct DiffConfig {
    pub(crate) side_by_side: bool,
    pub(crate) ignore_whitespace: bool,
    pub(crate) hide_git_lfs: bool,
    pub(crate) enable_git_submodule_support: bool,
    pub tree_file_list: bool,
    pub hide_reviewed_files: bool,
    pub auto_expand_description: bool,
    pub palette: DiffPaletteSelection,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            side_by_side: false,
            ignore_whitespace: false,
            hide_git_lfs: true,
            enable_git_submodule_support: false,
            tree_file_list: false,
            hide_reviewed_files: false,
            auto_expand_description: false,
            palette: DiffPaletteSelection::default(),
        }
    }
}

/// Stored palette ids, kept verbatim even when this build does not know them.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct DiffPaletteSelection {
    pub light: String,
    pub dark: String,
}

impl Default for DiffPaletteSelection {
    fn default() -> Self {
        Self {
            light: DiffPalette::default_id(false).to_owned(),
            dark: DiffPalette::default_id(true).to_owned(),
        }
    }
}

impl DiffPaletteSelection {
    pub fn resolve(&self, is_dark: bool) -> DiffPalette {
        DiffPalette::resolve(if is_dark { &self.dark } else { &self.light }, is_dark)
    }
}
