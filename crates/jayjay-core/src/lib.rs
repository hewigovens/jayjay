pub use jj_diff as diff;
pub use jj_diff::syntax;
pub mod commit_message;
pub mod dag;
pub mod file_tree;
pub mod fonts;
pub mod fuzzy;
mod jj_command;
pub mod palette;
pub mod placeholder;
pub mod projection;
mod repo;
pub mod theme;
pub mod tools;
mod types;

pub use fonts::{
    MONO_FONT_FALLBACK_NAMES, MONO_FONT_OPTIONS, MonoFontOption, SYSTEM_MONO_FONT_ID,
    mono_font_option,
};
pub use jj_command::{JjCommand, JjCommandResult};
pub use repo::{
    COMMIT_MESSAGE_PROMPT, DEFAULT_REVSET, DEFAULT_REVSET_DEPTH, Repo, ReviewNoteOutputFormat,
    ReviewNotesReport, RevsetPreset, add_review_note, build_default_revset, check_gh_environment,
    check_glab_environment, check_jj_environment, detect_ai_provider, find_existing_binary,
    generate_branch_name_cli, generate_commit_message_cli, home_dir, init_jj_git_repo,
    is_executable_file, is_valid_bookmark_name, is_valid_workspace_name, jj_binary, login_shell,
    login_shell_path, resolve_review_note, review_notes_output, revset_presets,
};
pub use theme::{DiffThemeColors, change_id_prefix_color, diff_theme_colors};
pub use tools::{
    EDITOR_OPTIONS, TERMINAL_OPTIONS, ToolsConfig, open_in_editor, open_in_terminal, repo_file_url,
};
pub use types::*;
