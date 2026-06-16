//! Classifies diff content as editable text vs. a synthetic placeholder the
//! diff layer emits for content it cannot show inline. Single source of truth
//! so the Rust diff-edit safety check and the Swift shell cannot drift apart;
//! producers live in `repo/diff/materialize.rs` and `repo/diff/entry.rs`.

/// Every placeholder prefix the diff layer can emit. Keep in sync with the
/// producers in `repo/diff` — a new placeholder must be listed here.
const PLACEHOLDER_PREFIXES: &[&str] = &[
    "<binary file",
    "<directory>",
    "<git lfs ",
    "<git submodule",
    "<conflict",
    "<access denied",
    "<image ",
];

/// True when `text` is editable text rather than a placeholder.
pub fn is_editable_text(text: &str) -> bool {
    !PLACEHOLDER_PREFIXES
        .iter()
        .any(|prefix| text.starts_with(prefix))
}

/// True when `text` is a Git LFS pointer/object placeholder.
pub fn is_git_lfs_placeholder(text: &str) -> bool {
    text.starts_with("<git lfs ")
}

/// True when `text` is a Git submodule placeholder.
pub fn is_git_submodule_placeholder(text: &str) -> bool {
    text.starts_with("<git submodule")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_text_is_editable() {
        assert!(is_editable_text("let x = 1;"));
        assert!(is_editable_text(""));
        // A literal '<' that is not one of our placeholders stays editable.
        assert!(is_editable_text("<html>"));
    }

    #[test]
    fn every_placeholder_is_non_editable() {
        for sample in [
            "<binary file (742800 bytes)>",
            "<directory>",
            "<git lfs pointer sha256:abc (10 bytes)>",
            "<git lfs object sha256:abc (10 bytes)>",
            "<git submodule deadbeef>",
            "<conflict>",
            "<access denied: permission>",
            "<image (100 bytes)>",
        ] {
            assert!(
                !is_editable_text(sample),
                "should be non-editable: {sample}"
            );
        }
    }

    #[test]
    fn lfs_and_submodule_classifiers() {
        assert!(is_git_lfs_placeholder(
            "<git lfs pointer sha256:abc (10 bytes)>"
        ));
        assert!(!is_git_lfs_placeholder("<git submodule deadbeef>"));
        assert!(is_git_submodule_placeholder("<git submodule deadbeef>"));
        assert!(!is_git_submodule_placeholder(
            "<git lfs object sha256:abc (1 bytes)>"
        ));
    }
}
