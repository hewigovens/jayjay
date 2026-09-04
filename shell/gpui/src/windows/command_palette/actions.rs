use gpui::{App, Context, Entity};

use crate::app::config::{self, AppearanceMode};
use crate::app::links::GUIDE_URL;
use crate::app::tools;
use crate::repo::window::RepoWindow;
use crate::ui::icons::glyph;
use crate::windows::keyboard_shortcuts::KeyboardShortcutsView;
use crate::windows::settings::SettingsView;

/// Context passed to a palette action's dispatcher.
pub(super) struct PaletteCtx<'a> {
    pub repo_path: &'a str,
    pub repo_window: Option<Entity<RepoWindow>>,
}

/// One row in the palette: human label, search keywords, and a side-effect.
pub(super) struct PaletteAction {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub glyph_str: &'static str,
    pub dispatch: fn(&PaletteCtx, &mut App),
}

impl PaletteAction {
    pub(super) fn display_name(&self, cx: &App) -> String {
        match self.name {
            "Open in Editor" => tools::open_in_editor_label(cx),
            "Open in Terminal" => tools::open_in_terminal_label(cx),
            name => name.to_owned(),
        }
    }
}

pub(super) const ACTIONS: &[PaletteAction] = &[
    PaletteAction {
        name: "Refresh",
        keywords: &["reload", "repository", "working", "copy"],
        glyph_str: glyph::ARROW_CLOCKWISE,
        dispatch: |ctx, cx| {
            with_repo_window(ctx, cx, |view, cx| {
                let vm = view.view_model();
                vm.update(cx, |vm, cx| vm.refresh(false, cx));
            });
        },
    },
    PaletteAction {
        name: "Open Settings",
        keywords: &["settings", "preferences", "config"],
        glyph_str: glyph::GEAR,
        dispatch: |_, cx| SettingsView::open(cx),
    },
    PaletteAction {
        name: "Keyboard Shortcuts",
        keywords: &["keyboard", "shortcut", "shortcuts", "keys", "help"],
        glyph_str: glyph::INFO,
        dispatch: |_, cx| KeyboardShortcutsView::open(cx),
    },
    PaletteAction {
        name: "Open User Guide",
        keywords: &["help", "guide", "manual", "docs", "documentation"],
        glyph_str: glyph::INFO,
        dispatch: |_, cx| crate::app::links::open_url(cx, GUIDE_URL),
    },
    PaletteAction {
        name: "Send Feedback",
        keywords: &["email", "contact", "support", "help"],
        glyph_str: glyph::EXTERNAL_LINK,
        dispatch: |_, cx| crate::app::feedback::open(cx),
    },
    PaletteAction {
        name: "Open Bookmark Manager",
        keywords: &["bookmark", "bookmarks", "manager", "branch", "branches"],
        glyph_str: glyph::GIT_BRANCH,
        dispatch: |ctx, cx| {
            if let Some(view) = ctx.repo_window.clone() {
                view.update(cx, |view, cx| view.open_bookmark_manager(cx));
            }
        },
    },
    PaletteAction {
        name: "New Workspace",
        keywords: &["workspace", "workspaces", "add", "create", "sibling"],
        glyph_str: glyph::PLUS_CIRCLE,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::open_create_workspace),
    },
    PaletteAction {
        name: "Operation Log",
        keywords: &["operation", "operations", "op", "log", "undo", "restore"],
        glyph_str: glyph::ARROW_CLOCKWISE,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::open_operation_log),
    },
    PaletteAction {
        name: "Open in Terminal",
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
        name: "Open in Editor",
        keywords: &["editor", "code", "open"],
        glyph_str: glyph::FILE_CODE,
        dispatch: |ctx, cx| {
            if let Some(view) = ctx.repo_window.clone() {
                view.update(cx, |view, cx| view.open_repo_in_editor(cx));
            } else {
                tools::open_in_editor(ctx.repo_path, ".", cx);
            }
        },
    },
    PaletteAction {
        name: "Theme: System",
        keywords: &["theme", "appearance", "system"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::System),
    },
    PaletteAction {
        name: "Theme: Light",
        keywords: &["theme", "appearance", "light"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::Light),
    },
    PaletteAction {
        name: "Theme: Dark",
        keywords: &["theme", "appearance", "dark"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::Dark),
    },
    PaletteAction {
        name: "Toggle Side-by-side Diff",
        keywords: &["diff", "split", "side", "by", "unified"],
        glyph_str: glyph::COLUMNS,
        dispatch: |_, cx| config::update(cx, |c| c.diff.side_by_side ^= true),
    },
    PaletteAction {
        name: "Toggle Ignore Whitespace",
        keywords: &["whitespace", "diff", "ignore"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| config::update(cx, |c| c.diff.ignore_whitespace ^= true),
    },
    PaletteAction {
        name: "Toggle Hide Git LFS-backed Files",
        keywords: &["lfs", "large", "file", "diff", "hide"],
        glyph_str: glyph::HARD_DRIVE,
        dispatch: |_, cx| config::update(cx, |c| c.diff.hide_git_lfs ^= true),
    },
    PaletteAction {
        name: "Toggle Tree File List",
        keywords: &["tree", "file", "folder", "list"],
        glyph_str: glyph::FOLDER_SIMPLE,
        dispatch: |_, cx| config::update(cx, |c| c.diff.tree_file_list ^= true),
    },
    PaletteAction {
        name: "Git Pull (fetch + rebase)",
        keywords: &["git", "pull", "fetch", "rebase", "sync"],
        glyph_str: glyph::ARROW_DOWN,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::git_fetch_origin),
    },
    PaletteAction {
        name: "Git Push",
        keywords: &["git", "push", "sync"],
        glyph_str: glyph::ARROW_UP,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::git_push_default),
    },
    PaletteAction {
        name: "Clean Up Stale Bookmarks",
        keywords: &["bookmark", "bookmarks", "stale", "forget", "cleanup"],
        glyph_str: glyph::BOOKMARK,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::forget_stale_bookmarks),
    },
    PaletteAction {
        name: "Show in File Manager",
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
        name: "View Remote Repository",
        keywords: &[
            "remote", "origin", "web", "browser", "github", "codeberg", "gitlab", "cursor",
        ],
        glyph_str: glyph::ARROW_CIRCLE_RIGHT,
        dispatch: |ctx, cx| with_repo_window(ctx, cx, RepoWindow::open_remote_repository),
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

    #[test]
    fn actions_include_web_user_guide() {
        let action = ACTIONS
            .iter()
            .find(|action| action.name == "Open User Guide")
            .expect("guide action");
        assert!(action.keywords.contains(&"help"));
        assert!(action.keywords.contains(&"docs"));
    }

    #[test]
    fn actions_include_operation_log() {
        let action = ACTIONS
            .iter()
            .find(|action| action.name == "Operation Log")
            .expect("operation log action");
        assert!(action.keywords.contains(&"op"));
        assert!(action.keywords.contains(&"restore"));
    }

    #[gpui::test]
    fn open_editor_action_display_name_uses_configured_editor(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let mut cfg = AppConfig::default();
            cfg.tools.external_editor = "zed".to_owned();
            cx.set_global(AppConfigStore::new(cfg));
            let action = ACTIONS
                .iter()
                .find(|action| action.name == "Open in Editor")
                .expect("open editor action");

            assert_eq!(action.display_name(cx), "Open in Zed");
        });
    }

    #[gpui::test]
    fn open_terminal_action_display_name_uses_configured_terminal(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let mut cfg = AppConfig::default();
            cfg.tools.terminal = "ghostty".to_owned();
            cx.set_global(AppConfigStore::new(cfg));
            let action = ACTIONS
                .iter()
                .find(|action| action.name == "Open in Terminal")
                .expect("open terminal action");

            assert_eq!(action.display_name(cx), "Open in Ghostty");
        });
    }
}
