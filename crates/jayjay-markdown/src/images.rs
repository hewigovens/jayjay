use std::borrow::Cow;

use crate::MarkdownImageAlign;
use crate::text::{character_reference, escape_html, url_scheme};

/// Where a rendered image may load from; inline `data:` images are allowed either way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkdownImages {
    /// Document-relative paths, for previewing a checkout file.
    #[default]
    Local,
    /// `https` URLs only, for a web page.
    Remote,
}

pub(crate) struct RawHtmlImage<'a> {
    pub(crate) source: String,
    pub(crate) alt: String,
    pub(crate) title: Option<String>,
    pub(crate) size: ImageSize,
    pub(crate) align: MarkdownImageAlign,
    pub(crate) rest: &'a str,
}

#[derive(Default)]
pub(crate) struct ImageSize {
    width: Option<String>,
    height: Option<String>,
}

/// A `<p align>` wrapper that pulldown splits across raw HTML events: images inside keep its alignment until the swallowed `</p>`; a wrapper that never held an image is replayed as text.
#[derive(Default)]
pub(crate) struct RawImageParagraphs {
    wrapper: Option<Wrapper>,
}

struct Wrapper {
    align: MarkdownImageAlign,
    opening: String,
    matched: bool,
}

pub(crate) fn keeps_raw_image_wrapper(event: &pulldown_cmark::Event<'_>) -> bool {
    use pulldown_cmark::Event;
    match event {
        Event::Html(_) | Event::InlineHtml(_) | Event::SoftBreak => true,
        Event::Text(text) => text.trim().is_empty(),
        _ => false,
    }
}

pub(crate) enum RawHtml<'a> {
    Image {
        image: RawHtmlImage<'a>,
        align: MarkdownImageAlign,
        rest: Option<&'a str>,
    },
    Skip,
    Text(Cow<'a, str>),
}

impl RawImageParagraphs {
    pub(crate) fn classify<'a>(&mut self, html: &'a str, images: MarkdownImages) -> RawHtml<'a> {
        if self.consume_paragraph_end(html) {
            return RawHtml::Skip;
        }
        if let Some(image) = raw_html_image(html, images) {
            let align = match self.wrapper.as_mut() {
                Some(wrapper) if image.align == MarkdownImageAlign::None => {
                    wrapper.matched = true;
                    wrapper.align
                }
                _ => image.align,
            };
            let rest = image.rest.trim();
            let rest = (!rest.is_empty() && !self.consume_paragraph_end(rest)).then_some(rest);
            return RawHtml::Image { image, align, rest };
        }
        let unmatched = self.take_unmatched();
        if let Some(align) = raw_html_image_paragraph_start(html) {
            self.wrapper = Some(Wrapper {
                align,
                opening: html.to_owned(),
                matched: false,
            });
            return match unmatched {
                Some(opening) => RawHtml::Text(opening.into()),
                None => RawHtml::Skip,
            };
        }
        RawHtml::Text(match unmatched {
            Some(opening) => (opening + html).into(),
            None => html.into(),
        })
    }

    pub(crate) fn take_unmatched(&mut self) -> Option<String> {
        self.wrapper
            .take_if(|wrapper| !wrapper.matched)
            .map(|wrapper| wrapper.opening)
    }

    fn consume_paragraph_end(&mut self, html: &str) -> bool {
        if self.wrapper.as_ref().is_some_and(|wrapper| wrapper.matched)
            && raw_html_image_paragraph_end(html)
        {
            self.wrapper = None;
            return true;
        }
        false
    }
}

fn raw_html_image_paragraph_start(input: &str) -> Option<MarkdownImageAlign> {
    let input = input.trim();
    let after_paragraph = raw_html_tag_end("p", input)?;
    input[after_paragraph..]
        .trim()
        .is_empty()
        .then(|| paragraph_alignment(&html_attributes(&input[..after_paragraph])))
}

fn raw_html_image_paragraph_end(input: &str) -> bool {
    input.trim().eq_ignore_ascii_case("</p>")
}

pub(crate) fn raw_html_image(input: &str, images: MarkdownImages) -> Option<RawHtmlImage<'_>> {
    let mut image_input = input.trim();
    let mut wrapped_in_paragraph = false;
    let mut align = MarkdownImageAlign::None;

    if let Some(after_paragraph) = raw_html_tag_end("p", image_input) {
        wrapped_in_paragraph = true;
        align = paragraph_alignment(&html_attributes(&image_input[..after_paragraph]));
        image_input = image_input[after_paragraph..].trim_start();
    }

    if !is_raw_html_tag("img", image_input) {
        return None;
    }
    let image_tag_end = closing_angle_bracket(image_input)?;
    let image_tag = &image_input[..=image_tag_end];
    let attributes = html_attributes(image_tag);
    let source = images.sanitized_source(attributes.get("src")?)?;

    let mut rest = &image_input[image_tag_end + 1..];
    if wrapped_in_paragraph {
        rest = rest.trim_start();
        if rest.to_ascii_lowercase().starts_with("</p>") {
            rest = &rest[4..];
        }
    }

    Some(RawHtmlImage {
        source,
        alt: attributes.get("alt").cloned().unwrap_or_default(),
        title: attributes
            .get("title")
            .filter(|title| !title.is_empty())
            .cloned(),
        size: ImageSize {
            width: attributes.get("width").and_then(|value| dimension(value)),
            height: attributes.get("height").and_then(|value| dimension(value)),
        },
        align,
        rest,
    })
}

impl RawHtmlImage<'_> {
    pub(crate) fn html(&self, align: MarkdownImageAlign) -> String {
        let image = image_html(&self.source, &self.alt, self.title.as_deref(), &self.size);
        match align {
            MarkdownImageAlign::None => image,
            MarkdownImageAlign::Center => format!("<p class=\"image-align-center\">{image}</p>"),
        }
    }
}

/// A pixel count or percentage; a `px` suffix is dropped.
fn dimension(value: &str) -> Option<String> {
    let value = value.trim();
    let digits = value.bytes().take_while(u8::is_ascii_digit).count();
    let percent = match &value[digits..] {
        "%" => "%",
        unit if unit.is_empty() || unit.eq_ignore_ascii_case("px") => "",
        _ => return None,
    };
    (1..=5)
        .contains(&digits)
        .then(|| format!("{}{percent}", &value[..digits]))
}

/// A sanitized image source; a relative path is document-relative and the consumer still contains it to the checkout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkdownImageSource<'a> {
    Data { subtype: &'a str, base64: &'a str },
    Relative(String),
}

impl<'a> MarkdownImageSource<'a> {
    pub fn parse(source: &'a str) -> Self {
        if let Some((subtype, base64)) = data_image_parts(source) {
            return Self::Data { subtype, base64 };
        }
        let path = source.split(['?', '#']).next().unwrap_or(source);
        Self::Relative(percent_decode_path(path))
    }
}

fn data_image_parts(source: &str) -> Option<(&str, &str)> {
    sanitized_data_image_source(source)?;
    let rest = &source["data:image/".len()..];
    let lower = rest.to_ascii_lowercase();
    let subtype_end = lower.find([';', ','])?;
    let payload = lower.find(";base64,")? + ";base64,".len();
    Some((&rest[..subtype_end], &rest[payload..]))
}

impl MarkdownImages {
    /// `source` is already decoded, by pulldown-cmark or `html_attributes`, so references in it are literal file name text.
    pub(crate) fn sanitized_source(self, source: &str) -> Option<String> {
        let trimmed = source.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(safe_data_source) = sanitized_data_image_source(trimmed) {
            return Some(safe_data_source.to_owned());
        }
        match self {
            Self::Local => sanitized_relative_source(trimmed),
            Self::Remote => is_https(trimmed).then(|| trimmed.to_owned()),
        }
    }
}

fn is_https(source: &str) -> bool {
    source
        .get(.."https://".len())
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
}

fn sanitized_relative_source(trimmed: &str) -> Option<String> {
    // Absolute schemes (incl. http/https) are rejected since the preview WebView only loads jayjay-preview/data: URIs; scan a copy trimmed of C0 controls and spaces and stripped of tabs/newlines since URL parsing drops those before reading the scheme, or "ht\ttp://" would evade this check.
    let scheme_probe = strip_ascii_tab_and_newline(trimmed.trim_matches(|ch: char| ch <= ' '));
    if url_scheme(&scheme_probe).is_some() {
        return None;
    }
    if scheme_probe.starts_with("//")
        || scheme_probe.starts_with('/')
        || scheme_probe.starts_with('\\')
    {
        return None;
    }

    // `..` stays: containment is checked where the path is resolved against the checkout.
    Some(trimmed.to_owned())
}

fn strip_ascii_tab_and_newline(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !matches!(ch, '\t' | '\n' | '\r'))
        .collect()
}

pub(crate) fn image_html(source: &str, alt: &str, title: Option<&str>, size: &ImageSize) -> String {
    let mut attributes = format!(
        "src=\"{}\" alt=\"{}\"",
        escape_html(source),
        escape_html(alt)
    );
    let optional = [
        ("title", title.filter(|title| !title.is_empty())),
        ("width", size.width.as_deref()),
        ("height", size.height.as_deref()),
        ("referrerpolicy", is_https(source).then_some("no-referrer")),
    ];
    for (name, value) in optional {
        if let Some(value) = value {
            attributes.push_str(&format!(" {name}=\"{}\"", escape_html(value)));
        }
    }
    format!("<img {attributes} loading=\"lazy\" decoding=\"async\">")
}

fn paragraph_alignment(
    attributes: &std::collections::HashMap<String, String>,
) -> MarkdownImageAlign {
    match attributes.get("align").map(|value| value.trim()) {
        Some(value) if value.eq_ignore_ascii_case("center") => MarkdownImageAlign::Center,
        _ => MarkdownImageAlign::None,
    }
}

fn sanitized_data_image_source(source: &str) -> Option<&str> {
    let lower = source.to_ascii_lowercase();
    let rest = lower.strip_prefix("data:image/")?;
    let subtype_end = rest.find([';', ','])?;
    let subtype = &rest[..subtype_end];
    if matches!(subtype, "png" | "jpeg" | "jpg" | "gif" | "webp" | "bmp")
        && lower.contains(";base64,")
    {
        Some(source)
    } else {
        None
    }
}

fn raw_html_tag_end(name: &str, input: &str) -> Option<usize> {
    is_raw_html_tag(name, input)
        .then(|| closing_angle_bracket(input).map(|index| index + 1))
        .flatten()
}

fn is_raw_html_tag(name: &str, input: &str) -> bool {
    let Some(rest) = input.strip_prefix('<') else {
        return false;
    };
    if !rest
        .get(..name.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(name))
    {
        return false;
    }
    rest[name.len()..]
        .chars()
        .next()
        .is_some_and(|ch| ch.is_whitespace() || ch == '>' || ch == '/')
}

fn closing_angle_bracket(input: &str) -> Option<usize> {
    let mut quote = None;
    for (index, ch) in input.char_indices() {
        if let Some(current_quote) = quote {
            if ch == current_quote {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '>' {
            return Some(index);
        }
    }
    None
}

fn html_attributes(tag: &str) -> std::collections::HashMap<String, String> {
    let mut attributes = std::collections::HashMap::new();
    let mut index = 0;

    while let Some(ch) = char_at(tag, index) {
        if ch.is_whitespace() || ch == '>' {
            break;
        }
        index += ch.len_utf8();
    }

    while index < tag.len() {
        skip_html_attribute_separators(tag, &mut index);
        if index >= tag.len() || char_at(tag, index) == Some('>') {
            break;
        }

        let name_start = index;
        while let Some(ch) = char_at(tag, index) {
            if !is_html_attribute_name_character(ch) {
                break;
            }
            index += ch.len_utf8();
        }
        if name_start == index {
            index += char_at(tag, index).map_or(1, char::len_utf8);
            continue;
        }
        let name = tag[name_start..index].to_ascii_lowercase();

        skip_html_attribute_separators(tag, &mut index);
        if char_at(tag, index) != Some('=') {
            continue;
        }
        index += 1;
        skip_html_attribute_separators(tag, &mut index);

        if let Some(value) = scan_html_attribute_value(tag, &mut index) {
            attributes.insert(name, unescape_html_attribute(&value));
        }
    }

    attributes
}

fn skip_html_attribute_separators(tag: &str, index: &mut usize) {
    while let Some(ch) = char_at(tag, *index) {
        if !(ch.is_whitespace() || ch == '/') {
            break;
        }
        *index += ch.len_utf8();
    }
}

fn scan_html_attribute_value(tag: &str, index: &mut usize) -> Option<String> {
    let ch = char_at(tag, *index)?;
    if ch == '>' {
        return None;
    }

    if ch == '"' || ch == '\'' {
        let quote = ch;
        *index += ch.len_utf8();
        let value_start = *index;
        while let Some(current) = char_at(tag, *index) {
            if current == quote {
                let value = tag[value_start..*index].to_owned();
                *index += current.len_utf8();
                return Some(value);
            }
            *index += current.len_utf8();
        }
        return None;
    }

    let value_start = *index;
    while let Some(current) = char_at(tag, *index) {
        if current.is_whitespace() || current == '>' {
            break;
        }
        *index += current.len_utf8();
    }
    let mut value_end = *index;
    if char_at(tag, *index) == Some('>')
        && value_end > value_start
        && tag[..value_end].ends_with('/')
    {
        value_end -= 1;
    }
    (value_start < value_end).then(|| tag[value_start..value_end].to_owned())
}

fn is_html_attribute_name_character(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':')
}

pub(crate) fn unescape_html_attribute(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('&') {
        output.push_str(&rest[..start]);
        rest = &rest[start..];
        let decoded =
            character_reference(rest).and_then(|name| Some((name, decode_reference(name)?)));
        match decoded {
            Some((name, ch)) => {
                output.push(ch);
                rest = &rest[name.len() + 2..];
            }
            None => {
                output.push('&');
                rest = &rest[1..];
            }
        }
    }
    output.push_str(rest);
    output
}

fn decode_reference(name: &str) -> Option<char> {
    let code = match name.strip_prefix('#') {
        Some(hex) if hex.starts_with(['x', 'X']) => u32::from_str_radix(&hex[1..], 16).ok(),
        Some(digits) => digits.parse().ok(),
        None => {
            return match name {
                "amp" => Some('&'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "nbsp" => Some('\u{a0}'),
                _ => None,
            };
        }
    };
    Some(
        code.and_then(char::from_u32)
            .filter(|&ch| ch != '\0')
            .unwrap_or('\u{fffd}'),
    )
}

fn percent_decode_path(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let Ok(hex) = std::str::from_utf8(&bytes[index + 1..index + 3])
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            decoded.push(byte);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn char_at(value: &str, index: usize) -> Option<char> {
    value.get(index..)?.chars().next()
}
