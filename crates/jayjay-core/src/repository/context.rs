use crate::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RepositoryContext {
    pub bookmarks: Vec<BookmarkInfo>,
    pub tags: Vec<TagInfo>,
    pub vocabulary: RevsetVocabulary,
    pub workspaces: Option<Vec<WorkspaceInfo>>,
    pub trunk_name: String,
    pub pr_host_name: Option<String>,
    pub upload_title: Option<String>,
    pub gerrit_update_title: Option<String>,
    pub fix_unavailable_reason: Option<String>,
    pub working_copy_stats: Option<DiffStats>,
    pub operation: String,
    #[serde(default)]
    pub undo: UndoState,
    pub bisect_state: Option<BisectState>,
}

impl crate::repo::Repo {
    pub(crate) fn repository_context(&self) -> JayResult<RepositoryContext> {
        let bookmarks = self.list_bookmarks()?;
        let vocabulary = self.revset_vocabulary(&bookmarks);
        Ok(RepositoryContext {
            bookmarks,
            vocabulary,
            tags: self.list_tags()?,
            workspaces: self.workspace_list().ok(),
            trunk_name: self.trunk_name(),
            pr_host_name: self.pr_host_name(),
            upload_title: self.upload_title(),
            gerrit_update_title: self.gerrit_update_title(),
            fix_unavailable_reason: self.fix_unavailable_reason(),
            working_copy_stats: self.diff_stats("@").ok(),
            operation: self.current_operation_description(),
            undo: self.undo_state(),
            bisect_state: self.bisect_state(),
        })
    }
}
