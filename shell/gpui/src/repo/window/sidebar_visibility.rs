use std::time::Duration;

use gpui::Context;

use super::{ActivePane, FocusStop, RepoWindow};

pub(crate) const SIDEBAR_SLIDE: Duration = Duration::from_millis(250);

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
            let slide = self.layout.sidebar_slide;
            let duration = crate::app::motion::animation_duration(cx, SIDEBAR_SLIDE);
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(duration).await;
                let _ = this.update(cx, |view, cx| {
                    if view.layout.sidebar_slide == slide {
                        view.layout.sidebar_closing = false;
                        cx.notify();
                    }
                });
            })
            .detach();
            if self.active_pane == ActivePane::Sidebar {
                self.active_pane = ActivePane::FileColumn;
            }
            if self.focused_control.is_some_and(FocusStop::is_in_sidebar) {
                self.focused_control = None;
            }
            if self.revset_filter.is_some() {
                self.close_revset_filter(cx);
            }
        }
        cx.notify();
    }

    pub fn sidebar_hidden(&self) -> bool {
        self.layout.sidebar_hidden
    }
}
