use std::path::PathBuf;

use jayjay_core::diff::FileDiff;
use jayjay_core::external_tools::{
    ExternalMergeResolution, conflict_marker_count, load_external_merge, save_external_merge,
};
use jayjay_core::{CoreResult, MergeEditorHunk, MergeHunkSource, merge_hunk_display_diff};

pub(super) struct ExternalMergeSession {
    pub left_path: PathBuf,
    pub base_path: PathBuf,
    pub right_path: PathBuf,
    pub output_path: PathBuf,
    pub repo_path: String,
    pub marker_length: usize,
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
        marker_length: usize,
    ) -> CoreResult<Self> {
        let merge = load_external_merge(
            &left_path,
            &base_path,
            &right_path,
            &output_path,
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
        self.selected_source(result).is_some() || self.is_text_merge()
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
