#[uniffi::export]
fn is_trunk_bookmark(name: &str) -> bool {
    jayjay_core::trunk::is_trunk_bookmark(name)
}

#[uniffi::export]
fn can_remove_bookmark_from_chip(name: &str, conflicted: bool) -> bool {
    jayjay_core::trunk::can_remove_bookmark_from_chip(name, conflicted)
}

#[uniffi::export]
fn can_delete_bookmark(name: &str, conflicted: bool) -> bool {
    jayjay_core::trunk::can_delete_bookmark(name, conflicted)
}
