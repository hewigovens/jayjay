pub use jayjay_primitives::{
    AppDirs, EvologRow, JAYJAY_CONFIG_COMMAND, JAYJAY_REVIEW_COMMAND, JAYJAY_TOOL_COMMAND,
    JJ_TOOL_CONFIG, evolog_rows, is_snapshot_operation, os_version,
};
pub use jj_diff as diff;
pub use jj_diff::syntax;
#[cfg(feature = "repository")]
mod cli;
pub mod commit_message;
pub mod compare;
pub mod dag;
pub mod diff_edit;
pub mod error_message;
#[cfg(feature = "repository")]
pub mod external_tools;
#[cfg(feature = "repository")]
mod file_display;
pub mod file_tree;
#[cfg(feature = "repository")]
mod filesystem;
pub mod fonts;
pub mod fuzzy;
pub mod github_alias;
#[cfg(feature = "repository")]
mod jj_command;
#[cfg(feature = "repository")]
mod json_store;
#[cfg(feature = "repository")]
mod merge_editor;
#[cfg(any(test, feature = "mock"))]
pub mod mock;
pub mod overview;
pub mod palette;
pub mod placeholder;
pub mod projection;
#[cfg(feature = "repository")]
mod repo;
#[cfg(feature = "repository")]
pub mod repositories;
#[cfg(feature = "repository")]
pub mod revset;
pub mod theme;
#[cfg(feature = "repository")]
pub mod tools;
pub mod trunk;
mod types;
pub mod utf16;

#[cfg(feature = "repository")]
pub use cli::run_app_cli_command;
#[cfg(feature = "repository")]
pub use file_display::MAX_IMAGE_BYTES;
#[cfg(feature = "repository")]
pub use filesystem::write_atomically;
pub use fonts::{
    CodeFontChoices, FontSpecimen, coding_font_families, font_family_choices, is_user_font_path,
    mono_font_family,
};
#[cfg(feature = "repository")]
pub use jj_command::{JjCommand, JjCommandResult};
#[cfg(feature = "repository")]
pub use merge_editor::{MergeEditorHunkExt, MergeScrollMap};
#[cfg(feature = "repository")]
pub(crate) use repo::jj_binary;
#[cfg(feature = "repository")]
pub use repo::{
    AiProvider, CloneRequest, DiffExcerpt, JjConfigEntry, JjConfigSection, JjUserConfig, Repo,
    ReviewNotesReport, ReviewOutputFormat, SyncToken, UserIdentity, add_review_note,
    branch_name_slug, check_gh_environment, check_glab_environment, check_jj_environment,
    check_origin_environment, compose_bisect_range, find_existing_binary, home_dir,
    init_jj_git_repo, is_executable_file, is_valid_bookmark_name, is_valid_workspace_name,
    jj_user_config, login_shell, login_shell_path, mark_review_file, resolve_review_note,
    review_display_group_map_from_hunk, review_notes_output, review_snapshot_from_hunk,
    review_status_output, unmark_review_files, workspace_primary_root, workspace_root,
};
#[cfg(feature = "repository")]
pub use revset::{
    BookmarkFilterTarget, DEFAULT_REVSET, DEFAULT_REVSET_DEPTH, RecentRevsetStore,
    RevsetCompletion, RevsetCompletionKind, RevsetFilter, RevsetFilterKind, RevsetFilterState,
    RevsetName, RevsetPreset, RevsetSuggestion, RevsetSuggestionKind, RevsetVocabulary,
    ancestors_revset, bookmark_filter_revset, build_default_revset, default_revset_depth,
    default_revset_preset, revset_completions, revset_presets, typed_revset,
    workspace_filter_revset,
};
pub use theme::{DiffPalette, DiffThemeColors, RepoColors, ThemeSeed, diff_palettes, repo_colors};
#[cfg(feature = "repository")]
pub use tools::{
    EDITOR_OPTIONS, TERMINAL_OPTIONS, ToolsConfig, editor_opens_at_line, open_in_editor,
    open_in_terminal, repo_file_path, repo_file_url,
};
pub use types::*;
