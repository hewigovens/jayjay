use std::sync::Arc;

use gpui::{App, Context, Entity};

use crate::app::config::{self, AppearanceMode};
use crate::app::links::GUIDE_URL;
use crate::app::tools;
use crate::repo::window::{ChangeAction, RepoWindow};
use crate::ui::icons::glyph;
use crate::windows::keyboard_shortcuts::KeyboardShortcutsView;
use crate::windows::open_remote_repository::OpenRemoteRepositoryView;
use crate::windows::repo_list::RepoListWindow;
use crate::windows::settings::SettingsView;

/// Context passed to a palette action's dispatcher.
pub(super) struct PaletteCtx<'a> {
    pub repo_path: &'a str,
    pub repo_window: Option<Entity<RepoWindow>>,
}

/// One row in the palette: stable id, catalog label, search keywords, and a side-effect.
pub(super) struct PaletteAction {
    pub id: &'static str,
    pub label: fn() -> String,
    pub keywords: &'static [&'static str],
    pub glyph_str: &'static str,
    pub dispatch: fn(&PaletteCtx, &mut App),
}

impl PaletteAction {
    pub(super) fn display_name(&self, cx: &App) -> String {
        match self.id {
            "open-in-editor" => tools::open_in_editor_label(cx),
            "open-in-terminal" => tools::open_in_terminal_label(cx),
            _ => (self.label)(),
        }
    }
}

pub(super) const ACTIONS: &[PaletteAction] = &[
    PaletteAction {
        id: "refresh",
        label: jayjay_i18n::repo::refresh,
        keywords: &["reload", "repository", "working", "copy"],
        glyph_str: glyph::SYNC,
        dispatch: |ctx, cx| {
            with_repo_window(ctx, cx, |view, cx| {
                let vm = view.view_model();
                vm.update(cx, |vm, cx| vm.run_refresh(cx));
            });
        },
    },
    PaletteAction {
        id: "bisect",
        label: jayjay_i18n::palette::bisect_ellipsis,
        keywords: &["bisect", "binary", "search", "bug", "regression"],
        glyph_str: glyph::ARROWS_LEFT_RIGHT,
        dispatch: |ctx, cx| {
            with_repo_window(ctx, cx, |view, cx| view.open_bisect_sheet(None, cx));
        },
    },
    PaletteAction {
        id: "open-settings",
        label: jayjay_i18n::palette::open_settings,
        keywords: &["settings", "preferences", "config"],
        glyph_str: glyph::GEAR,
        dispatch: |_, cx| SettingsView::open(cx),
    },
    PaletteAction {
        id: "keyboard-shortcuts",
        label: jayjay_i18n::menu::keyboard_shortcuts,
        keywords: &["keyboard", "shortcut", "shortcuts", "keys", "help"],
        glyph_str: glyph::INFO,
        dispatch: |_, cx| KeyboardShortcutsView::open(cx),
    },
    PaletteAction {
        id: "open-user-guide",
        label: jayjay_i18n::palette::open_user_guide,
        keywords: &["help", "guide", "manual", "docs", "documentation"],
        glyph_str: glyph::INFO,
        dispatch: |_, cx| crate::app::links::open_url(cx, GUIDE_URL),
    },
    PaletteAction {
        id: "send-feedback",
        label: jayjay_i18n::menu::send_feedback,
        keywords: &["email", "contact", "support", "help"],
        glyph_str: glyph::EXTERNAL_LINK,
        dispatch: |_, cx| crate::app::feedback::open(cx),
    },
    PaletteAction {
        id: "open-bookmark-manager",
        label: jayjay_i18n::palette::open_bookmark_manager,
        keywords: &["bookmark", "bookmarks", "manager", "branch", "branches"],
        glyph_str: glyph::GIT_BRANCH,
        dispatch: |ctx, cx| {
            if let Some(view) = ctx.repo_window.clone() {
                view.update(cx, |view, cx| view.open_bookmark_manager(cx));
            }
        },
    },
    PaletteAction {
        id: "repo-overview",
        label: jayjay_i18n::menu::repo_overview,
        keywords: &[
            "overview",
            "lanes",
            "workspaces",
            "work",
            "stacks",
            "branches",
        ],
        glyph_str: glyph::COLUMNS,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::open_overview),
    },
    PaletteAction {
        id: "repository-list",
        label: jayjay_i18n::repo::repository_list,
        keywords: &[
            "repository",
            "repositories",
            "recent",
            "pinned",
            "welcome",
            "open",
        ],
        glyph_str: glyph::DATABASE,
        dispatch: |_, cx| RepoListWindow::open(cx),
    },
    PaletteAction {
        id: "open-remote-repository",
        label: jayjay_i18n::ssh::open_remote_repository_ellipsis,
        keywords: &["open", "remote", "ssh", "repository", "machine", "host"],
        glyph_str: glyph::GLOBE,
        dispatch: |_, cx| OpenRemoteRepositoryView::open(cx),
    },
    PaletteAction {
        id: "new-workspace",
        label: jayjay_i18n::repo::new_workspace,
        keywords: &["workspace", "workspaces", "add", "create", "sibling"],
        glyph_str: glyph::PLUS,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::open_create_workspace),
    },
    PaletteAction {
        id: "new-workspace-from-pull-request",
        label: jayjay_i18n::repo::new_workspace_from_pull_request,
        keywords: &[
            "workspace",
            "pull",
            "request",
            "pr",
            "import",
            "github",
            "fetch",
        ],
        glyph_str: glyph::GIT_MERGE,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::open_pr_import),
    },
    PaletteAction {
        id: "operation-log",
        label: jayjay_i18n::repo::operation_log,
        keywords: &["operation", "operations", "op", "log", "undo", "restore"],
        glyph_str: glyph::HISTORY,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::open_operation_log),
    },
    PaletteAction {
        id: "open-in-terminal",
        label: jayjay_i18n::palette::open_in_terminal,
        keywords: &["terminal", "shell", "open"],
        glyph_str: glyph::TERMINAL,
        dispatch: |ctx, cx| {
            if let Some(view) = ctx.repo_window.clone() {
                view.update(cx, |view, cx| view.open_repo_in_terminal(cx));
            } else {
                tools::open_in_terminal(ctx.repo_path, cx);
            }
        },
    },
    PaletteAction {
        id: "open-in-editor",
        label: jayjay_i18n::palette::open_in_editor,
        keywords: &["editor", "code", "open"],
        glyph_str: glyph::FILE_CODE,
        dispatch: |ctx, cx| {
            if let Some(view) = ctx.repo_window.clone() {
                view.update(cx, |view, cx| view.open_repo_in_editor(cx));
            } else {
                tools::open_in_editor(ctx.repo_path, ".", None, cx);
            }
        },
    },
    PaletteAction {
        id: "theme-system",
        label: jayjay_i18n::palette::theme_system,
        keywords: &["theme", "appearance", "system"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::System),
    },
    PaletteAction {
        id: "theme-light",
        label: jayjay_i18n::palette::theme_light,
        keywords: &["theme", "appearance", "light"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::Light),
    },
    PaletteAction {
        id: "theme-dark",
        label: jayjay_i18n::palette::theme_dark,
        keywords: &["theme", "appearance", "dark"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::Dark),
    },
    PaletteAction {
        id: "toggle-side-by-side-diff",
        label: jayjay_i18n::palette::toggle_side_by_side_diff,
        keywords: &["diff", "split", "side", "by", "unified"],
        glyph_str: glyph::COLUMNS,
        dispatch: |_, cx| config::update(cx, |c| c.diff.side_by_side ^= true),
    },
    PaletteAction {
        id: "toggle-ignore-whitespace",
        label: jayjay_i18n::palette::toggle_ignore_whitespace,
        keywords: &["whitespace", "diff", "ignore"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| config::update(cx, |c| c.diff.ignore_whitespace ^= true),
    },
    PaletteAction {
        id: "toggle-hide-git-lfs-backed-files",
        label: jayjay_i18n::palette::toggle_hide_git_lfs_backed_files,
        keywords: &["lfs", "large", "file", "diff", "hide"],
        glyph_str: glyph::DATABASE,
        dispatch: |_, cx| config::update(cx, |c| c.diff.hide_git_lfs ^= true),
    },
    PaletteAction {
        id: "toggle-tree-file-list",
        label: jayjay_i18n::palette::toggle_tree_file_list,
        keywords: &["tree", "file", "folder", "list"],
        glyph_str: glyph::FOLDER,
        dispatch: |_, cx| config::update(cx, |c| c.diff.tree_file_list ^= true),
    },
    PaletteAction {
        id: "toggle-auto-expand-descriptions",
        label: jayjay_i18n::palette::toggle_auto_expand_descriptions,
        keywords: &["description", "commit", "message", "expand", "auto"],
        glyph_str: glyph::CARET_DOWN,
        dispatch: |_, cx| config::update(cx, |c| c.diff.auto_expand_description ^= true),
    },
    PaletteAction {
        id: "filter-by-revset",
        label: jayjay_i18n::menu::filter_by_revset_ellipsis,
        keywords: &["revset", "filter", "query", "graph", "log"],
        glyph_str: glyph::FILTER,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::request_revset_edit),
    },
    PaletteAction {
        id: "hide-show-sidebar",
        label: jayjay_i18n::palette::hide_show_sidebar,
        keywords: &[
            "sidebar", "hide", "show", "panel", "history", "dag", "graph",
        ],
        glyph_str: glyph::PANEL_LEFT,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::toggle_sidebar),
    },
    PaletteAction {
        id: "git-pull-fetch-rebase",
        label: jayjay_i18n::repo::git_pull_fetch_rebase,
        keywords: &["git", "pull", "fetch", "rebase", "sync"],
        glyph_str: glyph::ARROW_DOWN,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::git_fetch_origin),
    },
    PaletteAction {
        id: "rebase-working-copy-onto-trunk",
        label: jayjay_i18n::palette::rebase_working_copy_onto_trunk,
        keywords: &["rebase", "trunk", "main", "update", "working copy"],
        glyph_str: glyph::ARROW_UP,
        dispatch: |ctx, cx| {
            with_repo_window(ctx, cx, |view, cx| {
                let action = ChangeAction::RebaseOntoTrunk {
                    rev: "@".to_owned(),
                };
                view.run_change_action(Arc::new(action), cx);
            })
        },
    },
    PaletteAction {
        id: "parallelize-selected-changes",
        label: jayjay_i18n::palette::parallelize_selected_changes,
        keywords: &[
            "parallelize",
            "parallel",
            "sibling",
            "independent",
            "split",
            "selection",
        ],
        glyph_str: glyph::GIT_BRANCH,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::parallelize_selection),
    },
    PaletteAction {
        id: "run-formatters-on-selected-changes-jj-fix",
        label: jayjay_i18n::palette::run_formatters_on_selected_changes_jj,
        keywords: &["fix", "format", "formatter", "selection"],
        glyph_str: glyph::BRACES,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::fix_selection),
    },
    PaletteAction {
        id: "git-push",
        label: jayjay_i18n::repo::git_push,
        keywords: &["git", "push", "sync"],
        glyph_str: glyph::ARROW_UP,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::git_push_default),
    },
    PaletteAction {
        id: "clean-up-stale-bookmarks",
        label: jayjay_i18n::palette::clean_up_stale_bookmarks,
        keywords: &["bookmark", "bookmarks", "stale", "forget", "cleanup"],
        glyph_str: glyph::BOOKMARK,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::forget_stale_bookmarks),
    },
    PaletteAction {
        id: "show-in-file-manager",
        label: jayjay_i18n::palette::show_in_file_manager,
        keywords: &["finder", "file", "manager", "reveal", "folder"],
        glyph_str: glyph::FOLDER,
        dispatch: |ctx, cx| {
            if let Some(view) = ctx.repo_window.clone() {
                view.update(cx, |view, cx| view.show_repo_in_file_manager(cx));
            } else {
                tools::show_in_file_manager(ctx.repo_path, None);
            }
        },
    },
    PaletteAction {
        id: "view-remote-repository",
        label: jayjay_i18n::menu::view_remote_repository,
        keywords: &[
            "remote", "origin", "web", "browser", "github", "codeberg", "gitlab", "cursor",
        ],
        glyph_str: glyph::ARROW_CIRCLE_RIGHT,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::open_remote_repository),
    },
    PaletteAction {
        id: "zoom-in",
        label: jayjay_i18n::menu::zoom_in,
        keywords: &["zoom", "font", "size", "larger"],
        glyph_str: glyph::ZOOM_IN,
        dispatch: |_, cx| config::update(cx, |c| c.zoom(1.)),
    },
    PaletteAction {
        id: "zoom-out",
        label: jayjay_i18n::menu::zoom_out,
        keywords: &["zoom", "font", "size", "smaller"],
        glyph_str: glyph::ZOOM_OUT,
        dispatch: |_, cx| config::update(cx, |c| c.zoom(-1.)),
    },
    PaletteAction {
        id: "reset-zoom",
        label: jayjay_i18n::menu::reset_zoom,
        keywords: &["zoom", "font", "size", "reset", "default"],
        glyph_str: glyph::A_LARGE_SMALL,
        dispatch: |_, cx| config::update(cx, |c| c.reset_zoom()),
    },
    PaletteAction {
        id: "expand-all-unmodified-lines",
        label: jayjay_i18n::palette::expand_all_unmodified_lines,
        keywords: &[
            "context",
            "expand",
            "unmodified",
            "collapsed",
            "show",
            "diff",
        ],
        glyph_str: glyph::UNFOLD_VERTICAL,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::expand_all_context),
    },
];

fn set_appearance(cx: &mut App, mode: AppearanceMode) {
    config::update(cx, move |c| c.appearance = mode);
}

fn with_repo_window(
    ctx: &PaletteCtx,
    cx: &mut App,
    update: impl FnOnce(&mut RepoWindow, &mut Context<RepoWindow>) + 'static,
) {
    if let Some(view) = ctx.repo_window.clone() {
        view.update(cx, update);
    }
}

#[cfg(test)]
mod tests {
    use super::ACTIONS;
    use crate::app::config::{AppConfig, AppConfigStore};

    #[gpui::test]
    fn open_editor_action_display_name_uses_configured_editor(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let mut cfg = AppConfig::default();
            cfg.tools.external_editor = "zed".to_owned();
            cx.set_global(AppConfigStore::new(cfg));
            let action = ACTIONS
                .iter()
                .find(|action| action.id == "open-in-editor")
                .expect("open editor action");

            assert_eq!(action.display_name(cx), "Open in Zed");
        });
    }

    #[gpui::test]
    fn open_terminal_action_display_name_uses_configured_terminal(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let (terminal, title) = jayjay_core::tools::TERMINAL_OPTIONS[1];
            let label = jayjay_i18n::menu::open_in(title);
            let mut cfg = AppConfig::default();
            cfg.tools.terminal = terminal.to_owned();
            cx.set_global(AppConfigStore::new(cfg));
            let action = ACTIONS
                .iter()
                .find(|action| action.id == "open-in-terminal")
                .expect("open terminal action");

            assert_eq!(action.display_name(cx), label);
        });
    }
}
