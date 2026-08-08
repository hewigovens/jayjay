use pulldown_cmark::{Event, Parser};

use super::model::{MarkdownBlock, MarkdownListItem, MarkdownTableRow, block_text};
use crate::images::raw_html_image;
use crate::markdown_options;

pub fn parse_markdown_blocks(markdown: &str) -> Vec<MarkdownBlock> {
    let mut parser = BlockParser::default();
    for event in Parser::new_ext(markdown, markdown_options()) {
        parser.handle(event);
    }
    parser.finish()
}

#[derive(Default)]
pub(super) struct BlockParser {
    blocks: Vec<MarkdownBlock>,
    text: Option<TextBuilder>,
    pub(super) code: Option<CodeBuilder>,
    pub(super) list_stack: Vec<ListBuilder>,
    pub(super) item: Option<MarkdownListItem>,
    pub(super) quote_depth: usize,
    pub(super) table: Option<TableBuilder>,
    pub(super) row: Option<TableRowBuilder>,
    pub(super) cell: Option<String>,
    pub(super) image_destinations: Vec<String>,
}

pub(super) struct TextBuilder {
    kind: TextKind,
    text: String,
}

#[derive(Clone, Copy)]
pub(super) enum TextKind {
    Paragraph,
    Heading(u8),
    BlockQuote,
}

pub(super) struct CodeBuilder {
    pub(super) language: Option<String>,
    pub(super) text: String,
}

pub(super) struct ListBuilder {
    pub(super) start: Option<u64>,
    pub(super) items: Vec<MarkdownListItem>,
}

#[derive(Default)]
pub(super) struct TableBuilder {
    pub(super) rows: Vec<MarkdownTableRow>,
    pub(super) in_head: bool,
}

pub(super) struct TableRowBuilder {
    pub(super) header: bool,
    pub(super) cells: Vec<String>,
}

impl BlockParser {
    fn handle(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.append_text(text.as_ref()),
            Event::Code(code) => {
                self.append_text("`");
                self.append_text(code.as_ref());
                self.append_text("`");
            }
            Event::InlineMath(math) => {
                self.append_text("$");
                self.append_text(math.as_ref());
                self.append_text("$");
            }
            Event::DisplayMath(math) => {
                self.flush_text();
                self.push_block(MarkdownBlock::CodeBlock {
                    language: Some("math".to_owned()),
                    text: math.to_string(),
                });
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                if let Some(image) = raw_html_image(html.as_ref()) {
                    self.push_block(MarkdownBlock::Image {
                        source: image.source,
                        alt: image.alt,
                        title: image.title,
                        align: image.align,
                    });
                } else {
                    self.append_text(html.as_ref());
                }
            }
            Event::FootnoteReference(label) => {
                self.append_text("[^");
                self.append_text(label.as_ref());
                self.append_text("]");
            }
            Event::SoftBreak => self.append_text(" "),
            Event::HardBreak => self.append_text("\n"),
            Event::Rule => {
                self.flush_text();
                self.push_block(MarkdownBlock::Rule);
            }
            Event::TaskListMarker(checked) => {
                if let Some(item) = self.item.as_mut() {
                    item.checked = Some(checked);
                }
            }
        }
    }

    pub(super) fn start_text(&mut self, kind: TextKind) {
        if self.cell.is_some() || self.item.is_some() || self.code.is_some() {
            return;
        }
        self.text = Some(TextBuilder {
            kind,
            text: String::new(),
        });
    }

    pub(super) fn append_text(&mut self, text: &str) {
        if let Some(cell) = self.cell.as_mut() {
            cell.push_str(text);
        } else if let Some(code) = self.code.as_mut() {
            code.text.push_str(text);
        } else if let Some(item) = self.item.as_mut() {
            item.text.push_str(text);
        } else {
            if self.text.is_none() {
                self.start_text(if self.quote_depth > 0 {
                    TextKind::BlockQuote
                } else {
                    TextKind::Paragraph
                });
            }
            if let Some(current) = self.text.as_mut() {
                current.text.push_str(text);
            }
        }
    }

    pub(super) fn flush_text(&mut self) {
        let Some(text) = self.text.take() else {
            return;
        };
        let value = text.text.trim();
        if value.is_empty() {
            return;
        }
        let block = match text.kind {
            TextKind::Paragraph => MarkdownBlock::Paragraph(value.to_owned()),
            TextKind::Heading(level) => MarkdownBlock::Heading {
                level,
                text: value.to_owned(),
            },
            TextKind::BlockQuote => MarkdownBlock::BlockQuote(value.to_owned()),
        };
        self.push_block(block);
    }

    pub(super) fn push_block(&mut self, block: MarkdownBlock) {
        if let Some(item) = self.item.as_mut() {
            append_nested_block(&mut item.text, &block);
        } else {
            self.blocks.push(block);
        }
    }

    pub(super) fn flush_table_row(&mut self) {
        if let Some(row) = self.row.take()
            && let Some(table) = self.table.as_mut()
        {
            table.rows.push(MarkdownTableRow {
                header: row.header,
                cells: row.cells,
            });
        }
    }

    fn finish(mut self) -> Vec<MarkdownBlock> {
        self.flush_text();
        self.blocks
    }
}

fn append_nested_block(target: &mut String, block: &MarkdownBlock) {
    let text = block_text(block);
    if text.is_empty() {
        return;
    }
    if !target.trim().is_empty() {
        target.push('\n');
    }
    target.push_str(&text);
}
