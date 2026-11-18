use super::change::ShortId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpLogEntry {
    /// Operation id. Its `short_len` is the prefix unique among the listed operations (op ids have no templater `shortest()`, so it's computed in `op_log`).
    pub id: ShortId,
    #[serde(default)]
    pub parent_ids: Vec<String>,
    pub description: String,
    pub timestamp_millis: i64,
    pub is_current: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RecentOperations {
    Entries(Vec<OpLogEntry>),
    Failed(String),
}

impl Default for RecentOperations {
    fn default() -> Self {
        Self::Entries(Vec::new())
    }
}

impl RecentOperations {
    pub fn entries(&self) -> &[OpLogEntry] {
        match self {
            Self::Entries(entries) => entries,
            Self::Failed(_) => &[],
        }
    }

    /// The only listed parent of `undoing`, or of the current operation without one.
    pub fn restore_target(&self, undoing: Option<&str>) -> Option<&str> {
        let entries = self.entries();
        let entry = entries
            .iter()
            .find(|entry| undoing.map_or(entry.is_current, |id| entry.id.as_str() == id))?;
        let [parent] = entry.parent_ids.as_slice() else {
            return None;
        };
        entries
            .iter()
            .any(|entry| entry.id.as_str() == parent)
            .then_some(parent.as_str())
    }
}

/// What `jj undo` and `jj redo` would act on from the loaded operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UndoState {
    /// Full id of the loaded operation; undo and redo refuse to act once the repo has moved past it.
    pub operation_id: String,
    pub undo_description: Option<String>,
    pub redo_description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceInfo {
    pub name: String,
    /// Display path, possibly lossy; empty for repositories that predate recorded roots.
    pub path: String,
    pub is_path_resolved: bool,
    /// Canonical UTF-8 path for pinning; absent for unresolved or non-UTF-8 roots.
    pub pinnable_path: Option<String>,
    pub is_current: bool,
    pub change_id: ShortId,
    pub description: String,
    pub timestamp: i64,
    pub has_conflict: bool,
    pub files_changed: u32,
}

impl WorkspaceInfo {
    pub fn is_switch_target(&self) -> bool {
        !self.is_current && self.is_path_resolved
    }
}

/// Not a bool: a repo that momentarily fails to load is undecided, not forgotten.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkspacePresence {
    Exists,
    Gone,
    #[default]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub depth: u32,
    /// If Some, this is a file entry with associated hunk index. If None, it's a directory.
    pub hunk_index: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnotationLine {
    pub change_id: ShortId,
    pub author: String,
    pub timestamp: String,
    pub line_number: u32,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, parents: &[&str], is_current: bool) -> OpLogEntry {
        OpLogEntry {
            id: ShortId {
                id: id.to_owned(),
                short_len: 1,
            },
            parent_ids: parents.iter().map(|&parent| parent.to_owned()).collect(),
            description: String::new(),
            timestamp_millis: 0,
            is_current,
        }
    }

    #[test]
    fn restore_target_is_the_only_listed_parent() {
        let operations = RecentOperations::Entries(vec![
            entry("c", &["b"], true),
            entry("b", &["a"], false),
            entry("a", &["root"], false),
            entry("m", &["a", "b"], false),
        ]);
        assert_eq!(operations.restore_target(None), Some("b"));
        assert_eq!(operations.restore_target(Some("b")), Some("a"));
        assert_eq!(operations.restore_target(Some("a")), None);
        assert_eq!(operations.restore_target(Some("m")), None);
        assert_eq!(
            RecentOperations::Failed("x".into()).restore_target(None),
            None
        );
    }
}
