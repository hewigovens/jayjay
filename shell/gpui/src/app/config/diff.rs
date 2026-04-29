use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct DiffConfig {
    pub side_by_side: bool,
    pub ignore_whitespace: bool,
    pub hide_git_lfs: bool,
    pub enable_git_submodule_support: bool,
    pub tree_file_list: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            side_by_side: false,
            ignore_whitespace: false,
            hide_git_lfs: true,
            enable_git_submodule_support: false,
            tree_file_list: false,
        }
    }
}
