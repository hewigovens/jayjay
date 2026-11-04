use std::collections::HashSet;

use jayjay_primitives::{
    HunkType, JayError, JayResult, ReviewDiffProvider, ReviewFileDiff, ReviewHunk,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReviewBundle {
    pub schema_version: u32,
    pub repository_id: String,
    pub change_id: String,
    #[serde(default)]
    pub change_id_prefix: Option<usize>,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub author_email: String,
    pub files: Vec<ReviewFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReviewFile {
    pub path: String,
    #[serde(default)]
    pub old_path: Option<String>,
    pub review_identity: String,
    /// Both sides are absent until the host loads the file with `ReviewSession::load`.
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub after: Option<String>,
    #[serde(default)]
    pub kind: Option<FileKind>,
    #[serde(default)]
    pub added: Option<u32>,
    #[serde(default)]
    pub removed: Option<u32>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FileKind {
    Added,
    Modified,
    Removed,
    Renamed,
    Copied,
}

impl FileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Removed => "removed",
            Self::Renamed => "renamed",
            Self::Copied => "copied",
        }
    }
}

impl ReviewFile {
    pub fn content(&self) -> Option<(&str, &str)> {
        Some((self.before.as_deref()?, self.after.as_deref()?))
    }

    pub fn kind(&self) -> FileKind {
        if let Some(kind) = self.kind {
            return kind;
        }
        match (self.content(), self.old_path.is_some()) {
            (_, true) => FileKind::Renamed,
            (Some(("", after)), _) if !after.is_empty() => FileKind::Added,
            (Some((before, "")), _) if !before.is_empty() => FileKind::Removed,
            _ => FileKind::Modified,
        }
    }

    pub fn load(&mut self, before: &str, after: &str) -> Result<(), String> {
        if !valid_content(before, after) {
            return Err(format!("Invalid review file content: {}", self.path));
        }
        self.before = Some(before.to_owned());
        self.after = Some(after.to_owned());
        Ok(())
    }
}

impl ReviewBundle {
    pub fn parse(json: &str) -> Result<Self, String> {
        if json.len() > 32 * 1024 * 1024 {
            return Err("Review bundle exceeds 32 MiB".into());
        }
        let bundle: Self = serde_json::from_str(json).map_err(|error| error.to_string())?;
        if bundle.schema_version != 1
            || bundle.repository_id.is_empty()
            || bundle.change_id.is_empty()
            || bundle.change_id.contains('|')
            || bundle.files.len() > 2000
            || bundle.author_email.len() > 320
        {
            return Err("Unsupported review bundle or missing repository/change identity".into());
        }
        let mut paths = HashSet::new();
        for file in &bundle.files {
            if !valid_path(&file.path)
                || file
                    .old_path
                    .as_deref()
                    .is_some_and(|path| !valid_path(path))
                || file.review_identity.is_empty()
                || !paths.insert(&file.path)
                || file.before.is_some() != file.after.is_some()
                || file
                    .content()
                    .is_some_and(|(before, after)| !valid_content(before, after))
            {
                return Err(format!("Invalid or duplicate review file: {}", file.path));
            }
        }
        Ok(bundle)
    }
}

fn valid_content(before: &str, after: &str) -> bool {
    !before.contains('\0') && !after.contains('\0')
}

fn valid_path(path: &str) -> bool {
    !path.contains(['\\', '\0'])
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

impl ReviewDiffProvider for ReviewBundle {
    fn review_hunks(&self) -> JayResult<Vec<ReviewHunk>> {
        Ok(self
            .files
            .iter()
            .map(|file| ReviewHunk {
                path: file.path.clone(),
                old_path: file.old_path.clone(),
                hunk_type: match file.kind() {
                    FileKind::Added => HunkType::Added,
                    FileKind::Removed => HunkType::Removed,
                    _ => HunkType::Modified,
                },
                review_identity: file.review_identity.clone(),
            })
            .collect())
    }

    fn review_file_diff(&self, hunk: &ReviewHunk) -> JayResult<ReviewFileDiff> {
        let file = self
            .files
            .iter()
            .find(|file| file.path == hunk.path)
            .ok_or_else(|| JayError::review("Review file is unavailable"))?;
        let (before, after) = file
            .content()
            .ok_or_else(|| JayError::review("Review file is not loaded"))?;
        Ok(ReviewFileDiff {
            old_content: Some(before.to_owned()),
            new_content: Some(after.to_owned()),
        })
    }
}
