pub use jj_diff as diff;
pub use jj_diff::syntax;
pub mod commit_message;
pub mod dag;
pub mod file_tree;
pub mod fuzzy;
pub mod hash;
mod jj_command;
pub mod palette;
pub mod placeholder;
mod repo;
pub mod review;
pub mod theme;
pub mod tools;
mod types;

pub use jj_command::{JjCommand, JjCommandResult};
pub use repo::{
    COMMIT_MESSAGE_PROMPT, DEFAULT_REVSET, DEFAULT_REVSET_DEPTH, Repo, build_default_revset,
    check_gh_environment, check_glab_environment, check_jj_environment, detect_ai_provider,
    find_existing_binary, init_jj_git_repo, is_valid_bookmark_name, jj_binary, login_shell,
    login_shell_path,
};
pub use theme::{DiffThemeColors, change_id_prefix_color, diff_theme_colors};
pub use tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS, ToolsConfig, open_in_editor, open_in_terminal};
pub use types::*;
