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
        }
    }
}
