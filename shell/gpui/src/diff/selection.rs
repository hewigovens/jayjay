/// Line-granularity selection over a unified diff body. Anchor + focus are
/// indices into the rendered `DiffLine` list; `dragging` is true while the
/// user is mid-drag.
#[derive(Debug, Clone, Copy)]
pub struct DiffSelection {
    pub anchor: usize,
    pub focus: usize,
    pub dragging: bool,
}

impl DiffSelection {
    pub fn start(line: usize) -> Self {
        Self {
            anchor: line,
            focus: line,
            dragging: true,
        }
    }

    pub fn extend(&mut self, line: usize) {
        self.focus = line;
    }

    /// Inclusive line range, ordered low → high.
    pub fn range(&self) -> std::ops::RangeInclusive<usize> {
        let lo = self.anchor.min(self.focus);
        let hi = self.anchor.max(self.focus);
        lo..=hi
    }

    pub fn covers(&self, line_ix: usize) -> bool {
        self.range().contains(&line_ix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_orders_endpoints_low_to_high() {
        let mut sel = DiffSelection::start(5);
        sel.extend(2);
        assert_eq!(*sel.range().start(), 2);
        assert_eq!(*sel.range().end(), 5);
    }

    #[test]
    fn covers_is_inclusive_on_both_ends() {
        let mut sel = DiffSelection::start(2);
        sel.extend(5);
        assert!(sel.covers(2));
        assert!(sel.covers(5));
        assert!(sel.covers(3));
        assert!(!sel.covers(1));
        assert!(!sel.covers(6));
    }

    #[test]
    fn single_line_selection_covers_only_that_line() {
        let sel = DiffSelection::start(7);
        assert!(sel.covers(7));
        assert!(!sel.covers(6));
        assert!(!sel.covers(8));
    }
}
