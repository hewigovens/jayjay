use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone};
use gpui::{
    AnyElement, App, AppContext, Bounds, ClickEvent, Context, FocusHandle, Focusable,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement, Pixels, Point,
    Render, SharedString, Size, StatefulInteractiveElement, Styled, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, px, rgb, uniform_list,
};
use jayjay_core::diff::FileDiff;
use jayjay_core::{DiffHunk, EvologEntry, EvologRow, Repo};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::config::AppConfigStore;
use crate::app::fonts;
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::ui::icons::{self, glyph};
use crate::ui::ordered_selection::OrderedSelection;
use crate::ui::primitives::{checkbox_row, no_scrollbar_gutter};

mod context_menu;
mod diff;
mod selection;

use context_menu::{EvologContextMenuState, render_context_menu};

pub struct EvologView {
    repo: Arc<Repo>,
    rev: String,
    title: SharedString,
    entries: Option<Arc<Vec<EvologEntry>>>,
    error: Option<SharedString>,
    loading: bool,
    hide_snapshots: bool,
    expanded_runs: HashSet<u32>,
    selection: OrderedSelection<usize>,
    comparison_reversed: bool,
    files: Option<Arc<Vec<DiffHunk>>>,
    selected_file_ix: Option<usize>,
    current_hunk: Option<DiffHunk>,
    current_diff: Option<Arc<FileDiff>>,
    diff_error: Option<SharedString>,
    diff_loading: bool,
    selection_generation: u64,
    file_generation: u64,
    context_menu: Option<EvologContextMenuState>,
    focus_handle: FocusHandle,
}

impl EvologView {
    pub(crate) fn open(repo: Arc<Repo>, rev: String, title: SharedString, cx: &mut App) {
        let bounds = Bounds::centered(
            None,
            Size {
                width: px(1040.),
                height: px(640.),
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
                            hide_snapshots: true,
                            expanded_runs: HashSet::new(),
                            selection: OrderedSelection::default(),
                            comparison_reversed: false,
                            files: None,
                            selected_file_ix: None,
                            current_hunk: None,
                            current_diff: None,
                            diff_error: None,
                            diff_loading: false,
                            selection_generation: 0,
                            file_generation: 0,
                            context_menu: None,
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

    fn open_context_menu(
        &mut self,
        anchor: Point<Pixels>,
        commit_id: String,
        cx: &mut Context<Self>,
    ) {
        self.context_menu = Some(EvologContextMenuState { anchor, commit_id });
        cx.notify();
    }

    fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.take().is_some() {
            cx.notify();
        }
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
        let context_menu = self
            .context_menu
            .as_ref()
            .map(|state| render_context_menu(state, &t, &cx.entity()));
        let hide_snapshots = self.hide_snapshots;
        let body = if self.loading {
            placeholder("Loading evolution…", &t)
        } else if let Some(err) = self.error.clone() {
            placeholder_err(&err, &t)
        } else if let Some(entries) = self.entries.clone() {
            if entries.is_empty() {
                placeholder("No history", &t)
            } else {
                evolog_body(self, entries, t.clone(), cx)
            }
        } else {
            placeholder("Unable to load history", &t)
        };

        let mut root = div()
            .track_focus(&self.focus_handle)
            .key_context("EvologView")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _cx| {
                window.remove_window();
            }))
            .on_action(cx.listener(|_, _: &Dismiss, window, _cx| {
                window.remove_window();
            }))
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(header(&self.title, hide_snapshots, &t, cx))
            .child(body);
        if let Some(menu) = context_menu {
            root = root.child(menu);
        }
        root
    }
}

fn header(
    title: &SharedString,
    hide_snapshots: bool,
    t: &Theme,
    cx: &mut Context<EvologView>,
) -> AnyElement {
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
            checkbox_row("evolog-hide-snapshots", "Hide snapshots", hide_snapshots, t).on_click(
                cx.listener(|view, _: &ClickEvent, _, cx| {
                    view.set_hide_snapshots(!view.hide_snapshots, cx);
                }),
            ),
        )
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(title.clone()),
        )
        .into_any_element()
}

fn evolog_body(
    view: &EvologView,
    entries: Arc<Vec<EvologEntry>>,
    theme: Theme,
    cx: &mut Context<EvologView>,
) -> AnyElement {
    let rows = Arc::new(view.displayed_rows());
    let count = rows.len();
    let theme = Arc::new(theme);
    let list_theme = theme.clone();
    let selection = view.selection.clone();
    let list = uniform_list(
        "evolog",
        count,
        cx.processor(move |_this, range: std::ops::Range<usize>, _w, cx| {
            range
                .map(|ix| {
                    let row = rows[ix];
                    evolog_row(
                        &entries,
                        row,
                        selection.contains(&(row.start as usize)),
                        &list_theme,
                        cx,
                    )
                })
                .collect()
        }),
    );
    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_h_0()
        .child(
            div()
                .w(px(340.))
                .h_full()
                .border_r_1()
                .border_color(rgb(theme.border))
                .child(no_scrollbar_gutter(list).h_full()),
        )
        .child(diff::comparison(view, &theme, cx))
        .into_any_element()
}

fn evolog_row(
    entries: &Arc<Vec<EvologEntry>>,
    row: EvologRow,
    selected: bool,
    t: &Theme,
    cx: &mut Context<EvologView>,
) -> AnyElement {
    let entry = &entries[row.start as usize];
    let collapsed = row.is_collapsed_run();
    let operation: SharedString = if collapsed {
        format!("{} snapshots", row.count).into()
    } else {
        entry.operation.clone().into()
    };

    let short_commit = entry.commit_id.id.chars().take(12).collect::<String>();
    let n = (entry.commit_id.short_len as usize).min(short_commit.len());
    let commit_prefix = short_commit[..n].to_owned();
    let commit_rest = short_commit[n..].to_owned();
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

    let commit_for_menu = entry.commit_id.id.clone();
    let commit_selector = format!("commit-{commit_for_menu}");
    let debug_commit_selector = commit_selector.clone();
    let selector = if collapsed {
        format!("evolog-snapshot-run-{}-{}", row.start, row.count)
    } else {
        format!("evolog-row-{}", entry.commit_id.id)
    };
    let label_selector = format!("{selector}-label");
    let debug_selector = selector.clone();
    let debug_label = label_selector.clone();

    let operation_row = div()
        .id(SharedString::from(label_selector))
        .debug_selector(move || debug_label.clone())
        .flex()
        .flex_row()
        .items_baseline()
        .gap(px(8.))
        .w_full()
        .child(
            div()
                .text_size(px(12.))
                .text_color(rgb(t.fg))
                .child(operation),
        )
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(10.))
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(when)),
        );
    div()
        .id(SharedString::from(selector))
        .debug_selector(move || debug_selector.clone())
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .px(px(16.))
        .py(px(10.))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .bg(rgb(if selected { t.selected_bg } else { t.detail_bg }))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |view, event: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                view.open_context_menu(event.position, commit_for_menu.clone(), cx);
            }),
        )
        .on_click(cx.listener(move |view, event: &ClickEvent, _, cx| {
            view.select_version(row.start as usize, event.modifiers(), cx);
        }))
        .child(operation_row)
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(description)),
        )
        .child(
            div()
                .id(SharedString::from(commit_selector))
                .debug_selector(move || debug_commit_selector.clone())
                .flex()
                .flex_row()
                .font_family(fonts::mono())
                .text_size(px(10.))
                .child(
                    div()
                        .text_color(rgb(t.change_id_prefix))
                        .child(SharedString::from(commit_prefix)),
                )
                .child(
                    div()
                        .text_color(rgb(t.fg_dim))
                        .child(SharedString::from(commit_rest)),
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
