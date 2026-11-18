use std::time::Duration;

use gpui::Context;

use super::{ActivePane, FocusStop, LayoutState, RepoWindow};

pub(crate) const COLUMN_SLIDE: Duration = Duration::from_millis(250);

impl RepoWindow {
    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        let hidden = !self.layout.sidebar_hidden;
        crate::app::config::update(cx, move |c| c.layout.sidebar_hidden = hidden);
        self.apply_sidebar_hidden(hidden, cx);
    }

    pub(crate) fn show_sidebar(&mut self, cx: &mut Context<Self>) {
        if !self.layout.sidebar_hidden {
            return;
        }
        crate::app::config::update(cx, |c| c.layout.sidebar_hidden = false);
        self.apply_sidebar_hidden(false, cx);
        self.active_pane = ActivePane::Sidebar;
    }

    pub(crate) fn apply_sidebar_hidden(&mut self, hidden: bool, cx: &mut Context<Self>) {
        if self.layout.sidebar_hidden == hidden {
            return;
        }
        self.layout.sidebar_hidden = hidden;
        self.layout.sidebar_slide += 1;
        self.layout.sidebar_closing = hidden;
        if hidden {
            self.finish_closing_slide(
                |layout| (layout.sidebar_slide, &mut layout.sidebar_closing),
                cx,
            );
            if self.active_pane == ActivePane::Sidebar {
                self.active_pane = ActivePane::FileColumn;
            }
            if self.focused_control.is_some_and(FocusStop::is_in_sidebar) {
                self.focused_control = None;
            }
        }
        cx.notify();
    }

    /// Unmounts a closing column once its slide ends, unless a later toggle started another.
    pub(crate) fn finish_closing_slide(
        &mut self,
        column: fn(&mut LayoutState) -> (usize, &mut bool),
        cx: &mut Context<Self>,
    ) {
        let (slide, _) = column(&mut self.layout);
        let duration = if cx.reduce_motion() {
            Duration::ZERO
        } else {
            COLUMN_SLIDE
        };
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(duration).await;
            let _ = this.update(cx, |view, cx| {
                let (current, closing) = column(&mut view.layout);
                if current == slide {
                    *closing = false;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn sidebar_hidden(&self) -> bool {
        self.layout.sidebar_hidden
    }
}
