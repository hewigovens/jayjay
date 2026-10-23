use std::path::Path;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::ui::icons::{glyph, icon};
use crate::ui::loading::loading_status;
use crate::ui::primitives::button;

pub(super) fn repo_init_error_pane(
    repo_path: gpui::SharedString,
    message: gpui::SharedString,
    initializing: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let path = Path::new(repo_path.as_ref());
    let has_jj = path.join(".jj").exists();
    let enclosing = |marker: &str| {
        path.ancestors()
            .skip(1)
            .find(|dir| dir.join(marker).exists())
    };
    let enclosing_jj = enclosing(".jj").filter(|_| !has_jj);
    let enclosing_git = enclosing(".git").filter(|_| !path.join(".git").exists());
    let can_initialize = path.is_dir() && !has_jj && enclosing_jj.is_none();
    // `RepoNotFound` means jj just isn't set up here; any other message (e.g. a failed init) is a real error.
    let needs_setup = can_initialize && message.starts_with("repository not found at ");
    let (glyph, color, title, message): (_, _, _, gpui::SharedString) = if let Some(root) =
        enclosing_jj
    {
        (
            glyph::INFO,
            t.fg_dim,
            "This folder is inside a jj repository",
            format!("Open the repository root instead: {}", root.display()).into(),
        )
    } else if let Some(root) = enclosing_git.filter(|_| needs_setup) {
        (
            glyph::INFO,
            t.fg_dim,
            "This folder is inside a Git repository",
            format!(
                "Open {} to use its history, or initialize jj here to start a separate repository.",
                root.display()
            )
            .into(),
        )
    } else if !path.is_dir() {
        (
            glyph::WARNING,
            t.compare_accent,
            "Failed to open repository",
            "This folder no longer exists.".into(),
        )
    } else if !needs_setup {
        (
            glyph::WARNING,
            t.compare_accent,
            "Failed to open repository",
            repo_init_error_message(message.as_ref(), repo_path.as_ref()),
        )
    } else if path.join(".git").exists() {
        (
            glyph::GIT_BRANCH,
            t.fg_dim,
            "Set up jj for this Git repository",
            "JayJay works on jj repositories. jj git init adds one alongside Git; your Git history and branches stay as they are.".into(),
        )
    } else {
        (
            glyph::FOLDER,
            t.fg_dim,
            "No repository here yet",
            "jj git init creates a new jj repository in this folder.".into(),
        )
    };

    let mut actions = div().flex().justify_center();
    if can_initialize {
        let label = if initializing {
            "Initializing…"
        } else {
            "Initialize jj"
        };
        let mut init_button = button("repo-init", label, t, true);
        if !initializing {
            init_button = init_button.on_click(cx.listener(|view, _, _, cx| {
                let task = view.vm.update(cx, |vm, cx| vm.initialize_repo(cx));
                task.detach();
            }));
        }
        actions = actions.child(init_button);
    }

    div()
        .id("repo-init-pane")
        .flex()
        .flex_1()
        .size_full()
        .items_center()
        .justify_center()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(14.))
                .max_w(px(420.))
                .px(px(24.))
                .child(icon(glyph, 40., color))
                .child(
                    div()
                        .text_size(ui_font_size(16.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(t.fg))
                        .child(title),
                )
                .child(
                    div()
                        .code_text(11., t)
                        .text_color(rgb(t.fg_faint))
                        .child(repo_path),
                )
                .child(
                    div()
                        .text_size(ui_font_size(12.))
                        .line_height(ui_font_size(18.))
                        .text_color(rgb(t.fg_dim))
                        .text_align(gpui::TextAlign::Center)
                        .child(message),
                )
                .child(actions),
        )
        .into_any_element()
}

/// Shown while the repo is opening off the main thread (see `RepoViewModel::open_async`).
pub(super) fn repo_loading_pane(t: &Theme) -> AnyElement {
    loading_status("Opening repository…", t)
}

fn repo_init_error_message(message: &str, repo_path: &str) -> gpui::SharedString {
    let without_path = message.replace(repo_path, "this path");
    without_path
        .trim()
        .trim_start_matches("repository not found at ")
        .trim()
        .trim_start_matches(':')
        .trim()
        .into()
}
