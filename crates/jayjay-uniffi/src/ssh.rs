use jayjay_core::{
    BisectState, BookmarkInfo, DiffStats, RepositoryContext, RevsetVocabulary, TagInfo,
    WorkspaceInfo,
};
use jayjay_core::{
    JayError,
    ssh::{self, ConnectionState, RemoteDirectory, RemoteFolder, RemoteLocation},
};
use std::path::PathBuf;

#[uniffi::remote(Enum)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
}

#[uniffi::remote(Record)]
pub struct RemoteFolder {
    pub name: String,
    pub path: String,
}
#[uniffi::remote(Record)]
pub struct RemoteDirectory {
    pub path: String,
    pub parent: Option<String>,
    pub folders: Vec<RemoteFolder>,
}

#[uniffi::export]
fn ssh_machines() -> Vec<String> {
    ssh::machines()
}
#[uniffi::export]
fn ssh_last_machine() -> Option<String> {
    ssh::last_machine()
}
#[uniffi::export]
fn ssh_location(machine: String, path: String) -> Result<String, JayError> {
    Ok(RemoteLocation::new(&machine, &path)?.encode())
}
#[uniffi::export]
fn ssh_machine(location: String) -> Option<String> {
    RemoteLocation::from_path(&location).map(|location| location.machine)
}
#[uniffi::export]
fn ssh_path(location: String) -> String {
    RemoteLocation::from_path(&location).map_or(location, |location| location.path)
}
#[uniffi::export]
fn ssh_browse(machine: String, path: String) -> Result<RemoteDirectory, JayError> {
    ssh::browse(&machine, &path)
}
#[uniffi::export]
fn set_ssh_helper_directory(path: String) {
    ssh::set_helper_directory(PathBuf::from(path));
}

#[uniffi::remote(Record)]
pub struct RepositoryContext {
    pub bookmarks: Vec<BookmarkInfo>,
    pub tags: Vec<TagInfo>,
    pub vocabulary: RevsetVocabulary,
    pub workspaces: Option<Vec<WorkspaceInfo>>,
    pub trunk_name: String,
    pub pr_host_name: Option<String>,
    pub fix_unavailable_reason: Option<String>,
    pub working_copy_stats: Option<DiffStats>,
    pub operation: String,
    pub bisect_state: Option<BisectState>,
}
