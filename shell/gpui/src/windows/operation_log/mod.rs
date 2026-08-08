use std::sync::Arc;

mod chrome;
mod rows;

use gpui::{
    App, AppContext, Bounds, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Size, Styled, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, px, rgb,
};
use jayjay_core::{OpLogEntry, Repo};

use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::repo::window::RepoWindow;
use chrome::{footer, header, placeholder, placeholder_err};
use rows::operation_list;

pub struct OperationLogView {
    repo: Arc<Repo>,
    parent: Entity<RepoWindow>,
    entries: Option<Arc<Vec<OpLogEntry>>>,
    selected_id: Option<String>,
    error: Option<SharedString>,
    loading: bool,
    restoring: bool,
    focus_handle: FocusHandle,
}

impl OperationLogView {
    pub(crate) fn open(repo: Arc<Repo>, parent: Entity<RepoWindow>, cx: &mut App) {
        let bounds = Bounds::centered(
            None,
            Size {
                width: px(600.),
                height: px(480.),
            },
            cx,
        );
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Operation Log".into()),
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
                            entries: None,
                            selected_id: None,
                            error: None,
                            loading: true,
                            restoring: false,
                            focus_handle: cx.focus_handle(),
                        };
                        view.load(cx);
                        view
                    })
                },
            )
            .ok();
        if let Some(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                let focus = view.focus_handle(cx);
                window.focus(&focus, cx);
            });
        }
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        let repo = self.repo.clone();
        self.loading = true;
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx.background_spawn(async move { repo.op_log() }).await;
            let _ = this.update(cx, move |view, cx| {
                view.loading = false;
                match result {
                    Ok(entries) => {
                        if !entries
                            .iter()
                            .any(|entry| view.selected_id.as_deref() == Some(entry.id.as_str()))
                        {
                            view.selected_id = None;
                        }
                        view.entries = Some(Arc::new(entries));
                    }
                    Err(error) => view.error = Some(format!("{error}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn select_operation(&mut self, op_id: String, cx: &mut Context<Self>) {
        self.selected_id = Some(op_id);
        cx.notify();
    }

    fn restore_selected(&mut self, cx: &mut Context<Self>) {
        let Some(op_id) = self.selected_id.clone() else {
            return;
        };
        if self.selected_is_current() || self.restoring {
            return;
        }
        let repo = self.repo.clone();
        self.restoring = true;
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.op_restore(&op_id) })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.restoring = false;
                match result {
                    Ok(()) => {
                        view.selected_id = None;
                        view.refresh_parent(cx);
                        view.load(cx);
                    }
                    Err(error) => {
                        view.error = Some(format!("{error}").into());
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    fn refresh_parent(&self, cx: &mut Context<Self>) {
        let parent = self.parent.clone();
        parent.update(cx, |view, cx| {
            let vm = view.view_model();
            vm.update(cx, |vm, cx| vm.refresh(false, cx));
            view.show_toast("Restored operation", cx);
        });
    }

    fn selected_is_current(&self) -> bool {
        let Some(selected) = self.selected_id.as_deref() else {
            return false;
        };
        self.entries
            .as_ref()
            .and_then(|entries| entries.iter().find(|entry| entry.id.as_str() == selected))
            .is_some_and(|entry| entry.is_current)
    }
}

impl Focusable for OperationLogView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for OperationLogView {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let body = if self.loading {
            placeholder("Loading operations...", &t)
        } else if let Some(error) = self.error.clone() {
            placeholder_err(&error, &t)
        } else if let Some(entries) = self.entries.clone() {
            if entries.is_empty() {
                placeholder("No operations found", &t)
            } else {
                operation_list(entries, self.selected_id.clone(), t.clone(), cx)
            }
        } else {
            placeholder("Unable to load operations", &t)
        };

        div()
            .track_focus(&self.focus_handle)
            .key_context("OperationLogView")
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
            .child(header(&t))
            .child(body)
            .child(footer(self.can_restore(), self.restoring, &t, cx))
    }
}

impl OperationLogView {
    fn can_restore(&self) -> bool {
        !self.loading
            && self.error.is_none()
            && self.selected_id.is_some()
            && !self.selected_is_current()
    }
}
