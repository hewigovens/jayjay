use gpui::{App, Context};
use jayjay_core::TagInfo;

use super::RepoWindow;
use super::confirmation::{Confirmation, ConfirmedAction};
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

impl RepoWindow {
    /// A tag is a git ref: it is pushed once, and deleting it on the remote is the only later sync.
    pub(super) fn build_tag_menu(&self, name: &str, cx: &App) -> Vec<ContextMenuItem> {
        let on_remote = TagInfo::is_on_remote(&self.vm.read(cx).graph.tags, name);
        let mut items = Vec::new();
        if !on_remote {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::push(),
                glyph::ARROW_UP,
                ContextAction::PushTag(name.to_owned().into()),
            ));
        }
        items.push(ContextMenuItem::new(
            jayjay_i18n::menu::copy_tag_name(),
            glyph::COPY,
            ContextAction::CopyText(name.to_owned().into()),
        ));
        items.push(ContextMenuItem::new(
            jayjay_i18n::menu::delete_tag(),
            glyph::X_CIRCLE,
            ContextAction::DeleteTag(name.to_owned().into()),
        ));
        if on_remote {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::delete_tag_on_remote(),
                glyph::X_CIRCLE,
                ContextAction::DeleteRemoteTag(name.to_owned().into()),
            ));
        }
        items
    }

    /// A release on the tag turns into a draft on GitHub, so the remote deletion asks first.
    pub(super) fn request_remote_tag_delete(&mut self, name: String, cx: &mut Context<Self>) {
        let remotes = TagInfo::remotes_of(&self.vm.read(cx).graph.tags, &name).join(", ");
        self.request_confirmation(
            Confirmation {
                title: jayjay_i18n::repo::delete_tag_title(&name, &remotes).into(),
                message: jayjay_i18n::repo::delete_tag_message(&remotes).into(),
                confirm_label: jayjay_i18n::menu::delete_on_remote().into(),
                action: ConfirmedAction::DeleteRemoteTag { name },
                dont_ask_again: None,
            },
            cx,
        );
    }

    pub(super) fn delete_tag(&mut self, tag: String, cx: &mut Context<Self>) {
        let task = self.vm.update(cx, |vm, cx| vm.delete_tag(tag.clone(), cx));
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, move |view, cx| {
                    view.show_toast(jayjay_i18n::repo::deleted_tag(&tag), cx);
                });
            }
        })
        .detach();
    }
}
