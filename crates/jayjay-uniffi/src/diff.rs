#[uniffi::export]
pub fn build_side_by_side_rows(
    lines: Vec<jayjay_core::diff::DiffLine>,
) -> Vec<jayjay_core::diff::SideBySideRow> {
    jayjay_core::diff::build_side_by_side_rows(&lines)
}
