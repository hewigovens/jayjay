use crate::{MarkdownBlock, MarkdownImageAlign, parse_markdown_blocks, render_markdown_html};

#[test]
fn renders_common_blocks() {
    let html = render_markdown_html(
        r#"# Title

- One
- **Two**

| Name | Value |
| --- | --- |
| Code | `ok` |

```swift
print("hi")
```
"#,
    );

    assert!(html.contains("<h1 id=\"title\">Title</h1>"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("<strong>Two</strong>"));
    assert!(html.contains("<table>"));
    assert!(html.contains("<code class=\"language-swift\">print(&quot;hi&quot;)"));
}

#[test]
fn renders_gfm_extensions() {
    let html = render_markdown_html(
        r#"- [x] Done
- [ ] Todo

~~removed~~

https://example.com/path?a=1&b=2, user@example.com, and <https://example.org>.
"#,
    );

    assert!(html.contains("<ul class=\"contains-task-list\">"));
    assert!(html.contains("<input type=\"checkbox\" disabled checked> Done"));
    assert!(html.contains("<input type=\"checkbox\" disabled> Todo"));
    assert!(html.contains("<del>removed</del>"));
    assert!(html.contains("href=\"https://example.com/path?a=1&amp;b=2\""));
    assert!(html.contains(">https://example.com/path?a=1&amp;b=2</a>,"));
    assert!(html.contains("href=\"mailto:user@example.com\""));
    assert!(html.contains("href=\"https://example.org\""));
}

#[test]
fn renders_image_syntax_and_raw_image_tags() {
    let html = render_markdown_html(
        r#"![Diagram & flow](images/flow.png "Flow")

<p><img src="./screens/a.png" alt="A &amp; B" title="Preview" onerror="alert(1)"></p>
<p align="center"> <img src=images/raw-flow.png alt=Raw></p>
<p align="center">
  <img src="docs/imgs/home.webp" width="100%" alt="JayJay - DAG graph and side-by-side diff">
</p>

![Data](data:image/png;base64,abc123)
"#,
    );

    assert!(
        html.contains("<img src=\"images/flow.png\" alt=\"Diagram &amp; flow\" title=\"Flow\"")
    );
    assert!(html.contains("<img src=\"./screens/a.png\" alt=\"A &amp; B\" title=\"Preview\""));
    assert!(html.contains("<p class=\"image-align-center\">"));
    assert!(html.contains("<img src=\"images/raw-flow.png\" alt=\"Raw\""));
    assert!(html.contains(
        "<img src=\"docs/imgs/home.webp\" alt=\"JayJay - DAG graph and side-by-side diff\""
    ));
    assert!(html.contains("<img src=\"data:image/png;base64,abc123\" alt=\"Data\""));
    assert!(!html.contains("onerror"));
    assert!(!html.contains("&lt;p align=&quot;center&quot;&gt;"));
}

#[test]
fn rejects_unsafe_image_sources() {
    let html = render_markdown_html(
        r#"![bad](javascript:alert(1))
<img src="file:///etc/passwd" alt="secret">
<img src="../secret.png" alt="secret">
<img src="..%2fsecret.png" alt="secret">
<img src="data:image/svg+xml;base64,PHN2Zz4=" alt="svg">
"#,
    );

    assert!(!html.contains("<img src=\"javascript:alert(1)\""));
    assert!(!html.contains("<img src=\"file:///etc/passwd\""));
    assert!(!html.contains("<img src=\"../secret.png\""));
    assert!(!html.contains("<img src=\"..%2fsecret.png\""));
    assert!(!html.contains("<img src=\"data:image/svg+xml"));
    assert!(html.contains("&lt;img src=&quot;file:///etc/passwd&quot; alt=&quot;secret&quot;&gt;"));
}

#[test]
fn escapes_raw_html_and_unsafe_links() {
    let html = render_markdown_html(
        r#"<script>alert(1)</script>

[bad](javascript:alert(1)) [good](https://example.com?a=1&b=2)
"#,
    );

    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!html.contains("href=\"javascript:alert(1)\""));
    assert!(html.contains("href=\"https://example.com?a=1&amp;b=2\""));
}

#[test]
fn parses_blocks_for_native_renderers() {
    let blocks = parse_markdown_blocks(
        r#"# Title

Intro `code`.

- [x] Done
- [ ] Todo

| Name | Value |
| --- | --- |
| Code | `ok` |

```rust
fn main() {}
```

![Diagram](images/flow.png)

<p align="center"> <img src=images/raw-flow.png alt=Raw></p>
"#,
    );

    assert_eq!(
        blocks.first(),
        Some(&MarkdownBlock::Heading {
            level: 1,
            text: "Title".to_owned(),
        })
    );
    assert!(blocks.contains(&MarkdownBlock::Paragraph("Intro `code`.".to_owned())));
    let list = blocks
        .iter()
        .find_map(|block| match block {
            MarkdownBlock::List { items, .. } => Some(items),
            _ => None,
        })
        .expect("task list");
    assert_eq!(list[0].checked, Some(true));
    assert_eq!(list[1].checked, Some(false));
    assert!(
        blocks
            .iter()
            .any(|block| matches!(block, MarkdownBlock::Table { rows } if rows.len() == 2))
    );
    assert!(blocks.iter().any(|block| {
        matches!(
            block,
            MarkdownBlock::CodeBlock {
                language: Some(language),
                text,
            } if language == "rust" && text.contains("fn main")
        )
    }));
    assert!(blocks.contains(&MarkdownBlock::Paragraph(
        "Image: Diagram (images/flow.png)".to_owned()
    )));
    assert!(blocks.contains(&MarkdownBlock::Image {
        source: "images/raw-flow.png".to_owned(),
        alt: "Raw".to_owned(),
        title: None,
        align: MarkdownImageAlign::Center,
    }));
}
