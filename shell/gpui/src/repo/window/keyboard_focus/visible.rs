use super::stop::FocusStop;

/// The conditional stops; everything else in the cycle is on screen whenever the repo window renders its panes.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct VisibleStops {
    pub(super) file_column: bool,
    pub(super) review_mode: bool,
    pub(super) expand_description: bool,
    pub(super) mark_reviewed: bool,
    pub(super) diff_layout: bool,
    pub(super) edit_description: bool,
    pub(super) edit_diff: bool,
    pub(super) edit_file: bool,
    pub(super) revset_back: bool,
    pub(super) revset_reset: bool,
    pub(super) commit_box: bool,
    pub(super) sidebar_hidden: bool,
}

impl VisibleStops {
    fn includes(self, stop: FocusStop) -> bool {
        if self.sidebar_hidden && stop.is_in_sidebar() {
            return false;
        }
        match stop {
            FocusStop::FileList | FocusStop::TreeToggle | FocusStop::FilterToggle => {
                self.file_column
            }
            FocusStop::ReviewMode => self.file_column && self.review_mode,
            FocusStop::ExpandDescription => self.expand_description,
            FocusStop::MarkReviewed => self.mark_reviewed,
            FocusStop::DiffLayout => self.diff_layout,
            FocusStop::EditDescription => self.edit_description,
            FocusStop::EditDiff => self.edit_diff,
            FocusStop::EditFile => self.edit_file,
            FocusStop::RevsetBack => self.revset_back,
            FocusStop::RevsetReset => self.revset_reset,
            FocusStop::CommitSummary | FocusStop::CommitDescription => self.commit_box,
            _ => true,
        }
    }

    /// The stop after `current`, wrapping at both ends; a `current` that has left the screen keeps its place in the cycle.
    pub(super) fn next(self, current: FocusStop, backward: bool) -> FocusStop {
        let rank = |stop: FocusStop| FocusStop::CYCLE.iter().position(|s| *s == stop);
        let stops = FocusStop::CYCLE
            .into_iter()
            .filter(|stop| self.includes(*stop));
        let found = if backward {
            stops.clone().rev().find(|stop| rank(*stop) < rank(current))
        } else {
            stops.clone().find(|stop| rank(*stop) > rank(current))
        };
        found
            .or_else(|| {
                if backward {
                    stops.clone().next_back()
                } else {
                    stops.clone().next()
                }
            })
            .unwrap_or(FocusStop::Dag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: VisibleStops = VisibleStops {
        file_column: true,
        review_mode: true,
        expand_description: true,
        mark_reviewed: true,
        diff_layout: true,
        edit_description: true,
        edit_diff: true,
        edit_file: true,
        revset_back: true,
        revset_reset: true,
        commit_box: true,
        sidebar_hidden: false,
    };

    fn walk(stops: VisibleStops, start: FocusStop, steps: usize, backward: bool) -> Vec<FocusStop> {
        let mut current = start;
        (0..steps)
            .map(|_| {
                current = stops.next(current, backward);
                current
            })
            .collect()
    }

    #[test]
    fn cycle_runs_in_order_and_wraps_both_ways() {
        assert_eq!(
            walk(ALL, FocusStop::Dag, FocusStop::CYCLE.len(), false),
            [&FocusStop::CYCLE[1..], &[FocusStop::Dag][..]].concat()
        );
        assert_eq!(
            walk(ALL, FocusStop::Dag, 2, true),
            [FocusStop::CommitDescription, FocusStop::CommitSummary]
        );
    }

    #[test]
    fn hidden_stops_are_skipped() {
        let stops = VisibleStops {
            diff_layout: true,
            ..VisibleStops::default()
        };
        assert_eq!(
            walk(stops, FocusStop::Push, 3, false),
            [
                FocusStop::RevsetPresets,
                FocusStop::RevsetFilter,
                FocusStop::Editor
            ]
        );
        assert_eq!(walk(stops, FocusStop::Dag, 1, true), [FocusStop::Terminal]);
    }

    #[test]
    fn a_stop_that_left_the_screen_keeps_its_place() {
        let stops = VisibleStops {
            expand_description: true,
            sidebar_hidden: true,
            ..VisibleStops::default()
        };
        assert_eq!(
            walk(stops, FocusStop::FileList, 1, false),
            [FocusStop::ExpandDescription]
        );
        assert_eq!(
            walk(stops, FocusStop::EditDiff, 1, false),
            [FocusStop::SidebarToggle]
        );
    }

    #[test]
    fn hidden_sidebar_drops_its_stops_but_keeps_the_toggle() {
        let stops = VisibleStops {
            diff_layout: true,
            commit_box: true,
            sidebar_hidden: true,
            ..VisibleStops::default()
        };
        assert_eq!(
            walk(stops, FocusStop::DiffLayout, 3, false),
            [
                FocusStop::SidebarToggle,
                FocusStop::Refresh,
                FocusStop::Pull
            ]
        );
        assert_eq!(
            walk(stops, FocusStop::FileList, 1, true),
            [FocusStop::Terminal]
        );
    }
}
