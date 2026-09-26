use super::change::ShortId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewChange {
    pub change_id: ShortId,
    pub commit_id: ShortId,
    pub description: String,
    pub full_description: String,
    pub timestamp_millis: i64,
    pub is_empty: bool,
    pub has_conflict: bool,
    pub bookmarks: Vec<String>,
    pub workspaces: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverviewBaseKind {
    Trunk,
    OlderTrunk,
    Mutable,
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
    pub behind_trunk: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewWorkspace {
    pub name: String,
    pub is_current: bool,
    pub changes_above: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverviewLane {
    pub changes: Vec<OverviewChange>,
    pub base: OverviewBase,
    pub workspaces: Vec<OverviewWorkspace>,
    pub latest_timestamp_millis: i64,
    pub attention: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overview {
    pub lanes: Vec<OverviewLane>,
    pub workspace_count: u32,
}

impl OverviewChange {
    pub fn title(&self) -> &str {
        match (self.description.as_str(), self.is_empty) {
            ("", true) => "(empty)",
            ("", false) => "(no description)",
            (description, _) => description,
        }
    }
}

impl OverviewLane {
    pub fn new(
        changes: Vec<OverviewChange>,
        base: OverviewBase,
        workspaces: Vec<OverviewWorkspace>,
    ) -> Self {
        let latest_timestamp_millis = changes
            .iter()
            .map(|change| change.timestamp_millis)
            .max()
            .unwrap_or(0);
        let attention = Self::attention(&changes);
        Self {
            changes,
            base,
            workspaces,
            latest_timestamp_millis,
            attention,
        }
    }

    fn attention(changes: &[OverviewChange]) -> Vec<String> {
        let plural = |count: usize| if count == 1 { "" } else { "s" };
        let mut attention = Vec::new();
        let head = &changes[0];
        if head.is_empty && head.description.is_empty() && changes.len() > 1 {
            let below = changes.len() - 1;
            attention.push(format!(
                "Empty, undescribed checkout above {below} change{}",
                plural(below)
            ));
        }
        let conflicted = changes.iter().filter(|change| change.has_conflict).count();
        if conflicted > 0 {
            attention.push(format!(
                "{conflicted} change{} with conflicts",
                plural(conflicted)
            ));
        }
        let undescribed = changes
            .iter()
            .filter(|change| change.description.is_empty() && !change.is_empty)
            .count();
        if undescribed > 0 {
            attention.push(format!(
                "{undescribed} change{} without a description",
                plural(undescribed)
            ));
        }
        attention
    }

    pub fn head(&self) -> &OverviewChange {
        &self.changes[0]
    }

    pub fn title(&self) -> &str {
        match self.head().description.as_str() {
            "" => "(no description)",
            description => description,
        }
    }

    pub fn matches(&self, filter: &str) -> bool {
        let needle = filter.trim().to_lowercase();
        needle.is_empty()
            || self.changes.iter().any(|change| {
                [&change.description, &change.change_id.id]
                    .into_iter()
                    .chain(&change.bookmarks)
                    .chain(&change.workspaces)
                    .any(|haystack| haystack.to_lowercase().contains(&needle))
            })
    }

    pub fn has_current_workspace(&self) -> bool {
        self.workspaces.iter().any(|workspace| workspace.is_current)
    }

    pub fn is_behind_trunk(&self) -> bool {
        self.base.kind == OverviewBaseKind::OlderTrunk
    }
}
