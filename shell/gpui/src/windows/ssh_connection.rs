use crate::app::theme::{Theme, ui_font_size};
use crate::ui::primitives::button;
use gpui::{
    App, AppContext, Bounds, Context, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    px, rgb, size,
};
use jayjay_core::{
    Repo,
    ssh::{ConnectionState, RemoteLocation},
};
use std::sync::Arc;

pub struct SSHConnectionView {
    repo: Arc<Repo>,
    location: RemoteLocation,
    busy: bool,
    error: Option<String>,
}
impl SSHConnectionView {
    pub fn open(repo: Arc<Repo>, location: RemoteLocation, cx: &mut App) {
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(480.), px(250.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some(jayjay_i18n::ssh::connection().into()),
                    ..Default::default()
                }),
                ..crate::app::window_options()
            },
            |_, cx| {
                cx.new(|cx| {
                    cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                    Self {
                        repo,
                        location,
                        busy: false,
                        error: None,
                    }
                })
            },
        );
    }
    fn toggle(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.error = None;
        cx.notify();
        let repo = self.repo.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    if repo.connection_state() == ConnectionState::Connected {
                        repo.disconnect()
                    } else {
                        repo.reconnect()
                    }
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                view.busy = false;
                view.error = result.err().map(|e| e.to_string());
                cx.notify();
            });
        })
        .detach();
    }
}
impl Render for SSHConnectionView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let connected = self.repo.connection_state() == ConnectionState::Connected;
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .size_full()
            .p(px(20.))
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .text_size(ui_font_size(12.))
            .child(self.location.machine.clone())
            .child(self.location.path.clone())
            .child(if self.busy {
                jayjay_i18n::ssh::connecting()
            } else if connected {
                jayjay_i18n::ssh::connected()
            } else {
                jayjay_i18n::ssh::disconnected()
            })
            .children(
                self.error
                    .clone()
                    .map(|error| div().text_color(rgb(t.error_fg)).child(error)),
            )
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(
                        button("ssh-copy-path", jayjay_i18n::ssh::copy_path(), &t, false).on_click(
                            cx.listener(|view, _, _, cx| {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    view.location.path.clone(),
                                ))
                            }),
                        ),
                    )
                    .child(
                        button(
                            "ssh-toggle",
                            if connected {
                                jayjay_i18n::ssh::disconnect()
                            } else {
                                jayjay_i18n::ssh::reconnect()
                            },
                            &t,
                            true,
                        )
                        .on_click(cx.listener(|view, _, _, cx| view.toggle(cx))),
                    ),
            )
    }
}
