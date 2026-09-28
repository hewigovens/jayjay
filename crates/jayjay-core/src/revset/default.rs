pub const DEFAULT_REVSET_DEPTH: u32 = 20;
pub const DEFAULT_REVSET: &str = "present(@) | ancestors(immutable_heads().., 20) | trunk()";

const DEFAULT_REVSET_PREFIX: &str = "present(@) | ancestors(immutable_heads().., ";
const DEFAULT_REVSET_SUFFIX: &str = ") | trunk()";

pub fn build_default_revset(depth: u32) -> String {
    format!("{DEFAULT_REVSET_PREFIX}{depth}{DEFAULT_REVSET_SUFFIX}")
}

/// `None` when the user wrote their own revset; only the default shape can be paged.
pub fn default_revset_depth(revset: &str) -> Option<u32> {
    revset
        .strip_prefix(DEFAULT_REVSET_PREFIX)?
        .strip_suffix(DEFAULT_REVSET_SUFFIX)?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_revset_depth_round_trips_and_rejects_other_revsets() {
        assert_eq!(
            default_revset_depth(DEFAULT_REVSET),
            Some(DEFAULT_REVSET_DEPTH)
        );
        assert_eq!(default_revset_depth(&build_default_revset(140)), Some(140));
        assert_eq!(default_revset_depth("all()"), None);
        assert_eq!(
            default_revset_depth(&build_default_revset(20).replace("20", "x")),
            None
        );
    }
}
