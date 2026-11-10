use jayjay_core::DiffStats;

pub(super) fn working_copy_stat_label(stats: &DiffStats) -> Option<String> {
    if stats.files_changed == 0 {
        return None;
    }
    let mut text = jayjay_i18n::repo::file_count(stats.files_changed);
    if stats.insertions > 0 || stats.deletions > 0 {
        text.push_str(&format!(" +{} -{}", stats.insertions, stats.deletions));
    }
    Some(text)
}
