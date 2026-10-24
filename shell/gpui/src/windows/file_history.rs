use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Bounds, ClickEvent, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, ParentElement,
    Render, ScrollStrategy, SharedString, Size, StatefulInteractiveElement, Styled,
    TitlebarOptions, UniformListScrollHandle, WeakEntity, Window, WindowBounds, WindowOptions, div,
    px, rgb, uniform_list,
};
use jayjay_core::diff::FileDiff;
use jayjay_core::{ChangeInfo, DiffHunk, Repo};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::config::AppConfigStore;
use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, observe_window_appearance, ui_font_size};
use crate::repo::view_model::RepoViewModel;
use crate::repo::window::{RepoWindow, compact_id, format_when, id_cell};
use crate::ui::icons::{self, glyph};
use crate::ui::loading::loading_status;
use crate::ui::pane_drag::TrackPaneDrag as _;
use crate::ui::primitives::{
    capsule, icon_button, no_scrollbar_gutter, placeholder, placeholder_err, small_button,
    text_tooltip,
};
use crate::ui::resizable_pane::ResizablePane;
use crate::ui::resize_handle::resize_handle;

mod comparison;
mod context_menu;
mod diff;
mod layout;

use comparison::{FileHistoryComparison, FileHistoryEndpoint};
use context_menu::{FileHistoryMenu, render_menu};

pub struct FileHistoryView {
    repo: Arc<Repo>,
    parent: WeakEntity<RepoWindow>,
    path: SharedString,
    history: Option<Arc<Vec<ChangeInfo>>>,
    error: Option<SharedString>,
    loading: bool,
    comparison: FileHistoryComparison,
    current_hunk: Option<DiffHunk>,
    current_diff: Option<Arc<FileDiff>>,
    diff_error: Option<SharedString>,
    menu: Option<FileHistoryMenu>,
    history_scroll: UniformListScrollHandle,
    list_layout: ResizablePane,
    focus_handle: FocusHandle,
}

impl FileHistoryView {
    pub(crate) fn open(
        repo: Arc<Repo>,
        path: String,
        parent: Entity<RepoWindow>,
        vm: Entity<RepoViewModel>,
        cx: &mut App,
    ) {
        let bounds = Bounds::centered(
            None,
            Size {
                width: px(1100.),
                height: px(700.),
            },
            cx,
        );
        let title = jayjay_i18n::history::window_title(&path);
        let path_for_view: SharedString = path.into();
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(title.into()),
                        ..Default::default()
                    }),
                    ..crate::app::window_options()
                },
                |window, cx| {
                    cx.new(|cx| {
                        cx.observe_release_in(&vm, window, |_, _, window, _| {
                            window.remove_window()
                        })
                        .detach();
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        let mut view = Self {
                            repo,
                            parent: parent.downgrade(),
                            path: path_for_view,
                            history: None,
                            error: None,
                            loading: true,
                            comparison: FileHistoryComparison::default(),
                            current_hunk: None,
                            current_diff: None,
                            diff_error: None,
                            menu: None,
                            history_scroll: UniformListScrollHandle::new(),
                            list_layout: Self::list_pane(cx),
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

    pub fn comparison_endpoints(&self) -> (Option<String>, Option<String>) {
        (
            self.comparison
                .base()
                .map(|endpoint| endpoint.commit_id.id.clone()),
            self.comparison
                .target()
                .map(|endpoint| endpoint.commit_id.id.clone()),
        )
    }

    pub fn loaded_file_contents(&self) -> Option<(Option<String>, Option<String>)> {
        self.current_hunk
            .as_ref()
            .map(|hunk| (hunk.old.content.clone(), hunk.new.content.clone()))
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
                    Ok(entries) => {
                        view.history = Some(Arc::new(entries));
                        view.select_newest(cx);
                    }
                    Err(e) => view.error = Some(crate::app::error_text(e)),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn select_newest(&mut self, cx: &mut Context<Self>) {
        if self.comparison.target().is_some() {
            return;
        }
        let Some(entry) = self
            .history
            .as_ref()
            .and_then(|history| history.first().cloned())
        else {
            return;
        };
        self.select_entry(&entry, cx);
    }

    fn select_entry(&mut self, entry: &ChangeInfo, cx: &mut Context<Self>) {
        let endpoint = FileHistoryEndpoint::from_change(entry);
        if self
            .comparison
            .target()
            .is_some_and(|target| target.commit_id.id == endpoint.commit_id.id)
        {
            return;
        }
        self.comparison.select(endpoint);
        self.refresh_target(cx);
    }

    fn pin_selected(&mut self, cx: &mut Context<Self>) {
        let Some(target) = self.comparison.target().cloned() else {
            return;
        };
        self.pin_endpoint(target, cx);
    }

    fn pin_endpoint(&mut self, endpoint: FileHistoryEndpoint, cx: &mut Context<Self>) {
        self.comparison.pin(endpoint);
        self.refresh_target(cx);
    }

    fn swap_endpoints(&mut self, cx: &mut Context<Self>) {
        if !self.comparison.is_pinned() {
            return;
        }
        self.comparison.swap();
        self.refresh_target(cx);
    }

    fn refresh_target(&mut self, cx: &mut Context<Self>) {
        self.scroll_target_into_view();
        self.load_comparison(cx);
    }

    fn clear_base(&mut self, cx: &mut Context<Self>) {
        if !self.comparison.is_pinned() {
            return;
        }
        self.comparison.clear();
        self.load_comparison(cx);
    }

    fn move_target(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(history) = self.history.clone() else {
            return;
        };
        if history.is_empty() {
            return;
        }
        let current = self
            .comparison
            .target()
            .and_then(|target| {
                history
                    .iter()
                    .position(|entry| entry.commit_id.id == target.commit_id.id)
            })
            .unwrap_or(0);
        let next = current.saturating_add_signed(delta).min(history.len() - 1);
        if self.comparison.target().is_some() && next == current {
            return;
        }
        self.select_entry(&history[next], cx);
    }

    fn reveal_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.selected_entry() else {
            return;
        };
        self.reveal_entry(&entry, window, cx);
    }

    fn reveal_entry(&mut self, entry: &ChangeInfo, window: &mut Window, cx: &mut Context<Self>) {
        let revision = entry.selection_revision().to_owned();
        let _ = self
            .parent
            .update(cx, |view, cx| view.reveal_change_id(&revision, cx));
        window.remove_window();
    }

    fn selected_entry(&self) -> Option<ChangeInfo> {
        let target = self.comparison.target()?;
        self.history
            .as_ref()?
            .iter()
            .find(|entry| entry.commit_id.id == target.commit_id.id)
            .cloned()
    }

    fn scroll_target_into_view(&self) {
        let Some(history) = &self.history else {
            return;
        };
        let Some(target) = self.comparison.target() else {
            return;
        };
        if let Some(ix) = history
            .iter()
            .position(|entry| entry.commit_id.id == target.commit_id.id)
        {
            self.history_scroll
                .scroll_to_item(ix, ScrollStrategy::Nearest);
        }
    }

    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_handle.is_focused(window) || self.menu.is_some() {
            return;
        }
        match event.keystroke.key.as_str() {
            "up" => self.move_target(-1, cx),
            "down" => self.move_target(1, cx),
            "enter" => self.reveal_selected(window, cx),
            _ => return,
        }
        cx.stop_propagation();
    }

    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.menu.is_some() {
            self.close_menu(cx);
            return;
        }
        if self.comparison.is_pinned() {
            self.clear_base(cx);
            return;
        }
        window.remove_window();
    }
}

impl Focusable for FileHistoryView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FileHistoryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let count = self.history.as_ref().map(|h| h.len()).unwrap_or(0);
        let menu = self
            .menu
            .as_ref()
            .map(|menu| render_menu(menu, &t, &cx.entity()));

        let body = if self.loading {
            loading_status(jayjay_i18n::history::loading_history_ellipsis(), &t)
        } else if let Some(err) = self.error.clone() {
            placeholder_err(&err, &t)
        } else if let Some(history) = self.history.clone() {
            if history.is_empty() {
                placeholder(jayjay_i18n::repo::no_revisions_modified_this_file(), &t)
            } else {
                history_body(self, history, &t, window, cx)
            }
        } else {
            placeholder(jayjay_i18n::history::unable_to_load_history(), &t)
        };

        let mut root = div()
            .font_family(crate::app::fonts::interface())
            .track_focus(&self.focus_handle)
            .key_context("FileHistoryView")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _cx| {
                window.remove_window();
            }))
            .on_action(cx.listener(|view, _: &Dismiss, window, cx| {
                view.dismiss(window, cx);
            }))
            .on_key_down(cx.listener(Self::on_key))
            .track_pane_drag(
                self.list_layout.is_dragging(),
                FileHistoryView::drag_list_to,
                FileHistoryView::end_list_drag,
                cx,
            )
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(header(self, count, &t, cx))
            .child(body);
        if let Some(menu) = menu {
            root = root.child(menu);
        }
        root
    }
}

fn header(
    view: &FileHistoryView,
    count: usize,
    t: &Theme,
    cx: &mut Context<FileHistoryView>,
) -> AnyElement {
    let revisions = jayjay_i18n::history::revision_count(count);
    div()
        .flex()
        .flex_col()
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .px(px(16.))
                .pt(px(10.))
                .child(icons::icon(glyph::FILE_CODE, 14., t.fg_dim))
                .child(
                    div()
                        .code_text(12., t)
                        .text_color(rgb(t.fg))
                        .child(view.path.clone()),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(ui_font_size(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(SharedString::from(revisions)),
                ),
        )
        .child(comparison_bar(view, t, cx))
        .into_any_element()
}

fn comparison_bar(
    view: &FileHistoryView,
    t: &Theme,
    cx: &mut Context<FileHistoryView>,
) -> AnyElement {
    let mut bar = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(16.))
        .py(px(8.));
    if let Some(base) = view.comparison.base() {
        bar = bar
            .child(endpoint_label(
                "base",
                &jayjay_i18n::repo::base(),
                base,
                t.compare_accent,
                t,
            ))
            .child(
                icon_button(
                    "file-history-swap",
                    glyph::ARROWS_LEFT_RIGHT,
                    12.,
                    22.,
                    20.,
                    t.fg_dim,
                    t,
                )
                .debug_selector(|| "file-history-swap".to_owned())
                .tooltip(text_tooltip(jayjay_i18n::repo::swap_base_and_target()))
                .on_click(cx.listener(|view, _: &ClickEvent, _, cx| view.swap_endpoints(cx))),
            );
    }
    if let Some(target) = view.comparison.target() {
        bar = bar.child(endpoint_label(
            "target",
            &jayjay_i18n::repo::target(),
            target,
            t.selected_accent,
            t,
        ));
    } else {
        bar = bar.child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child(jayjay_i18n::repo::select_a_revision()),
        );
    }
    bar = bar.child(div().flex_1()).child(pin_button(view, t, cx));
    if view.comparison.is_pinned() {
        bar = bar.child(
            small_button("file-history-clear", jayjay_i18n::common::clear(), t)
                .debug_selector(|| "file-history-clear".to_owned())
                .tooltip(text_tooltip(jayjay_i18n::repo::clear_the_pinned_base_esc()))
                .on_click(cx.listener(|view, _: &ClickEvent, _, cx| view.clear_base(cx))),
        );
    }
    bar.into_any_element()
}

fn pin_button(view: &FileHistoryView, t: &Theme, cx: &mut Context<FileHistoryView>) -> AnyElement {
    let mut button = small_button("file-history-pin", jayjay_i18n::repo::pin_as_base(), t)
        .debug_selector(|| "file-history-pin".to_owned())
        .tooltip(text_tooltip(
            jayjay_i18n::repo::pin_the_selected_revision_as_the(),
        ));
    if view.comparison.target().is_some() {
        button = button.on_click(cx.listener(|view, _: &ClickEvent, _, cx| view.pin_selected(cx)));
    } else {
        button = button.opacity(0.45);
    }
    button.into_any_element()
}

fn endpoint_label(
    id: &str,
    role: &str,
    endpoint: &FileHistoryEndpoint,
    color: u32,
    t: &Theme,
) -> AnyElement {
    let selector = format!(
        "file-history-{id}-{}-{}",
        endpoint.change_id.id, endpoint.commit_id.id
    );
    let change_id = compact_id(&endpoint.change_id);
    let commit_id = compact_id(&endpoint.commit_id);
    let help =
        jayjay_i18n::history::endpoint_help(role, &endpoint.change_id.id, &endpoint.commit_id.id);
    div()
        .id(SharedString::from(selector.clone()))
        .debug_selector(move || selector)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .tooltip(text_tooltip(help))
        .child(
            div()
                .text_size(ui_font_size(10.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(color))
                .child(role.to_owned()),
        )
        .child(id_cell(
            &change_id,
            endpoint.change_id.short_len,
            t.change_id_prefix,
            11.,
            t,
        ))
        .child(
            div()
                .code_text(11., t)
                .text_color(rgb(t.fg_dim))
                .child(commit_id),
        )
        .into_any_element()
}

fn history_body(
    view: &FileHistoryView,
    history: Arc<Vec<ChangeInfo>>,
    theme: &Theme,
    window: &mut Window,
    cx: &mut Context<FileHistoryView>,
) -> AnyElement {
    let count = history.len();
    let theme_for_rows = Arc::new(theme.clone());
    let base_commit = view
        .comparison
        .base()
        .map(|endpoint| endpoint.commit_id.id.clone());
    let target_commit = view
        .comparison
        .target()
        .map(|endpoint| endpoint.commit_id.id.clone());
    let scroll = view.history_scroll.clone();
    let list = uniform_list(
        "file-history",
        count,
        cx.processor(move |_this, range: std::ops::Range<usize>, _w, cx| {
            range
                .map(|ix| {
                    history_row(
                        history[ix].clone(),
                        base_commit.clone(),
                        target_commit.clone(),
                        theme_for_rows.clone(),
                        cx,
                    )
                })
                .collect()
        }),
    );
    let list_width = view
        .list_layout
        .fitted(f32::from(window.viewport_size().width));
    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_h_0()
        .child(
            div()
                .w(px(list_width))
                .h_full()
                .flex_none()
                .child(no_scrollbar_gutter(list).h_full().track_scroll(&scroll)),
        )
        .child(resize_handle(
            "file-history-list-resize-handle",
            theme,
            |view: &mut FileHistoryView, x, viewport_width, cx| {
                view.start_list_drag(x, viewport_width, cx)
            },
            cx,
        ))
        .child(diff::comparison_diff(view, theme, window, cx))
        .into_any_element()
}

fn history_row(
    entry: ChangeInfo,
    base_commit: Option<String>,
    target_commit: Option<String>,
    t: Arc<Theme>,
    cx: &mut Context<FileHistoryView>,
) -> AnyElement {
    let short_id = compact_id(&entry.change_id);
    let when = format_when(entry.author.timestamp_millis);
    let description = if entry.description.trim().is_empty() {
        jayjay_i18n::repo::no_description_placeholder()
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
    let commit_id = entry.commit_id.id.clone();
    let selected = target_commit.as_deref() == Some(commit_id.as_str());
    let role = if base_commit.as_deref() == Some(commit_id.as_str()) {
        Some((jayjay_i18n::repo::base(), t.compare_accent, t.compare_bg))
    } else if selected {
        Some((
            jayjay_i18n::repo::target(),
            t.selected_accent,
            t.selected_bg,
        ))
    } else {
        None
    };
    // Divergent revisions share a change id, so the row key is the commit id.
    let selector = format!("file-history-row-{commit_id}");
    let entry_for_click = entry.clone();
    let entry_for_menu = entry.clone();

    let mut row = div()
        .id(SharedString::from(commit_id))
        .debug_selector(move || selector)
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .px(px(16.))
        .py(px(10.))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |view, event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                view.select_entry(&entry_for_menu, cx);
                view.open_menu(event.position, entry_for_menu.clone(), cx);
                window.focus(&view.focus_handle, cx);
            }),
        )
        .on_click(cx.listener(move |view, event: &ClickEvent, window, cx| {
            view.select_entry(&entry_for_click, cx);
            if event.click_count() == 2 {
                view.reveal_entry(&entry_for_click, window, cx);
            } else {
                window.focus(&view.focus_handle, cx);
            }
        }));
    if selected {
        row = row.bg(rgb(t.selected_bg));
    } else {
        let hover = t.row_alt_bg;
        row = row.hover(move |style| style.bg(rgb(hover)));
    }

    row.child(
        div()
            .flex()
            .flex_row()
            .items_baseline()
            .gap(px(8.))
            .child(id_cell(
                &short_id,
                entry.change_id.short_len,
                t.change_id_prefix,
                11.,
                &t,
            ))
            .child(
                div()
                    .text_size(ui_font_size(11.))
                    .text_color(rgb(t.fg_dim))
                    .child(SharedString::from(author)),
            )
            .child(div().flex_1())
            .children(role.map(|(label, fg, bg)| capsule(label, bg, fg, 9.)))
            .child(
                div()
                    .text_size(ui_font_size(10.))
                    .text_color(rgb(t.fg_faint))
                    .child(SharedString::from(when)),
            ),
    )
    .child(
        div()
            .w_full()
            .truncate()
            .text_size(ui_font_size(12.))
            .text_color(rgb(t.fg))
            .child(SharedString::from(description)),
    )
    .into_any_element()
}
