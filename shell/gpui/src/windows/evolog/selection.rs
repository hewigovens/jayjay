use gpui::{Context, Modifiers};
use jayjay_core::{EvologRow, evolog_rows};

use super::EvologView;
use crate::ui::selection::click_from_modifiers;

impl EvologView {
    pub(super) fn displayed_rows(&self) -> Vec<EvologRow> {
        let expanded: Vec<_> = self.expanded_runs.iter().copied().collect();
        self.entries
            .as_deref()
            .map(|entries| evolog_rows(entries, self.hide_snapshots, &expanded))
            .unwrap_or_default()
    }

    pub(super) fn set_hide_snapshots(&mut self, hide: bool, cx: &mut Context<Self>) {
        if self.hide_snapshots == hide {
            return;
        }
        let previous_endpoints = self.selected_endpoints();
        self.hide_snapshots = hide;
        if hide {
            self.expanded_runs.clear();
            let rows = self.displayed_rows();
            self.selection.retarget(|id| {
                let index = id.parse().ok()?;
                rows.iter()
                    .find(|row| row.contains(index))
                    .map(|row| row.start.to_string())
            });
        }
        if previous_endpoints != self.selected_endpoints() {
            self.load_interdiff(cx);
        }
        cx.notify();
    }

    pub fn select_version(&mut self, index: usize, modifiers: Modifiers, cx: &mut Context<Self>) {
        let rows = self.displayed_rows();
        let Some(row) = rows.iter().find(|row| row.start as usize == index).copied() else {
            return;
        };
        let order: Vec<_> = rows.iter().map(|row| row.start.to_string()).collect();
        self.selection
            .apply_pair(click_from_modifiers(&modifiers), index.to_string(), &order);
        self.comparison_reversed = false;
        if row.is_collapsed_run() {
            self.expanded_runs.insert(row.start);
        }
        self.load_interdiff(cx);
        cx.notify();
    }

    pub fn selected_version_indices(&self) -> Vec<usize> {
        let Some(entries) = self.entries.as_deref() else {
            return Vec::new();
        };
        let order: Vec<_> = (0..entries.len()).map(|index| index.to_string()).collect();
        self.selection
            .ordered(&order)
            .iter()
            .filter_map(|id| id.parse().ok())
            .collect()
    }

    pub fn selected_endpoints(&self) -> Option<(String, String)> {
        let (from, to) = self.chronological_endpoints()?;
        if self.comparison_reversed {
            Some((to, from))
        } else {
            Some((from, to))
        }
    }

    pub(super) fn can_reverse_comparison(&self) -> bool {
        self.chronological_endpoints()
            .is_some_and(|(from, to)| from != to)
    }

    pub(super) fn reverse_comparison(&mut self, cx: &mut Context<Self>) {
        if !self.can_reverse_comparison() {
            return;
        }
        self.comparison_reversed = !self.comparison_reversed;
        self.load_interdiff(cx);
        cx.notify();
    }

    fn chronological_endpoints(&self) -> Option<(String, String)> {
        let entries = self.entries.as_deref()?;
        let selected = self.selected_version_indices();
        let newest = *selected.first()?;
        let oldest = *selected.last()?;
        let from = entries.get(oldest)?.commit_id.id.clone();
        let to = if selected.len() > 1 {
            entries.get(newest)?.commit_id.id.clone()
        } else {
            entries.first()?.commit_id.id.clone()
        };
        Some((from, to))
    }
}
