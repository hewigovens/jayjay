use super::change::ShortId;

#[derive(Debug, Clone)]
pub struct OpLogEntry {
    /// Operation id. Its `short_len` is the prefix unique among the listed operations (op ids have no templater `shortest()`, so it's computed in `op_log`).
    pub id: ShortId,
    pub description: String,
    pub timestamp: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: String,
    pub is_current: bool,
    /// Hex commit id of this workspace's working-copy commit. Empty when the view has no WC for the name.
    pub wc_commit_id: String,
    /// Hex commit id of the WC commit's first parent. Empty when there is no parent.
    pub parent_commit_id: String,
    /// Author timestamp of the WC commit, milliseconds since epoch.
    pub timestamp_millis: Option<i64>,
    /// Files different between the WC commit tree and its first parent. Not dirty disk.
    pub changed_file_count: Option<u32>,
    /// First line of the WC commit description. Empty when the commit has no description.
    pub description: String,
    /// Whether `path` exists on disk. A missing path is still listed, but dimmed in the UI.
    pub path_exists: bool,
}

impl WorkspaceInfo {
    pub fn new(name: impl Into<String>, path: impl Into<String>, is_current: bool) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_current,
            wc_commit_id: String::new(),
            parent_commit_id: String::new(),
            timestamp_millis: None,
            changed_file_count: None,
            description: String::new(),
            path_exists: false,
        }
    }

    /// Pin `default` when present, otherwise the workspace this window launched with. Remaining rows are newest `@` first.
    pub fn sort_for_sidebar(workspaces: &mut [Self], launched_name: &str) {
        let pin = if workspaces
            .iter()
            .any(|workspace| workspace.name == "default")
        {
            "default"
        } else {
            launched_name
        };
        workspaces.sort_by(|left, right| match (left.name == pin, right.name == pin) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => match (right.timestamp_millis, left.timestamp_millis) {
                (Some(newer), Some(older)) => {
                    newer.cmp(&older).then_with(|| left.name.cmp(&right.name))
                }
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => left.name.cmp(&right.name),
            },
        });
    }
}

#[cfg(test)]
mod workspace_info_tests {
    use super::WorkspaceInfo;

    fn workspace(name: &str, is_current: bool, timestamp_millis: Option<i64>) -> WorkspaceInfo {
        let mut info = WorkspaceInfo::new(name, format!("/{name}"), is_current);
        info.timestamp_millis = timestamp_millis;
        info
    }

    #[test]
    fn sort_pins_default_then_timestamp_descending() {
        let mut workspaces = vec![
            workspace("older", false, Some(1_000)),
            workspace("newest", false, Some(3_000)),
            workspace("default", true, Some(2_000)),
            workspace("mid", false, Some(2_500)),
        ];
        WorkspaceInfo::sort_for_sidebar(&mut workspaces, "default");
        let names: Vec<_> = workspaces
            .iter()
            .map(|workspace| workspace.name.as_str())
            .collect();
        assert_eq!(names, ["default", "newest", "mid", "older"]);
    }

    #[test]
    fn sort_pins_launched_workspace_when_default_is_absent() {
        let mut workspaces = vec![
            workspace("agent-pr", true, Some(1_000)),
            workspace("indexer", false, Some(4_000)),
            workspace("hotfix", false, Some(2_000)),
        ];
        WorkspaceInfo::sort_for_sidebar(&mut workspaces, "agent-pr");
        let names: Vec<_> = workspaces
            .iter()
            .map(|workspace| workspace.name.as_str())
            .collect();
        assert_eq!(names, ["agent-pr", "indexer", "hotfix"]);
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub depth: u32,
    /// If Some, this is a file entry with associated hunk index. If None, it's a directory.
    pub hunk_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AnnotationLine {
    pub change_id: ShortId,
    pub author: String,
    pub timestamp: String,
    pub line_number: u32,
    pub text: String,
}
