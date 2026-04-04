const NON_EDITABLE_PREFIXES: &[&str] = &[
    "<binary file",
    "<directory>",
    "<git lfs ",
    "<git submodule",
    "<conflict",
    "<access denied",
];

/// Check if text content represents editable text (not a binary/placeholder).
pub fn is_editable_text(text: Option<&str>) -> bool {
    match text {
        None => true,
        Some(t) => !NON_EDITABLE_PREFIXES.iter().any(|p| t.starts_with(p)),
    }
}

/// Check if text content is a Git LFS placeholder.
pub fn is_git_lfs(text: Option<&str>) -> bool {
    text.is_some_and(|t| t.starts_with("<git lfs "))
}

/// Check if text content is a Git submodule placeholder.
pub fn is_git_submodule(text: Option<&str>) -> bool {
    text.is_some_and(|t| t.starts_with("<git submodule"))
}
