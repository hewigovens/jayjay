//! Classifies diff content as editable text vs. a synthetic placeholder the
//! diff layer emits for content it cannot show inline. Single source of truth
//! so the Rust diff-edit safety check and the Swift shell cannot drift apart;
//! producers live in `repo/diff/materialize.rs` and `repo/diff/entry.rs`.

/// Every placeholder prefix the diff layer can emit. Keep in sync with the
/// producers in `repo/diff` and `external_tools/scan.rs` — a new placeholder must be listed here.
const PLACEHOLDER_PREFIXES: &[&str] = &[
    "<binary file",
    "<directory>",
    "<git lfs ",
    "<git submodule",
    "<conflict",
    "<access denied",
    "<file too large",
    "<image ",
    "<not a regular file",
    "<unsupported file",
    "symlink -> ",
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
