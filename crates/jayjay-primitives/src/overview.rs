use super::change::ShortId;

/// One change inside a lane, in the order the lane lists them (head first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewChange {
    pub change_id: ShortId,
    pub commit_id: ShortId,
    /// First line of the description; empty when undescribed.
    pub description: String,
    /// The whole description, for a detail view.
    pub full_description: String,
    pub timestamp_millis: i64,
    pub is_empty: bool,
    pub has_conflict: bool,
    pub bookmarks: Vec<String>,
    /// Every workspace checked out on this change, the current one included.
    pub workspaces: Vec<String>,
}

/// What a lane sits on. The kind decides how the base row is drawn and whether the lane counts as behind trunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverviewBaseKind {
    /// The current trunk head.
    Trunk,
    /// An immutable ancestor of trunk: trunk has moved on since this lane forked.
    OlderTrunk,
    /// A mutable change shared by several lanes (a fork point).
    Mutable,
    /// Immutable but not on trunk, such as a tagged release.
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewBase {
    pub change_id: ShortId,
    pub commit_id: ShortId,
    pub description: String,
    pub timestamp_millis: i64,
    pub kind: OverviewBaseKind,
    pub bookmarks: Vec<String>,
    /// Trunk commits after this base; 0 unless the kind is `OlderTrunk`.
    pub behind_trunk: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewWorkspace {
    pub name: String,
    pub is_current: bool,
    /// Changes in the lane above this checkout; 0 means the workspace sits on the head.
    pub changes_above: u32,
}

/// A maximal chain of mutable changes with no fork inside it: every change but the head has exactly one mutable child.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewLane {
    /// Head first, the change on the base last.
    pub changes: Vec<OverviewChange>,
    pub base: OverviewBase,
    pub workspaces: Vec<OverviewWorkspace>,
    pub latest_timestamp_millis: i64,
    /// Plain sentences describing conditions worth a look; empty when there are none.
    pub attention: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overview {
    pub lanes: Vec<OverviewLane>,
    pub workspace_count: u32,
}

impl OverviewLane {
    pub fn head(&self) -> &OverviewChange {
        &self.changes[0]
    }

    pub fn has_current_workspace(&self) -> bool {
        self.workspaces.iter().any(|workspace| workspace.is_current)
    }

    pub fn is_behind_trunk(&self) -> bool {
        self.base.kind == OverviewBaseKind::OlderTrunk
    }
}
