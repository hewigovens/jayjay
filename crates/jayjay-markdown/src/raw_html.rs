use crate::MarkdownImageAlign;
use crate::images::{MarkdownImages, raw_html_image, unescape_html_attribute};
use crate::text::{character_reference, escape_html, sanitized_link};

/// Formatting tags GitHub keeps in Markdown, such as Dependabot's `<details>` release notes; besides these and an `<img>` with an allowed source, markup is shown as text.
const ALLOWED_TAGS: &[&str] = &[
    "a",
    "b",
    "blockquote",
    "br",
    "code",
    "dd",
    "del",
    "details",
    "div",
    "dl",
    "dt",
    "em",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "kbd",
    "li",
    "ol",
    "p",
    "pre",
    "s",
    "span",
    "strong",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

/// Raw HTML from Markdown with allowed tags rebuilt from their safe attributes and everything else escaped; a comment or link stays open across events.
#[derive(Default)]
pub(crate) struct RawHtmlSanitizer {
    in_comment: bool,
    open_links: usize,
    images: MarkdownImages,
}

impl RawHtmlSanitizer {
    pub(crate) fn new(images: MarkdownImages) -> Self {
        Self {
            images,
            ..Self::default()
        }
    }

    pub(crate) fn in_comment(&self) -> bool {
        self.in_comment
    }

    /// Whether text is inside a raw `<a>`, where an autolink would nest anchors.
    pub(crate) fn in_link(&self) -> bool {
        self.open_links > 0
    }

    pub(crate) fn sanitize(&mut self, html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut rest = html;
        loop {
            if self.in_comment {
                let Some(end) = rest.find("-->") else {
                    return output;
                };
                rest = &rest[end + 3..];
                self.in_comment = false;
            }
            let Some(start) = rest.find(['<', '&', '>', '"']) else {
                break;
            };
            output.push_str(&rest[..start]);
            rest = &rest[start..];
            if let Some(after) = rest.strip_prefix("<!--") {
                self.in_comment = true;
                rest = after;
            } else if let Some((tag, after)) = rest
                .starts_with('<')
                .then(|| allowed_tag(rest, self.images))
                .flatten()
            {
                if tag == "</a>" {
                    self.open_links = self.open_links.saturating_sub(1);
                } else if tag.starts_with("<a ") {
                    self.open_links += 1;
                }
                output.push_str(&tag);
                rest = after;
            } else if let Some(name) = character_reference(rest) {
                output.push_str(&rest[..name.len() + 2]);
                rest = &rest[name.len() + 2..];
            } else {
                output.push_str(&escape_html(&rest[..1]));
                rest = &rest[1..];
            }
        }
        output.push_str(rest);
        output
    }
}

/// An allowed start or end tag at the front of `input`, rebuilt, and the input after it.
fn allowed_tag(input: &str, images: MarkdownImages) -> Option<(String, &str)> {
    let end = tag_end(input)?;
    let inner = &input[1..end];
    let (closing, inner) = match inner.strip_prefix('/') {
        Some(inner) => (true, inner),
        None => (false, inner),
    };
    let name_end = inner
        .find(|ch: char| ch.is_ascii_whitespace() || ch == '/')
        .unwrap_or(inner.len());
    let name = inner[..name_end].to_ascii_lowercase();
    if name == "img" && !closing {
        let image = raw_html_image(&input[..=end], images)?;
        return Some((image.html(MarkdownImageAlign::None), &input[end + 1..]));
    }
    if !ALLOWED_TAGS.contains(&name.as_str()) {
        return None;
    }
    let rebuilt = if closing {
        format!("</{name}>")
    } else {
        let attributes: String = attributes(&inner[name_end..])
            .into_iter()
            .filter_map(|(key, value)| allowed_attribute(&name, &key, value.as_deref()))
            .collect();
        let rel = if name == "a" {
            " rel=\"noopener noreferrer\""
        } else {
            ""
        };
        format!("<{name}{attributes}{rel}>")
    };
    Some((rebuilt, &input[end + 1..]))
}

fn allowed_attribute(tag: &str, key: &str, value: Option<&str>) -> Option<String> {
    match (tag, key, value) {
        ("a", "href", Some(href)) => {
            let href = unescape_html_attribute(href);
            sanitized_link(href.trim()).map(|href| format!(" href=\"{}\"", escape_html(href)))
        }
        ("details", "open", _) => Some(" open".into()),
        (
            "td" | "th" | "p" | "div" | "h1" | "h2" | "h3",
            "align",
            Some(align @ ("left" | "center" | "right")),
        ) => Some(format!(" align=\"{align}\"")),
        ("ol", "start", Some(start))
            if !start.is_empty() && start.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            Some(format!(" start=\"{start}\""))
        }
        _ => None,
    }
}

/// The index of the `>` closing the tag that opens `input`, skipping `>` inside quoted values.
fn tag_end(input: &str) -> Option<usize> {
    let mut quote = None;
    for (index, ch) in input.char_indices().skip(1) {
        match (quote, ch) {
            (Some(open), _) if ch == open => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(ch),
            (None, '>') => return Some(index),
            (None, '<') => return None,
            _ => {}
        }
    }
    None
}

/// Lowercased attribute names with their unquoted values; a bare name has no value.
fn attributes(input: &str) -> Vec<(String, Option<String>)> {
    let mut found = Vec::new();
    let mut rest = input;
    loop {
        rest = rest.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '/');
        if rest.is_empty() {
            break;
        }
        let name_end = rest
            .find(|ch: char| ch.is_whitespace() || ch == '=' || ch == '/')
            .unwrap_or(rest.len());
        let name = rest[..name_end].to_ascii_lowercase();
        rest = rest[name_end..].trim_start();
        let value = if let Some(after) = rest.strip_prefix('=') {
            let after = after.trim_start();
            let (value, remaining) = match after.chars().next() {
                Some(quote @ ('"' | '\'')) => {
                    let close = after[1..].find(quote).map_or(after.len(), |end| end + 1);
                    (&after[1..close], after.get(close + 1..).unwrap_or(""))
                }
                _ => {
                    let end = after.find(char::is_whitespace).unwrap_or(after.len());
                    (&after[..end], &after[end..])
                }
            };
            rest = remaining.trim_start();
            Some(value.to_owned())
        } else {
            None
        };
        if !name.is_empty() {
            found.push((name, value));
        }
    }
    found
}
