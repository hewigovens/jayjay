use gpui::{App, AppContext, Bounds, Context, KeyDownEvent, Pixels, Window, point, px};
use jayjay_core::overview::OverviewGroup;
use jayjay_core::{FileDiffStats, OverviewLane};

use super::OverviewView;
use super::panel::PanelText;
use super::placement::Placement;

#[derive(Default)]
pub(super) struct Selection {
    pub(super) lane: Option<String>,
    pub(super) change: Option<String>,
    pub(super) lane_panel: bool,
    pub(super) files: Option<Vec<FileDiffStats>>,
    pub(super) text: Option<PanelText>,
}

impl OverviewView {
    pub(super) fn visible_groups(&self, cx: &App) -> Vec<OverviewGroup> {
        self.snapshot
            .as_ref()
            .map(|snapshot| snapshot.visible_groups(&self.filter.input.read(cx).text()))
            .unwrap_or_default()
    }

    pub(super) fn visible_lanes(&self, cx: &App) -> Vec<usize> {
        self.visible_groups(cx)
            .iter()
            .flat_map(|group| group.lanes.iter().map(|&lane| lane as usize))
            .collect()
    }

    pub(super) fn keep_selection_visible(&mut self, cx: &App) {
        let visible = self.visible_lanes(cx);
        if !self
            .selected_lane_ix()
            .is_some_and(|ix| visible.contains(&ix))
        {
            self.selection.lane = visible.first().map(|&ix| self.lane_ids[ix].clone());
            self.clear_change();
        } else if self.selection.change.is_some() && self.selected_change_offset().is_none() {
            self.clear_change();
        }
    }

    pub(super) fn selected_change_offset(&self) -> Option<usize> {
        let change = self.selection.change.as_ref()?;
        self.lane(self.selected_lane_ix()?)?
            .changes
            .iter()
            .position(|candidate| &candidate.commit_id.id == change)
    }

    pub(super) fn clear_change(&mut self) -> bool {
        self.selection.files = None;
        self.selection.change.take().is_some()
    }

    pub(super) fn selected_lane_ix(&self) -> Option<usize> {
        let selected = self.selection.lane.as_ref()?;
        self.lane_ids.iter().position(|id| id == selected)
    }

    pub(super) fn lane(&self, ix: usize) -> Option<&OverviewLane> {
        self.snapshot.as_ref()?.overview.lanes.get(ix)
    }

    pub(super) fn select_lane(&mut self, lane: usize, cx: &mut Context<Self>) {
        self.selection.lane = self.lane_ids.get(lane).cloned();
        self.clear_change();
        cx.notify();
    }

    pub(super) fn select_change(&mut self, lane: usize, commit_id: String, cx: &mut Context<Self>) {
        self.selection.lane = self.lane_ids.get(lane).cloned();
        if self.selection.change.as_ref() != Some(&commit_id) {
            self.selection.change = Some(commit_id);
            self.load_files(cx);
        }
        cx.notify();
    }

    pub(super) fn move_lane(&mut self, delta: isize, cx: &mut Context<Self>) {
        let order = self.visible_lanes(cx);
        if order.is_empty() {
            return;
        }
        let current = self
            .selected_lane_ix()
            .and_then(|ix| order.iter().position(|&lane| lane == ix))
            .unwrap_or(0);
        let next = current.saturating_add_signed(delta).min(order.len() - 1);
        self.select_lane(order[next], cx);
        self.reveal_selection = true;
    }

    pub(super) fn move_change(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(lane_ix) = self.selected_lane_ix() else {
            return;
        };
        let count = self.lane(lane_ix).map_or(0, |lane| lane.changes.len());
        let next = match self.selected_change_offset() {
            None if delta > 0 => Some(0),
            None => count.checked_sub(1),
            Some(current) => current
                .checked_add_signed(delta)
                .filter(|&next| next < count),
        };
        let commit_id = next
            .and_then(|offset| self.lane(lane_ix)?.changes.get(offset))
            .map(|change| change.commit_id.id.clone());
        match commit_id {
            Some(commit_id) => self.select_change(lane_ix, commit_id, cx),
            None => self.select_lane(lane_ix, cx),
        }
        self.reveal_selection = true;
    }

    pub(super) fn load_files(&mut self, cx: &mut Context<Self>) {
        self.selection.files = None;
        let (Some(repo), Some(commit_id)) = (self.load.repo.clone(), self.selection.change.clone())
        else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let requested = commit_id.clone();
            let files = cx
                .background_spawn(async move {
                    repo.diff_file_stats(&requested, false).unwrap_or_default()
                })
                .await;
            let _ = this.update(cx, |view, cx| {
                if view.selection.change.as_ref() == Some(&commit_id) {
                    view.selection.files = Some(files);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn handle_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.focus_handle.is_focused(window)
            || self.menu.is_some()
            || self.confirmation.is_some()
        {
            return;
        }
        match event.keystroke.key.as_str() {
            "left" => self.move_lane(-1, cx),
            "right" => self.move_lane(1, cx),
            "down" => self.move_change(1, cx),
            "up" => self.move_change(-1, cx),
            "enter" => {
                if let Some(lane) = self.selected_lane_ix() {
                    self.show_in_graph(lane, self.selection.change.clone(), cx);
                }
            }
            _ => return,
        }
        cx.stop_propagation();
    }

    pub(super) fn scroll_into_view(&self, target: Bounds<Pixels>) {
        let viewport = self.scroll.bounds().size;
        if viewport.width <= px(0.) {
            return;
        }
        let offset = self.scroll.offset();
        let max = self.scroll.max_offset();
        let reach = |start: Pixels, len: Pixels, lo: Pixels, hi: Pixels, max: Pixels| {
            let visible = if lo < start {
                lo
            } else if hi > start + len {
                hi - len
            } else {
                start
            };
            visible.clamp(px(0.), max.max(px(0.)))
        };
        let left = reach(
            -offset.x,
            viewport.width,
            target.left(),
            target.right(),
            max.x,
        );
        let top = reach(
            -offset.y,
            viewport.height,
            target.top(),
            target.bottom(),
            max.y,
        );
        self.scroll.set_offset(point(-left, -top));
    }

    pub(super) fn selection_bounds(&self, placement: &Placement) -> Option<Bounds<Pixels>> {
        let placed = placement.lane(self.selected_lane_ix()?)?;
        Some(placement.selection_bounds(placed, self.selected_change_offset()))
    }
}
