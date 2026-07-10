mod tags;

use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

use self::tags::heading_tag;
use crate::MarkdownImageAlign;
use crate::images::{
    image_html, image_html_with_align, raw_html_image, raw_html_image_markup,
    raw_html_image_paragraph_end, raw_html_image_paragraph_start, sanitized_image_source,
};
use crate::text::{escape_html, render_text_with_bare_autolinks};

pub(crate) fn render_markdown_events_html<'a>(
    events: impl IntoIterator<Item = Event<'a>>,
) -> String {
    let events = events.into_iter().collect::<Vec<_>>();
    let mut renderer = HtmlRenderer::new(events);
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
    table_depth: usize,
    pending_raw_image_paragraph_align: Option<MarkdownImageAlign>,
    skip_raw_image_paragraph_end: bool,
}

impl<'a> HtmlRenderer<'a> {
    fn new(events: Vec<Event<'a>>) -> Self {
        Self {
            events,
            index: 0,
            output: String::new(),
            heading_ids: std::collections::HashMap::new(),
            link_stack: Vec::new(),
            pending_item_starts: 0,
            table_head_depth: 0,
            table_depth: 0,
            pending_raw_image_paragraph_align: None,
            skip_raw_image_paragraph_end: false,
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
                if let Some(source) = sanitized_image_source(dest_url.as_ref()) {
                    self.output
                        .push_str(&image_html(&source, &alt, Some(title.as_ref())));
                } else {
                    self.output.push_str(&escape_html(&alt));
                }
                self.index = matching_end_index(&self.events, self.index, TagEnd::Image)
                    .unwrap_or(self.index);
            }
            Event::Start(tag) => self.render_start_tag(&tag),
            Event::End(tag) => self.render_end_tag(tag),
            Event::Text(text) => {
                self.flush_pending_items();
                self.output
                    .push_str(&render_text_with_bare_autolinks(text.as_ref()));
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
        if self.skip_raw_image_paragraph_end && raw_html_image_paragraph_end(html) {
            self.skip_raw_image_paragraph_end = false;
            return;
        }

        if let Some((image, rest)) = raw_html_image_markup(html) {
            self.output.push_str(&image);
            self.render_raw_html_rest(rest);
            return;
        }

        if let Some(align) = raw_html_image_paragraph_start(html) {
            self.pending_raw_image_paragraph_align = Some(align);
            return;
        }

        if let Some(align) = self.pending_raw_image_paragraph_align {
            if let Some(image) = raw_html_image(html) {
                self.output.push_str(&image_html_with_align(
                    &image.source,
                    &image.alt,
                    image.title.as_deref(),
                    align,
                ));
                self.pending_raw_image_paragraph_align = None;
                self.skip_raw_image_paragraph_end = true;
                self.render_raw_html_rest(image.rest);
                return;
            }
            self.pending_raw_image_paragraph_align = None;
        }

        self.output.push_str(&escape_html(html));
    }

    fn render_raw_html_rest(&mut self, rest: &str) {
        let rest = rest.trim();
        if rest.is_empty() {
            return;
        }
        if self.skip_raw_image_paragraph_end && raw_html_image_paragraph_end(rest) {
            self.skip_raw_image_paragraph_end = false;
        } else {
            self.output.push_str(&escape_html(rest));
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
