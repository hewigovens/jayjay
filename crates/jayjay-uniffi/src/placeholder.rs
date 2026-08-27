#[uniffi::export]
fn is_editable_diff_text(text: String) -> bool {
    jayjay_core::placeholder::is_editable_text(&text)
}

#[uniffi::export]
fn is_git_lfs_placeholder(text: String) -> bool {
    jayjay_core::placeholder::is_git_lfs_placeholder(&text)
}

#[uniffi::export]
fn is_git_submodule_placeholder(text: String) -> bool {
    jayjay_core::placeholder::is_git_submodule_placeholder(&text)
}
