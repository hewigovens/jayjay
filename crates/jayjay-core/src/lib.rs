pub use jj_diff as diff;
pub use jj_diff::syntax;
pub mod dag;
pub mod diffedit_plan;
pub mod evolog_display;
pub mod file_tree;
pub mod hash;
pub mod jj_command;
mod repo;
pub mod review;
pub mod revsets;
pub mod tools;
mod types;

pub use evolog_display::{evolog_is_snapshot, evolog_operation_kind, evolog_visible_rows};
pub use jj_command::{
    jj_command_body, parse_jj_command_args, record_jj_command_history, run_jj_command_in_path,
};
pub use repo::{
    COMMIT_MESSAGE_PROMPT, DEFAULT_REVSET, DEFAULT_REVSET_DEPTH, Repo, build_default_revset,
    check_gh_environment, check_jj_environment, detect_ai_provider, find_existing_binary,
    jj_binary,
};
pub use tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS, ToolsConfig, open_in_editor, open_in_terminal};
pub use types::*;
