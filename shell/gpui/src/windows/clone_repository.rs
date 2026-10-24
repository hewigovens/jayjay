use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, Focusable, InteractiveElement,
    IntoElement, ParentElement, PathPromptOptions, Render, SharedString,
    StatefulInteractiveElement, Styled, Subscription, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, px, rgb, size,
};
use jayjay_core::{CloneRequest, JayError, SyncToken};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::theme::{Theme, observe_window_appearance, ui_font_size};
use crate::ui::icons;
use crate::ui::primitives::button;
use crate::ui::text_area::{Newline, TextArea, TextAreaUpdated};

struct RunningClone {
    sync: SyncToken,
    finished: mpsc::Receiver<()>,
}

pub struct CloneRepositoryView {
    source: Entity<TextArea>,
    name: Entity<TextArea>,
    location: Entity<TextArea>,
    folder: String,
    name_follows_source: bool,
    /// Name and location rewrite each other, so the update event for our own write is skipped.
    echo: Option<String>,
    cloning: Option<RunningClone>,
    error: Option<SharedString>,
    _subscriptions: [Subscription; 3],
}

impl CloneRepositoryView {
    pub fn open(cx: &mut App) {
        let location =
            CloneRequest::default_location(&crate::app::config::current(cx).recent_repos);
        let bounds = Bounds::centered(None, size(px(520.), px(250.)), cx);
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(jayjay_i18n::repo::clone_repository().into()),
                        ..Default::default()
                    }),
                    ..crate::app::window_options()
                },
                |_, cx| cx.new(|cx| Self::new(location, cx)),
            )
            .ok();
        if let Some(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                window.focus(&view.source.read(cx).focus_handle(cx), cx);
            });
        }
    }

    fn new(folder: PathBuf, cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        let field = |text: String, placeholder: &'static str, cx: &mut Context<Self>| {
            cx.new(|cx| TextArea::new(text, placeholder, false, 28., cx))
        };
        let source = field(String::new(), "https://github.com/owner/repo.git", cx);
        let name = field(String::new(), "repo", cx);
        let folder = folder.display().to_string();
        let location = field(folder.clone(), "~/workspace/repo", cx);
        let subscriptions = [
            cx.subscribe(&source, |view, source, _: &TextAreaUpdated, cx| {
                if view.name_follows_source {
                    let derived = CloneRequest::directory_name(&source.read(cx).text());
                    view.name.update(cx, |name, cx| name.set_text(derived, cx));
                }
                cx.notify();
            }),
            // Typing a name detaches it from the URL; clearing it or matching the URL's name reattaches it.
            cx.subscribe(&name, |view, name, _: &TextAreaUpdated, cx| {
                let name = name.read(cx).text();
                let derived = CloneRequest::directory_name(&view.source.read(cx).text());
                view.name_follows_source = name.is_empty() || name == derived;
                if !view.take_echo(&name) {
                    view.sync_location(&name, cx);
                }
                cx.notify();
            }),
            cx.subscribe(&location, |view, location, _: &TextAreaUpdated, cx| {
                let location = location.read(cx).text();
                if !view.take_echo(&location) {
                    let (folder, name) = CloneRequest::split_location(&location);
                    view.folder = folder.to_owned();
                    view.sync_field(&view.name.clone(), name.to_owned(), cx);
                }
                cx.notify();
            }),
        ];
        // jj runs in its own process group, so quitting cancels the clone and waits briefly for its cleanup.
        cx.on_app_quit(|view: &mut Self, _| {
            if let Some(clone) = view.cloning.take() {
                clone.sync.cancel();
                let _ = clone.finished.recv_timeout(Duration::from_secs(2));
            }
            async {}
        })
        .detach();
        Self {
            source,
            name,
            location,
            folder,
            name_follows_source: true,
            echo: None,
            cloning: None,
            error: None,
            _subscriptions: subscriptions,
        }
    }

    fn take_echo(&mut self, text: &str) -> bool {
        let echoed = self.echo.as_deref() == Some(text);
        if echoed {
            self.echo = None;
        }
        echoed
    }

    fn sync_field(&mut self, field: &Entity<TextArea>, text: String, cx: &mut Context<Self>) {
        if field.read(cx).text() != text {
            self.echo = Some(text.clone());
            field.update(cx, |field, cx| field.set_text(text, cx));
        }
    }

    fn sync_location(&mut self, name: &str, cx: &mut Context<Self>) {
        let location = CloneRequest::location(&self.folder, name);
        self.sync_field(&self.location.clone(), location, cx);
    }

    fn can_clone(&self, cx: &App) -> bool {
        self.cloning.is_none()
            && !self.source.read(cx).text().trim().is_empty()
            && !self.name.read(cx).text().trim().is_empty()
    }

    fn clone_repository(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.can_clone(cx) {
            return;
        }
        let source = self.source.read(cx).text();
        let request = match CloneRequest::new(&source, &self.location.read(cx).text()) {
            Ok(request) => request,
            Err(error) => {
                self.error = Some(crate::app::error_text(error));
                cx.notify();
                return;
            }
        };
        let sync = SyncToken::standalone();
        let (finished, finished_rx) = mpsc::channel();
        self.cloning = Some(RunningClone {
            sync: sync.clone(),
            finished: finished_rx,
        });
        self.error = None;
        self.set_fields_read_only(true, cx);
        cx.notify();
        let clone_window = window.window_handle();
        cx.spawn(async move |this, cx| {
            let destination = request.destination.clone();
            let result = cx
                .background_spawn(async move {
                    let result = request.run(&sync);
                    let _ = finished.send(());
                    result
                })
                .await;
            let cloned = this.update(cx, |view, cx| {
                view.cloning = None;
                view.set_fields_read_only(false, cx);
                cx.notify();
                match result {
                    Ok(()) => true,
                    Err(JayError::Canceled) => false,
                    Err(error) => {
                        view.error = Some(crate::app::error_text(error));
                        false
                    }
                }
            });
            if cloned.unwrap_or(false) {
                let _ = clone_window.update(cx, |_, window, _| window.remove_window());
                cx.update(|cx| crate::repo::open_repo_window(destination, cx));
            }
        })
        .detach();
    }

    fn set_fields_read_only(&self, read_only: bool, cx: &mut Context<Self>) {
        for field in [&self.source, &self.name, &self.location] {
            field.update(cx, |field, cx| field.set_read_only(read_only, cx));
        }
    }

    /// Escape stops a running clone first, so a second press is needed to close the window.
    fn cancel(&mut self, window: &mut Window) {
        match &self.cloning {
            Some(clone) => clone.sync.cancel(),
            None => window.remove_window(),
        }
    }

    fn choose_location(&mut self, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(jayjay_i18n::clone::choose_where_to_clone().into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let _ = this.update(cx, |view, cx| {
                view.folder = path.display().to_string();
                let name = view.name.read(cx).text();
                view.sync_location(&name, cx);
            });
        })
        .detach();
    }

    fn status(&self, t: &Theme, cx: &App) -> Option<AnyElement> {
        let row = div()
            .flex()
            .flex_row()
            .items_start()
            .gap(px(6.))
            .text_size(ui_font_size(11.));
        if self.cloning.is_some() {
            return Some(
                row.text_color(rgb(t.fg_dim))
                    .child(jayjay_i18n::clone::cloning(
                        self.name.read(cx).text().trim(),
                    ))
                    .into_any_element(),
            );
        }
        let error = self.error.clone()?;
        Some(
            row.text_color(rgb(t.error_fg))
                .child(icons::icon(icons::glyph::WARNING, 12., t.error_fg))
                .child(div().flex_1().child(error))
                .into_any_element(),
        )
    }
}

impl Drop for CloneRepositoryView {
    fn drop(&mut self) {
        if let Some(clone) = self.cloning.take() {
            clone.sync.cancel();
        }
    }
}

fn field_row(label: String, field: impl IntoElement, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .w(px(110.))
                .flex_none()
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg_dim))
                .child(label),
        )
        .child(div().flex_1().min_w_0().child(field))
}

impl Render for CloneRepositoryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let cloning = self.cloning.is_some();
        let can_clone = self.can_clone(cx);
        let focus_handle = self.source.read(cx).focus_handle(cx);
        let input = |field: &Entity<TextArea>| {
            div()
                .w_full()
                .when(cloning, |field| field.opacity(0.6))
                .child(field.clone())
        };
        div()
            .font_family(crate::app::fonts::interface())
            .track_focus(&focus_handle)
            .key_context("CloneRepositoryView")
            .on_action(cx.listener(|view, _: &CloseWindow, window, _| view.cancel(window)))
            .on_action(cx.listener(|view, _: &Dismiss, window, _| view.cancel(window)))
            .on_action(
                cx.listener(|view, _: &Newline, window, cx| view.clone_repository(window, cx)),
            )
            .flex()
            .flex_col()
            .size_full()
            .gap(px(10.))
            .p(px(20.))
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(field_row(
                jayjay_i18n::clone::repository_url(),
                input(&self.source),
                &t,
            ))
            .child(field_row(jayjay_i18n::clone::name(), input(&self.name), &t))
            .child(field_row(
                jayjay_i18n::repo::location(),
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .id("clone-repository-location")
                            .debug_selector(|| "clone-repository-location".to_owned())
                            .flex_1()
                            .min_w_0()
                            .child(input(&self.location)),
                    )
                    .child(
                        button(
                            "clone-repository-choose",
                            jayjay_i18n::clone::choose_ellipsis(),
                            &t,
                            false,
                        )
                        .when(cloning, |choose| choose.opacity(0.5).cursor_default())
                        .when(!cloning, |choose| {
                            choose.on_click(cx.listener(|view, _, _, cx| view.choose_location(cx)))
                        }),
                    ),
                &t,
            ))
            .children(
                self.status(&t, cx)
                    .map(|status| div().pl(px(118.)).child(status)),
            )
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        button(
                            "clone-repository-cancel",
                            jayjay_i18n::common::cancel(),
                            &t,
                            false,
                        )
                        .on_click(cx.listener(|view, _, window, _| view.cancel(window))),
                    )
                    .child(
                        button(
                            "clone-repository-submit",
                            jayjay_i18n::onboarding::clone(),
                            &t,
                            true,
                        )
                        .when(!can_clone, |submit| submit.opacity(0.5).cursor_default())
                        .when(can_clone, |submit| {
                            submit.on_click(
                                cx.listener(|view, _, window, cx| {
                                    view.clone_repository(window, cx)
                                }),
                            )
                        }),
                    ),
            )
    }
}
