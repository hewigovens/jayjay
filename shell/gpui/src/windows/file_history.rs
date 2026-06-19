use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone};
use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString, Size,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    px, rgb, uniform_list,
};
use jayjay_core::{ChangeInfo, Repo};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::config::AppConfigStore;
use crate::app::fonts;
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::no_scrollbar_gutter;

pub struct FileHistoryView {
    repo: Arc<Repo>,
    parent: Entity<RepoWindow>,
    path: SharedString,
    history: Option<Arc<Vec<ChangeInfo>>>,
    error: Option<SharedString>,
    loading: bool,
    focus_handle: FocusHandle,
}

impl FileHistoryView {
    pub fn open(repo: Arc<Repo>, path: String, parent: Entity<RepoWindow>, cx: &mut App) {
        let bounds = Bounds::centered(
            None,
            Size {
                width: px(640.),
                height: px(480.),
            },
            cx,
        );
        let title = format!("History: {path}");
        let path_for_view: SharedString = path.into();
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(title.into()),
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
                            parent,
                            path: path_for_view,
                            history: None,
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
                observe_window_appearance(window, cx);
                let f = view.focus_handle(cx);
                window.focus(&f, cx);
            });
        }
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        let repo = self.repo.clone();
        let path = self.path.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.file_history(&path) })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.loading = false;
                match result {
                    Ok(entries) => view.history = Some(Arc::new(entries)),
                    Err(e) => view.error = Some(format!("{e}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Focusable for FileHistoryView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FileHistoryView {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let count = self.history.as_ref().map(|h| h.len()).unwrap_or(0);

        let body = if self.loading {
            placeholder("Loading history…", &t)
        } else if let Some(err) = self.error.clone() {
            placeholder_err(&err, &t)
        } else if let Some(history) = self.history.clone() {
            if history.is_empty() {
                placeholder("No revisions modified this file.", &t)
            } else {
                history_body(history, t.clone(), self.parent.clone(), cx)
            }
        } else {
            placeholder("Unable to load history.", &t)
        };

        div()
            .track_focus(&self.focus_handle)
            .key_context("FileHistoryView")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _cx| {
                window.remove_window();
            }))
            .on_action(cx.listener(|_, _: &Dismiss, window, _cx| {
                window.remove_window();
            }))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(header(&self.path, count, &t))
            .child(body)
    }
}

fn header(path: &SharedString, count: usize, t: &Theme) -> AnyElement {
    let revisions = format!("{count} revisions");
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
        .child(icons::icon(glyph::FILE_CODE, 14., t.fg_dim))
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(12.))
                .text_color(rgb(t.fg))
                .child(path.clone()),
        )
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(revisions)),
        )
        .into_any_element()
}

fn history_body(
    history: Arc<Vec<ChangeInfo>>,
    theme: Theme,
    parent: Entity<RepoWindow>,
    cx: &mut Context<FileHistoryView>,
) -> AnyElement {
    let count = history.len();
    let theme = Arc::new(theme);
    let list = uniform_list(
        "file-history",
        count,
        cx.processor(move |_this, range: std::ops::Range<usize>, _w, cx| {
            range
                .map(|ix| {
                    let entry = history[ix].clone();
                    let parent = parent.clone();
                    let theme = theme.clone();
                    history_row(entry, theme, parent, cx)
                })
                .collect()
        }),
    );
    no_scrollbar_gutter(list).h_full().into_any_element()
}

fn history_row(
    entry: ChangeInfo,
    t: Arc<Theme>,
    parent: Entity<RepoWindow>,
    cx: &mut Context<FileHistoryView>,
) -> AnyElement {
    // Highlight the shortest-unique prefix within the displayed 8 chars.
    let short_id = entry.change_id.id.chars().take(8).collect::<String>();
    let n = (entry.change_id.short_len as usize).min(short_id.len());
    let id_prefix = short_id[..n].to_owned();
    let id_rest = short_id[n..].to_owned();
    let when = format_when(entry.author.timestamp_millis);
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
    let author = entry.author.name.clone();
    let change_id_for_click = entry.change_id.clone();

    div()
        .id(SharedString::from(format!("hist-{}", entry.change_id.id)))
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .px(px(16.))
        .py(px(10.))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(cx.listener(move |_, _, window, cx| {
            let id = change_id_for_click.clone();
            parent.update(cx, |view, cx| view.reveal_change_id(&id, cx));
            window.remove_window();
        }))
        .child(
            div()
                .flex()
                .flex_row()
                .items_baseline()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .font_family(fonts::mono())
                        .text_size(px(11.))
                        .child(
                            div()
                                .text_color(rgb(t.change_id_prefix))
                                .child(SharedString::from(id_prefix)),
                        )
                        .child(
                            div()
                                .text_color(rgb(t.fg_dim))
                                .child(SharedString::from(id_rest)),
                        ),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(SharedString::from(author)),
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
                .text_size(px(12.))
                .text_color(rgb(t.fg))
                .child(SharedString::from(description)),
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
