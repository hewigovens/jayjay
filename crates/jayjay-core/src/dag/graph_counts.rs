use std::collections::HashSet;

use crate::ChangeInfo;

/// Changes in the loaded graph that need attention; divergence counts change ids, since each divergent id has several rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphCounts {
    pub divergent: u32,
    pub conflicted: u32,
}

impl GraphCounts {
    pub fn new<'a>(changes: impl IntoIterator<Item = &'a ChangeInfo>) -> Self {
        let mut divergent = HashSet::new();
        let mut conflicted = 0;
        for change in changes {
            if change.is_divergent {
                divergent.insert(change.change_id.id.as_str());
            }
            if change.has_conflict {
                conflicted += 1;
            }
        }
        Self {
            divergent: divergent.len() as u32,
            conflicted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::change_info;

    fn change(change_id: &str, divergent: bool, conflict: bool) -> ChangeInfo {
        let mut change = change_info(change_id, "commit");
        change.is_divergent = divergent;
        change.has_conflict = conflict;
        change
    }

    #[test]
    fn divergent_rows_of_one_change_count_once() {
        let changes = [
            change("aaaa", true, false),
            change("aaaa", true, true),
            change("bbbb", false, true),
            change("cccc", false, false),
        ];
        assert_eq!(
            GraphCounts::new(&changes),
            GraphCounts {
                divergent: 1,
                conflicted: 2
            }
        );
    }
}
