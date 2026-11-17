use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::ssh::{ConnectionState, RemoteLocation};

use super::RepoWindow;
use crate::app::theme::{FONT_BODY, FONT_META, Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::small_button;

impl RepoWindow {
    pub(crate) fn reconnect_ssh(&mut self, cx: &mut Context<Self>) {
        let disconnected = self
            .vm
            .read(cx)
            .repo
            .as_ref()
            .is_some_and(|repo| repo.connection_state() == ConnectionState::Disconnected);
        if !disconnected {
            return;
        }
        self.vm.update(cx, |vm, cx| {
            vm.repo_write_task(cx, |repo| repo.reconnect(), |vm, cx| vm.refresh(false, cx))
                .detach();
        });
    }
}

pub(super) fn ssh_reconnect_bar(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Option<AnyElement> {
    let vm = view.vm.read(cx);
    let repo = vm.repo.clone()?;
    let location = RemoteLocation::from_path(vm.repo_path.as_ref())?;
    let state = repo.connection_state();
    if state == ConnectionState::Connected {
        return None;
    }
    let connecting = state == ConnectionState::Connecting;
    let detail = repo
        .connection_error()
        .unwrap_or_else(jayjay_i18n::ssh::error_disconnected);
    let mut button = small_button(
        "ssh-reconnect",
        if connecting {
            jayjay_i18n::ssh::reconnecting()
        } else {
            jayjay_i18n::ssh::reconnect()
        },
        t,
    )
    .debug_selector(|| "ssh-reconnect".to_owned());
    button = if connecting {
        button.opacity(0.45)
    } else {
        button.on_click(cx.listener(|view, _, _, cx| view.reconnect_ssh(cx)))
    };
    Some(
        div()
            .debug_selector(|| "ssh-reconnect-bar".to_owned())
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.))
            .w_full()
            .px(px(12.))
            .py(px(8.))
            .bg(rgb(t.header_bg))
            .border_t_1()
            .border_color(rgb(t.border))
            .text_size(ui_font_size(FONT_BODY))
            .text_color(rgb(t.fg))
            .child(icons::icon(glyph::CLOUD_OFF, 13., t.error_fg))
            .child(div().flex_none().child(location.machine))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(ui_font_size(FONT_META))
                    .text_color(rgb(t.fg_dim))
                    .child(detail),
            )
            .child(button)
            .into_any_element(),
    )
}
