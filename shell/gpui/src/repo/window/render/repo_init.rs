use std::path::Path;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::fonts::{CodeText as _, mono_advance};
use crate::app::theme::{Theme, ui_font_size};
use crate::diff::middle_elide;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{glyph, icon};
use crate::ui::loading::loading_status;
use crate::ui::primitives::button;

const CARD_WIDTH: f32 = 420.;
const CARD_PADDING: f32 = 24.;
const PATH_FONT_SIZE: f32 = 11.;

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
    // Any other message, e.g. a failed init, is a real error.
    let needs_setup = can_initialize && message.starts_with("repository not found at ");
    let (glyph, color, title, message): (_, _, _, gpui::SharedString) =
        if let Some(root) = enclosing_jj {
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
                "Your Git history and branches stay as they are.".into(),
            )
        } else {
            (
                glyph::FOLDER,
                t.fg_dim,
                "No repository here yet",
                "jj git init creates a new jj repository in this folder.".into(),
            )
        };

    let advance = mono_advance(cx, px(t.scaled_code_font_size(PATH_FONT_SIZE)));
    let path_cols = ((CARD_WIDTH - 2. * CARD_PADDING) / f32::from(advance)) as usize;

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
                .max_w(px(CARD_WIDTH))
                .px(px(CARD_PADDING))
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
                        .code_text(PATH_FONT_SIZE, t)
                        .text_color(rgb(t.fg_faint))
                        .child(two_line_path(&repo_path, path_cols)),
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

/// GPUI's middle truncation is single-line.
fn two_line_path(path: &str, cols: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;
    let cols = cols.max(1);
    let mut lines = vec![String::new()];
    let mut used = 0;
    for grapheme in middle_elide(path, cols * 2).graphemes(true) {
        let width = grapheme.width();
        if used + width > cols {
            lines.push(String::new());
            used = 0;
        }
        used += width;
        lines.last_mut().unwrap().push_str(grapheme);
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::two_line_path;

    #[test]
    fn long_paths_elide_in_the_middle_to_two_lines() {
        assert_eq!(two_line_path("/a/b", 10), "/a/b");
        assert_eq!(two_line_path("/abcdefghij/klmnopqrst", 6), "/abcd…\nopqrst");
    }
}
