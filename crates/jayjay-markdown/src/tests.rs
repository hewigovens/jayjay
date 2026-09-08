use crate::blocks::parse_markdown_blocks;
use crate::{
    MarkdownBlock, MarkdownDocument, MarkdownImageAlign, MarkdownImageSource, render_markdown_html,
};

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
    assert!(html.contains("<ul><li>One</li><li><strong>Two</strong></li></ul>"));
    assert!(html.contains("<table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody><tr><td>Code</td><td><code>ok</code></td></tr></tbody></table>"));
    assert!(html.contains("<code class=\"language-swift\">print(&quot;hi&quot;)"));
}

#[test]
fn heading_ids_links_and_code_classes_are_sanitized() {
    let html = render_markdown_html(
        "# Hello, World!\n\n# Hello, World!\n\n[plain](https://example.com) [titled](https://example.com \"Site\")\n\n```c++\nint x;\n```\n\n```shell_session-2\n$ ls\n```\n",
    );

    assert!(html.contains("<h1 id=\"hello-world\">"));
    assert!(html.contains("<h1 id=\"hello-world-2\">"));
    assert!(html.contains("<a href=\"https://example.com\">plain</a>"));
    assert!(html.contains("<a href=\"https://example.com\" title=\"Site\">titled</a>"));
    assert!(html.contains("<code class=\"language-c\">"));
    assert!(html.contains("<code class=\"language-shell_session-2\">"));
}

#[test]
fn task_list_classes_apply_per_list_including_loose_and_nested_items() {
    let html = render_markdown_html(
        "- plain\n\n1. one\n\n- a\n  - b\n- [ ] c\n\n* [ ] loose\n\n* [x] second\n",
    );

    assert!(html.contains("<ul><li>plain</li></ul>"));
    assert!(html.contains("<ol start=\"1\"><li>one</li></ol>"));
    assert!(html.contains("<ul class=\"contains-task-list\"><li>a<ul><li>b</li></ul></li><li class=\"task-list-item\"><input type=\"checkbox\" disabled> c</li></ul>"), "{html}");
    assert!(html.contains("<ul class=\"contains-task-list\"><li><p><input type=\"checkbox\" disabled> loose</p></li><li><p><input type=\"checkbox\" disabled checked> second</p></li></ul>"), "{html}");
}

#[test]
fn image_alt_text_flattens_inline_markup_and_skips_inner_events() {
    let html = render_markdown_html("![**bold** `code`\nalt](a.png) after\n\n# Title `code`\n");

    assert!(html.contains(
        "<img src=\"a.png\" alt=\"bold code alt\" loading=\"lazy\" decoding=\"async\"> after"
    ));
    assert!(!html.contains("<strong>"));
    assert!(html.contains("<h1 id=\"title-code\">Title <code>code</code></h1>"));
}

#[test]
fn raw_image_tags_tolerate_attribute_syntax_variants() {
    let cases = [
        (
            "<IMG SRC='a.png' ALT='caps' />",
            "<img src=\"a.png\" alt=\"caps\"",
        ),
        ("<img src=a.png/>", "<img src=\"a.png\" alt=\"\""),
        (
            "<img src=\"a>b.png\" alt=\"q\">",
            "<img src=\"a&gt;b.png\" alt=\"q\"",
        ),
        (
            "<img  src = \"a.png\"  alt=\"sp\" >",
            "<img src=\"a.png\" alt=\"sp\"",
        ),
        ("<img src=\"a.png\" alt>", "<img src=\"a.png\" alt=\"\""),
        (
            "<img src=\"ü.png\" alt=\"ü\">",
            "<img src=\"ü.png\" alt=\"ü\"",
        ),
        ("<img src=a.png alt=x> tail", "decoding=\"async\"> tail"),
        ("<img src=a.png alt=x></p>", "decoding=\"async\">&lt;/p&gt;"),
        (
            "<p align=\"center\">\n<img src=\"a.png\" alt=\"c\">\n<span>x</span>\n</p>\n",
            "</p>&lt;span&gt;x&lt;/span&gt;\n\n",
        ),
        (
            "<p align=\"left\"><img src=\"a.png\" alt=\"l\"></p>",
            "\n<img src=\"a.png\" alt=\"l\"",
        ),
        (
            "<p align=\"center\"><img src=\"a.png\" alt=\"c\"><span>x</span></p>",
            "</p>&lt;span&gt;x&lt;/span&gt;&lt;/p&gt;",
        ),
    ];
    for (markdown, expected) in cases {
        let html = render_markdown_html(markdown);
        assert!(html.contains(expected), "{markdown:?} -> {html}");
    }
    let rejected = [
        "<imgx src=\"a.png\">",
        "<img>",
        "<img src=\"a.png\" alt=\"unterminated>",
        "<img src=\"/etc/x.png\" alt=\"abs\">",
        "<img src=\"\\\\x.png\" alt=\"unc\">",
    ];
    for markdown in rejected {
        let html = render_markdown_html(markdown);
        assert!(!html.contains("<img "), "{markdown:?} -> {html}");
    }
    let html = render_markdown_html("<p>\nplain\n</p>\n");
    assert!(html.contains("&lt;/p&gt;"), "{html}");
}

#[test]
fn wrapper_alignment_spans_every_image_and_unmatched_openings_stay_text() {
    let centered = "<p align=\"center\">\n<img src=\"a.png\" alt=\"A\">\n<img src=\"b.png\" alt=\"B\">\n</p>\n";
    let html = render_markdown_html(centered);
    assert_eq!(
        html.matches("<p class=\"image-align-center\">").count(),
        2,
        "{html}"
    );
    assert!(!html.contains("&lt;/p&gt;"), "{html}");
    let blocks = MarkdownDocument::parse(centered);
    assert!(blocks.blocks().iter().all(|block| matches!(
        block,
        MarkdownBlock::Image {
            align: MarkdownImageAlign::Center,
            ..
        }
    )));

    let inline = "- <p align=\"center\">text</p>\n- <p align=\"center\"> <img src=\"a.png\" alt=\"A\"> </p>\n";
    let html = render_markdown_html(inline);
    assert!(
        html.contains("<li>&lt;p align=&quot;center&quot;&gt;text&lt;/p&gt;"),
        "{html}"
    );
    assert!(
        html.contains("<li><p class=\"image-align-center\"><img src=\"a.png\" alt=\"A\""),
        "{html}"
    );
    assert!(matches!(
        &MarkdownDocument::parse(inline).blocks()[0],
        MarkdownBlock::List { items, .. }
            if items[0].text == "<p align=\"center\">text</p>" && items[1].text == "Image: A (a.png)"
    ));

    let captioned = "<img src=\"a.png\" alt=\"A\"> caption\n<img src=\"b.png\" alt=\"B\">\n";
    assert!(matches!(
        MarkdownDocument::parse(captioned).blocks(),
        [
            MarkdownBlock::Image { source: a, .. },
            MarkdownBlock::Paragraph(caption),
            MarkdownBlock::Image { source: b, .. },
        ] if a == "a.png" && caption == "caption" && b == "b.png"
    ));

    let unmatched = "<p align=\"center\">\ntext only\n</p>\n";
    let html = render_markdown_html(unmatched);
    assert!(
        html.contains("&lt;p align=&quot;center&quot;&gt;\ntext only\n&lt;/p&gt;"),
        "{html}"
    );
    assert_eq!(
        MarkdownDocument::parse(unmatched).blocks(),
        [MarkdownBlock::Paragraph(
            "<p align=\"center\">\ntext only\n</p>".to_owned()
        )]
    );
}

#[test]
fn bare_autolinks_need_boundaries_and_valid_domains() {
    let linked = render_markdown_html("(https://example.com now, [x]user@example.com\n");
    assert!(linked.contains("(<a href=\"https://example.com\">https://example.com</a> now"));
    assert!(linked.contains("<a href=\"mailto:user@example.com\">"));
    for text in [
        "xhttps://example.com",
        "user@.example.com",
        "user@localhost",
        "user@exa_mple.com",
    ] {
        let html = render_markdown_html(text);
        assert!(!html.contains("<a "), "{text:?} -> {html}");
    }
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
        "<p class=\"image-align-center\"><img src=\"docs/imgs/home.webp\" alt=\"JayJay - DAG graph and side-by-side diff\""
    ));
    assert!(html.contains("<img src=\"data:image/png;base64,abc123\" alt=\"Data\""));
    assert!(!html.contains("onerror"));
    assert!(!html.contains("&lt;p align=&quot;center&quot;&gt;"));
    assert!(!html.contains("&lt;/p&gt;"));
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
    assert!(html.contains("<img src=\"../secret.png\""));
    assert!(html.contains("<img src=\"..%2fsecret.png\""));
    assert!(!html.contains("<img src=\"data:image/svg+xml"));
    assert!(html.contains("&lt;img src=&quot;file:///etc/passwd&quot; alt=&quot;secret&quot;&gt;"));
}

#[test]
fn rejects_remote_and_obfuscated_image_sources() {
    // Markdown image syntax: rejection falls back to plain alt text, no <img> at all.
    for scheme in ["http", "https"] {
        let html = render_markdown_html(&format!("![remote]({scheme}://evil.example/pixel.png)"));
        assert!(
            !html.contains("<img"),
            "{scheme} image should be rejected: {html}"
        );
        assert!(html.contains("remote"));
    }

    // Raw <img> tags: rejection falls back to escaping the whole tag as text (existing mechanism).
    let rejected = [
        r#"<img src="//evil.example/pixel.png" alt="protocol-relative">"#,
        r#"<img src="data:text/html,hello" alt="data-html">"#,
        r#"<img src="HTTP://evil.example/pixel.png" alt="upper-scheme">"#,
    ];
    for markdown in rejected {
        let html = render_markdown_html(markdown);
        assert!(
            !html.contains("<img "),
            "should reject {markdown:?}, got {html}"
        );
    }

    // WebKit's URL parser strips ASCII tab/newline before reading the scheme, so an embedded tab
    // must not let an absolute URL slip through as a "relative" source.
    let tabbed = format!(
        "<img src=\"ht{}tp://evil.example/pixel.png\" alt=\"tabbed\">",
        '\t'
    );
    let html = render_markdown_html(&tabbed);
    assert!(
        !html.contains("<img "),
        "should reject tab-obfuscated scheme, got {html}"
    );

    // Still allowed: repo-relative paths and image data URIs.
    let allowed = render_markdown_html(
        r#"![local](assets/ok.png)

<img src="data:image/png;base64,abc123" alt="inline">
"#,
    );
    assert!(allowed.contains("<img src=\"assets/ok.png\" alt=\"local\""));
    assert!(allowed.contains("<img src=\"data:image/png;base64,abc123\" alt=\"inline\""));
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

### Sub

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

<p align="center">
  <img src=images/raw-flow.png alt=Raw>
</p>
"#,
    );

    assert!(
        !blocks
            .iter()
            .any(|block| matches!(block, MarkdownBlock::Paragraph(text) if text.contains("<p")))
    );
    assert_eq!(
        blocks.first(),
        Some(&MarkdownBlock::Heading {
            level: 1,
            text: "Title".to_owned(),
        })
    );
    assert!(blocks.contains(&MarkdownBlock::Heading {
        level: 3,
        text: "Sub".to_owned(),
    }));
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
    assert!(blocks.contains(&MarkdownBlock::Image {
        source: "images/flow.png".to_owned(),
        alt: "Diagram".to_owned(),
        title: None,
        align: MarkdownImageAlign::None,
    }));
    assert!(blocks.contains(&MarkdownBlock::Image {
        source: "images/raw-flow.png".to_owned(),
        alt: "Raw".to_owned(),
        title: None,
        align: MarkdownImageAlign::Center,
    }));
}

#[test]
fn nested_blocks_and_raw_html_fold_into_item_and_quote_text() {
    let document = MarkdownDocument::parse(
        "- item\n\n  ```\n  code\n  ```\n- <img src=\"a.png\" alt=\"A\">\n\n<div>top</div>\n\n> <div>quoted</div>\n",
    );
    assert_eq!(document.source().lines().next(), Some("- item"));
    let blocks = document.blocks();
    assert!(matches!(
        &blocks[0],
        MarkdownBlock::List { items, .. }
            if items[0].text == "item\ncode" && items[1].text == "Image: A (a.png)"
    ));
    assert_eq!(
        blocks[1],
        MarkdownBlock::Paragraph("<div>top</div>".to_owned())
    );
    assert_eq!(
        blocks[2],
        MarkdownBlock::BlockQuote("<div>quoted</div>".to_owned())
    );
}

#[test]
fn only_image_only_paragraphs_become_image_blocks() {
    let blocks = MarkdownDocument::parse(
        "![One](a.png) ![Two](b%20c.png \"Pair\")\n\nSee ![inline](c.png) here.\n\n![First](e.png) then text\n\n> ![Quoted](q.png)\n\n![Bad](https://example.com/x.png)\n\n- ![Item](d.png)\n\n![Md](m.png)<img src=\"h.png\" alt=\"Html\">\n",
    );
    let blocks = blocks.blocks();
    assert_eq!(
        blocks[0],
        MarkdownBlock::Image {
            source: "a.png".to_owned(),
            alt: "One".to_owned(),
            title: None,
            align: MarkdownImageAlign::None,
        }
    );
    assert_eq!(
        blocks[1],
        MarkdownBlock::Image {
            source: "b%20c.png".to_owned(),
            alt: "Two".to_owned(),
            title: Some("Pair".to_owned()),
            align: MarkdownImageAlign::None,
        }
    );
    assert_eq!(
        blocks[2],
        MarkdownBlock::Paragraph("See Image: inline (c.png) here.".to_owned())
    );
    assert_eq!(
        blocks[3],
        MarkdownBlock::Paragraph("Image: First (e.png) then text".to_owned())
    );
    assert_eq!(
        blocks[4],
        MarkdownBlock::BlockQuote("Image: Quoted (q.png)".to_owned())
    );
    assert_eq!(
        blocks[5],
        MarkdownBlock::Paragraph("Image: Bad (https://example.com/x.png)".to_owned())
    );
    assert!(matches!(
        &blocks[6],
        MarkdownBlock::List { items, .. } if items[0].text == "Image: Item (d.png)"
    ));
    assert_eq!(
        blocks[7..]
            .iter()
            .map(|block| match block {
                MarkdownBlock::Image { source, .. } => source.as_str(),
                _ => "?",
            })
            .collect::<Vec<_>>(),
        ["m.png", "h.png"]
    );
}

#[test]
fn image_sources_split_into_data_payloads_and_decoded_relative_paths() {
    for source in ["data:image/png;base64,AAAA", "data:image/PNG;BASE64,AAAA"] {
        assert!(matches!(
            MarkdownImageSource::parse(source),
            MarkdownImageSource::Data { subtype, base64: "AAAA" } if subtype.eq_ignore_ascii_case("png")
        ));
    }
    for (source, decoded) in [
        ("docs/a%20b.png", "docs/a b.png"),
        ("../assets/logo.png", "../assets/logo.png"),
        ("images/a.png?raw=1", "images/a.png"),
        ("images/a.png#frag", "images/a.png"),
        ("a%3Fb.png", "a?b.png"),
        ("%41.png", "A.png"),
        ("a%4", "a%4"),
        ("a%zz.png", "a%zz.png"),
    ] {
        assert_eq!(
            MarkdownImageSource::parse(source),
            MarkdownImageSource::Relative(decoded.to_owned())
        );
    }
}
