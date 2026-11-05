use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergePlan {
    pub change_id: String,
    pub versions: Vec<ConvergeVersion>,
    pub author: ConvergeAuthorAttribute,
    pub description: ConvergeDescriptionAttribute,
    pub parents: ConvergeParentsAttribute,
    pub conflicted_paths: Option<Vec<String>>,
    pub contents_source: Option<String>,
    pub effects: ConvergeEffects,
    pub basis: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergeVersion {
    pub commit_id: String,
    pub description: String,
    pub author: ConvergeAuthor,
    pub parents: Vec<String>,
    pub is_immutable: bool,
    pub workspaces: Vec<String>,
    pub bookmarks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergeAuthor {
    pub name: String,
    pub email: String,
    pub timestamp_millis: i64,
    pub timezone_offset_minutes: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvergeAuthorAttribute {
    Solved { value: ConvergeAuthor },
    Unsolved,
}

impl ConvergeAuthorAttribute {
    pub fn solved(&self) -> Option<&ConvergeAuthor> {
        match self {
            Self::Solved { value } => Some(value),
            Self::Unsolved => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvergeDescriptionAttribute {
    Solved { value: String },
    Unsolved,
}

impl ConvergeDescriptionAttribute {
    pub fn solved(&self) -> Option<&String> {
        match self {
            Self::Solved { value } => Some(value),
            Self::Unsolved => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvergeParentsAttribute {
    Solved { value: Vec<String> },
    Unsolved,
}

impl ConvergeParentsAttribute {
    pub fn solved(&self) -> Option<&Vec<String>> {
        match self {
            Self::Solved { value } => Some(value),
            Self::Unsolved => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergeChoices {
    pub author: Option<String>,
    pub parents: Option<String>,
    pub description: Option<DescriptionChoice>,
    pub contents: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DescriptionChoice {
    FromVersion(String),
    Custom(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergeEffects {
    pub descendants: Vec<ConvergeDescendant>,
    pub bookmarks: Vec<String>,
    pub workspaces: Vec<String>,
    pub blocked_reason: Option<String>,
}

impl ConvergeEffects {
    pub fn exceeds(&self, reviewed: &Self) -> bool {
        self.descendants
            .iter()
            .any(|descendant| !reviewed.descendants.contains(descendant))
            || self
                .bookmarks
                .iter()
                .any(|name| !reviewed.bookmarks.contains(name))
            || self
                .workspaces
                .iter()
                .any(|name| !reviewed.workspaces.contains(name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergeDescendant {
    pub commit_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergeOutcome {
    pub successor_commit_id: String,
    pub conflicted_paths: Vec<String>,
}
