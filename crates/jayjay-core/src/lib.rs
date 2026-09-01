pub use jayjay_primitives::{
    EvologRow, JAYJAY_CONFIG_COMMAND, JAYJAY_REVIEW_COMMAND, JAYJAY_TOOL_COMMAND, JJ_TOOL_CONFIG,
    evolog_rows, is_snapshot_operation,
};
pub use jj_diff as diff;
pub use jj_diff::syntax;
#[cfg(feature = "repository")]
mod cli;
pub mod commit_message;
pub mod dag;
#[cfg(feature = "repository")]
pub mod external_tools;
#[cfg(feature = "repository")]
mod file_display;
pub mod file_tree;
#[cfg(feature = "repository")]
mod filesystem;
pub mod fonts;
pub mod fuzzy;
#[cfg(feature = "repository")]
mod jj_command;
#[cfg(feature = "repository")]
mod merge_editor;
pub mod palette;
pub mod placeholder;
pub mod projection;
#[cfg(feature = "repository")]
mod repo;
#[cfg(feature = "repository")]
pub mod repositories;
pub mod theme;
#[cfg(feature = "repository")]
pub mod tools;
mod types;

#[cfg(feature = "repository")]
pub use cli::run_app_cli_command;
pub use fonts::{
    MONO_FONT_FALLBACK_NAMES, MONO_FONT_OPTIONS, MonoFontOption, SYSTEM_MONO_FONT_ID,
    mono_font_option,
};
#[cfg(feature = "repository")]
pub use jj_command::{JjCommand, JjCommandResult};
#[cfg(feature = "repository")]
pub use merge_editor::{
    merge_hunk_display_diff, merge_hunk_is_unresolved, merge_result_use_source,
};
#[cfg(feature = "repository")]
pub(crate) use repo::jj_binary;
#[cfg(feature = "repository")]
pub use repo::{
    COMMIT_MESSAGE_PROMPT, DEFAULT_REVSET, DEFAULT_REVSET_DEPTH, Repo, ReviewNoteOutputFormat,
    ReviewNotesReport, RevsetPreset, SyncToken, add_review_note, build_default_revset,
    check_gh_environment, check_glab_environment, check_jj_environment, check_origin_environment,
    combined_diff_revsets, detect_ai_provider, find_existing_binary, generate_branch_name_cli,
    generate_commit_message_cli, home_dir, init_jj_git_repo, is_executable_file,
    is_valid_bookmark_name, is_valid_workspace_name, login_shell, login_shell_path,
    resolve_review_note, review_display_group_map_from_hunk, review_notes_output,
    review_snapshot_from_hunk, revset_presets, workspace_primary_root,
};
pub use theme::{DiffThemeColors, change_id_prefix_color, diff_theme_colors};
#[cfg(feature = "repository")]
pub use tools::{
    EDITOR_OPTIONS, TERMINAL_OPTIONS, ToolsConfig, open_in_editor, open_in_terminal, repo_file_url,
};
pub use types::*;
