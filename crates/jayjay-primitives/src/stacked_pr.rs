/// One change in a stacked-PR plan, ordered bottom (closest to trunk) → top.
#[derive(Debug, Clone)]
pub struct StackLayer {
    pub change_id: String,
    pub commit_id: String,
    pub title: String,
    pub body: String,
    pub bookmark: String,
    /// PR base: the bookmark of the layer below, or the trunk branch for the bottom.
    pub base: String,
    /// Whether `bookmark` already existed on the change locally (vs. auto-assigned by the planner); says nothing about the remote or an existing PR.
    pub bookmark_existed: bool,
    /// Shortest unique change-id prefix — the suffix the UI appends to a generated branch name (e.g. `<ai-name>-<change_id_short>`).
    pub change_id_short: String,
}

/// One layer's finalized name as submitted from the UI — the bookmark may have been edited or AI-generated, so `submit_stack` uses it verbatim.
#[derive(Debug, Clone)]
pub struct SubmitStackLayer {
    pub change_id: String,
    pub bookmark: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Stack {
    pub layers: Vec<StackLayer>,
    /// Trunk branch (e.g. `main`).
    pub base_bookmark: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackLayerOutcome {
    Created,
    Updated,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SubmittedLayer {
    pub bookmark: String,
    pub base: String,
    pub title: String,
    pub outcome: StackLayerOutcome,
    /// PR number, or 0 when none could be determined.
    pub pr_number: u32,
    pub pr_url: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct StackedPrResult {
    pub layers: Vec<SubmittedLayer>,
    pub message: String,
}
