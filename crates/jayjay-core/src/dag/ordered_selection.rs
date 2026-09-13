use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionClick {
    Replace,
    Toggle,
    Extend,
}

/// Ordering questions are answered against the row order the caller passes in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrderedSelection {
    pub selected: Vec<String>,
    pub primary: Option<String>,
    pub anchor: Option<String>,
}

impl OrderedSelection {
    pub fn new(selected: Vec<String>, primary: Option<String>, anchor: Option<String>) -> Self {
        let mut seen = HashSet::with_capacity(selected.len());
        let deduped = selected
            .iter()
            .filter(|id| seen.insert(id.as_str()))
            .cloned()
            .collect();
        let mut selection = Self {
            selected: deduped,
            primary,
            anchor,
        };
        selection.settle();
        selection
    }

    pub fn len(&self) -> usize {
        self.selected.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.selected.iter().any(|candidate| candidate == id)
    }

    pub fn ordered(&self, order: &[String]) -> Vec<String> {
        let selected: HashSet<_> = self.selected.iter().collect();
        order
            .iter()
            .filter(|id| selected.contains(id))
            .cloned()
            .collect()
    }

    pub fn is_contiguous_in(&self, order: &[String]) -> bool {
        let selected: HashSet<_> = self.selected.iter().collect();
        let rows: Vec<_> = order
            .iter()
            .enumerate()
            .filter_map(|(row, id)| selected.contains(id).then_some(row))
            .collect();
        rows.len() == self.selected.len()
            && rows
                .first()
                .zip(rows.last())
                .is_some_and(|(first, last)| last - first + 1 == rows.len())
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn replace(&mut self, id: String) {
        self.selected = vec![id.clone()];
        self.primary = Some(id.clone());
        self.anchor = Some(id);
    }

    pub fn apply(&mut self, click: SelectionClick, id: String, order: &[String]) {
        match click {
            SelectionClick::Replace => self.replace(id),
            SelectionClick::Toggle => self.toggle(id, order),
            SelectionClick::Extend => self.extend(id, order),
        }
    }

    /// A pair selection compares two ends, so an extend keeps the anchor and the clicked row without the rows between them.
    pub fn apply_pair(&mut self, click: SelectionClick, id: String, order: &[String]) {
        match click {
            SelectionClick::Replace => self.replace(id),
            SelectionClick::Toggle if self.contains(&id) => self.toggle(id, order),
            SelectionClick::Toggle | SelectionClick::Extend => self.extend_pair(id),
        }
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        self.selected.retain(|id| keep(id));
        self.settle();
    }

    pub fn retarget(&mut self, mut map: impl FnMut(&str) -> Option<String>) {
        let selected = self
            .selected
            .iter()
            .filter_map(|id| map(id))
            .collect::<Vec<_>>();
        let primary = self.primary.as_deref().and_then(&mut map);
        let anchor = self.anchor.as_deref().and_then(map);
        *self = Self::new(selected, primary, anchor);
    }

    fn toggle(&mut self, id: String, order: &[String]) {
        let Some(row) = self.selected.iter().position(|candidate| *candidate == id) else {
            self.selected.push(id.clone());
            self.primary = Some(id.clone());
            self.anchor = Some(id);
            return;
        };
        self.selected.remove(row);
        if self.selected.is_empty() {
            self.primary = None;
            self.anchor = None;
            return;
        }
        if self.primary.as_deref().is_none_or(|primary| primary == id) {
            self.primary = self.ordered(order).into_iter().next();
        }
        if self.anchor.as_deref() == Some(id.as_str()) {
            self.anchor.clone_from(&self.primary);
        }
    }

    fn extend(&mut self, id: String, order: &[String]) {
        let anchor = self
            .anchor
            .clone()
            .or_else(|| self.primary.clone())
            .unwrap_or_else(|| id.clone());
        let row_of = |wanted: &str| order.iter().position(|candidate| candidate == wanted);
        let (Some(anchor_row), Some(id_row)) = (row_of(&anchor), row_of(&id)) else {
            self.replace(id);
            return;
        };
        let (start, end) = (anchor_row.min(id_row), anchor_row.max(id_row));
        self.selected = order[start..=end].to_vec();
        self.primary = Some(id);
        self.anchor = Some(anchor);
    }

    fn extend_pair(&mut self, id: String) {
        let anchor = self
            .anchor
            .clone()
            .filter(|anchor| self.contains(anchor))
            .or_else(|| self.primary.clone())
            .unwrap_or_else(|| id.clone());
        self.selected = if anchor == id {
            vec![id.clone()]
        } else {
            vec![anchor.clone(), id.clone()]
        };
        self.primary = Some(id);
        self.anchor = Some(anchor);
    }

    fn settle(&mut self) {
        if self
            .primary
            .as_deref()
            .is_some_and(|primary| !self.contains(primary))
        {
            self.primary = None;
        }
        if self
            .anchor
            .as_deref()
            .is_some_and(|anchor| !self.contains(anchor))
        {
            self.anchor.clone_from(&self.primary);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order() -> Vec<String> {
        ["a", "b", "c", "d"].map(str::to_owned).to_vec()
    }

    fn apply(selection: &mut OrderedSelection, click: SelectionClick, id: &str) {
        selection.apply(click, id.to_owned(), &order());
    }

    fn apply_pair(selection: &mut OrderedSelection, click: SelectionClick, id: &str) {
        selection.apply_pair(click, id.to_owned(), &order());
    }

    #[test]
    fn replace_toggle_and_extend_preserve_the_row_order() {
        let mut selection = OrderedSelection::default();

        apply(&mut selection, SelectionClick::Replace, "b");
        apply(&mut selection, SelectionClick::Toggle, "b");
        assert!(selection.selected.is_empty());
        assert_eq!(selection.primary, None);

        apply(&mut selection, SelectionClick::Replace, "b");
        apply(&mut selection, SelectionClick::Toggle, "d");
        assert_eq!(selection.ordered(&order()), ["b", "d"]);

        apply(&mut selection, SelectionClick::Toggle, "d");
        assert_eq!(selection.primary.as_deref(), Some("b"));

        apply(&mut selection, SelectionClick::Toggle, "d");
        apply(&mut selection, SelectionClick::Extend, "b");
        assert_eq!(selection.ordered(&order()), ["b", "c", "d"]);
        assert_eq!(selection.primary.as_deref(), Some("b"));
        assert_eq!(selection.anchor.as_deref(), Some("d"));
    }

    #[test]
    fn only_selections_without_gaps_are_contiguous() {
        let mut selection = OrderedSelection::new(vec!["a".to_owned()], Some("a".to_owned()), None);

        apply(&mut selection, SelectionClick::Toggle, "c");
        assert!(!selection.is_contiguous_in(&order()));

        apply(&mut selection, SelectionClick::Toggle, "b");
        assert!(selection.is_contiguous_in(&order()));
    }

    #[test]
    fn a_large_selection_dedupes_orders_and_toggles_by_row() {
        let order: Vec<_> = (0..12_000).map(|id| format!("{id:032x}")).collect();
        let selected: Vec<_> = order.iter().rev().chain(order.iter()).cloned().collect();
        let mut selection = OrderedSelection::new(selected, order.first().cloned(), None);

        assert_eq!(selection.len(), order.len());
        assert_eq!(selection.ordered(&order), order);
        assert!(selection.is_contiguous_in(&order));
        selection.apply(SelectionClick::Toggle, order[6000].clone(), &order);
        assert!(!selection.is_contiguous_in(&order));
        selection.apply(SelectionClick::Toggle, order[6000].clone(), &order);
        assert_eq!(selection.ordered(&order), order);
        assert!(selection.is_contiguous_in(&order));
    }

    #[test]
    fn extending_from_an_anchor_off_the_row_order_selects_only_the_clicked_row() {
        let mut selection =
            OrderedSelection::new(vec!["gone".to_owned()], Some("gone".to_owned()), None);

        apply(&mut selection, SelectionClick::Extend, "c");

        assert_eq!(selection.selected, ["c"]);
        assert_eq!(selection.anchor.as_deref(), Some("c"));
    }

    #[test]
    fn pair_selection_keeps_the_anchor_and_at_most_two_rows() {
        let mut selection = OrderedSelection::default();

        apply_pair(&mut selection, SelectionClick::Replace, "b");
        apply_pair(&mut selection, SelectionClick::Toggle, "d");
        assert_eq!(selection.ordered(&order()), ["b", "d"]);

        apply_pair(&mut selection, SelectionClick::Toggle, "c");
        assert_eq!(selection.ordered(&order()), ["b", "c"]);

        apply_pair(&mut selection, SelectionClick::Extend, "d");
        assert_eq!(selection.ordered(&order()), ["b", "d"]);

        apply_pair(&mut selection, SelectionClick::Toggle, "b");
        assert_eq!(selection.ordered(&order()), ["d"]);
        assert_eq!(selection.primary.as_deref(), Some("d"));
    }

    #[test]
    fn retargeting_collapses_rows_that_merge_and_drops_rows_that_vanish() {
        let mut selection = OrderedSelection::new(
            ["a", "b", "c"].map(str::to_owned).to_vec(),
            Some("c".to_owned()),
            Some("a".to_owned()),
        );

        selection.retarget(|id| match id {
            "a" | "b" => Some("a".to_owned()),
            _ => None,
        });

        assert_eq!(selection.selected, ["a"]);
        assert_eq!(selection.primary, None);
        assert_eq!(selection.anchor.as_deref(), Some("a"));
    }

    #[test]
    fn retaining_drops_the_primary_and_anchor_with_their_rows() {
        let mut selection = OrderedSelection::new(
            ["a", "b"].map(str::to_owned).to_vec(),
            Some("b".to_owned()),
            Some("b".to_owned()),
        );

        selection.retain(|id| id == "a");

        assert_eq!(selection.selected, ["a"]);
        assert_eq!(selection.primary, None);
        assert_eq!(selection.anchor, None);
    }
}
