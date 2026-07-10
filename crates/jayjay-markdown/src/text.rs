pub(crate) fn render_text_with_bare_autolinks(text: &str) -> String {
    let mut output = String::new();
    let mut remaining = text;
    while !remaining.is_empty() {
        if let Some((html, rest)) = bare_autolink_markup(remaining, text) {
            output.push_str(&html);
            remaining = rest;
        } else {
            let ch = remaining.chars().next().expect("non-empty text");
            output.push_str(&escape_html(&ch.to_string()));
            remaining = &remaining[ch.len_utf8()..];
        }
    }
    output
}

pub(crate) fn escape_html(text: &str) -> String {
    text.chars().fold(String::new(), |mut result, ch| {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(ch),
        }
        result
    })
}

pub(crate) fn sanitized_link(url: &str) -> Option<&str> {
    if url.starts_with('#') {
        return Some(url);
    }
    match url_scheme(url) {
        Some("http" | "https" | "mailto") => Some(url),
        _ => None,
    }
}

pub(crate) fn url_scheme(value: &str) -> Option<&str> {
    let colon = value.find(':')?;
    let candidate = &value[..colon];
    let mut chars = candidate.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    chars
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
        .then_some(candidate)
}

fn bare_autolink_markup<'a>(input: &'a str, full_text: &str) -> Option<(String, &'a str)> {
    if !is_autolink_boundary(input, full_text) {
        return None;
    }
    if input.starts_with("http://") || input.starts_with("https://") {
        bare_url_autolink_markup(input)
    } else {
        bare_email_autolink_markup(input)
    }
}

fn bare_url_autolink_markup(input: &str) -> Option<(String, &str)> {
    let end = input
        .find(|ch: char| ch.is_whitespace() || ch == '<')
        .unwrap_or(input.len());
    let link_end = trim_trailing_autolink_punctuation(&input[..end]);
    let url = &input[..link_end];
    let href = sanitized_link(url)?;
    Some((
        format!("<a href=\"{}\">{}</a>", escape_html(href), escape_html(url)),
        &input[link_end..],
    ))
}

fn bare_email_autolink_markup(input: &str) -> Option<(String, &str)> {
    let first = input.chars().next()?;
    if !(first.is_ascii_alphanumeric()) {
        return None;
    }

    let end = input
        .find(|ch: char| !is_email_character(ch))
        .unwrap_or(input.len());
    let link_end = trim_trailing_autolink_punctuation(&input[..end]);
    let email = &input[..link_end];
    if !is_valid_email_autolink(email) {
        return None;
    }
    Some((
        format!(
            "<a href=\"mailto:{}\">{}</a>",
            escape_html(email),
            escape_html(email)
        ),
        &input[link_end..],
    ))
}

fn trim_trailing_autolink_punctuation(value: &str) -> usize {
    value.trim_end_matches(['.', ',', ':', ';', '!', '?']).len()
}

fn is_autolink_boundary(input: &str, full_text: &str) -> bool {
    let offset = full_text.len().saturating_sub(input.len());
    if offset == 0 {
        return true;
    }
    full_text[..offset]
        .chars()
        .next_back()
        .is_some_and(|ch| ch.is_whitespace() || matches!(ch, '(' | '[' | '{' | '"' | '\''))
}

fn is_email_character(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '%' | '+' | '-' | '@')
}

fn is_valid_email_autolink(value: &str) -> bool {
    let parts = value.split('@').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0].is_empty() {
        return false;
    }
    let domain = parts[1];
    !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain.contains('.')
        && domain
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-')
}
