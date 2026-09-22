use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, AppContext as _, Context, Div, Entity, InteractiveElement as _, SharedString, Stateful,
    Window,
};
use jayjay_core::{MergePane, MergeScrollMap};

use crate::ui::merge_editor::merge_hunk_list_container;
use crate::ui::text_area::{TextArea, TextAreaScrolled, TextAreaUpdated};

use super::{MergeHunkListScroll, MergeScroll, MergeScrollTargets, SOURCE_PANES};

const REBUILD_DELAY: Duration = Duration::from_millis(120);
/// A remounted pane has no layout for a frame or two; give up rather than retry forever.
const ADOPT_ATTEMPTS: u32 = 4;

#[derive(Default)]
pub(crate) struct MergeSync {
    pub(crate) scroll: MergeScroll,
    pub(crate) hunk_list: MergeHunkListScroll,
    generation: u64,
    pristine_result: String,
    /// Result text the current scroll map reflects; highlight-only updates skip the rebuild.
    mapped_result: String,
    suppress_result: bool,
}

impl MergeSync {
    pub(crate) fn load(
        &mut self,
        map: MergeScrollMap,
        hunk_count: u32,
        is_raw: bool,
        result: String,
    ) {
        *self = Self::default();
        self.pristine_result = result.clone();
        self.mapped_result = result;
        self.scroll.load(map, hunk_count, is_raw);
    }
}

pub(crate) trait MergeSynchronized: Sized + 'static {
    fn merge_sync(&self) -> &MergeSync;
    fn merge_sync_mut(&mut self) -> &mut MergeSync;
    fn merge_source_areas(&self) -> Option<&[Entity<TextArea>; 3]>;
    fn merge_result_area(&self) -> Option<&Entity<TextArea>>;
    fn merge_shows_base(&self) -> bool;
    fn merge_session(&self) -> Option<u64>;

    fn merge_pane_area(&self, pane: MergePane) -> Option<Entity<TextArea>> {
        match SOURCE_PANES.iter().position(|source| *source == pane) {
            Some(index) => Some(self.merge_source_areas()?[index].clone()),
            None => self.merge_result_area().cloned(),
        }
    }

    fn observe_merge_panes(
        sources: &[Entity<TextArea>; 3],
        result: &Entity<TextArea>,
        cx: &mut Context<Self>,
    ) {
        for (pane, area) in SOURCE_PANES
            .into_iter()
            .zip(sources)
            .chain([(MergePane::Result, result)])
        {
            cx.subscribe(area, move |view, _, _: &TextAreaScrolled, cx| {
                view.merge_pane_scrolled(pane, cx);
            })
            .detach();
        }
        cx.subscribe(result, |view, _, _: &TextAreaUpdated, cx| {
            view.merge_result_edited(cx);
            cx.notify();
        })
        .detach();
    }

    /// A hidden pane keeps a stale layout, so it neither leads nor follows.
    fn merge_live_pane(&self, pane: MergePane) -> Option<Entity<TextArea>> {
        let on_screen = match pane {
            MergePane::Base => self.merge_shows_base(),
            MergePane::Left | MergePane::Right => !self.merge_shows_base(),
            MergePane::Result => self.merge_sync().scroll.is_raw(),
        };
        on_screen.then(|| self.merge_pane_area(pane)).flatten()
    }

    fn merge_pane_center_line(&self, pane: MergePane, cx: &App) -> Option<f64> {
        Some(self.merge_pane_area(pane)?.read(cx).center_logical_line())
    }

    fn merge_pane_scrolled(&mut self, pane: MergePane, cx: &mut Context<Self>) {
        if pane == MergePane::Result && self.merge_sync().suppress_result {
            return;
        }
        let Some(area) = self.merge_live_pane(pane) else {
            return;
        };
        let center = area.read(cx).center_logical_line();
        let targets = self.merge_sync_mut().scroll.did_scroll(pane, center);
        self.apply_merge_targets(targets, cx);
    }

    fn apply_merge_targets(&mut self, targets: MergeScrollTargets, cx: &mut Context<Self>) -> bool {
        let mut applied = true;
        for (pane, line) in targets.positions {
            if let Some(area) = self.merge_live_pane(pane) {
                applied &= area.update(cx, |area, cx| area.scroll_to_logical_line(line, cx));
            }
        }
        if let Some(hunk) = targets.reveal_hunk
            && self.merge_sync_mut().hunk_list.reveal(hunk)
        {
            cx.notify();
        }
        applied
    }

    fn merge_result_edited(&mut self, cx: &mut Context<Self>) {
        let (Some(session), Some(result)) = (
            self.merge_session(),
            self.merge_pane_area(MergePane::Result),
        ) else {
            return;
        };
        let text = result.read(cx).text();
        if text == self.merge_sync().mapped_result {
            return;
        }
        let sync = self.merge_sync_mut();
        let pristine = text == sync.pristine_result;
        sync.scroll.invalidate_result();
        sync.generation += 1;
        let generation = sync.generation;
        let Some(map) = sync.scroll.pristine() else {
            return;
        };
        if pristine {
            self.apply_merge_map(map, text, cx);
            return;
        }
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(REBUILD_DELAY).await;
            let rebuilt = cx
                .background_spawn({
                    let text = text.clone();
                    async move { Arc::new(map.with_result(&text)) }
                })
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.merge_session() != Some(session)
                    || view.merge_sync().generation != generation
                {
                    return;
                }
                view.apply_merge_map(rebuilt, text, cx);
            });
        })
        .detach();
    }

    fn apply_merge_map(&mut self, map: Arc<MergeScrollMap>, text: String, cx: &mut Context<Self>) {
        let center = self
            .merge_pane_center_line(MergePane::Result, cx)
            .unwrap_or_default();
        let sync = self.merge_sync_mut();
        sync.mapped_result = text;
        let targets = sync.scroll.update_map(map, center);
        self.apply_merge_targets(targets, cx);
        cx.notify();
    }

    fn merge_hunk_list(
        &self,
        id: impl Into<SharedString>,
        cards: Vec<Stateful<Div>>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        merge_hunk_list_container(id, cards, &self.merge_sync().hunk_list.handle).on_scroll_wheel(
            cx.listener(|_, _, window, cx| {
                // The container's own scroll handler applies the delta during this dispatch.
                cx.defer_in(window, |view, _, cx| view.merge_hunk_list_scrolled(cx));
            }),
        )
    }

    fn merge_hunk_list_scrolled(&mut self, cx: &mut Context<Self>) {
        let sync = self.merge_sync_mut();
        let Some(hunk) = sync.hunk_list.user_scrolled() else {
            return;
        };
        if sync.scroll.visible_hunk() == Some(hunk) {
            return;
        }
        let targets = sync.scroll.did_scroll_hunk(hunk);
        self.apply_merge_targets(targets, cx);
        cx.notify();
    }

    fn merge_hunk_selected(&mut self, hunk: u32, window: &mut Window, cx: &mut Context<Self>) {
        let targets = self.merge_sync_mut().scroll.reveal(hunk);
        self.apply_merge_targets(targets, cx);
        if !self.merge_sync_mut().hunk_list.reveal(hunk) {
            // The list may have just (re)mounted; retry once it has painted, then give up.
            cx.on_next_frame(window, |view, _, cx| {
                let sync = view.merge_sync_mut();
                if sync.hunk_list.apply_pending_reveal() {
                    cx.notify();
                } else {
                    sync.hunk_list.cancel_reveal();
                }
            });
        }
        cx.notify();
    }

    fn merge_raw_changed(
        &mut self,
        raw: bool,
        selected: u32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pane = if self.merge_shows_base() {
            MergePane::Base
        } else {
            MergePane::Left
        };
        let center = self
            .merge_live_pane(pane)
            .map(|area| (pane, area.read(cx).center_logical_line()));
        self.merge_sync_mut().scroll.set_raw(raw, center);
        if raw {
            // Focusing the result pulls its caret into view; that scroll must not lead while the result is still adopting.
            self.merge_sync_mut().suppress_result = true;
            cx.on_next_frame(window, |view, window, cx| {
                view.adopt_merge_result(0, window, cx);
            });
        } else {
            self.merge_hunk_selected(selected, window, cx);
        }
    }

    fn merge_base_toggled(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.on_next_frame(window, |view, window, cx| {
            view.adopt_merge_sources(0, window, cx);
        });
    }

    fn adopt_merge_sources(&mut self, attempt: u32, window: &mut Window, cx: &mut Context<Self>) {
        let mut applied = true;
        for pane in SOURCE_PANES {
            if self.merge_live_pane(pane).is_none() {
                continue;
            }
            let targets = self.merge_sync_mut().scroll.adopt(pane);
            applied &= self.apply_merge_targets(targets, cx);
        }
        if !applied && attempt < ADOPT_ATTEMPTS {
            cx.on_next_frame(window, move |view, window, cx| {
                view.adopt_merge_sources(attempt + 1, window, cx);
            });
        }
    }

    fn adopt_merge_result(&mut self, attempt: u32, window: &mut Window, cx: &mut Context<Self>) {
        if !self.merge_sync().scroll.is_raw() {
            self.merge_sync_mut().suppress_result = false;
            return;
        }
        let targets = self.merge_sync_mut().scroll.adopt(MergePane::Result);
        if self.apply_merge_targets(targets, cx) || attempt >= ADOPT_ATTEMPTS {
            self.merge_sync_mut().suppress_result = false;
        } else {
            cx.on_next_frame(window, move |view, window, cx| {
                view.adopt_merge_result(attempt + 1, window, cx);
            });
        }
    }
}
