#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownDocument {
    source: String,
    blocks: Vec<MarkdownBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkdownBlock {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph(String),
    CodeBlock {
        language: Option<String>,
        text: String,
    },
    Image {
        source: String,
        alt: String,
        title: Option<String>,
        align: MarkdownImageAlign,
    },
    BlockQuote(String),
    List {
        start: Option<u64>,
        items: Vec<MarkdownListItem>,
    },
    Table {
        rows: Vec<MarkdownTableRow>,
    },
    Rule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownListItem {
    pub checked: Option<bool>,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownTableRow {
    pub header: bool,
    pub cells: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkdownImageAlign {
    #[default]
    None,
    Center,
}

impl MarkdownDocument {
    pub fn parse(markdown: impl Into<String>) -> Self {
        let source = markdown.into();
        let blocks = super::parse_markdown_blocks(&source);
        Self { source, blocks }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn blocks(&self) -> &[MarkdownBlock] {
        &self.blocks
    }
}

pub(super) fn block_text(block: &MarkdownBlock) -> String {
    match block {
        MarkdownBlock::Heading { text, .. }
        | MarkdownBlock::Paragraph(text)
        | MarkdownBlock::CodeBlock { text, .. }
        | MarkdownBlock::BlockQuote(text) => text.clone(),
        MarkdownBlock::Image { source, alt, .. } => image_text(source, alt),
        MarkdownBlock::List { start, items } => items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let marker = match item.checked {
                    Some(true) => "[x]".to_owned(),
                    Some(false) => "[ ]".to_owned(),
                    None => start
                        .map(|start| format!("{}.", start + index as u64))
                        .unwrap_or_else(|| "-".to_owned()),
                };
                format!("{marker} {}", item.text)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::Table { rows } => rows
            .iter()
            .map(|row| row.cells.join(" | "))
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::Rule => "----".to_owned(),
    }
}

fn image_text(source: &str, alt: &str) -> String {
    let label = if alt.is_empty() { "Image" } else { alt };
    format!("Image: {label} ({source})")
}
