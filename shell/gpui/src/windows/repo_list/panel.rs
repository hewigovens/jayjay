use std::time::{Duration, Instant};

use gpui::{Context, Window, ease_in_out};

use super::window::{DETAIL_WIDTH, RepoListWindow};
use crate::ui::pane_drag::{PaneDrag, pane_max};
use crate::ui::resize_handle::RESIZE_HANDLE_WIDTH;

pub(crate) const RECENT_PANEL_DEFAULT: f32 = 270.;
const PANEL_MIN: f32 = 220.;
const PANEL_MAX: f32 = 560.;
const PANEL_SLIDE: Duration = Duration::from_millis(180);

#[derive(Clone, Copy)]
struct PanelSlide {
    started: Instant,
    from: f32,
    to: f32,
}

impl PanelSlide {
    fn width_at(&self, now: Instant) -> f32 {
        let progress = (now - self.started).as_secs_f32() / PANEL_SLIDE.as_secs_f32();
        self.from + (self.to - self.from) * ease_in_out(progress.min(1.))
    }
}

pub(super) struct RecentPanel {
    width: f32,
    shown: Option<bool>,
    slide: Option<PanelSlide>,
    drag: Option<PaneDrag<()>>,
}

/// The panel's width and its on-screen width this frame, which is narrower mid-slide.
#[derive(Clone, Copy)]
pub(super) struct PanelFrame {
    pub(super) full: f32,
    pub(super) visible: f32,
}

impl RecentPanel {
    pub(super) fn new(width: f32) -> Self {
        Self {
            width: width.clamp(PANEL_MIN, PANEL_MAX),
            shown: None,
            slide: None,
            drag: None,
        }
    }

    /// The panel gives way before the pinned column drops below its welcome width.
    fn fitted(&self, viewport_width: f32) -> f32 {
        self.width.min(Self::max_width(viewport_width))
    }

    fn max_width(viewport_width: f32) -> f32 {
        pane_max(
            PANEL_MIN,
            PANEL_MAX,
            viewport_width - RESIZE_HANDLE_WIDTH - DETAIL_WIDTH,
        )
    }
}

impl RepoListWindow {
    pub(super) fn panel_frame(
        &mut self,
        shown: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<PanelFrame> {
        let now = cx.background_executor().now();
        let full = self.panel.fitted(f32::from(window.viewport_size().width));
        let to = if shown { full } else { 0. };
        if self
            .panel
            .shown
            .replace(shown)
            .is_some_and(|was| was != shown)
            && !cx.reduce_motion()
        {
            let from = self
                .panel
                .slide
                .map_or(full - to, |slide| slide.width_at(now));
            self.panel.slide = Some(PanelSlide {
                started: now,
                from,
                to,
            });
        }
        if self
            .panel
            .slide
            .is_some_and(|slide| now - slide.started >= PANEL_SLIDE)
        {
            self.panel.slide = None;
        }
        let visible = match self.panel.slide {
            Some(slide) => {
                window.request_animation_frame();
                slide.width_at(now)
            }
            None if shown => full,
            None => return None,
        };
        Some(PanelFrame { full, visible })
    }

    pub(super) fn start_panel_drag(
        &mut self,
        start_x: f32,
        viewport_width: f32,
        cx: &mut Context<Self>,
    ) {
        self.panel.drag = Some(PaneDrag::new(
            (),
            start_x,
            self.panel.fitted(viewport_width),
        ));
        cx.notify();
    }

    pub(super) fn drag_panel_to(&mut self, x: f32, viewport_width: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.panel.drag else {
            return;
        };
        self.panel.width = drag.width_at(x, PANEL_MIN, RecentPanel::max_width(viewport_width));
        cx.notify();
    }

    pub(super) fn end_panel_drag(&mut self, cx: &mut Context<Self>) {
        if self.panel.drag.take().is_none() {
            return;
        }
        let width = self.panel.width;
        crate::app::config::update(cx, move |c| c.layout.recent_repos_panel_width = width);
    }
}
