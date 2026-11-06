use jayjay_primitives::{
    ConvergeAuthor, ConvergeAuthorAttribute, ConvergeChoices, ConvergeDescendant,
    ConvergeDescriptionAttribute, ConvergeEffects, ConvergeOutcome, ConvergeParentsAttribute,
    ConvergePlan, ConvergeVersion, DescriptionChoice,
};

#[uniffi::remote(Record)]
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

#[uniffi::remote(Enum)]
pub enum ConvergeAuthorAttribute {
    Solved { value: ConvergeAuthor },
    Unsolved,
}

#[uniffi::remote(Enum)]
pub enum ConvergeDescriptionAttribute {
    Solved { value: String },
    Unsolved,
}

#[uniffi::remote(Enum)]
pub enum ConvergeParentsAttribute {
    Solved { value: Vec<String> },
    Unsolved,
}

#[uniffi::remote(Record)]
pub struct ConvergeVersion {
    pub commit_id: String,
    pub description: String,
    pub author: ConvergeAuthor,
    pub parents: Vec<String>,
    pub is_immutable: bool,
    pub workspaces: Vec<String>,
    pub bookmarks: Vec<String>,
}

#[uniffi::remote(Record)]
pub struct ConvergeAuthor {
    pub name: String,
    pub email: String,
    pub timestamp_millis: i64,
    pub timezone_offset_minutes: i32,
}

#[uniffi::remote(Record)]
pub struct ConvergeChoices {
    pub author: Option<String>,
    pub parents: Option<String>,
    pub description: Option<DescriptionChoice>,
    pub contents: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum DescriptionChoice {
    FromVersion(String),
    Custom(String),
}

#[uniffi::remote(Record)]
pub struct ConvergeEffects {
    pub descendants: Vec<ConvergeDescendant>,
    pub bookmarks: Vec<String>,
    pub workspaces: Vec<String>,
    pub blocked_reason: Option<String>,
}

#[uniffi::remote(Record)]
pub struct ConvergeDescendant {
    pub commit_id: String,
    pub description: String,
}

#[uniffi::remote(Record)]
pub struct ConvergeOutcome {
    pub successor_commit_id: String,
    pub conflicted_paths: Vec<String>,
}
