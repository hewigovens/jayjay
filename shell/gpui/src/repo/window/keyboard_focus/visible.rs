use super::stop::FocusStop;

/// The conditional stops; everything else in the cycle is on screen whenever the repo window renders its panes.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct VisibleStops {
    pub(super) expand_description: bool,
    pub(super) diff_layout: bool,
    pub(super) edit_description: bool,
    pub(super) edit_diff: bool,
    pub(super) revset_input: bool,
    pub(super) commit_box: bool,
}

impl VisibleStops {
    fn includes(self, stop: FocusStop) -> bool {
        match stop {
            FocusStop::ExpandDescription => self.expand_description,
            FocusStop::DiffLayout => self.diff_layout,
            FocusStop::EditDescription => self.edit_description,
            FocusStop::EditDiff => self.edit_diff,
            FocusStop::RevsetInput => self.revset_input,
            FocusStop::CommitSummary | FocusStop::CommitDescription => self.commit_box,
            _ => true,
        }
    }

    /// The stop after `current`, wrapping at both ends; a `current` that has left the screen restarts the cycle.
    pub(super) fn next(self, current: FocusStop, backward: bool) -> FocusStop {
        let stops: Vec<FocusStop> = FocusStop::CYCLE
            .into_iter()
            .filter(|stop| self.includes(*stop))
            .collect();
        let index = stops.iter().position(|stop| *stop == current).unwrap_or(0);
        let step = if backward { stops.len() - 1 } else { 1 };
        stops[(index + step) % stops.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: VisibleStops = VisibleStops {
        expand_description: true,
        diff_layout: true,
        edit_description: true,
        edit_diff: true,
        revset_input: true,
        commit_box: true,
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
            walk(stops, FocusStop::FilterToggle, 3, false),
            [
                FocusStop::DiffLayout,
                FocusStop::RevsetFilter,
                FocusStop::Refresh
            ]
        );
        assert_eq!(walk(stops, FocusStop::Dag, 1, true), [FocusStop::Settings]);
    }

    #[test]
    fn a_stop_that_left_the_screen_restarts_the_cycle() {
        assert_eq!(
            walk(VisibleStops::default(), FocusStop::EditDiff, 1, false),
            [FocusStop::FileList]
        );
    }
}
