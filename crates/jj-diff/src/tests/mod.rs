use super::*;

mod basic;
mod collapse;
mod eof;
mod performance;
mod trim;
mod word;

fn span_info(line: &DiffLine) -> Vec<(&str, DiffSpanStyle)> {
    line.spans
        .iter()
        .map(|s| (s.text.as_str(), s.style))
        .collect()
}
