use std::collections::HashSet;
use std::sync::Arc;

use crate::repo::view_model::RepoViewModel;

pub(crate) struct DagDragSelection {
    pub(crate) commit_ids: Vec<String>,
    pub(crate) targets: HashSet<String>,
}

impl DagDragSelection {
    pub(in crate::repo::window) fn of(vm: &RepoViewModel) -> Option<Arc<Self>> {
        let selected = vm.selected_change_indices();
        if selected.len() < 2 {
            return None;
        }
        let changes = &vm.graph.changes;
        Some(Arc::new(Self {
            commit_ids: selected
                .iter()
                .map(|&ix| changes[ix].commit_id.id.clone())
                .collect(),
            targets: vm
                .selection_state()
                .can_rebase_onto
                .iter()
                .zip(changes.iter())
                .filter(|(allowed, _)| **allowed)
                .map(|(_, change)| change.commit_id.id.clone())
                .collect(),
        }))
    }
}
