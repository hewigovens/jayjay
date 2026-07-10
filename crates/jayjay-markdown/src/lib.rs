mod blocks;
mod html;
mod images;
mod text;

use pulldown_cmark::{Event, Options, Parser};

pub use blocks::{
    MarkdownBlock, MarkdownDocument, MarkdownImageAlign, MarkdownListItem, MarkdownTableRow,
    parse_markdown_blocks,
};
pub use pulldown_cmark::{
    CodeBlockKind as MarkdownCodeBlockKind, Event as MarkdownEvent,
    HeadingLevel as MarkdownHeadingLevel, Tag as MarkdownTag, TagEnd as MarkdownTagEnd,
};

const DOCUMENT_PREFIX: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
:root {
    color-scheme: light dark;
    font: 14px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    line-height: 1.48;
}
html, body {
    margin: 0;
    min-height: 100%;
    background: transparent;
    color: CanvasText;
}
main {
    box-sizing: border-box;
    max-width: 980px;
    padding: 18px 22px 28px;
}
h1, h2, h3, h4, h5, h6 {
    margin: 1.3em 0 0.55em;
    line-height: 1.2;
    font-weight: 650;
}
h1:first-child, h2:first-child, h3:first-child {
    margin-top: 0;
}
h1 { font-size: 1.7em; border-bottom: 1px solid rgba(128, 128, 128, 0.28); padding-bottom: 0.25em; }
h2 { font-size: 1.35em; border-bottom: 1px solid rgba(128, 128, 128, 0.2); padding-bottom: 0.2em; }
h3 { font-size: 1.14em; }
p, ul, ol, blockquote, pre, table { margin: 0 0 1em; }
ul, ol { padding-left: 1.55em; }
li + li { margin-top: 0.25em; }
a { color: LinkText; text-decoration-thickness: 0.08em; text-underline-offset: 0.12em; }
del { color: color-mix(in srgb, CanvasText 62%, transparent); }
ul.contains-task-list { padding-left: 0; }
li.task-list-item { list-style: none; }
li.task-list-item input {
    margin: 0 0.45em 0 0;
    vertical-align: -0.08em;
}
img {
    display: block;
    box-sizing: border-box;
    max-width: 100%;
    height: auto;
    margin: 0.65em 0;
    border-radius: 4px;
}
.image-align-center {
    text-align: center;
}
.image-align-center img {
    margin-left: auto;
    margin-right: auto;
}
code, pre {
    font-family: ui-monospace, "SF Mono", Menlo, Monaco, Consolas, monospace;
    font-size: 0.92em;
}
code {
    padding: 0.12em 0.32em;
    border-radius: 4px;
    background: rgba(128, 128, 128, 0.14);
}
pre {
    overflow-x: auto;
    padding: 12px;
    border-radius: 6px;
    background: rgba(128, 128, 128, 0.12);
}
pre code {
    padding: 0;
    border-radius: 0;
    background: transparent;
    font-size: 1em;
}
blockquote {
    padding-left: 1em;
    border-left: 3px solid rgba(128, 128, 128, 0.35);
    color: color-mix(in srgb, CanvasText 72%, transparent);
}
table {
    display: block;
    width: 100%;
    overflow-x: auto;
    border-collapse: collapse;
    border-spacing: 0;
}
th, td {
    padding: 6px 9px;
    border: 1px solid rgba(128, 128, 128, 0.28);
    vertical-align: top;
}
th {
    font-weight: 650;
    background: rgba(128, 128, 128, 0.1);
}
hr {
    border: 0;
    border-top: 1px solid rgba(128, 128, 128, 0.28);
    margin: 1.4em 0;
}
</style>
</head>
<body>
<main>
"#;

const DOCUMENT_SUFFIX: &str = r#"
</main>
</body>
</html>
"#;

pub fn parse_markdown(markdown: &str) -> Vec<MarkdownEvent<'static>> {
    Parser::new_ext(markdown, markdown_options())
        .map(Event::into_static)
        .collect()
}

pub fn render_markdown_html(markdown: &str) -> String {
    let body = html::render_markdown_events_html(Parser::new_ext(markdown, markdown_options()));
    let mut document = String::from(DOCUMENT_PREFIX);
    document.push_str(&body);
    document.push_str(DOCUMENT_SUFFIX);
    document
}

pub fn render_markdown_events_html<'a>(
    events: impl IntoIterator<Item = MarkdownEvent<'a>>,
) -> String {
    html::render_markdown_events_html(events)
}

fn markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options
}

#[cfg(test)]
mod tests;
