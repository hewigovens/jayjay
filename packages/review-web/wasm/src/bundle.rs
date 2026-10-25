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
    pub before: String,
    pub after: String,
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
                || file.before.contains('\0')
                || file.after.contains('\0')
            {
                return Err(format!("Invalid or duplicate review file: {}", file.path));
            }
        }
        Ok(bundle)
    }
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
                hunk_type: if file.before.is_empty() {
                    HunkType::Added
                } else if file.after.is_empty() {
                    HunkType::Removed
                } else {
                    HunkType::Modified
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
        Ok(ReviewFileDiff {
            old_content: Some(file.before.clone()),
            new_content: Some(file.after.clone()),
        })
    }
}
