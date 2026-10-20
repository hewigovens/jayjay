use crate::{MarkdownBlock, MarkdownDocument, render_markdown_html};

#[test]
fn mermaid_fences_render_in_html_and_native_documents_with_unique_ids() {
    let source = "```mermaid\nflowchart LR\naccTitle: Sample\n A --> B\n```\n\n```mermaid\nflowchart LR\naccTitle: Sample\n A --> B\n```";
    let html = render_markdown_html(source);
    assert_eq!(html.matches("<svg ").count(), 2);
    let body = format!(
        "<root>{}</root>",
        html.split("<main>")
            .nth(1)
            .unwrap()
            .split("</main>")
            .next()
            .unwrap()
    );
    let xml = resvg::usvg::roxmltree::Document::parse(&body).unwrap();
    let ids = xml
        .descendants()
        .filter_map(|node| node.attribute("id"))
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    assert_eq!(
        ids.len(),
        ids.iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len()
    );
    let document = MarkdownDocument::parse(source);
    let diagrams = document
        .blocks()
        .iter()
        .filter_map(|block| match block {
            MarkdownBlock::Mermaid(diagram) => Some(diagram),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(diagrams.len(), 2);
    assert_ne!(
        diagrams[0].svg(0, 0xffffff, "monospace"),
        diagrams[1].svg(0, 0xffffff, "monospace")
    );
}

#[test]
fn invalid_and_oversized_mermaid_falls_back_to_escaped_source() {
    for source in [
        "not a diagram <script>bad()</script>".to_owned(),
        format!("flowchart LR\n A[{}]", "x".repeat(10001)),
    ] {
        let markdown = format!("```mermaid\n{source}\n```");
        let html = render_markdown_html(&markdown);
        assert!(!html.contains("<svg "));
        assert!(!html.contains("<script>"));
        assert!(html.contains("<pre><code class=\"language-mermaid\">"));
        assert!(
            matches!(&MarkdownDocument::parse(markdown).blocks()[0], MarkdownBlock::CodeBlock { text, .. } if text.trim_end() == source)
        );
    }
}

#[test]
fn mermaid_gantt_fills_missing_date_fields_from_the_current_clock() {
    let html = render_markdown_html(
        "```mermaid\ngantt\n dateFormat MM-DD\n section S\n Task :a1, 01-05, 3d\n```",
    );
    assert!(html.contains("<svg "));
    assert!(!html.contains("1970-"));
}

#[test]
fn mermaid_svg_preserves_authored_text_and_disables_links_and_images() {
    let source = r#"```mermaid
flowchart LR
 A["var(--_fg) 日本語 & text"] --> B["&lt;script&gt;alert(1)&lt;/script&gt;"]
 click A "https://example.com"
```

```mermaid
flowchart LR
 A@{ img: "https://example.com/a.png", label: "Image" }
```"#;
    let html = render_markdown_html(source);
    assert_eq!(html.matches("<svg ").count(), 2);
    assert!(html.contains("var(--_fg)"));
    assert!(html.contains("日本語"));
    assert!(!html.contains("<script>"));
    assert!(!html.contains("href="));
    assert!(!html.contains("<image"));
}

#[test]
fn mermaid_native_svg_resolves_theme_fonts_and_series_for_resvg() {
    use resvg::{tiny_skia, usvg};
    let document = MarkdownDocument::parse("```mermaid\npie\n \"A\": 40\n \"B\": 60\n```");
    let MarkdownBlock::Mermaid(diagram) = &document.blocks()[0] else {
        panic!("expected diagram")
    };
    let mut fonts = usvg::fontdb::Database::new();
    fonts.load_system_fonts();
    let options = usvg::Options {
        fontdb: std::sync::Arc::new(fonts),
        ..Default::default()
    };
    for (foreground, background) in [(0, 0xffffff), (0xffffff, 0x151515)] {
        let svg = diagram.svg(foreground, background, "Menlo, monospace");
        assert!(!svg.contains("var("));
        assert!(!svg.contains("color-mix("));
        let tree = usvg::Tree::from_str(&svg, &options).unwrap();
        let mut image = tiny_skia::Pixmap::new(diagram.width, diagram.height).unwrap();
        resvg::render(&tree, tiny_skia::Transform::identity(), &mut image.as_mut());
        let mut colors = std::collections::HashSet::new();
        for pixel in image.pixels().iter().filter(|p| p.alpha() == 255) {
            colors.insert((pixel.red(), pixel.green(), pixel.blue()));
        }
        let fg = (
            ((foreground >> 16) & 255) as u8,
            ((foreground >> 8) & 255) as u8,
            (foreground & 255) as u8,
        );
        assert!(colors.contains(&fg));
        assert!(
            colors
                .iter()
                .any(|&(red, green, blue)| red == green && green == blue && red > 64 && red < 192)
        );
    }
    let svg = diagram.svg(0, 0xffffff, "name\" onload=\"bad");
    assert!(usvg::roxmltree::Document::parse(&svg).is_ok());
    assert!(!svg.contains(" onload=\""));
}

fn native_svg(source: &str) -> String {
    let document = MarkdownDocument::parse(format!("```mermaid\n{source}\n```"));
    let MarkdownBlock::Mermaid(diagram) = &document.blocks()[0] else {
        panic!("expected diagram")
    };
    let svg = diagram.svg(0, 0xffffff, "monospace");
    resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default()).unwrap();
    svg
}

fn node_radii(svg: &str) -> Vec<(Option<String>, Option<String>)> {
    roxmltree::Document::parse(svg)
        .unwrap()
        .descendants()
        .filter(|node| node.has_tag_name("rect") && node.attribute("rx").is_some())
        .map(|node| {
            assert!(!node.attribute("style").unwrap_or_default().contains("rx:"));
            (
                node.attribute("rx").map(str::to_owned),
                node.attribute("ry").map(str::to_owned),
            )
        })
        .collect()
}

#[test]
fn mermaid_native_svg_rounds_corners_and_lets_author_radii_win() {
    let svg = native_svg("flowchart LR\n A --> B");
    assert!(svg.contains("stroke=\"#b3b3b3\""));
    assert!(node_radii(&svg).contains(&(Some("4px".to_owned()), None)));

    for source in [
        "flowchart LR\n A[Box] --> B\n style A rx:10,ry:10",
        "flowchart LR\n A[Box] --> B\n classDef r rx:10,ry:10\n class A r",
    ] {
        let radii = node_radii(&native_svg(source));
        assert!(radii.contains(&(Some("10".to_owned()), Some("10".to_owned()))));
    }
    assert!(
        node_radii(&native_svg(
            "stateDiagram-v2\n A --> B\n classDef c rx:5\n class A c"
        ))
        .contains(&(Some("5".to_owned()), None))
    );
}

#[test]
fn mermaid_variable_substitution_never_builds_new_css_tokens() {
    let unresolved = "stateDiagram-v2\n A --> B\n classDef c fill:uvar( --q)rl(https://evil.example/p)\n class A c";
    assert!(!native_svg(unresolved).contains("evil.example"));
    for source in [
        unresolved,
        "C4Context\n Person(a, \"A\")\n UpdateElementStyle(a, $bgColor=\"uvar( --q,r)l(https://evil.example/x)\", $fontColor=\"red /var( --q,*) x\")",
    ] {
        for output in [
            native_svg(source),
            render_markdown_html(&format!("```mermaid\n{source}\n```")),
        ] {
            assert!(output.contains("<svg "));
            assert!(!output.contains("url("));
            assert!(!output.contains("/*"));
        }
    }
}
