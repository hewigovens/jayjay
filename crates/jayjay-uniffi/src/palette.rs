#[derive(uniffi::Record, Debug, Clone)]
pub struct PaletteRecall {
    query: String,
    history_index: Option<u32>,
}

/// Push `command` onto `history` newest-first, deduped, capped at the limit.
#[uniffi::export]
fn palette_record_history(command: String, history: Vec<String>) -> Vec<String> {
    jayjay_core::palette::record(&command, &history)
}

#[uniffi::export]
fn palette_recall_history(
    history: Vec<String>,
    history_index: Option<u32>,
    older: bool,
) -> Option<PaletteRecall> {
    jayjay_core::palette::recall(&history, history_index.map(|ix| ix as usize), older).map(
        |recall| PaletteRecall {
            query: recall.query,
            history_index: recall.index.map(|ix| ix as u32),
        },
    )
}
