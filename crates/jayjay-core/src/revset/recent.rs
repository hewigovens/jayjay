use super::filter::{RevsetFilter, RevsetFilterKind};

pub(super) const RECENT_REVSET_LIMIT: usize = 15;

pub(super) fn remember_revset(recent: &[String], revset: &str) -> Vec<String> {
    let revset = revset.trim();
    if revset.is_empty() || RevsetFilter::of(revset).kind != RevsetFilterKind::Custom {
        return recent.to_vec();
    }
    std::iter::once(revset.to_owned())
        .chain(recent.iter().filter(|entry| *entry != revset).cloned())
        .take(RECENT_REVSET_LIMIT)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revset::DEFAULT_REVSET;

    #[test]
    fn recent_revsets_keep_typed_ones_newest_first_without_duplicates() {
        let mut recent = Vec::new();
        for revset in ["a()", "mine()", DEFAULT_REVSET, " b() ", "a()", ""] {
            recent = remember_revset(&recent, revset);
        }
        assert_eq!(recent, ["a()", "b()"]);

        for n in 0..RECENT_REVSET_LIMIT {
            recent = remember_revset(&recent, &format!("r{n}()"));
        }
        assert_eq!(recent.len(), RECENT_REVSET_LIMIT);
        assert_eq!(recent[0], format!("r{}()", RECENT_REVSET_LIMIT - 1));
    }
}
