use std::ops::Range;

use roxmltree::{Document, Node};

use crate::text::escape_html;

// Ondine's monochrome defaults draw square, full-contrast boxes; these soften them the way Pele's palette did.
const THEME: [(&str, &str); 7] = [
    ("--ondine-radius", "4px"),
    ("--ondine-stroke", "1.5px"),
    (
        "--ondine-muted",
        "color-mix(in srgb, var(--ondine-fg) 62%, var(--ondine-bg))",
    ),
    (
        "--ondine-line",
        "color-mix(in srgb, var(--ondine-fg) 72%, var(--ondine-bg))",
    ),
    (
        "--ondine-border",
        "color-mix(in srgb, var(--ondine-fg) 30%, var(--ondine-bg))",
    ),
    (
        "--ondine-surface",
        "color-mix(in srgb, var(--ondine-fg) 7%, var(--ondine-bg))",
    ),
    (
        "--ondine-surface-alt",
        "color-mix(in srgb, var(--ondine-fg) 4%, var(--ondine-bg))",
    ),
];

const VAR: &str = "var(";
const COLOR_MIX: &str = "color-mix(";
const RESOLVED_ATTRIBUTES: [&str; 8] = [
    "fill",
    "stroke",
    "stroke-width",
    "color",
    "font-family",
    "font-weight",
    "rx",
    "ry",
];

type Declarations = [(String, String)];

// Ondine's SVG themes itself through CSS variables; resvg needs concrete presentation attributes, including fonts on the root.
pub(super) fn flatten(svg: &str, foreground: &str, background: &str, font_family: &str) -> String {
    let Ok(document) = Document::parse(svg) else {
        return svg.to_owned();
    };
    let root = document.root_element();
    let mut declarations = split(root.attribute("style").unwrap_or_default(), ';')
        .filter_map(|declaration| declaration.split_once(':'))
        .filter(|(name, _)| name.trim().starts_with("--"))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .collect::<Vec<_>>();
    declarations.extend(
        [
            ("--ondine-fg", foreground),
            ("--ondine-bg", background),
            ("--ondine-font", font_family),
        ]
        .into_iter()
        .chain(THEME)
        .map(|(name, value)| (name.to_owned(), value.to_owned())),
    );
    let root_attributes = vec![
        ("color", foreground.to_owned()),
        (
            "stroke-width",
            resolve_value("var(--_sw)", &declarations).unwrap_or_default(),
        ),
        ("font-family", font_family.to_owned()),
        ("font-size", "16".to_owned()),
        (
            "style",
            "font-kerning:none;font-variant-ligatures:none".to_owned(),
        ),
    ];
    let mut edits = Vec::new();
    for element in document.descendants().filter(Node::is_element) {
        let overrides = if element == root {
            root_attributes.clone()
        } else {
            style_overrides(element, &declarations)
        };
        rewrite_attributes(element, &declarations, overrides, &mut edits);
    }
    let mut output = String::with_capacity(svg.len());
    let mut copied = 0;
    for (range, text) in edits {
        output.push_str(&svg[copied..range.start]);
        output.push_str(&text);
        copied = range.end;
    }
    output.push_str(&svg[copied..]);
    output
}

// resvg ignores corner radii set as CSS properties, so they move to attributes, the last one winning as in CSS.
fn style_overrides<'a>(
    element: Node<'a, '_>,
    declarations: &Declarations,
) -> Vec<(&'a str, String)> {
    let Some(style) = element.attribute("style") else {
        return Vec::new();
    };
    let mut overrides = Vec::new();
    let mut kept = Vec::new();
    for (property, value) in split(style, ';').filter_map(|declaration| declaration.split_once(':'))
    {
        let Some(value) = resolve_value(value, declarations) else {
            continue;
        };
        match property.trim() {
            radius @ ("rx" | "ry") => {
                overrides.retain(|(name, _)| *name != radius);
                overrides.push((radius, value));
            }
            property => kept.push(format!("{property}:{value}")),
        }
    }
    overrides.push(("style", kept.join(";")));
    overrides
}

fn rewrite_attributes(
    element: Node<'_, '_>,
    declarations: &Declarations,
    mut overrides: Vec<(&str, String)>,
    edits: &mut Vec<(Range<usize>, String)>,
) {
    let Some(end) = element
        .attributes()
        .next_back()
        .map(|attribute| attribute.range().end)
    else {
        return;
    };
    let text = |name: &str, value: &str| format!("{name}=\"{}\"", escape_html(value));
    for attribute in element.attributes() {
        let name = attribute.name();
        if attribute.namespace().is_some() {
            continue;
        }
        let replacement = if let Some(index) = overrides.iter().position(|(key, _)| *key == name) {
            let (name, value) = overrides.remove(index);
            text(name, &value)
        } else if RESOLVED_ATTRIBUTES.contains(&name) {
            resolve_value(attribute.value(), declarations)
                .map_or_else(String::new, |value| text(name, &value))
        } else {
            continue;
        };
        edits.push((attribute.range(), replacement));
    }
    let appended = overrides
        .iter()
        .map(|(name, value)| format!(" {}", text(name, value)))
        .collect::<String>();
    if !appended.is_empty() {
        edits.push((end..end, appended));
    }
}

fn resolve_value(value: &str, declarations: &Declarations) -> Option<String> {
    Some(resolve_mixes(resolve(value, declarations, 0)?.trim()))
}

// An unknown variable without a fallback invalidates the whole declaration, as in CSS.
fn resolve(text: &str, declarations: &Declarations, depth: usize) -> Option<String> {
    if depth > 8 {
        return None;
    }
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(VAR) {
        output.push_str(&rest[..start]);
        let open = start + VAR.len() - 1;
        let Some(length) = closing_parenthesis(&rest[open..]) else {
            break;
        };
        let end = open + length;
        let inner = &rest[open + 1..end];
        let (name, fallback) = match inner.split_once(',') {
            Some((name, fallback)) => (name, Some(fallback)),
            None => (inner, None),
        };
        let value = declarations
            .iter()
            .rev()
            .find(|(key, _)| key == name.trim())
            .map(|(_, value)| value.as_str())
            .or(fallback)?;
        // CSS substitutes whole tokens, so `uvar(--x,r)l(` must never join into `url(`.
        output.push(' ');
        output.push_str(&resolve(value, declarations, depth + 1)?);
        output.push(' ');
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    Some(output)
}

fn resolve_mixes(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(COLOR_MIX) {
        output.push_str(&rest[..start]);
        let open = start + COLOR_MIX.len() - 1;
        let Some(length) = closing_parenthesis(&rest[open..]) else {
            break;
        };
        let end = open + length;
        let inner = resolve_mixes(&rest[open + 1..end]);
        output.push_str(&mix_srgb(&inner).unwrap_or_else(|| format!("{COLOR_MIX}{inner})")));
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    output
}

fn mix_srgb(arguments: &str) -> Option<String> {
    let parts = split(arguments, ',').map(str::trim).collect::<Vec<_>>();
    let ["in srgb", first, second] = parts.as_slice() else {
        return None;
    };
    let (first, percent) = first.rsplit_once(' ')?;
    let weight = percent.strip_suffix('%')?.parse::<f32>().ok()? / 100.;
    let first = hex_color(first.trim())?;
    let second = hex_color(second)?;
    let color = [16, 8, 0].map(|shift| {
        let a = ((first >> shift) & 255) as f32;
        let b = ((second >> shift) & 255) as f32;
        (a * weight + b * (1. - weight)).round().clamp(0., 255.) as u32
    });
    Some(format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]))
}

fn hex_color(text: &str) -> Option<u32> {
    let hex = text.strip_prefix('#')?;
    match hex.len() {
        6 => u32::from_str_radix(hex, 16).ok(),
        3 => {
            let color = u32::from_str_radix(hex, 16).ok()?;
            Some(((color & 0xf00) << 8 | (color & 0xf0) << 4 | color & 0xf) * 17)
        }
        _ => None,
    }
}

fn closing_parenthesis(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split(text: &str, separator: char) -> impl Iterator<Item = &str> {
    let mut depth = 0usize;
    text.split(move |ch| {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        ch == separator && depth == 0
    })
}
