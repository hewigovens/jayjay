use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Tag, TagEnd};

use super::HtmlRenderer;
use crate::text::{escape_html, sanitized_link};

impl<'a> HtmlRenderer<'a> {
    pub(super) fn render_start_tag(&mut self, tag: &Tag<'a>) {
        match tag {
            Tag::Paragraph => {
                self.flush_pending_items();
                self.output.push_str("<p>");
            }
            Tag::Heading { .. } | Tag::Image { .. } => {}
            Tag::BlockQuote(_) => {
                self.flush_pending_items();
                self.output.push_str("<blockquote>");
            }
            Tag::CodeBlock(kind) => self.render_code_block_start(kind),
            Tag::HtmlBlock => {}
            Tag::List(start) => self.render_list_start(*start),
            Tag::Item => {
                self.pending_item_starts += 1;
            }
            Tag::Emphasis => self.inline_start("em"),
            Tag::Strong => self.inline_start("strong"),
            Tag::Strikethrough => self.inline_start("del"),
            Tag::Superscript => self.inline_start("sup"),
            Tag::Subscript => self.inline_start("sub"),
            Tag::Link {
                dest_url, title, ..
            } => self.render_link_start(dest_url.as_ref(), title.as_ref()),
            Tag::Table(_) => {
                self.flush_pending_items();
                self.table_depth += 1;
                self.output.push_str("<table>");
            }
            Tag::TableHead => {
                self.flush_pending_items();
                self.table_head_depth += 1;
                self.output.push_str("<thead>");
            }
            Tag::TableRow => {
                self.flush_pending_items();
                self.output.push_str("<tr>");
            }
            Tag::TableCell => {
                self.flush_pending_items();
                self.output.push_str(if self.table_head_depth > 0 {
                    "<th>"
                } else {
                    "<td>"
                });
            }
            Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::MetadataBlock(_) => self.flush_pending_items(),
        }
    }

    pub(super) fn render_end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.output.push_str("</p>"),
            TagEnd::Heading(level) => self.output.push_str(&format!("</{}>", heading_tag(level))),
            TagEnd::BlockQuote(_) => self.output.push_str("</blockquote>"),
            TagEnd::CodeBlock => self.output.push_str("</code></pre>"),
            TagEnd::HtmlBlock => {}
            TagEnd::List(ordered) => self
                .output
                .push_str(if ordered { "</ol>" } else { "</ul>" }),
            TagEnd::Item => {
                self.flush_pending_items();
                self.output.push_str("</li>");
            }
            TagEnd::Emphasis => self.output.push_str("</em>"),
            TagEnd::Strong => self.output.push_str("</strong>"),
            TagEnd::Strikethrough => self.output.push_str("</del>"),
            TagEnd::Superscript => self.output.push_str("</sup>"),
            TagEnd::Subscript => self.output.push_str("</sub>"),
            TagEnd::Link => {
                if self.link_stack.pop().unwrap_or(false) {
                    self.output.push_str("</a>");
                }
            }
            TagEnd::Image => {}
            TagEnd::Table => {
                self.table_depth = self.table_depth.saturating_sub(1);
                self.output.push_str("</tbody></table>");
            }
            TagEnd::TableHead => {
                self.table_head_depth = self.table_head_depth.saturating_sub(1);
                self.output.push_str("</thead><tbody>");
            }
            TagEnd::TableRow => self.output.push_str("</tr>"),
            TagEnd::TableCell => self.output.push_str(if self.table_head_depth > 0 {
                "</th>"
            } else {
                "</td>"
            }),
            TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::MetadataBlock(_) => {}
        }
    }

    fn render_code_block_start(&mut self, kind: &CodeBlockKind<'a>) {
        self.flush_pending_items();
        self.output.push_str("<pre><code");
        if let CodeBlockKind::Fenced(language) = kind {
            let class = sanitized_language_class(language.as_ref());
            if !class.is_empty() {
                self.output
                    .push_str(&format!(" class=\"language-{}\"", class));
            }
        }
        self.output.push('>');
    }

    fn render_list_start(&mut self, start: Option<u64>) {
        self.flush_pending_items();
        if let Some(start) = start {
            self.output.push_str(&format!("<ol start=\"{}\">", start));
        } else if list_contains_task_marker(&self.events, self.index + 1) {
            self.output.push_str("<ul class=\"contains-task-list\">");
        } else {
            self.output.push_str("<ul>");
        }
    }

    fn inline_start(&mut self, tag: &str) {
        self.flush_pending_items();
        self.output.push_str(&format!("<{tag}>"));
    }

    fn render_link_start(&mut self, dest_url: &str, title: &str) {
        self.flush_pending_items();
        if let Some(href) = sanitized_link(dest_url) {
            self.link_stack.push(true);
            self.output
                .push_str(&format!("<a href=\"{}\"", escape_html(href)));
            if !title.is_empty() {
                self.output
                    .push_str(&format!(" title=\"{}\"", escape_html(title)));
            }
            self.output.push('>');
        } else {
            self.link_stack.push(false);
        }
    }
}

pub(super) fn heading_tag(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "h1",
        HeadingLevel::H2 => "h2",
        HeadingLevel::H3 => "h3",
        HeadingLevel::H4 => "h4",
        HeadingLevel::H5 => "h5",
        HeadingLevel::H6 => "h6",
    }
}

fn list_contains_task_marker(events: &[Event<'_>], start: usize) -> bool {
    let mut depth = 0usize;
    for event in &events[start..] {
        match event {
            Event::Start(Tag::List(_)) => depth += 1,
            Event::End(TagEnd::List(false)) if depth == 0 => return false,
            Event::End(TagEnd::List(_)) => depth = depth.saturating_sub(1),
            Event::TaskListMarker(_) => return true,
            _ => {}
        }
    }
    false
}

fn sanitized_language_class(language: &str) -> String {
    language
        .chars()
        .take(32)
        .filter(|ch| ch.is_alphanumeric() || *ch == '-' || *ch == '_')
        .collect()
}
