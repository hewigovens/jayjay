use std::path::{Path, PathBuf};

use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Bounds, Context, Entity, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, TitlebarOptions,
    Window, WindowBounds, WindowOptions, div, px, rgb, size,
};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::ui::primitives::button;
use crate::ui::text_area::{Newline, TextArea};

pub struct OpenRepositoryPathView {
    input: Entity<TextArea>,
    error: Option<SharedString>,
}

impl OpenRepositoryPathView {
    pub fn open(cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(520.), px(220.)), cx);
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Open Repository by Path".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        Self {
                            input: cx
                                .new(|cx| TextArea::new("", "~/workspace/my-repo", false, 32., cx)),
                            error: None,
                        }
                    })
                },
            )
            .ok();
        if let Some(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                window.focus(&view.input.read(cx).focus_handle(cx), cx);
            });
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.input.read(cx).text();
        match resolve_directory(&input) {
            Ok(path) => {
                window.remove_window();
                cx.defer(move |cx| crate::repo::open_repo_window(path, cx));
            }
            Err(error) => {
                self.error = Some(error.into());
                cx.notify();
            }
        }
    }
}

impl Render for OpenRepositoryPathView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let error = self.error.clone();
        let focus_handle = self.input.read(cx).focus_handle(cx);
        div()
            .track_focus(&focus_handle)
            .key_context("OpenRepositoryPathView")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _| window.remove_window()))
            .on_action(cx.listener(|_, _: &Dismiss, window, _| window.remove_window()))
            .on_action(cx.listener(|view, _: &Newline, window, cx| view.submit(window, cx)))
            .flex()
            .flex_col()
            .size_full()
            .gap(px(12.))
            .p(px(24.))
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(
                div()
                    .text_size(px(18.))
                    .child("Choose a repository directory"),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(t.fg_dim))
                    .child("The desktop file picker is unavailable. Enter a path instead."),
            )
            .child(
                div()
                    .id("open-repository-path-input")
                    .debug_selector(|| "open-repository-path-input".to_owned())
                    .w_full()
                    .child(self.input.clone()),
            )
            .when_some(error, |root, error| {
                root.child(
                    div()
                        .id("open-repository-path-error")
                        .text_size(px(11.))
                        .text_color(rgb(t.error_fg))
                        .child(error),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        button("open-repository-path-cancel", "Cancel", &t, false)
                            .on_click(cx.listener(|_, _, window, _| window.remove_window())),
                    )
                    .child(
                        button("open-repository-path-submit", "Open", &t, true)
                            .on_click(cx.listener(|view, _, window, cx| view.submit(window, cx))),
                    ),
            )
    }
}

fn resolve_directory(input: &str) -> Result<PathBuf, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Enter a repository path.".to_owned());
    }
    let path = expand_home(input);
    let path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .map_err(|error| format!("Could not resolve the current directory: {error}"))?
            .join(path)
    };
    let path = path
        .canonicalize()
        .map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    if !path.is_dir() {
        return Err(format!("{} is not a directory.", path.display()));
    }
    Ok(path)
}

fn expand_home(input: &str) -> PathBuf {
    if input == "~" {
        return jayjay_core::home_dir().unwrap_or_else(|| PathBuf::from(input));
    }
    if let Some(relative) = input.strip_prefix("~/")
        && let Some(home) = jayjay_core::home_dir()
    {
        return home.join(relative);
    }
    Path::new(input).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::resolve_directory;

    #[test]
    fn typed_repository_path_must_resolve_to_a_directory() {
        let directory = tempfile::tempdir().expect("temp directory");
        assert_eq!(
            resolve_directory(directory.path().to_str().unwrap()).unwrap(),
            directory.path().canonicalize().unwrap()
        );
        assert!(resolve_directory("").is_err());
        assert!(resolve_directory(directory.path().join("missing").to_str().unwrap()).is_err());
    }
}
