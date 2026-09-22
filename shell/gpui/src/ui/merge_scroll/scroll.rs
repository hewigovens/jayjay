use std::cmp::Ordering;
use std::sync::Arc;

use jayjay_core::{MergePane, MergeScrollMap};

pub(crate) const SOURCE_PANES: [MergePane; 3] =
    [MergePane::Left, MergePane::Base, MergePane::Right];
const PANES: [MergePane; 4] = [
    MergePane::Left,
    MergePane::Base,
    MergePane::Right,
    MergePane::Result,
];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Position {
    Text(MergePane, f64),
    Hunk(u32),
}

/// Mirrors SwiftUI's `MergeScrollCoordinator`: the pane scrolled last leads, followers center the mapped line.
#[derive(Default)]
pub(crate) struct MergeScroll {
    is_raw: bool,
    pristine: Option<Arc<MergeScrollMap>>,
    map: Option<Arc<MergeScrollMap>>,
    hunk_count: u32,
    last_scroll: Option<Position>,
    visible_hunk: Option<u32>,
    result_is_current: bool,
    result_needs_restore: bool,
}

#[derive(Default)]
#[must_use]
pub(crate) struct MergeScrollTargets {
    pub(crate) positions: Vec<(MergePane, f64)>,
    pub(crate) reveal_hunk: Option<u32>,
}

impl MergeScroll {
    pub(crate) fn load(&mut self, map: MergeScrollMap, hunk_count: u32, is_raw: bool) {
        let map = Arc::new(map);
        *self = Self {
            is_raw,
            pristine: Some(map.clone()),
            map: Some(map),
            hunk_count,
            result_is_current: true,
            ..Default::default()
        };
    }

    pub(crate) fn is_raw(&self) -> bool {
        self.is_raw
    }

    pub(crate) fn visible_hunk(&self) -> Option<u32> {
        self.visible_hunk
    }

    pub(crate) fn pristine(&self) -> Option<Arc<MergeScrollMap>> {
        self.pristine.clone()
    }

    pub(crate) fn invalidate_result(&mut self) {
        self.result_is_current = false;
    }

    pub(crate) fn set_raw(&mut self, raw: bool, source_center: Option<(MergePane, f64)>) {
        if raw == self.is_raw {
            return;
        }
        self.is_raw = raw;
        if let Some((pane, line)) = source_center {
            self.last_scroll = Some(Position::Text(pane, line));
        }
        if !raw {
            self.result_needs_restore = false;
        }
    }

    pub(crate) fn did_scroll(&mut self, pane: MergePane, center: f64) -> MergeScrollTargets {
        if self.map.is_none() {
            return MergeScrollTargets::default();
        }
        if pane == MergePane::Result {
            if !self.is_raw {
                return MergeScrollTargets::default();
            }
            self.result_needs_restore = false;
            if !self.result_is_current {
                return MergeScrollTargets::default();
            }
        } else if !self.result_is_current {
            self.result_needs_restore = true;
        }
        self.last_scroll = Some(Position::Text(pane, center));
        self.synchronize()
    }

    pub(crate) fn did_scroll_hunk(&mut self, hunk: u32) -> MergeScrollTargets {
        if self.is_raw {
            return MergeScrollTargets::default();
        }
        self.visible_hunk = Some(hunk);
        self.last_scroll = Some(Position::Hunk(hunk));
        self.synchronize()
    }

    pub(crate) fn reveal(&mut self, hunk: u32) -> MergeScrollTargets {
        if self.is_raw {
            return MergeScrollTargets::default();
        }
        self.last_scroll = Some(Position::Hunk(hunk));
        self.apply(Position::Hunk(hunk), None)
    }

    pub(crate) fn adopt(&mut self, pane: MergePane) -> MergeScrollTargets {
        if pane == MergePane::Result {
            self.result_needs_restore = true;
        }
        let Some(last_scroll) = self.last_scroll else {
            return MergeScrollTargets::default();
        };
        self.apply(last_scroll, Some(pane))
    }

    pub(crate) fn update_map(
        &mut self,
        map: Arc<MergeScrollMap>,
        result_center: f64,
    ) -> MergeScrollTargets {
        let reconcile = self.map.is_some() && !self.result_is_current && self.is_raw;
        self.map = Some(map);
        self.result_is_current = true;
        if self.is_raw && self.result_needs_restore {
            self.synchronize()
        } else if reconcile {
            self.did_scroll(MergePane::Result, result_center)
        } else {
            MergeScrollTargets::default()
        }
    }

    fn synchronize(&mut self) -> MergeScrollTargets {
        let Some(last_scroll) = self.last_scroll else {
            return MergeScrollTargets::default();
        };
        self.apply(last_scroll, None)
    }

    fn apply(&mut self, position: Position, only: Option<MergePane>) -> MergeScrollTargets {
        let Some(map) = self.map.as_ref() else {
            return MergeScrollTargets::default();
        };
        let result_leads = self.is_raw && self.result_is_current;
        let mut positions = Vec::new();
        let mut nearest = None;
        match position {
            Position::Text(leader, line) => {
                if leader == MergePane::Result && !result_leads {
                    return MergeScrollTargets::default();
                }
                for follower in PANES {
                    let included = only.map_or(follower != leader, |pane| follower == pane);
                    if !included || (follower == MergePane::Result && !result_leads) {
                        continue;
                    }
                    positions.push((follower, map.map_line(leader, follower, line)));
                }
                if !self.is_raw && only.is_none() {
                    nearest = (0..self.hunk_count).min_by(|a, b| {
                        let distance = |hunk: &u32| {
                            (map.hunk_line(leader, *hunk).unwrap_or(f64::INFINITY) - line).abs()
                        };
                        distance(a)
                            .partial_cmp(&distance(b))
                            .unwrap_or(Ordering::Equal)
                    });
                }
            }
            Position::Hunk(hunk) => {
                for pane in SOURCE_PANES {
                    if only.is_some_and(|only| only != pane) {
                        continue;
                    }
                    if let Some(line) = map.hunk_line(pane, hunk) {
                        positions.push((pane, line));
                    }
                }
            }
        }
        if positions.iter().any(|(pane, _)| *pane == MergePane::Result) {
            self.result_needs_restore = false;
        }
        let mut reveal_hunk = None;
        match position {
            Position::Hunk(hunk) if only.is_none() => self.visible_hunk = Some(hunk),
            Position::Text(..) if nearest.is_some() && nearest != self.visible_hunk => {
                self.visible_hunk = nearest;
                reveal_hunk = nearest;
            }
            _ => {}
        }
        MergeScrollTargets {
            positions,
            reveal_hunk,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform_map(lines: usize) -> MergeScrollMap {
        let text = (0..lines).fold(String::new(), |text, line| text + &format!("line {line}\n"));
        MergeScrollMap::new(&text, &text, &text, &text, &[])
    }

    fn hunk_map() -> MergeScrollMap {
        let context = (0..50).fold(String::new(), |text, line| {
            text + &format!("context {line}\n")
        });
        let raw = "<<<<<<<\nleft\n=======\nright\n>>>>>>>\n";
        let original = format!("{context}{raw}{context}{raw}{context}");
        let source = format!("{context}source\n{context}source\n{context}");
        let hunks = (0..2)
            .map(|index| jayjay_core::MergeEditorHunk {
                index,
                occurrence: index,
                raw: raw.to_owned(),
                left: "source\n".to_owned(),
                base: "source\n".to_owned(),
                right: "source\n".to_owned(),
            })
            .collect::<Vec<_>>();
        MergeScrollMap::new(&source, &source, &source, &original, &hunks)
    }

    #[test]
    fn leader_scroll_centers_mapped_lines_on_followers() {
        let mut scroll = MergeScroll::default();
        scroll.load(uniform_map(100), 0, true);
        let targets = scroll.did_scroll(MergePane::Left, 20.4);
        assert_eq!(
            targets.positions,
            vec![
                (MergePane::Base, 20.4),
                (MergePane::Right, 20.4),
                (MergePane::Result, 20.4),
            ]
        );
    }

    #[test]
    fn result_participates_only_in_raw_mode() {
        let mut scroll = MergeScroll::default();
        scroll.load(uniform_map(100), 0, false);
        let targets = scroll.did_scroll(MergePane::Left, 55.);
        assert_eq!(
            targets.positions,
            vec![(MergePane::Base, 55.), (MergePane::Right, 55.)]
        );
        assert!(
            scroll
                .did_scroll(MergePane::Result, 10.)
                .positions
                .is_empty(),
            "the raw-hidden result may not lead"
        );
    }

    #[test]
    fn reveal_scrolls_sources_to_the_hunk_and_tracks_visible_hunk() {
        let mut scroll = MergeScroll::default();
        scroll.load(hunk_map(), 2, false);
        let targets = scroll.reveal(1);
        assert_eq!(
            targets.positions,
            vec![
                (MergePane::Left, 101.),
                (MergePane::Base, 101.),
                (MergePane::Right, 101.),
            ]
        );
        assert_eq!(scroll.visible_hunk(), Some(1));
        scroll.set_raw(true, Some((MergePane::Left, 30.)));
        assert!(scroll.reveal(0).positions.is_empty());
    }

    #[test]
    fn source_scroll_in_hunks_mode_updates_the_visible_hunk() {
        let mut scroll = MergeScroll::default();
        scroll.load(hunk_map(), 2, false);
        assert_eq!(
            scroll.did_scroll(MergePane::Right, 110.).reveal_hunk,
            Some(1)
        );
        assert_eq!(
            scroll.did_scroll(MergePane::Right, 10.).reveal_hunk,
            Some(0)
        );
        assert_eq!(
            scroll.did_scroll(MergePane::Right, 12.).reveal_hunk,
            None,
            "staying on the same hunk must not fight the list's own scrolling"
        );
    }

    #[test]
    fn newly_mounted_pane_adopts_the_last_position() {
        let mut scroll = MergeScroll::default();
        scroll.load(uniform_map(100), 0, false);
        let _ = scroll.did_scroll(MergePane::Left, 42.);
        assert_eq!(
            scroll.adopt(MergePane::Base).positions,
            vec![(MergePane::Base, 42.)]
        );
        let mut scroll = MergeScroll::default();
        scroll.load(hunk_map(), 2, false);
        let _ = scroll.reveal(1);
        assert_eq!(
            scroll.adopt(MergePane::Base).positions,
            vec![(MergePane::Base, 101.)]
        );
    }

    #[test]
    fn edited_result_stays_put_until_the_map_rebuild_restores_it() {
        let source = (0..120).fold(String::new(), |text, line| text + &format!("line {line}\n"));
        let edited = format!("{}{}", "inserted\n".repeat(10), source);
        let mut scroll = MergeScroll::default();
        scroll.load(uniform_map(120), 0, true);
        let _ = scroll.did_scroll(MergePane::Result, 30.);
        scroll.invalidate_result();
        let targets = scroll.did_scroll(MergePane::Left, 60.);
        assert_eq!(
            targets.positions,
            vec![(MergePane::Base, 60.), (MergePane::Right, 60.)]
        );
        let rebuilt = Arc::new(scroll.pristine().unwrap().with_result(&edited));
        let targets = scroll.update_map(rebuilt, 0.);
        assert_eq!(
            targets.positions,
            vec![
                (MergePane::Base, 60.),
                (MergePane::Right, 60.),
                (MergePane::Result, 70.),
            ]
        );
    }

    #[test]
    fn result_leads_after_rebuild_when_it_scrolled_last() {
        let source = (0..120).fold(String::new(), |text, line| text + &format!("line {line}\n"));
        let edited = format!("{}{}", "inserted\n".repeat(10), source);
        let mut scroll = MergeScroll::default();
        scroll.load(uniform_map(120), 0, true);
        let _ = scroll.did_scroll(MergePane::Result, 30.);
        scroll.invalidate_result();
        let _ = scroll.did_scroll(MergePane::Left, 60.);
        assert!(
            scroll
                .did_scroll(MergePane::Result, 90.)
                .positions
                .is_empty()
        );
        let rebuilt = Arc::new(scroll.pristine().unwrap().with_result(&edited));
        let targets = scroll.update_map(rebuilt, 90.);
        assert_eq!(
            targets.positions,
            vec![
                (MergePane::Left, 80.),
                (MergePane::Base, 80.),
                (MergePane::Right, 80.),
            ]
        );
    }

    #[test]
    fn mode_switch_rebases_from_a_source_center() {
        let mut scroll = MergeScroll::default();
        scroll.load(hunk_map(), 2, false);
        let _ = scroll.reveal(1);
        scroll.set_raw(true, Some((MergePane::Left, 101.)));
        let targets = scroll.did_scroll(MergePane::Result, 101.);
        assert_eq!(
            targets.positions,
            vec![
                (MergePane::Left, 97.),
                (MergePane::Base, 97.),
                (MergePane::Right, 97.),
            ]
        );
        scroll.set_raw(false, Some((MergePane::Left, 50.)));
        assert_eq!(
            scroll.reveal(1).positions,
            vec![
                (MergePane::Left, 101.),
                (MergePane::Base, 101.),
                (MergePane::Right, 101.),
            ]
        );
    }
}
