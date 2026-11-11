mod tags;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Tag, TagEnd};

use self::tags::heading_tag;
use crate::images::{
    ImageSize, MarkdownImages, RawHtml, RawImageParagraphs, image_html, keeps_raw_image_wrapper,
};
use crate::raw_html::RawHtmlSanitizer;
use crate::text::{escape_html, render_text_with_bare_autolinks};

pub(crate) fn render_markdown_events_html<'a>(
    events: impl IntoIterator<Item = Event<'a>>,
    images: MarkdownImages,
) -> String {
    let events = events.into_iter().collect::<Vec<_>>();
    let mut renderer = HtmlRenderer::new(events, images);
    renderer.render()
}

struct HtmlRenderer<'a> {
    events: Vec<Event<'a>>,
    index: usize,
    output: String,
    heading_ids: std::collections::HashMap<String, usize>,
    link_stack: Vec<bool>,
    pending_item_starts: usize,
    table_head_depth: usize,
    images: MarkdownImages,
    raw_images: RawImageParagraphs,
    raw_html: RawHtmlSanitizer,
}

impl<'a> HtmlRenderer<'a> {
    fn new(events: Vec<Event<'a>>, images: MarkdownImages) -> Self {
        Self {
            events,
            index: 0,
            output: String::new(),
            heading_ids: std::collections::HashMap::new(),
            link_stack: Vec::new(),
            pending_item_starts: 0,
            table_head_depth: 0,
            images,
            raw_images: RawImageParagraphs::default(),
            raw_html: RawHtmlSanitizer::new(images),
        }
    }

    fn render(&mut self) -> String {
        while self.index < self.events.len() {
            self.render_current_event();
            self.index += 1;
        }
        std::mem::take(&mut self.output)
    }

    fn render_current_event(&mut self) {
        let event = self.events[self.index].clone();
        if !keeps_raw_image_wrapper(&event)
            && let Some(opening) = self.raw_images.take_unmatched()
        {
            self.output.push_str(&self.raw_html.sanitize(&opening));
        }
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                self.flush_pending_items();
                let text = self.plain_text_until_heading_end(self.index + 1, level);
                let id = self.heading_id(&text);
                self.output
                    .push_str(&format!("<{} id=\"{}\">", heading_tag(level), id));
            }
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                self.flush_pending_items();
                let alt = self.plain_text_until_image_end(self.index + 1);
                if let Some(source) = self.images.sanitized_source(dest_url.as_ref()) {
                    self.output.push_str(&image_html(
                        &source,
                        &alt,
                        Some(title.as_ref()),
                        &ImageSize::default(),
                    ));
                } else {
                    self.output.push_str(&escape_html(&alt));
                }
                self.index = matching_end_index(&self.events, self.index, TagEnd::Image)
                    .unwrap_or(self.index);
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))
                if crate::MermaidDiagram::is_language(&language) =>
            {
                self.flush_pending_items();
                let end = matching_end_index(&self.events, self.index, TagEnd::CodeBlock)
                    .unwrap_or(self.index);
                let source = self.events[self.index + 1..end]
                    .iter()
                    .filter_map(|event| match event {
                        Event::Text(text) => Some(text.as_ref()),
                        _ => None,
                    })
                    .collect::<String>();
                if let Some(diagram) = crate::MermaidDiagram::render(&source, self.index) {
                    self.output.push_str("<div class=\"mermaid-diagram\">");
                    self.output.push_str(&diagram.html_svg());
                    self.output.push_str("</div>");
                } else {
                    self.output
                        .push_str("<pre><code class=\"language-mermaid\">");
                    self.output.push_str(&escape_html(&source));
                    self.output.push_str("</code></pre>");
                }
                self.index = end;
            }
            Event::Start(tag) => self.render_start_tag(&tag),
            Event::End(tag) => self.render_end_tag(tag),
            Event::Text(text) => {
                self.flush_pending_items();
                let text = if self.raw_html.in_link() {
                    escape_html(text.as_ref())
                } else {
                    render_text_with_bare_autolinks(text.as_ref())
                };
                self.output.push_str(&text);
            }
            Event::Code(code) => {
                self.flush_pending_items();
                self.output
                    .push_str(&format!("<code>{}</code>", escape_html(code.as_ref())));
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                self.flush_pending_items();
                self.render_raw_html(html.as_ref());
            }
            Event::SoftBreak => {
                self.flush_pending_items();
                self.output.push('\n');
            }
            Event::HardBreak => {
                self.flush_pending_items();
                self.output.push_str("<br>");
            }
            Event::Rule => {
                self.flush_pending_items();
                self.output.push_str("<hr>");
            }
            Event::TaskListMarker(checked) => self.render_task_list_marker(checked),
            Event::FootnoteReference(reference) => {
                self.flush_pending_items();
                self.output
                    .push_str(&format!("<sup>{}</sup>", escape_html(reference.as_ref())));
            }
            Event::InlineMath(math) => {
                self.flush_pending_items();
                self.output
                    .push_str(&format!("<code>{}</code>", escape_html(math.as_ref())));
            }
            Event::DisplayMath(math) => {
                self.flush_pending_items();
                self.output.push_str(&format!(
                    "<pre><code>{}</code></pre>",
                    escape_html(math.as_ref())
                ));
            }
        }
    }

    fn render_task_list_marker(&mut self, checked: bool) {
        if self.pending_item_starts > 0 {
            self.pending_item_starts -= 1;
            self.output.push_str("<li class=\"task-list-item\">");
        } else {
            self.flush_pending_items();
        }

        let checked_attribute = if checked { " checked" } else { "" };
        self.output.push_str(&format!(
            "<input type=\"checkbox\" disabled{}> ",
            checked_attribute
        ));
    }

    fn render_raw_html(&mut self, html: &str) {
        if self.raw_html.in_comment() {
            self.output.push_str(&self.raw_html.sanitize(html));
            return;
        }
        match self.raw_images.classify(html, self.images) {
            RawHtml::Image { image, align, rest } => {
                self.output.push_str(&image.html(align));
                if let Some(rest) = rest {
                    self.output.push_str(&self.raw_html.sanitize(rest));
                }
            }
            RawHtml::Skip => {}
            RawHtml::Text(text) => self.output.push_str(&self.raw_html.sanitize(&text)),
        }
    }

    fn flush_pending_items(&mut self) {
        for _ in 0..self.pending_item_starts {
            self.output.push_str("<li>");
        }
        self.pending_item_starts = 0;
    }

    fn plain_text_until_heading_end(&self, start: usize, level: HeadingLevel) -> String {
        plain_text_until(&self.events, start, TagEnd::Heading(level))
    }

    fn plain_text_until_image_end(&self, start: usize) -> String {
        plain_text_until(&self.events, start, TagEnd::Image)
    }

    fn heading_id(&mut self, text: &str) -> String {
        let mut slug = String::new();
        for ch in text.to_lowercase().chars() {
            if ch.is_alphanumeric() {
                slug.push(ch);
            } else if !slug.ends_with('-') {
                slug.push('-');
            }
        }

        let base = slug.trim_matches('-');
        let base = if base.is_empty() { "section" } else { base };
        let count = self.heading_ids.entry(base.to_owned()).or_insert(0);
        *count += 1;
        if *count == 1 {
            base.to_owned()
        } else {
            format!("{}-{}", base, count)
        }
    }
}

fn plain_text_until(events: &[Event<'_>], start: usize, end_tag: TagEnd) -> String {
    let mut text = String::new();
    let mut depth = 0usize;
    for event in &events[start..] {
        match event {
            Event::Start(_) => depth += 1,
            Event::End(tag) if depth == 0 && *tag == end_tag => break,
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Text(value)
            | Event::Code(value)
            | Event::InlineMath(value)
            | Event::DisplayMath(value) => text.push_str(value.as_ref()),
            Event::SoftBreak | Event::HardBreak => text.push(' '),
            _ => {}
        }
    }
    text
}

fn matching_end_index(events: &[Event<'_>], start: usize, end_tag: TagEnd) -> Option<usize> {
    let mut depth = 0usize;
    for (index, event) in events.iter().enumerate().skip(start + 1) {
        match event {
            Event::Start(_) => depth += 1,
            Event::End(tag) if depth == 0 && *tag == end_tag => return Some(index),
            Event::End(_) => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}
