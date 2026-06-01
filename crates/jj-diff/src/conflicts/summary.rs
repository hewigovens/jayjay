use crate::types::{ConflictBlock, ConflictLineKind};

const SOURCE_LABEL_MAX_CHARS: usize = 56;

pub(super) fn conflict_summary(block: &ConflictBlock) -> String {
    let mut parts = vec![block.title.clone()];
    for section in &block.sections {
        if section.kind != ConflictLineKind::Section {
            continue;
        }
        let label = summary_label(&section.label);
        if !parts.iter().any(|part| part == &label) {
            parts.push(label);
        }
    }
    parts.join(" · ")
}

fn summary_label(label: &str) -> String {
    for (prefix, glyph) in [
        ("Base: ", "◇"),
        ("Destination: ", "→"),
        ("Rebased: ", "↻"),
        ("Side: ", "◆"),
    ] {
        if let Some(rest) = label.strip_prefix(prefix) {
            return format!("{glyph} {}", compact_source(rest));
        }
    }
    match label {
        "Base" => "◇ base".to_owned(),
        "Destination" => "→ destination".to_owned(),
        "Rebased" => "↻ rebased".to_owned(),
        "Side" => "◆ side".to_owned(),
        _ if label.starts_with("Side #") => format!("◆ {}", compact_source(label)),
        _ => compact_source(label),
    }
}

fn compact_source(raw: &str) -> String {
    let text = raw
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    truncate_chars(text, SOURCE_LABEL_MAX_CHARS)
}

fn truncate_chars(text: String, max_chars: usize) -> String {
    let mut chars = text.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        text
    }
}
