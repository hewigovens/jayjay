use crate::{EvologEntry, EvologOperationKind, EvologVisibleRow};

pub fn evolog_operation_kind(raw: &str) -> EvologOperationKind {
    if raw.starts_with("snapshot working copy") {
        EvologOperationKind::Snapshot
    } else if raw.starts_with("describe commit ") {
        EvologOperationKind::Describe
    } else if raw.starts_with("rebase commit ") {
        EvologOperationKind::Rebase
    } else if raw.starts_with("squash commits ") {
        EvologOperationKind::Squash
    } else if raw.starts_with("split commit ") {
        EvologOperationKind::Split
    } else if raw.starts_with("new empty commit") {
        EvologOperationKind::New
    } else if raw.is_empty() {
        EvologOperationKind::Rewrite
    } else {
        EvologOperationKind::Other
    }
}

pub fn evolog_is_snapshot(raw: &str) -> bool {
    evolog_operation_kind(raw) == EvologOperationKind::Snapshot
}

pub fn evolog_visible_rows(
    entries: &[EvologEntry],
    hide_snapshots: bool,
    collapse_snapshot_runs: bool,
) -> Vec<EvologVisibleRow> {
    let mut rows = Vec::new();
    let mut index = 0;
    while index < entries.len() {
        let entry = &entries[index];
        if hide_snapshots && evolog_is_snapshot(&entry.operation) {
            index += 1;
            continue;
        }

        if collapse_snapshot_runs && evolog_is_snapshot(&entry.operation) {
            let start = index;
            let mut group_entries = Vec::new();
            let mut group_indices = Vec::new();
            while index < entries.len() && evolog_is_snapshot(&entries[index].operation) {
                group_entries.push(entries[index].clone());
                group_indices.push(index as u32);
                index += 1;
            }
            rows.push(EvologVisibleRow {
                id: format!("snapshots-{start}-{}", group_entries.len()),
                primary_index: start as u32,
                indices: group_indices,
                is_snapshot_run: group_entries.len() > 1,
                entries: group_entries,
            });
        } else {
            rows.push(EvologVisibleRow {
                id: format!("entry-{index}"),
                primary_index: index as u32,
                indices: vec![index as u32],
                entries: vec![entry.clone()],
                is_snapshot_run: false,
            });
            index += 1;
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_operation_prefixes() {
        assert_eq!(
            evolog_operation_kind("snapshot working copy 123"),
            EvologOperationKind::Snapshot
        );
        assert_eq!(evolog_operation_kind(""), EvologOperationKind::Rewrite);
        assert_eq!(
            evolog_operation_kind("rebase commit abc"),
            EvologOperationKind::Rebase
        );
    }

    #[test]
    fn collapses_snapshot_runs() {
        let entries = vec![
            entry("a", "snapshot working copy"),
            entry("b", "snapshot working copy"),
            entry("c", "describe commit x"),
            entry("d", "snapshot working copy"),
        ];

        let rows = evolog_visible_rows(&entries, false, true);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].indices, vec![0, 1]);
        assert!(rows[0].is_snapshot_run);
        assert_eq!(rows[1].primary_index, 2);
        assert_eq!(rows[2].indices, vec![3]);
        assert!(!rows[2].is_snapshot_run);
    }

    #[test]
    fn hides_snapshots_without_collapsing() {
        let entries = vec![
            entry("a", "snapshot working copy"),
            entry("b", "describe commit x"),
            entry("c", "snapshot working copy"),
        ];

        let rows = evolog_visible_rows(&entries, true, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].primary_index, 1);
        assert_eq!(rows[0].entries[0].commit_id, "b");
    }

    #[test]
    fn hide_snapshots_takes_precedence_over_collapse() {
        let entries = vec![
            entry("a", "snapshot working copy"),
            entry("b", "snapshot working copy"),
            entry("c", "describe commit x"),
        ];

        let rows = evolog_visible_rows(&entries, true, true);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].primary_index, 2);
        assert!(!rows[0].is_snapshot_run);
    }

    fn entry(commit_id: &str, operation: &str) -> EvologEntry {
        EvologEntry {
            change_id: "change".to_owned(),
            commit_id: commit_id.to_owned(),
            timestamp_millis: 0,
            operation: operation.to_owned(),
            description: String::new(),
        }
    }
}
