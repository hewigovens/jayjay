use crate::MarkdownImageAlign;
use crate::text::{escape_html, url_scheme};

pub(crate) struct RawHtmlImage<'a> {
    pub(crate) source: String,
    pub(crate) alt: String,
    pub(crate) title: Option<String>,
    pub(crate) align: MarkdownImageAlign,
    pub(crate) rest: &'a str,
}

pub(crate) fn raw_html_image_markup(input: &str) -> Option<(String, &str)> {
    let image = raw_html_image(input)?;
    Some((
        image_html_with_align(
            &image.source,
            &image.alt,
            image.title.as_deref(),
            image.align,
        ),
        image.rest,
    ))
}

pub(crate) fn raw_html_image_paragraph_start(input: &str) -> Option<MarkdownImageAlign> {
    let input = input.trim();
    let after_paragraph = raw_html_tag_end("p", input)?;
    input[after_paragraph..]
        .trim()
        .is_empty()
        .then(|| paragraph_alignment(&html_attributes(&input[..after_paragraph])))
}

pub(crate) fn raw_html_image_paragraph_end(input: &str) -> bool {
    input.trim().eq_ignore_ascii_case("</p>")
}

pub(crate) fn raw_html_image(input: &str) -> Option<RawHtmlImage<'_>> {
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
    let source = sanitized_image_source(attributes.get("src")?)?;

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
        align,
        rest,
    })
}

pub(crate) fn sanitized_image_source(source: &str) -> Option<String> {
    let unescaped = unescape_html_attribute(source);
    let trimmed = unescaped.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(safe_data_source) = sanitized_data_image_source(trimmed) {
        return Some(safe_data_source.to_owned());
    }

    // Absolute schemes (incl. http/https) are rejected since the preview WebView only loads jayjay-preview/data: URIs; scan a tab/newline-stripped copy since WebKit's parser strips those before reading the scheme, or "ht\ttp://" would evade this check.
    let scheme_probe = strip_ascii_tab_and_newline(trimmed);
    if url_scheme(&scheme_probe).is_some() {
        return None;
    }
    if scheme_probe.starts_with("//")
        || scheme_probe.starts_with('/')
        || scheme_probe.starts_with('\\')
    {
        return None;
    }

    let decoded = percent_decode_path(trimmed);
    let has_parent_component = decoded
        .split(['/', '\\'])
        .any(|component| component == "..");
    (!has_parent_component).then(|| trimmed.to_owned())
}

fn strip_ascii_tab_and_newline(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !matches!(ch, '\t' | '\n' | '\r'))
        .collect()
}

pub(crate) fn image_html(source: &str, alt: &str, title: Option<&str>) -> String {
    image_html_with_align(source, alt, title, MarkdownImageAlign::None)
}

pub(crate) fn image_html_with_align(
    source: &str,
    alt: &str,
    title: Option<&str>,
    align: MarkdownImageAlign,
) -> String {
    let mut attributes = format!(
        "src=\"{}\" alt=\"{}\"",
        escape_html(source),
        escape_html(alt)
    );
    if let Some(title) = title.filter(|title| !title.is_empty()) {
        attributes.push_str(&format!(" title=\"{}\"", escape_html(title)));
    }
    let image = format!("<img {attributes} loading=\"lazy\" decoding=\"async\">");
    match align {
        MarkdownImageAlign::None => image,
        MarkdownImageAlign::Center => format!("<p class=\"image-align-center\">{image}</p>"),
    }
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
    if rest.len() < name.len() || !rest[..name.len()].eq_ignore_ascii_case(name) {
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

fn unescape_html_attribute(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
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
