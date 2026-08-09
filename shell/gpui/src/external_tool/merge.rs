use std::path::PathBuf;

use jayjay_core::diff::FileDiff;
use jayjay_core::external_tools::{
    ExternalMergeResolution, conflict_marker_count, has_conflict_marker_remnants,
    load_external_merge, save_external_merge,
};
use jayjay_core::{CoreResult, MergeEditorHunk, MergeHunkSource, merge_hunk_display_diff};

pub(super) struct ExternalMergeSession {
    pub left_path: PathBuf,
    pub base_path: PathBuf,
    pub right_path: PathBuf,
    pub output_path: PathBuf,
    pub repo_path: String,
    pub marker_length: usize,
    output_is_initialized: bool,
    pub left: String,
    pub base: String,
    pub right: String,
    pub initial_result: String,
    pub is_text: bool,
    pub hunks: Vec<MergeEditorHunk>,
    pub hunk_diffs: Vec<FileDiff>,
    pub selected_source: Option<(PathBuf, String)>,
}

pub(super) enum ExternalMergeSave {
    Content { output: PathBuf, content: String },
    Source { output: PathBuf, source: PathBuf },
}

impl ExternalMergeSession {
    pub fn load(
        left_path: PathBuf,
        base_path: PathBuf,
        right_path: PathBuf,
        output_path: PathBuf,
        repo_path: String,
        output_is_initialized: bool,
        marker_length: usize,
    ) -> CoreResult<Self> {
        let merge = load_external_merge(
            &left_path,
            &base_path,
            &right_path,
            &output_path,
            output_is_initialized,
            marker_length,
        )?;
        let hunk_diffs = merge
            .hunks
            .iter()
            .map(|hunk| merge_hunk_display_diff(&repo_path, &merge.result, hunk))
            .collect();
        Ok(Self {
            left_path,
            base_path,
            right_path,
            output_path,
            repo_path,
            marker_length,
            output_is_initialized,
            left: merge.left,
            base: merge.base,
            right: merge.right,
            initial_result: merge.result,
            is_text: merge.is_text,
            hunks: merge.hunks,
            hunk_diffs,
            selected_source: None,
        })
    }

    pub fn is_text_merge(&self) -> bool {
        self.is_text
    }

    pub fn source(&self, source: MergeHunkSource) -> (&PathBuf, &str) {
        match source {
            MergeHunkSource::Left => (&self.left_path, &self.left),
            MergeHunkSource::Base => (&self.base_path, &self.base),
            MergeHunkSource::Right => (&self.right_path, &self.right),
        }
    }

    pub fn unresolved_count(&self, result: &str) -> usize {
        if self.selected_source(result).is_some() {
            0
        } else {
            conflict_marker_count(result, self.marker_length)
        }
    }

    pub fn save_request(&self, result: String) -> ExternalMergeSave {
        if let Some(source) = self.selected_source(&result) {
            ExternalMergeSave::Source {
                output: self.output_path.clone(),
                source: source.clone(),
            }
        } else {
            ExternalMergeSave::Content {
                output: self.output_path.clone(),
                content: result,
            }
        }
    }

    pub fn can_save(&self, result: &str) -> bool {
        self.selected_source(result).is_some()
            || self.is_text_merge()
                && (!self.output_is_initialized
                    || !has_conflict_marker_remnants(result, self.marker_length))
    }

    fn selected_source(&self, result: &str) -> Option<&PathBuf> {
        self.selected_source
            .as_ref()
            .filter(|(_, snapshot)| snapshot == result)
            .map(|(path, _)| path)
    }
}

impl ExternalMergeSave {
    pub fn run(self) -> CoreResult<()> {
        match self {
            Self::Content { output, content } => {
                save_external_merge(&output, ExternalMergeResolution::Content(content.as_str()))
            }
            Self::Source { output, source } => {
                save_external_merge(&output, ExternalMergeResolution::Source(&source))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn initialized_merge_rejects_partial_conflict_markers() {
        let directory = tempfile::tempdir().expect("directory");
        let left = directory.path().join("left.rs");
        let base = directory.path().join("base.rs");
        let right = directory.path().join("right.rs");
        let output = directory.path().join("output.rs");
        fs::write(&left, "fn value() -> i32 { 1 }\n").expect("left");
        fs::write(&base, "fn value() -> i32 { 0 }\n").expect("base");
        fs::write(&right, "fn value() -> i32 { 2 }\n").expect("right");
        fs::write(&output, "").expect("output");
        let generated =
            load_external_merge(&left, &base, &right, &output, false, 7).expect("generated merge");
        let partial = generated.result.replacen("<<<<<<< side #1\n", "", 1);
        fs::write(&output, partial).expect("partial merge");

        let session = ExternalMergeSession::load(
            left,
            base,
            right,
            output,
            "src/value.rs".to_owned(),
            true,
            7,
        )
        .expect("session");

        assert_eq!(session.unresolved_count(&session.initial_result), 0);
        assert!(!session.can_save(&session.initial_result));
        assert!(session.can_save("Markdown heading\n=======\nbody\n"));
    }
}
