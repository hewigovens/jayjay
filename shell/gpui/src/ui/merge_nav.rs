use jayjay_core::MergeEditorHunk;

pub(crate) fn next_unresolved_hunk_index(
    hunks: &[MergeEditorHunk],
    result: &str,
    selected: usize,
    delta: isize,
) -> Option<usize> {
    let unresolved = hunks
        .iter()
        .enumerate()
        .filter_map(|(index, hunk)| {
            jayjay_core::merge_hunk_is_unresolved(result, hunk).then_some(index)
        })
        .collect::<Vec<_>>();
    // Search relative to the selected index itself: after a resolution the selected hunk is no longer in `unresolved`, and a position-based +1 would skip its immediate neighbor.
    if delta >= 0 {
        unresolved
            .iter()
            .find(|index| **index > selected)
            .or_else(|| unresolved.first())
            .copied()
    } else {
        unresolved
            .iter()
            .rev()
            .find(|index| **index < selected)
            .or_else(|| unresolved.last())
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunk(index: u32, marker: &str) -> MergeEditorHunk {
        MergeEditorHunk {
            index,
            occurrence: 0,
            raw: format!("<<<<<<< {marker}\nx\n>>>>>>> {marker}\n"),
            left: "left\n".to_owned(),
            base: "base\n".to_owned(),
            right: "right\n".to_owned(),
        }
    }

    #[test]
    fn advances_to_the_immediate_neighbor_after_a_resolution() {
        let hunks = [hunk(0, "a"), hunk(1, "b"), hunk(2, "c")];
        let result = format!("{}{}", hunks[1].raw, hunks[2].raw);
        assert_eq!(next_unresolved_hunk_index(&hunks, &result, 0, 1), Some(1));
        assert_eq!(next_unresolved_hunk_index(&hunks, &result, 1, 1), Some(2));
        assert_eq!(next_unresolved_hunk_index(&hunks, &result, 2, 1), Some(1));
        assert_eq!(next_unresolved_hunk_index(&hunks, &result, 1, -1), Some(2));
        assert_eq!(next_unresolved_hunk_index(&hunks, &result, 2, -1), Some(1));
    }

    #[test]
    fn returns_none_when_everything_is_resolved() {
        let hunks = [hunk(0, "a"), hunk(1, "b")];
        assert_eq!(
            next_unresolved_hunk_index(&hunks, "no markers left", 0, 1),
            None
        );
    }
}
