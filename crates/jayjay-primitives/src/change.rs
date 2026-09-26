#[derive(Debug, Clone)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub timestamp_millis: i64,
}

impl CommitAuthor {
    pub fn new(name: impl Into<String>, email: impl Into<String>, timestamp_millis: i64) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            timestamp_millis,
        }
    }

    pub fn empty(timestamp_millis: i64) -> Self {
        Self::new("", "", timestamp_millis)
    }
}

/// An id (change-id or commit-id) paired with the length of its shortest unique prefix among visible commits; shells highlight that prefix and dim the rest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortId {
    pub id: String,
    pub short_len: u32,
}

impl ShortId {
    pub fn new(id: String, short_len: u32) -> Self {
        Self { id, short_len }
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }

    pub const LABEL_CHARS: usize = 8;

    pub fn prefix(&self, max_chars: usize) -> String {
        self.id.chars().take(max_chars).collect()
    }

    pub fn unique_prefix(&self) -> String {
        self.prefix(self.short_len as usize)
    }
}

impl std::ops::Deref for ShortId {
    type Target = str;
    fn deref(&self) -> &str {
        &self.id
    }
}

impl PartialEq<str> for ShortId {
    fn eq(&self, other: &str) -> bool {
        self.id == other
    }
}

impl PartialEq<String> for ShortId {
    fn eq(&self, other: &String) -> bool {
        &self.id == other
    }
}

#[derive(Debug, Clone)]
pub struct ChangeInfo {
    pub change_id: ShortId,
    pub commit_id: ShortId,
    pub description: String,
    pub author: CommitAuthor,
    pub parents: Vec<String>,
    pub bookmarks: Vec<String>,
    pub tags: Vec<String>,
    /// Other workspaces whose working copy sits on this commit; the current workspace shows as `@` instead.
    pub workspaces: Vec<String>,
    pub is_working_copy: bool,
    pub has_conflict: bool,
    pub is_empty: bool,
    pub is_immutable: bool,
    pub is_divergent: bool,
    pub new_change: NewChangeEligibility,
}

impl ChangeInfo {
    /// Divergent siblings share a change id, so only the commit id names one of them.
    pub fn selection_revision(&self) -> &str {
        if self.is_divergent {
            &self.commit_id.id
        } else {
            &self.change_id.id
        }
    }

    pub fn label(&self) -> String {
        if let Some(name) = self.bookmarks.first().or(self.tags.first())
            && !name.is_empty()
        {
            return name.clone();
        }
        if self.is_working_copy {
            return "@".to_owned();
        }
        self.change_id.prefix(ShortId::LABEL_CHARS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewChangeEligibility {
    pub on_top: bool,
    pub before: bool,
    pub after: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertPosition {
    Before,
    After,
}

#[derive(Debug, Clone)]
pub struct EvologEntry {
    pub change_id: ShortId,
    pub commit_id: ShortId,
    /// Operation timestamp (when this rewrite happened).
    pub timestamp_millis: i64,
    pub operation: String,
    /// Commit description at this point in evolution (often empty for snapshots).
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct GraphEntry {
    pub change: ChangeInfo,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// Target commit_id (hex) this edge points to.
    pub target: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Direct,
    Indirect,
    Missing,
}

#[derive(Debug, Clone)]
pub struct ChangeDetail {
    pub info: ChangeInfo,
    pub diff: Vec<super::DiffHunk>,
}

#[cfg(test)]
mod tests {
    use super::ShortId;

    #[test]
    fn short_id_prefix_is_bounded_by_the_available_id() {
        let id = ShortId::new("abcdefghijklmnop".to_owned(), 4);
        assert_eq!(id.prefix(12), "abcdefghijkl");
        assert_eq!(id.prefix(24), id.as_str());
    }
}

#[cfg(test)]
mod change_info_tests {
    use super::*;

    fn change(change_id: &str) -> ChangeInfo {
        ChangeInfo {
            change_id: ShortId::new(change_id.to_owned(), 1),
            commit_id: ShortId::new(format!("{change_id}-commit"), 1),
            description: String::new(),
            author: CommitAuthor::empty(0),
            parents: Vec::new(),
            bookmarks: Vec::new(),
            tags: Vec::new(),
            workspaces: Vec::new(),
            is_working_copy: false,
            has_conflict: false,
            is_empty: false,
            is_immutable: false,
            is_divergent: false,
            new_change: NewChangeEligibility {
                on_top: true,
                before: true,
                after: true,
            },
        }
    }

    #[test]
    fn a_change_resolves_by_commit_id_only_when_it_is_divergent() {
        let mut change = change("change-id");
        assert_eq!(change.selection_revision(), "change-id");

        change.is_divergent = true;
        assert_eq!(change.selection_revision(), "change-id-commit");
    }

    #[test]
    fn labels_prefer_bookmarks_then_tags_then_the_working_copy_over_change_ids() {
        let mut change = change("change-id-long");
        assert_eq!(change.label(), "change-i");

        change.is_working_copy = true;
        assert_eq!(change.label(), "@");

        change.tags.push("v1.0.0".to_owned());
        assert_eq!(change.label(), "v1.0.0");

        change.bookmarks.push("main".to_owned());
        assert_eq!(change.label(), "main");
    }
}
