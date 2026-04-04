uniffi::setup_scaffolding!();

mod error;
mod repo;
mod types;

pub use error::*;
pub use repo::*;
pub use types::*;

#[uniffi::export]
pub fn build_side_by_side_rows(
    lines: Vec<jayjay_core::diff::DiffLine>,
) -> Vec<jayjay_core::diff::SideBySideRow> {
    jayjay_core::diff::build_side_by_side_rows(&lines)
}
