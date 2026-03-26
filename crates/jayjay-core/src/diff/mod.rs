mod compute;
mod context;
mod highlights;
mod line_diff;
mod types;
mod word_diff;

#[cfg(test)]
mod tests;

pub use compute::{compute_file_diff, compute_file_diff_full};
pub use types::{DiffLine, DiffSpan, DiffSpanStyle, FileDiff};
