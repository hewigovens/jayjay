use crate::app::actions::{CloseWindow, Dismiss};
use crate::app::theme::{Theme, observe_window_appearance, ui_font_size};
use crate::ui::primitives::button;
use crate::ui::text_area::{Newline, TextArea};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Bounds, Context, Entity, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, TitlebarOptions,
    Window, WindowBounds, WindowOptions, div, px, rgb, size,
};
use jayjay_core::{
    JayError, Repo,
    ssh::{self, RemoteDirectory, RemoteLocation},
};
use std::path::PathBuf;

pub struct OpenRemoteRepositoryView {
    machine: Entity<TextArea>,
    folder: Entity<TextArea>,
    machines: Vec<String>,
    connected_machine: Option<String>,
    directory: Option<RemoteDirectory>,
    busy: bool,
    error: Option<SharedString>,
}

impl OpenRemoteRepositoryView {
    pub fn open(cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(580.), px(460.)), cx);
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(jayjay_i18n::ssh::open_remote_repository().into()),
                        ..Default::default()
                    }),
                    ..crate::app::window_options()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        Self {
                            machine: cx.new(|cx| {
                                TextArea::new(
                                    "",
                                    jayjay_i18n::ssh::host_placeholder(),
                                    false,
                                    32.,
                                    cx,
                                )
                            }),
                            folder: cx
                                .new(|cx| TextArea::new("", "~/workspace/project", false, 32., cx)),
                            machines: vec![],
                            connected_machine: None,
                            directory: None,
                            busy: false,
                            error: None,
                        }
                    })
                },
            )
            .ok();
        if let Some(handle) = handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                window.focus(&view.machine.read(cx).focus_handle(cx), cx);
                if !crate::app::fs_watcher::is_watcher_suppressed(cx) {
                    cx.spawn(async move |this, cx| {
                        let (machines, last) = cx
                            .background_spawn(async { (ssh::machines(), ssh::last_machine()) })
                            .await;
                        let _ = this.update(cx, |view, cx| {
                            view.machines = machines;
                            if let Some(last) = last
                                && view.host(cx).is_empty()
                            {
                                view.machine
                                    .update(cx, |field, cx| field.set_text(&last, cx));
                            }
                            cx.notify();
                        });
                    })
                    .detach();
                }
            });
        }
    }

    fn host(&self, cx: &App) -> String {
        self.machine.read(cx).text().trim().to_owned()
    }

    fn connected(&self, cx: &App) -> bool {
        self.directory.is_some() && self.connected_machine.as_deref() == Some(&self.host(cx))
    }

    fn set_busy(&mut self, busy: bool, cx: &mut Context<Self>) {
        self.busy = busy;
        for field in [&self.machine, &self.folder] {
            field.update(cx, |field, cx| field.set_read_only(busy, cx));
        }
        cx.notify();
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if !self.connected(cx) {
            self.browse(None, cx);
            return;
        }
        let location = match RemoteLocation::new(&self.host(cx), self.folder.read(cx).text().trim())
        {
            Ok(location) => location,
            Err(error) => {
                self.error = Some(crate::app::error_text(error));
                cx.notify();
                return;
            }
        };
        self.set_busy(true, cx);
        self.error = None;
        let handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let path = PathBuf::from(location.encode());
                    match Repo::open(&path) {
                        Ok(repo) => Ok(repo.path().to_owned()),
                        // The repository window offers `jj git init` for a folder that is not a repository yet.
                        Err(JayError::RepoNotFound { .. }) => Ok(path),
                        Err(error) => Err(error),
                    }
                })
                .await;
            let path = this
                .update(cx, |view, cx| {
                    view.set_busy(false, cx);
                    result
                        .map_err(|error| view.error = Some(crate::app::error_text(error)))
                        .ok()
                })
                .ok()
                .flatten();
            if let Some(path) = path {
                let _ = handle.update(cx, |_, window, _| window.remove_window());
                cx.update(|cx| crate::repo::open_repo_window(path, cx));
            }
        })
        .detach();
    }

    fn browse(&mut self, folder: Option<String>, cx: &mut Context<Self>) {
        let machine = self.host(cx);
        if self.busy || machine.is_empty() {
            return;
        }
        let path = folder
            .or_else(|| {
                self.connected(cx)
                    .then(|| self.folder.read(cx).text().trim().to_owned())
            })
            .filter(|path| !path.is_empty())
            .unwrap_or_else(|| "~".into());
        self.set_busy(true, cx);
        self.error = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn({
                    let machine = machine.clone();
                    async move { ssh::browse(&machine, &path) }
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.set_busy(false, cx);
                match result {
                    Ok(directory) => {
                        view.folder
                            .update(cx, |field, cx| field.set_text(&directory.path, cx));
                        view.connected_machine = Some(machine);
                        view.directory = Some(directory);
                    }
                    Err(error) => view.error = Some(crate::app::error_text(error)),
                };
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for OpenRemoteRepositoryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let host = self.host(cx);
        let connected = self.connected(cx);
        let suggestions: Vec<String> = self
            .machines
            .iter()
            .filter(|machine| machine.to_lowercase().contains(&host.to_lowercase()))
            .take(5)
            .cloned()
            .collect();
        div()
            .font_family(crate::app::fonts::interface())
            .track_focus(&self.machine.read(cx).focus_handle(cx))
            .key_context("OpenRemoteRepositoryView")
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
            .text_size(ui_font_size(12.))
            .child(jayjay_i18n::ssh::machine())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().flex_1().child(self.machine.clone()))
                    .when(!connected, |row| {
                        row.child(
                            button("ssh-connect", jayjay_i18n::ssh::connect(), &t, false)
                                .on_click(cx.listener(|view, _, _, cx| view.browse(None, cx))),
                        )
                    }),
            )
            .when(!connected, |root| {
                root.child(div().flex().flex_wrap().gap(px(6.)).children(
                    suggestions.into_iter().map(|host| {
                        button(format!("ssh-host-{host}"), host.clone(), &t, false).on_click(
                            cx.listener(move |view, _, _, cx| {
                                if !view.busy {
                                    view.machine
                                        .update(cx, |field, cx| field.set_text(&host, cx));
                                    view.browse(None, cx);
                                }
                            }),
                        )
                    }),
                ))
            })
            .when_some(
                self.directory.clone().filter(|_| connected),
                |root, directory| {
                    root.child(jayjay_i18n::ssh::folder())
                        .child(self.folder.clone())
                        .child(
                            div()
                                .id("ssh-folders")
                                .flex()
                                .flex_col()
                                .gap(px(4.))
                                .h(px(165.))
                                .overflow_y_scroll()
                                .when_some(directory.parent, |root, parent| {
                                    root.child(
                                        button("ssh-parent", jayjay_i18n::ssh::parent(), &t, false)
                                            .on_click(cx.listener(move |view, _, _, cx| {
                                                view.browse(Some(parent.clone()), cx)
                                            })),
                                    )
                                })
                                .children(directory.folders.into_iter().map(|folder| {
                                    button(
                                        format!("ssh-folder-{}", folder.path),
                                        folder.name,
                                        &t,
                                        false,
                                    )
                                    .on_click(cx.listener(
                                        move |view, _, _, cx| {
                                            view.browse(Some(folder.path.clone()), cx)
                                        },
                                    ))
                                })),
                        )
                },
            )
            .when(self.busy, |root| root.child(jayjay_i18n::ssh::connecting()))
            .when_some(self.error.clone(), |root, error| {
                root.child(
                    div()
                        .id("open-remote-repository-error")
                        .text_color(rgb(t.error_fg))
                        .child(error),
                )
            })
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        button(
                            "open-remote-repository-cancel",
                            jayjay_i18n::common::cancel(),
                            &t,
                            false,
                        )
                        .on_click(cx.listener(|_, _, window, _| window.remove_window())),
                    )
                    .child(
                        button(
                            "open-remote-repository-submit",
                            jayjay_i18n::common::open(),
                            &t,
                            true,
                        )
                        .when(self.busy || !connected, |item| item.opacity(0.5))
                        .on_click(cx.listener(|view, _, window, cx| {
                            if view.connected(cx) {
                                view.submit(window, cx)
                            }
                        })),
                    ),
            )
    }
}
