use pulldown_cmark::{CodeBlockKind, HeadingLevel, Tag, TagEnd};

use super::model::{MarkdownBlock, MarkdownListItem};
use super::parser::{
    BlockParser, CodeBuilder, ListBuilder, TableBuilder, TableRowBuilder, TextKind,
};

impl BlockParser {
    pub(super) fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => self.start_text(if self.quote_depth > 0 {
                TextKind::BlockQuote
            } else {
                TextKind::Paragraph
            }),
            Tag::Heading { level, .. } => self.start_text(TextKind::Heading(heading_level(level))),
            Tag::BlockQuote(_) => self.quote_depth += 1,
            Tag::CodeBlock(kind) => {
                self.flush_text();
                self.code = Some(CodeBuilder {
                    language: code_block_language(&kind),
                    text: String::new(),
                });
            }
            Tag::List(start) => self.list_stack.push(ListBuilder {
                start,
                items: Vec::new(),
            }),
            Tag::Item => {
                self.item = Some(MarkdownListItem {
                    checked: None,
                    text: String::new(),
                });
            }
            Tag::Table(_) => {
                self.flush_text();
                self.table = Some(TableBuilder::default());
            }
            Tag::TableHead => {
                if let Some(table) = self.table.as_mut() {
                    table.in_head = true;
                }
            }
            Tag::TableRow => self.start_table_row(),
            Tag::TableCell => {
                if self.row.is_none() {
                    self.start_table_row();
                }
                self.cell = Some(String::new());
            }
            Tag::Image { dest_url, .. } => {
                self.image_destinations.push(dest_url.to_string());
                self.append_text("Image: ");
            }
            Tag::HtmlBlock
            | Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Superscript
            | Tag::Subscript
            | Tag::Link { .. }
            | Tag::MetadataBlock(_) => {}
        }
    }

    pub(super) fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::HtmlBlock => self.flush_text(),
            TagEnd::BlockQuote(_) => {
                self.flush_text();
                self.quote_depth = self.quote_depth.saturating_sub(1);
            }
            TagEnd::CodeBlock => {
                if let Some(code) = self.code.take() {
                    self.push_block(MarkdownBlock::CodeBlock {
                        language: code.language,
                        text: code.text,
                    });
                }
            }
            TagEnd::Item => {
                if let Some(mut item) = self.item.take() {
                    item.text = item.text.trim().to_owned();
                    if let Some(list) = self.list_stack.last_mut() {
                        list.items.push(item);
                    }
                }
            }
            TagEnd::List(_) => {
                if let Some(list) = self.list_stack.pop() {
                    self.push_block(MarkdownBlock::List {
                        start: list.start,
                        items: list.items,
                    });
                }
            }
            TagEnd::TableCell => {
                if let Some(cell) = self.cell.take()
                    && let Some(row) = self.row.as_mut()
                {
                    row.cells.push(cell.trim().to_owned());
                }
            }
            TagEnd::TableRow => self.flush_table_row(),
            TagEnd::TableHead => {
                self.flush_table_row();
                if let Some(table) = self.table.as_mut() {
                    table.in_head = false;
                }
            }
            TagEnd::Table => {
                self.flush_table_row();
                if let Some(table) = self.table.take() {
                    self.push_block(MarkdownBlock::Table { rows: table.rows });
                }
            }
            TagEnd::Image => {
                if let Some(destination) = self.image_destinations.pop()
                    && !destination.is_empty()
                {
                    self.append_text(" (");
                    self.append_text(&destination);
                    self.append_text(")");
                }
            }
            TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript
            | TagEnd::Link
            | TagEnd::MetadataBlock(_) => {}
        }
    }

    fn start_table_row(&mut self) {
        let header = self.table.as_ref().is_some_and(|table| table.in_head);
        if self.row.is_none() {
            self.row = Some(TableRowBuilder {
                header,
                cells: Vec::new(),
            });
        }
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn code_block_language(kind: &CodeBlockKind<'_>) -> Option<String> {
    match kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(language) => {
            let language = language.trim();
            (!language.is_empty()).then(|| language.to_owned())
        }
    }
}
