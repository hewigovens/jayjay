/// Check if text content is a Git LFS placeholder.
pub fn is_git_lfs(text: Option<&str>) -> bool {
    text.is_some_and(|t| t.starts_with("<git lfs "))
}

/// Check if text content is a Git submodule placeholder.
pub fn is_git_submodule(text: Option<&str>) -> bool {
    text.is_some_and(|t| t.starts_with("<git submodule"))
}
