use std::path::{Path, PathBuf};

use gpui::{
    Anchor, AnyElement, AnyWindowHandle, Context, Entity, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, Pixels, Point, SharedString, Styled, anchored,
    deferred, div, px, rgb,
};

use super::RepoWindow;
use crate::app::repositories;
use crate::app::theme::Theme;
use crate::ui::icons;
use crate::windows::repo_list::RepoListWindow;

#[derive(Clone)]
pub(crate) struct RepoSwitcherState {
    anchor: Point<Pixels>,
    current: String,
    open: Vec<String>,
    pinned: Vec<String>,
}

#[derive(Clone)]
enum RepoSwitcherAction {
    Activate(String),
    Open(String),
    ShowRepositoryList,
    OpenRepository,
}

impl RepoWindow {
    pub fn open_repo_switcher(
        &mut self,
        anchor: Point<Pixels>,
        current_window: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        let current = self.vm.read(cx).repo_path.to_string();
        let open = super::open::open_repo_paths(current_window, &current, cx);
        let open_set = open.iter().collect::<std::collections::HashSet<_>>();
        let pinned = repositories::current(cx)
            .into_iter()
            .filter(|path| !open_set.contains(path))
            .collect();
        self.app_menu = None;
        self.context_menu = None;
        self.repo_switcher = Some(RepoSwitcherState {
            anchor,
            current,
            open,
            pinned,
        });
        cx.notify();
    }

    pub fn close_repo_switcher(&mut self, cx: &mut Context<Self>) {
        if self.repo_switcher.take().is_some() {
            cx.notify();
        }
    }

    fn dispatch_repo_switcher(&mut self, action: RepoSwitcherAction, cx: &mut Context<Self>) {
        self.repo_switcher = None;
        cx.notify();
        match action {
            RepoSwitcherAction::Activate(path) => {
                if path == self.vm.read(cx).repo_path.as_ref() {
                    return;
                }
                cx.defer(move |cx| {
                    super::open::activate_repo_window(Path::new(&path), cx);
                });
            }
            RepoSwitcherAction::Open(path) => {
                cx.defer(move |cx| super::open::open_repo_window(PathBuf::from(path), cx));
            }
            RepoSwitcherAction::ShowRepositoryList => {
                cx.defer(RepoListWindow::open);
            }
            RepoSwitcherAction::OpenRepository => {
                cx.defer(crate::app::menus::prompt_open_repository);
            }
        }
    }
}

pub(crate) fn render_repo_switcher(
    state: &RepoSwitcherState,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let backdrop_view = view.clone();
    let backdrop = div()
        .id("repo-switcher-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            backdrop_view.update(cx, |view, cx| view.close_repo_switcher(cx));
        });
    let panel = anchored()
        .anchor(Anchor::TopLeft)
        .position(state.anchor)
        .snap_to_window_with_margin(px(6.))
        .child(menu_panel(state, t, view));

    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(backdrop)
            .child(panel),
    )
    .with_priority(3)
    .into_any_element()
}

fn menu_panel(state: &RepoSwitcherState, t: &Theme, view: &Entity<RepoWindow>) -> AnyElement {
    let mut panel = div()
        .debug_selector(|| "repo-switcher-panel".to_owned())
        .flex()
        .flex_col()
        .min_w(px(280.))
        .max_w(px(360.))
        .py(px(6.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm()
        .child(section_header("Open Windows", t));
    for (index, path) in state.open.iter().enumerate() {
        panel = panel.child(repository_row(
            format!("repo-switcher-open-{index}"),
            path,
            if path == &state.current {
                icons::glyph::CHECK
            } else {
                icons::glyph::FOLDER
            },
            RepoSwitcherAction::Activate(path.clone()),
            t,
            view,
        ));
    }
    if !state.pinned.is_empty() {
        panel = panel.child(separator(t)).child(section_header("Pinned", t));
        for (index, path) in state.pinned.iter().enumerate() {
            panel = panel.child(repository_row(
                format!("repo-switcher-pinned-{index}"),
                path,
                icons::glyph::PIN,
                RepoSwitcherAction::Open(path.clone()),
                t,
                view,
            ));
        }
    }
    panel
        .child(separator(t))
        .child(action_row(
            "repo-switcher-list",
            "Repository List...",
            icons::glyph::LIST,
            RepoSwitcherAction::ShowRepositoryList,
            t,
            view,
        ))
        .child(action_row(
            "repo-switcher-open-repository",
            "Open Repository...",
            icons::glyph::FOLDER,
            RepoSwitcherAction::OpenRepository,
            t,
            view,
        ))
        .into_any_element()
}

fn repository_row(
    id: String,
    path: &str,
    glyph: &'static str,
    action: RepoSwitcherAction,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let name = repositories::repository_name(path);
    let view = view.clone();
    div()
        .id(SharedString::from(id.clone()))
        .debug_selector(move || id.clone())
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(5.))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(t.selected_bg)))
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            let action = action.clone();
            view.update(cx, |view, cx| {
                view.dispatch_repo_switcher(action, cx);
            });
        })
        .child(icons::icon(glyph, 13., t.fg_dim))
        .child(
            div()
                .flex()
                .min_w_0()
                .flex_1()
                .flex_col()
                .child(div().truncate().text_size(px(12.)).child(name))
                .child(
                    div()
                        .truncate()
                        .text_size(px(10.))
                        .text_color(rgb(t.fg_dim))
                        .child(path.to_owned()),
                ),
        )
        .into_any_element()
}

fn action_row(
    id: &'static str,
    label: &'static str,
    glyph: &'static str,
    action: RepoSwitcherAction,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let view = view.clone();
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(6.))
        .text_size(px(12.))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(t.selected_bg)))
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            let action = action.clone();
            view.update(cx, |view, cx| view.dispatch_repo_switcher(action, cx));
        })
        .child(icons::icon(glyph, 13., t.fg_dim))
        .child(label)
        .into_any_element()
}

fn section_header(label: &'static str, t: &Theme) -> AnyElement {
    div()
        .px(px(10.))
        .pt(px(4.))
        .pb(px(3.))
        .text_size(px(11.))
        .text_color(rgb(t.fg_faint))
        .child(label)
        .into_any_element()
}

fn separator(t: &Theme) -> AnyElement {
    div()
        .mx(px(8.))
        .my(px(4.))
        .h(px(1.))
        .bg(rgb(t.border))
        .into_any_element()
}
