use gpui::{App, Context};

use super::EvologView;
use crate::repo::window::{SECONDARY_PANE_MAX, SECONDARY_PANE_MIN, pane_max};
use crate::ui::resize_handle::RESIZE_HANDLE_WIDTH;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum EvologPane {
    EntryList,
    FileList,
}

#[derive(Clone, Copy)]
struct PaneDrag {
    pane: EvologPane,
    start_x: f32,
    start_width: f32,
}

/// Entry list and file list widths; both seed from the shared secondary-pane preference, only the entry list writes it back.
pub(super) struct EvologLayout {
    entry_list_width: f32,
    file_list_width: f32,
    drag: Option<PaneDrag>,
}

impl EvologLayout {
    pub(super) fn from_config(cx: &App) -> Self {
        let width = crate::app::config::current(cx)
            .layout
            .secondary_pane_width
            .clamp(SECONDARY_PANE_MIN, SECONDARY_PANE_MAX);
        Self {
            entry_list_width: width,
            file_list_width: width,
            drag: None,
        }
    }

    fn entry_list_max(viewport_width: f32) -> f32 {
        let room = viewport_width - 2. * RESIZE_HANDLE_WIDTH - 2. * SECONDARY_PANE_MIN;
        pane_max(SECONDARY_PANE_MIN, SECONDARY_PANE_MAX, room)
    }

    fn file_list_max(viewport_width: f32, entry_list_width: f32) -> f32 {
        let room =
            viewport_width - entry_list_width - 2. * RESIZE_HANDLE_WIDTH - SECONDARY_PANE_MIN;
        pane_max(SECONDARY_PANE_MIN, SECONDARY_PANE_MAX, room)
    }

    pub(super) fn fitted(&self, viewport_width: f32) -> (f32, f32) {
        let entry_list = self
            .entry_list_width
            .min(Self::entry_list_max(viewport_width));
        let file_list = self
            .file_list_width
            .min(Self::file_list_max(viewport_width, entry_list));
        (entry_list, file_list)
    }
}

impl EvologView {
    pub(super) fn start_pane_drag(
        &mut self,
        pane: EvologPane,
        start_x: f32,
        viewport_width: f32,
        cx: &mut Context<Self>,
    ) {
        let (entry_list, file_list) = self.layout.fitted(viewport_width);
        let start_width = match pane {
            EvologPane::EntryList => entry_list,
            EvologPane::FileList => file_list,
        };
        self.layout.drag = Some(PaneDrag {
            pane,
            start_x,
            start_width,
        });
        cx.notify();
    }

    pub(super) fn drag_pane_to(&mut self, x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.layout.drag else {
            return;
        };
        let width = drag.start_width + (x - drag.start_x);
        match drag.pane {
            EvologPane::EntryList => {
                self.layout.entry_list_width = width.clamp(
                    SECONDARY_PANE_MIN,
                    EvologLayout::entry_list_max(viewport_width),
                );
            }
            EvologPane::FileList => {
                let (entry_list, _) = self.layout.fitted(viewport_width);
                self.layout.file_list_width = width.clamp(
                    SECONDARY_PANE_MIN,
                    EvologLayout::file_list_max(viewport_width, entry_list),
                );
            }
        }
        cx.notify();
    }

    pub(super) fn end_pane_drag(&mut self, cx: &mut Context<Self>) {
        let Some(drag) = self.layout.drag.take() else {
            return;
        };
        if drag.pane == EvologPane::EntryList {
            let width = self.layout.entry_list_width;
            crate::app::config::update(cx, move |c| c.layout.secondary_pane_width = width);
        }
        cx.notify();
    }
}
