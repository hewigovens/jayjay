use std::collections::HashSet;
use std::hash::Hash;

use gpui::Modifiers;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectionClick {
    Replace,
    Toggle,
    Extend,
}

impl SelectionClick {
    pub(crate) fn from_modifiers(modifiers: &Modifiers) -> Self {
        if modifiers.secondary() {
            Self::Toggle
        } else if modifiers.shift {
            Self::Extend
        } else {
            Self::Replace
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct OrderedSelection<T> {
    selected: HashSet<T>,
    primary: Option<T>,
    anchor: Option<T>,
}

impl<T> Default for OrderedSelection<T> {
    fn default() -> Self {
        Self {
            selected: HashSet::new(),
            primary: None,
            anchor: None,
        }
    }
}

impl<T: Clone + Eq + Hash> OrderedSelection<T> {
    pub(crate) fn len(&self) -> usize {
        self.selected.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub(crate) fn contains(&self, id: &T) -> bool {
        self.selected.contains(id)
    }

    pub(crate) fn primary(&self) -> Option<&T> {
        self.primary.as_ref()
    }

    pub(crate) fn retarget(&mut self, mut map: impl FnMut(&T) -> Option<T>) {
        self.selected = self.selected.iter().filter_map(&mut map).collect();
        self.primary = self.primary.as_ref().and_then(&mut map);
        self.anchor = self.anchor.as_ref().and_then(map);
        if self
            .primary
            .as_ref()
            .is_some_and(|primary| !self.selected.contains(primary))
        {
            self.primary = None;
        }
        if self
            .anchor
            .as_ref()
            .is_some_and(|anchor| !self.selected.contains(anchor))
        {
            self.anchor.clone_from(&self.primary);
        }
    }

    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        self.selected.retain(&mut keep);
        if self
            .primary
            .as_ref()
            .is_some_and(|primary| !self.selected.contains(primary))
        {
            self.primary = None;
        }
        if self
            .anchor
            .as_ref()
            .is_some_and(|anchor| !self.selected.contains(anchor))
        {
            self.anchor.clone_from(&self.primary);
        }
    }

    pub(crate) fn ordered(&self, order: &[T]) -> Vec<T> {
        order
            .iter()
            .filter(|id| self.selected.contains(*id))
            .cloned()
            .collect()
    }

    pub(crate) fn is_contiguous_in(&self, order: &[T]) -> bool {
        let indices: Vec<_> = order
            .iter()
            .enumerate()
            .filter_map(|(ix, id)| self.selected.contains(id).then_some(ix))
            .collect();
        indices.len() == self.selected.len()
            && indices
                .first()
                .zip(indices.last())
                .is_some_and(|(first, last)| last - first + 1 == indices.len())
    }

    pub(crate) fn clear(&mut self) {
        self.selected.clear();
        self.primary = None;
        self.anchor = None;
    }

    pub(crate) fn replace(&mut self, id: T) {
        self.selected.clear();
        self.selected.insert(id.clone());
        self.primary = Some(id.clone());
        self.anchor = Some(id);
    }

    pub(crate) fn apply(&mut self, click: SelectionClick, id: T, order: &[T]) {
        match click {
            SelectionClick::Replace => self.replace(id),
            SelectionClick::Toggle => self.toggle(id, order),
            SelectionClick::Extend => self.extend(id, order),
        }
    }

    pub(crate) fn apply_pair(&mut self, click: SelectionClick, id: T, order: &[T]) {
        match click {
            SelectionClick::Replace => self.replace(id),
            SelectionClick::Toggle if self.selected.contains(&id) => self.toggle(id, order),
            SelectionClick::Toggle | SelectionClick::Extend => self.extend_pair(id),
        }
    }

    fn toggle(&mut self, id: T, order: &[T]) {
        if !self.selected.remove(&id) {
            self.selected.insert(id.clone());
            self.primary = Some(id.clone());
            self.anchor = Some(id);
            return;
        }
        if self.selected.is_empty() {
            self.primary = None;
            self.anchor = None;
            return;
        }
        if self.primary.as_ref().is_none_or(|primary| primary == &id) {
            self.primary = self.ordered(order).into_iter().next();
        }
        if self.anchor.as_ref() == Some(&id) {
            self.anchor.clone_from(&self.primary);
        }
    }

    fn extend(&mut self, id: T, order: &[T]) {
        let anchor = self
            .anchor
            .clone()
            .or_else(|| self.primary.clone())
            .unwrap_or_else(|| id.clone());
        let Some(anchor_ix) = order.iter().position(|candidate| candidate == &anchor) else {
            self.replace(id);
            return;
        };
        let Some(id_ix) = order.iter().position(|candidate| candidate == &id) else {
            self.replace(id);
            return;
        };
        let (start, end) = (anchor_ix.min(id_ix), anchor_ix.max(id_ix));
        self.selected = order[start..=end].iter().cloned().collect();
        self.primary = Some(id);
        self.anchor = Some(anchor);
    }

    fn extend_pair(&mut self, id: T) {
        let anchor = self
            .anchor
            .clone()
            .filter(|anchor| self.selected.contains(anchor))
            .or_else(|| self.primary.clone())
            .unwrap_or_else(|| id.clone());
        self.selected.clear();
        self.selected.insert(anchor.clone());
        self.selected.insert(id.clone());
        self.primary = Some(id);
        self.anchor = Some(anchor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_toggle_and_extend_preserve_order() {
        let order = vec!["a", "b", "c", "d"];
        let mut selection = OrderedSelection::default();

        selection.apply(SelectionClick::Replace, "b", &order);
        selection.apply(SelectionClick::Toggle, "d", &order);
        assert_eq!(selection.ordered(&order), vec!["b", "d"]);

        selection.apply(SelectionClick::Toggle, "d", &order);
        assert_eq!(selection.primary(), Some(&"b"));

        selection.apply(SelectionClick::Toggle, "d", &order);
        selection.apply(SelectionClick::Extend, "b", &order);
        assert_eq!(selection.ordered(&order), vec!["b", "c", "d"]);
        assert!(selection.is_contiguous_in(&order));
    }

    #[test]
    fn selection_with_a_gap_is_not_contiguous() {
        let order = vec!["a", "b", "c"];
        let mut selection = OrderedSelection::default();
        selection.replace("a");
        selection.apply(SelectionClick::Toggle, "c", &order);

        assert!(!selection.is_contiguous_in(&order));
    }

    #[test]
    fn pair_selection_keeps_the_anchor_and_at_most_two_items() {
        let order = vec!["a", "b", "c", "d"];
        let mut selection = OrderedSelection::default();

        selection.apply_pair(SelectionClick::Replace, "b", &order);
        selection.apply_pair(SelectionClick::Toggle, "d", &order);
        assert_eq!(selection.ordered(&order), vec!["b", "d"]);

        selection.apply_pair(SelectionClick::Toggle, "c", &order);
        assert_eq!(selection.ordered(&order), vec!["b", "c"]);

        selection.apply_pair(SelectionClick::Extend, "d", &order);
        assert_eq!(selection.ordered(&order), vec!["b", "d"]);

        selection.apply_pair(SelectionClick::Toggle, "b", &order);
        assert_eq!(selection.ordered(&order), vec!["d"]);
        assert_eq!(selection.primary(), Some(&"d"));
    }
}
