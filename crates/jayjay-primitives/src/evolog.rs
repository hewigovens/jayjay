use crate::EvologEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EvologRow {
    pub start: u32,
    pub count: u32,
}

impl EvologRow {
    fn entry(index: usize) -> Self {
        Self {
            start: index as u32,
            count: 1,
        }
    }

    pub fn is_collapsed_run(self) -> bool {
        self.count > 1
    }

    pub fn contains(self, index: u32) -> bool {
        index >= self.start && index < self.start + self.count
    }
}

pub fn is_snapshot_operation(operation: &str) -> bool {
    operation.starts_with("snapshot working copy")
}

impl EvologEntry {
    pub fn is_snapshot(&self) -> bool {
        is_snapshot_operation(&self.operation)
    }
}

pub fn evolog_rows(
    entries: &[EvologEntry],
    hide_snapshots: bool,
    expanded_runs: &[u32],
) -> Vec<EvologRow> {
    if !hide_snapshots {
        return (0..entries.len()).map(EvologRow::entry).collect();
    }
    let mut rows = Vec::new();
    let mut index = 0;
    while index < entries.len() {
        if index == 0 || !entries[index].is_snapshot() {
            rows.push(EvologRow::entry(index));
            index += 1;
            continue;
        }
        let start = index;
        while index < entries.len() && entries[index].is_snapshot() {
            index += 1;
        }
        let count = index - start;
        if count == 1 || expanded_runs.contains(&(start as u32)) {
            rows.extend((start..index).map(EvologRow::entry));
        } else {
            rows.push(EvologRow {
                start: start as u32,
                count: count as u32,
            });
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShortId;

    fn entries(operations: &[&str]) -> Vec<EvologEntry> {
        operations
            .iter()
            .enumerate()
            .map(|(index, operation)| EvologEntry {
                change_id: ShortId {
                    id: "change".to_owned(),
                    short_len: 1,
                },
                commit_id: ShortId {
                    id: format!("c{index}"),
                    short_len: 1,
                },
                timestamp_millis: index as i64,
                operation: (*operation).to_owned(),
                description: String::new(),
            })
            .collect()
    }

    fn run(start: u32, count: u32) -> EvologRow {
        EvologRow { start, count }
    }

    #[test]
    fn evolog_rows_collapse_snapshot_runs_behind_the_newest_entry() {
        let snapshot = "snapshot working copy";
        let mut ops = vec!["squash commits abc"];
        ops.extend(std::iter::repeat_n(snapshot, 12));
        ops.push("describe commit def");
        let history = entries(&ops);

        assert_eq!(
            evolog_rows(&history, true, &[]),
            vec![run(0, 1), run(1, 12), run(13, 1)]
        );
        assert_eq!(evolog_rows(&history, false, &[]).len(), 14);
        assert_eq!(evolog_rows(&history, true, &[1]).len(), 14);
        assert_eq!(
            evolog_rows(&entries(&[snapshot, snapshot, snapshot]), true, &[]),
            vec![run(0, 1), run(1, 2)]
        );
        assert_eq!(
            evolog_rows(&entries(&["squash", snapshot, "describe"]), true, &[]),
            vec![run(0, 1), run(1, 1), run(2, 1)]
        );
        assert_eq!(
            evolog_rows(
                &entries(&[
                    "squash", snapshot, snapshot, "describe", snapshot, snapshot, snapshot
                ]),
                true,
                &[]
            ),
            vec![run(0, 1), run(1, 2), run(3, 1), run(4, 3)]
        );
        assert!(run(1, 12).contains(12));
        assert!(!run(1, 12).contains(13));
    }
}
