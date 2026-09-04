use std::path::Path;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::button;

pub(super) fn repo_init_error_pane(
    repo_path: gpui::SharedString,
    message: gpui::SharedString,
    initializing: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let can_initialize = !Path::new(repo_path.as_ref()).join(".jj").exists();
    let message = repo_init_error_message(message.as_ref(), repo_path.as_ref());
    let button_label = if initializing {
        "Initializing..."
    } else {
        "Initialize with jj git init"
    };
    let mut init_button = button("repo-init", button_label, t, true);
    if can_initialize && !initializing {
        init_button = init_button.on_click(cx.listener(|view, _, _, cx| {
            let task = view.vm.update(cx, |vm, cx| vm.initialize_repo(cx));
            task.detach();
        }));
    }

    let mut actions = div().flex().justify_center();
    if can_initialize {
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
                .child(icon(glyph::WARNING, 40., t.compare_accent))
                .child(
                    div()
                        .text_size(ui_font_size(16.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(t.fg))
                        .child("Failed to open repository"),
                )
                .child(
                    div()
                        .font_family(crate::app::fonts::mono())
                        .text_size(ui_font_size(11.))
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
    div()
        .flex()
        .flex_1()
        .size_full()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(ui_font_size(13.))
                .text_color(rgb(t.fg_dim))
                .child("Opening repository…"),
        )
        .into_any_element()
}

fn repo_init_error_message(message: &str, repo_path: &str) -> gpui::SharedString {
    if message.contains("There is no Jujutsu repo in") {
        return "There is no Jujutsu repo at this path.".into();
    }

    let without_path = message.replace(repo_path, "this path");
    without_path
        .trim()
        .trim_start_matches("repository not found at ")
        .trim()
        .trim_start_matches(':')
        .trim()
        .into()
}
