//! Which bookmark names are trunk, and the menu rules that depend on it.

const TRUNK_BOOKMARKS: &[&str] = &["main", "master", "trunk"];

/// Matches bare "main" as well as remote-qualified forms like "main@origin".
pub fn is_trunk_bookmark(name: &str) -> bool {
    let bare = name.split('@').next().unwrap_or(name);
    TRUNK_BOOKMARKS.contains(&bare)
}

/// DAG chips may drop a conflicted target even on trunk; whole-bookmark delete stays hidden for resolved trunk names.
pub fn can_remove_bookmark_from_chip(name: &str, conflicted: bool) -> bool {
    conflicted || !is_trunk_bookmark(name)
}

/// Deleting a whole bookmark is offered for neither trunk nor a conflicted bookmark, whose targets are dropped per change instead.
pub fn can_delete_bookmark(name: &str, conflicted: bool) -> bool {
    !conflicted && !is_trunk_bookmark(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_can_remove_a_conflicted_trunk_target() {
        assert!(can_remove_bookmark_from_chip("main", true));
        assert!(can_remove_bookmark_from_chip("main@origin", true));
        assert!(can_remove_bookmark_from_chip("feature", false));
        assert!(!can_remove_bookmark_from_chip("main", false));
        assert!(!can_remove_bookmark_from_chip("master", false));
    }

    #[test]
    fn whole_bookmark_delete_skips_trunk_and_conflicted_bookmarks() {
        assert!(can_delete_bookmark("feature", false));
        assert!(!can_delete_bookmark("feature", true));
        assert!(!can_delete_bookmark("main", false));
        assert!(!can_delete_bookmark("main@origin", false));
    }
}
