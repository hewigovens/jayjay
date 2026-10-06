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
                "Push",
                glyph::ARROW_UP,
                ContextAction::PushTag(name.to_owned().into()),
            ));
        }
        items.push(ContextMenuItem::new(
            "Copy Tag Name",
            glyph::COPY,
            ContextAction::CopyText(name.to_owned().into()),
        ));
        items.push(ContextMenuItem::new(
            "Delete Tag",
            glyph::X_CIRCLE,
            ContextAction::DeleteTag(name.to_owned().into()),
        ));
        if on_remote {
            items.push(ContextMenuItem::new(
                "Delete Tag on Remote",
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
                title: format!("Delete Tag {name} on {remotes}?").into(),
                message: format!(
                    "This removes the tag locally and pushes the deletion to {remotes}. A GitHub release on this tag turns into a draft and its downloads stop working until the tag exists again."
                )
                .into(),
                confirm_label: "Delete on Remote".into(),
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
                    view.show_toast(format!("Deleted tag {tag}"), cx);
                });
            }
        })
        .detach();
    }
}
