mod summary;

use summary::conflict_summary;

use crate::syntax::SyntaxToken;
use crate::types::{
    ConflictBlock, ConflictBlockSection, ConflictLineKind, DiffDisplayItem, DiffLine, DiffSpan,
    DiffSpanStyle,
};

const BASE_MARKER: &str = "◇";
const DESTINATION_MARKER: &str = "→";
const REBASED_MARKER: &str = "←";
const SIDE_MARKER: &str = "◆";

const BASE_CONTENT_PREFIX: &str = "◇ │ ";
const DESTINATION_CONTENT_PREFIX: &str = "→ │ ";
const REBASED_CONTENT_PREFIX: &str = "← │ ";
const SIDE_CONTENT_PREFIX: &str = "◆ │ ";

pub fn annotate_conflict_lines(lines: &mut [DiffLine]) {
    let mut open_start: Option<usize> = None;

    for (index, line) in lines.iter_mut().enumerate() {
        let text = line.text();
        if is_marker(&text, '<') {
            line.conflict_kind = ConflictLineKind::Start;
            open_start = Some(open_start.unwrap_or(index));
            continue;
        }

        if open_start.is_some() && is_marker(&text, '>') {
            line.conflict_kind = ConflictLineKind::End;
            open_start = None;
            continue;
        }

        if open_start.is_none() {
            line.conflict_kind = ConflictLineKind::None;
            continue;
        }

        line.conflict_kind = if is_section_marker(&text) {
            ConflictLineKind::Section
        } else if text.starts_with('-') {
            ConflictLineKind::Removed
        } else if text.starts_with('+') {
            ConflictLineKind::Added
        } else {
            ConflictLineKind::Content
        };
    }

    // A block that never closes is quoted marker text, not a conflict; annotating it would conflict-style and pin the file's whole tail.
    if let Some(start) = open_start {
        for line in &mut lines[start..] {
            line.conflict_kind = ConflictLineKind::None;
        }
    }
}

pub fn conflict_display_text(kind: ConflictLineKind, raw: &str) -> Option<String> {
    match kind {
        ConflictLineKind::None
        | ConflictLineKind::Content
        | ConflictLineKind::Removed
        | ConflictLineKind::Added => None,
        ConflictLineKind::Start | ConflictLineKind::End | ConflictLineKind::Section => {
            let label = marker_payload(raw)?;
            Some(normalize_label(label))
        }
    }
}

pub fn build_diff_display_items(lines: &[DiffLine]) -> Vec<DiffDisplayItem> {
    let mut items = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        if is_raw_conflict_start(lines, index) {
            let block = conflict_block(lines, index);
            index = block.line_end as usize;
            items.push(DiffDisplayItem::ConflictBlock { block });
            continue;
        }

        let start = index;
        while index < lines.len() && !is_raw_conflict_start(lines, index) {
            index += 1;
        }
        if start < index {
            items.push(DiffDisplayItem::Lines {
                line_start: start as u32,
                line_end: index as u32,
            });
        }
    }

    items
}

pub fn build_diff_display_lines(lines: &[DiffLine]) -> Vec<DiffLine> {
    let mut display_lines = Vec::new();
    for item in build_diff_display_items(lines) {
        match item {
            DiffDisplayItem::Lines {
                line_start,
                line_end,
            } => {
                let start = display_lines.len();
                display_lines.extend_from_slice(&lines[line_start as usize..line_end as usize]);
                // Keep raw ordering for word pairs and persisted review fingerprints; group only display rows.
                for group in display_lines[start..].chunk_by_mut(|left, right| {
                    left.is_changed()
                        && right.is_changed()
                        && left.conflict_kind == ConflictLineKind::None
                        && right.conflict_kind == ConflictLineKind::None
                }) {
                    group.sort_by_key(|line| line.style == DiffSpanStyle::Added);
                }
            }
            DiffDisplayItem::ConflictBlock { block } => {
                display_lines.push(conflict_summary_line(lines, &block));
                for section in &block.sections {
                    display_lines.extend(
                        lines[section.content_start as usize..section.line_end as usize]
                            .iter()
                            .map(|line| conflict_content_line(line, &section.label)),
                    );
                }
            }
        }
    }
    display_lines
}

fn conflict_content_line(line: &DiffLine, section_label: &str) -> DiffLine {
    let Some(prefix) = conflict_content_prefix(line, section_label) else {
        return line.clone();
    };
    let mut line = line.clone();
    line.spans.insert(
        0,
        DiffSpan {
            text: prefix.to_owned(),
            style: DiffSpanStyle::Unchanged,
            token: SyntaxToken::Plain,
        },
    );
    line
}

fn conflict_content_prefix(line: &DiffLine, section_label: &str) -> Option<&'static str> {
    if section_label == "Base" || section_label.starts_with("Base:") {
        Some(BASE_CONTENT_PREFIX)
    } else if section_label == "Destination" || section_label.starts_with("Destination:") {
        match line.conflict_kind {
            ConflictLineKind::Removed => Some(BASE_CONTENT_PREFIX),
            ConflictLineKind::Added => Some(DESTINATION_CONTENT_PREFIX),
            _ => Some(DESTINATION_CONTENT_PREFIX),
        }
    } else if section_label == "Rebased" || section_label.starts_with("Rebased:") {
        Some(REBASED_CONTENT_PREFIX)
    } else if section_label == "Side"
        || section_label.starts_with("Side ")
        || section_label.starts_with("Side:")
    {
        Some(SIDE_CONTENT_PREFIX)
    } else {
        None
    }
}

fn conflict_block(lines: &[DiffLine], start: usize) -> ConflictBlock {
    let mut end = start + 1;
    while end < lines.len() {
        if lines[end].conflict_kind == ConflictLineKind::End {
            end += 1;
            break;
        }
        end += 1;
    }

    let mut sections = Vec::new();
    let mut marker = start;
    while marker < end {
        let kind = lines[marker].conflict_kind;
        if !is_block_marker(kind) {
            marker += 1;
            continue;
        }

        let mut next_marker = marker + 1;
        while next_marker < end && !is_block_marker(lines[next_marker].conflict_kind) {
            next_marker += 1;
        }

        let raw = lines[marker].text();
        sections.push(ConflictBlockSection {
            label: conflict_display_text(kind, &raw).unwrap_or_else(|| default_section_label(kind)),
            marker_line: marker as u32,
            content_start: (marker + 1) as u32,
            line_end: next_marker as u32,
            kind,
        });
        marker = next_marker;
    }

    let title = sections
        .first()
        .map(|section| section.label.clone())
        .unwrap_or_else(|| "Conflict".to_owned());
    ConflictBlock {
        title,
        line_start: start as u32,
        line_end: end as u32,
        sections,
    }
}

fn conflict_summary_line(lines: &[DiffLine], block: &ConflictBlock) -> DiffLine {
    let mut line = lines[block.line_start as usize].clone();
    line.old_line_no = None;
    line.new_line_no = None;
    line.style = DiffSpanStyle::Context;
    line.conflict_kind = ConflictLineKind::Start;
    line.no_eof_newline = false;
    line.spans = vec![DiffSpan {
        text: conflict_summary(block),
        style: DiffSpanStyle::Unchanged,
        token: SyntaxToken::Plain,
    }];
    line
}

fn is_raw_conflict_start(lines: &[DiffLine], index: usize) -> bool {
    lines[index].conflict_kind == ConflictLineKind::Start && is_marker(&lines[index].text(), '<')
}

fn is_block_marker(kind: ConflictLineKind) -> bool {
    matches!(
        kind,
        ConflictLineKind::Start | ConflictLineKind::Section | ConflictLineKind::End
    )
}

fn default_section_label(kind: ConflictLineKind) -> String {
    match kind {
        ConflictLineKind::Start => "Conflict".to_owned(),
        ConflictLineKind::End => "Conflict ends".to_owned(),
        ConflictLineKind::Section => "Conflict section".to_owned(),
        _ => "Conflict content".to_owned(),
    }
}

fn is_section_marker(text: &str) -> bool {
    ['%', '\\', '+']
        .iter()
        .any(|marker| is_marker(text, *marker))
}

fn is_marker(text: &str, marker: char) -> bool {
    text.chars().take_while(|ch| *ch == marker).count() >= 7
}

fn marker_payload(raw: &str) -> Option<&str> {
    let marker = raw.chars().next()?;
    let mut count = 0;
    for ch in raw.chars() {
        if ch == marker {
            count += ch.len_utf8();
        } else {
            break;
        }
    }
    if count < 7 {
        return None;
    }
    let label = raw[count..].trim();
    if label.is_empty() { None } else { Some(label) }
}

fn normalize_label(label: &str) -> String {
    let label = label.trim();
    if let Some(rest) = label.strip_prefix("diff from:") {
        return source_label("Base", rest);
    }
    if let Some(rest) = label.strip_prefix("to:") {
        return source_label("Destination", rest);
    }
    if let Some(conflict) = label.strip_suffix(" ends") {
        return format!("End {}", capitalize_label(conflict));
    }
    if label.contains("(rebased revision)") {
        return source_label("Rebased", label);
    }
    if let Some(description) = quoted_description(label) {
        return format!("Side: {description}");
    }
    capitalize_label(label)
}

fn source_label(prefix: &str, raw: &str) -> String {
    quoted_description(raw)
        .map(|description| format!("{prefix}: {description}"))
        .unwrap_or_else(|| prefix.to_owned())
}

fn quoted_description(raw: &str) -> Option<&str> {
    let start = raw.find('"')? + 1;
    let end = raw[start..].find('"')?;
    let description = raw[start..start + end].trim();
    (!description.is_empty()).then_some(description)
}

fn capitalize_label(label: &str) -> String {
    let mut chars = label.chars();
    let Some(first) = chars.next() else {
        return label.to_owned();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}

#[cfg(test)]
mod tests;
