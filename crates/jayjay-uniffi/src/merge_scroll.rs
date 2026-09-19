use std::sync::Arc;

use jayjay_core::{MergeEditorHunk, MergePane};

#[derive(uniffi::Object)]
pub struct MergeScrollMap(jayjay_core::MergeScrollMap);

#[uniffi::export]
impl MergeScrollMap {
    #[uniffi::constructor]
    fn new(
        left: String,
        base: String,
        right: String,
        original: String,
        hunks: Vec<MergeEditorHunk>,
    ) -> Arc<Self> {
        Arc::new(Self(jayjay_core::MergeScrollMap::new(
            &left, &base, &right, &original, &hunks,
        )))
    }

    fn with_result(&self, result: String) -> Arc<Self> {
        Arc::new(Self(self.0.with_result(&result)))
    }

    fn map_line(&self, from: MergePane, to: MergePane, line: f64) -> f64 {
        self.0.map_line(from, to, line)
    }

    fn hunk_line(&self, pane: MergePane, index: u32) -> Option<f64> {
        self.0.hunk_line(pane, index)
    }
}
