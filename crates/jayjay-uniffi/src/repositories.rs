use std::path::PathBuf;

use jayjay_core::repositories::{Store, normalize_repository_path};

fn store(store_path: Option<String>) -> Store {
    match store_path {
        Some(path) => Store::load_from(PathBuf::from(path)),
        None => Store::load(),
    }
}

#[uniffi::export]
pub fn normalized_repository_path(path: String) -> String {
    let normalized = normalize_repository_path(PathBuf::from(&path).as_path());
    normalized.into_os_string().into_string().unwrap_or(path)
}

#[uniffi::export]
pub fn repositories(store_path: Option<String>) -> Vec<String> {
    store(store_path).repositories()
}

#[uniffi::export]
pub fn set_repository_pinned(
    path: String,
    pinned: bool,
    store_path: Option<String>,
) -> Vec<String> {
    store(store_path).set_pinned(PathBuf::from(path).as_path(), pinned)
}
