use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone};
use gpui::{
    AnyElement, App, AppContext, Bounds, ClipboardItem, Context, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString, Size,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    px, rgb, uniform_list,
};
use jayjay_core::{EvologEntry, Repo};

use crate::app::actions::CloseWindow;
use crate::ui::primitives::no_scrollbar_gutter;
use crate::app::config::AppConfigStore;
use crate::app::fonts;
use crate::app::theme::{Theme, theme};
use crate::ui::icons::{self, glyph};

pub struct EvologView {
    repo: Arc<Repo>,
    rev: String,
    title: SharedString,
    entries: Option<Arc<Vec<EvologEntry>>>,
    error: Option<SharedString>,
    loading: bool,
    focus_handle: FocusHandle,
}

impl EvologView {
    pub fn open(repo: Arc<Repo>, rev: String, title: SharedString, cx: &mut App) {
        let bounds = Bounds::centered(
            None,
            Size {
                width: px(720.),
                height: px(540.),
            },
            cx,
        );
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(format!("Evolution: {title}").into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        let mut view = Self {
                            repo,
                            rev,
                            title,
                            entries: None,
                            error: None,
                            loading: true,
                            focus_handle: cx.focus_handle(),
                        };
                        view.load(cx);
                        view
                    })
                },
            )
            .ok();
        if let Some(h) = handle {
            let _ = h.update(cx, |view, window, cx| {
                let f = view.focus_handle(cx);
                window.focus(&f, cx);
            });
        }
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        let repo = self.repo.clone();
        let rev = self.rev.clone();
        cx.spawn(async move |this, cx| {
            let result = cx.background_spawn(async move { repo.evolog(&rev) }).await;
            let _ = this.update(cx, move |view, cx| {
                view.loading = false;
                match result {
                    Ok(entries) => view.entries = Some(Arc::new(entries)),
                    Err(e) => view.error = Some(format!("{e}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Focusable for EvologView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EvologView {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let body = if self.loading {
            placeholder("Loading evolution…", &t)
        } else if let Some(err) = self.error.clone() {
            placeholder_err(&err, &t)
        } else if let Some(entries) = self.entries.clone() {
            if entries.is_empty() {
                placeholder("No history", &t)
            } else {
                evolog_body(entries, t.clone())
            }
        } else {
            placeholder("Unable to load history", &t)
        };

        div()
            .track_focus(&self.focus_handle)
            .key_context("EvologView")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _cx| {
                window.remove_window();
            }))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(header(&self.title, &t))
            .child(body)
    }
}

fn header(title: &SharedString, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(16.))
        .py(px(10.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(icons::icon(glyph::ARROW_CLOCKWISE, 14., t.fg_dim))
        .child(
            div()
                .text_size(px(13.))
                .text_color(rgb(t.fg))
                .child("Evolution history"),
        )
        .child(div().flex_1())
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(title.clone()),
        )
        .into_any_element()
}

fn evolog_body(entries: Arc<Vec<EvologEntry>>, theme: Theme) -> AnyElement {
    let count = entries.len();
    let theme = Arc::new(theme);
    let list = uniform_list(
        "evolog",
        count,
        move |range: std::ops::Range<usize>, _w, _cx| {
            range.map(|ix| evolog_row(&entries[ix], &theme)).collect()
        },
    );
    no_scrollbar_gutter(list)
        .h_full()
        .into_any_element()
}

fn evolog_row(entry: &EvologEntry, t: &Theme) -> AnyElement {
    let short_commit = entry.commit_id.chars().take(12).collect::<String>();
    let when = format_when(entry.timestamp_millis);
    let description = if entry.description.trim().is_empty() {
        "(no description)".to_owned()
    } else {
        entry
            .description
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_owned()
    };

    let commit_for_copy = entry.commit_id.clone();
    let restore_cmd = format!("jj restore --from {commit_for_copy}");
    let restore_for_copy = restore_cmd.clone();

    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .px(px(16.))
        .py(px(10.))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .child(
            div()
                .flex()
                .flex_row()
                .items_baseline()
                .gap(px(8.))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(t.fg))
                        .child(SharedString::from(entry.operation.clone())),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(rgb(t.fg_faint))
                        .child(SharedString::from(when)),
                ),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(description)),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .id(SharedString::from(format!("commit-{commit_for_copy}")))
                        .font_family(fonts::mono())
                        .text_size(px(10.))
                        .text_color(rgb(t.fg_dim))
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(
                                commit_for_copy.clone(),
                            ));
                        })
                        .child(SharedString::from(short_commit)),
                )
                .child(
                    div()
                        .id(SharedString::from(format!("restore-{}", entry.commit_id)))
                        .px(px(6.))
                        .py(px(1.))
                        .rounded_sm()
                        .bg(rgb(t.toggle_inactive_bg))
                        .text_size(px(10.))
                        .text_color(rgb(t.toggle_inactive_fg))
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(
                                restore_for_copy.clone(),
                            ));
                        })
                        .child(SharedString::from("Copy `jj restore` cmd")),
                ),
        )
        .into_any_element()
}

fn format_when(ts: i64) -> String {
    let dt: DateTime<Local> = match Local.timestamp_millis_opt(ts).single() {
        Some(d) => d,
        None => return String::new(),
    };
    dt.format("%Y-%m-%d %H:%M").to_string()
}

fn placeholder(text: &'static str, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .text_color(rgb(t.fg_dim))
        .child(text)
        .into_any_element()
}

fn placeholder_err(text: &SharedString, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .text_color(rgb(t.error_fg))
        .child(text.clone())
        .into_any_element()
}
