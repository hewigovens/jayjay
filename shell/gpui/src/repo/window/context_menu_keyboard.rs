use std::sync::Arc;

use gpui::{Bounds, Context, KeyDownEvent, Keystroke, Pixels, ScrollStrategy, Window, point, px};

use crate::app::config;
use crate::ui::context_menu::ContextMenuItem;

use super::{ActivePane, RepoWindow};

impl RepoWindow {
    pub(super) fn handle_context_menu_key(
        &mut self,
        key: &Keystroke,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(menu) = self.context_menu.as_mut() else {
            return false;
        };
        if !key.modifiers.modified() {
            let key = key.key.as_str();
            if key == "escape" && menu.submenu_index.is_none() {
                self.close_context_menu(cx);
            } else if let Some(action) = menu.handle_key(key) {
                self.dispatch_context_action(action, cx);
            } else {
                cx.notify();
            }
        }
        true
    }

    pub(super) fn handle_context_menu_opener(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let key = &event.keystroke;
        let modifiers = &key.modifiers;
        let only = |shift: bool, control: bool| {
            modifiers.shift == shift
                && modifiers.control == control
                && !modifiers.platform
                && !modifiers.alt
                && !modifiers.function
        };
        let opener = match key.key.as_str() {
            "menu" => only(false, false),
            "f10" => only(true, false),
            "enter" => only(false, true),
            _ => false,
        };
        if !opener
            || self.has_refresh_sensitive_interaction()
            || self.rating_prompt
            || self.find.query.is_some()
            || self.focused_control.is_some()
            || self.is_text_input_focused(window, cx)
        {
            return false;
        }
        let vm = self.vm.read(cx);
        if vm.repo.is_none() || vm.error.is_some() || self.selected_context_menu_items(cx).is_none()
        {
            return false;
        }
        if self.selected_row_bounds().is_some() {
            self.open_selected_context_menu(window, cx);
        } else {
            self.reveal_selection(cx);
            self.context_menu_reveal_frames = 2;
            cx.notify();
        }
        true
    }

    fn selected_context_menu_items(&self, cx: &Context<Self>) -> Option<Vec<ContextMenuItem>> {
        let vm = self.vm.read(cx);
        match self.active_pane {
            ActivePane::Sidebar => Some(self.build_change_menu(vm.selected_change()?, cx)),
            ActivePane::FileColumn => {
                let file = vm.files.as_ref()?.get(vm.selected_file_ix?)?;
                Some(self.build_file_context_menu(&file.path, cx))
            }
        }
    }

    fn selected_row_bounds(&self) -> Option<Bounds<Pixels>> {
        match self.active_pane {
            ActivePane::Sidebar => self.selected_change_bounds.get(),
            ActivePane::FileColumn => self.selected_file_bounds.get(),
        }
    }

    fn reveal_selection(&mut self, cx: &mut Context<Self>) {
        match self.active_pane {
            ActivePane::Sidebar => {
                if let Some(ix) = self.vm.read(cx).selected {
                    self.scrolls
                        .changes
                        .scroll_to_item(ix, ScrollStrategy::Center);
                }
            }
            ActivePane::FileColumn => {
                let vm = self.vm.read(cx);
                let (Some(files), Some(ix)) = (vm.files.clone(), vm.selected_file_ix) else {
                    return;
                };
                let (show_review, change_id) = self.review_file_context(cx);
                let visible =
                    Arc::new(self.visible_indices(&files, change_id.as_deref(), show_review, cx));
                self.reveal_file(
                    ix,
                    &files,
                    &visible,
                    config::current(cx).diff.tree_file_list,
                );
            }
        }
    }

    /// The revealed row records its bounds while painting, so the menu opens on the render after that paint.
    pub(super) fn open_revealed_context_menu(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.context_menu_reveal_frames == 0 {
            return;
        }
        self.context_menu_reveal_frames -= 1;
        if self.context_menu_reveal_frames == 0 || self.selected_row_bounds().is_some() {
            self.context_menu_reveal_frames = 0;
            self.open_selected_context_menu(window, cx);
        } else {
            cx.spawn(async move |this, cx| this.update(cx, |_, cx| cx.notify()))
                .detach();
        }
    }

    fn open_selected_context_menu(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Some(items) = self.selected_context_menu_items(cx) else {
            return;
        };
        let (sidebar_width, file_width) =
            self.layout.fitted(f32::from(window.viewport_size().width));
        let fallback_x = match self.active_pane {
            ActivePane::Sidebar => sidebar_width / 2.,
            ActivePane::FileColumn => sidebar_width + file_width / 2.,
        };
        let anchor = self
            .selected_row_bounds()
            .map(|bounds| bounds.center())
            .unwrap_or_else(|| point(px(fallback_x), window.viewport_size().height / 2.));
        self.open_context_menu(anchor, items, cx);
        if let Some(menu) = self.context_menu.as_mut() {
            menu.handle_key("home");
        }
    }
}
