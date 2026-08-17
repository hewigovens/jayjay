use std::path::PathBuf;

use jayjay_core::repositories::{RepoGroup, RepoListGroups, Store, normalize_repository_path};

fn store(store_path: Option<String>) -> Store {
    match store_path {
        Some(path) => Store::load_from(PathBuf::from(path)),
        None => Store::load(),
    }
}

#[uniffi::export]
fn normalized_repository_path(path: String) -> String {
    let normalized = normalize_repository_path(PathBuf::from(&path).as_path());
    normalized.into_os_string().into_string().unwrap_or(path)
}

#[uniffi::export]
fn repositories(store_path: Option<String>) -> Vec<String> {
    store(store_path).repositories()
}

#[uniffi::export]
fn set_repository_pinned(path: String, pinned: bool, store_path: Option<String>) -> Vec<String> {
    store(store_path).set_pinned(PathBuf::from(path).as_path(), pinned)
}

#[uniffi::remote(Record)]
pub struct RepoGroup {
    pub path: String,
    pub workspaces: Vec<String>,
}

#[uniffi::remote(Record)]
pub struct RepoListGroups {
    pub pinned: Vec<RepoGroup>,
    pub recent: Vec<RepoGroup>,
}

#[uniffi::export]
fn repository_list_groups(pinned: Vec<String>, recents: Vec<String>) -> RepoListGroups {
    jayjay_core::repositories::group_repositories(&pinned, &recents)
}
