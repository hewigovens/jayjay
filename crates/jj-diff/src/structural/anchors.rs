use std::cmp::Reverse;
use std::collections::HashMap;
use std::ops::Range;

use super::tree::SyntaxTree;

type Anchor<'t> = (&'t Range<usize>, &'t Range<usize>);

/// Unique unchanged subtrees outside the chain that agrees with the displayed rows: code that changed position.
pub(super) fn moved_subtrees(
    old: &SyntaxTree<'_>,
    new: &SyntaxTree<'_>,
    aligned: &dyn Fn(usize, usize) -> bool,
) -> Vec<(Range<usize>, Range<usize>)> {
    let mut occurrences: HashMap<u64, [(u32, usize); 2]> = HashMap::new();
    for (side, tree) in [old, new].into_iter().enumerate() {
        for (index, subtree) in tree.subtrees.iter().enumerate() {
            let occurrence = &mut occurrences.entry(subtree.hash).or_default()[side];
            *occurrence = (occurrence.0 + 1, index);
        }
    }
    let mut unique: Vec<Anchor<'_>> = occurrences
        .into_values()
        .filter(|[o, n]| o.0 == 1 && n.0 == 1)
        .map(|[o, n]| (&old.subtrees[o.1].atoms, &new.subtrees[n.1].atoms))
        .collect();
    unique.sort_unstable_by_key(|(o, _)| (Reverse(o.len()), o.start));
    let mut old_covered = vec![false; old.atoms.len()];
    let mut new_covered = vec![false; new.atoms.len()];
    unique.retain(|(o, n)| {
        let outermost = !old_covered[o.start]
            && !new_covered[n.start]
            && old.keys((*o).clone()).eq(new.keys((*n).clone()));
        if outermost {
            old_covered[(*o).clone()].fill(true);
            new_covered[(*n).clone()].fill(true);
        }
        outermost
    });
    unique.sort_unstable_by_key(|(o, _)| o.start);
    // Agreeing with the displayed rows dominates; size breaks ties.
    let weights: Vec<_> = unique
        .iter()
        .map(|(o, n)| {
            let agreeing = (*o)
                .clone()
                .zip((*n).clone())
                .filter(|&(o, n)| aligned(o, n))
                .count();
            agreeing * 65_536 + o.len()
        })
        .collect();
    let in_chain = heaviest_chain(&unique, &weights, new.atoms.len());
    unique
        .into_iter()
        .zip(in_chain)
        .filter(|(_, in_chain)| !in_chain)
        .map(|((o, n), _)| (o.clone(), n.clone()))
        .collect()
}

/// Weighted longest increasing subsequence on new positions, using a prefix-max Fenwick tree.
fn heaviest_chain(anchors: &[Anchor<'_>], weights: &[usize], new_len: usize) -> Vec<bool> {
    let mut fenwick = vec![(0, usize::MAX); new_len + 1];
    let mut previous = vec![usize::MAX; anchors.len()];
    let mut best = (0, usize::MAX);
    for (index, (_, n)) in anchors.iter().enumerate() {
        let mut prior = (0, usize::MAX);
        let mut k = n.start;
        while k > 0 {
            prior = prior.max(fenwick[k]);
            k &= k - 1;
        }
        previous[index] = prior.1;
        let chain = (prior.0 + weights[index], index);
        best = best.max(chain);
        let mut k = n.start + 1;
        while k <= new_len {
            fenwick[k] = fenwick[k].max(chain);
            k += k.isolate_lowest_one();
        }
    }
    let mut in_chain = vec![false; anchors.len()];
    let mut index = best.1;
    while index != usize::MAX {
        in_chain[index] = true;
        index = previous[index];
    }
    in_chain
}
