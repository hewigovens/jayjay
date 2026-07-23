# Format Projections Guide

This guide records the issue 104 implementation contract for rich diff projections. Load it before adding or changing projected file formats.

## Pipeline

The projection pipeline is:

```text
repo blob bytes -> projection -> jj-diff -> renderer
```

Format adapters live in `jayjay-core` beside diff materialization. `jj-diff` stays the plain text diff/highlight engine. SwiftUI decides whether the user sees raw source text or the processed rich preview.

## Behavior Matrix

| File shape | Projection availability | Default view | Rich preview behavior |
| --- | --- | --- | --- |
| Text source, projection available | `.ipynb`, `.csv`, `.tsv`, `.sarif`, `.sarif.json` | Raw source diff | Header icon switches to processed content, then back to source. |
| Binary source, projection available | Binary `.plist` | Processed content | Banner explains the binary file is previewed as text; no separate rich toggle is needed. |
| Text source, no useful projection | Markdown files, SVG files, HTML files, plain XML `.plist`, normal code/data | Raw source diff | No projection button. Markdown and SVG render in JayJay; HTML can open the working-copy file in the default app so relative CSS and images work, and SwiftUI additionally offers an inline sandboxed preview alongside that external-open action. |
| Projection parse fails | Any projected format | Raw source if possible, otherwise binary placeholder | Surface diagnostics in the projection banner; do not block the source diff. |

## Current Formats

| Format | Raw bytes on disk | Processed content | Render kind | Default |
| --- | --- | --- | --- | --- |
| `.ipynb` | Notebook JSON | Markdown with markdown/code/raw cells | Markdown | Raw |
| `.csv`, `.tsv` | Delimited text | Escaped Markdown table | Table | Raw |
| `.sarif`, `.sarif.json` | SARIF JSON | Markdown report summary | Markdown | Raw |
| Binary `.plist` | Binary property list | Sorted XML property list | Text | Processed |
| Plain XML `.plist` | XML property list | None | Text | Raw |

## Source And External Previews

Markdown (`.md`, `.markdown`), SVG (`.svg`), and HTML (`.html`, `.htm`) are not projections. They stay raw by default. Markdown and SVG use header preview buttons to render the post-change content in JayJay. HTML uses a header external-open button for working-copy files that exist on disk, delegating rendering to the user's default app so linked CSS, images, fonts, and browser security policy behave like a normal local file.

SwiftUI also offers an inline HTML preview button (same on-disk-file requirement as external-open) that renders the file through the sandboxed `PreviewWebView`/`RepoPreviewSchemeHandler` used for Markdown, loading the actual file as the main document rather than diff content, with content JavaScript disabled and a help-text hint when the file contains `<script>`. External-open remains available alongside it for HTML that needs real script execution. This inline preview is SwiftUI-only for now; GPUI keeps external-open only (see [Shell Feature Parity Guide](shell-parity.md)).

## Implementation Rules

1. Path matching is only a cheap affordance for the file list. Byte-aware `matches_input` must clear ambiguous projections, such as plain XML plist files.
2. If the raw file is readable text, the default view is raw source. Processed content is opt-in through the rich preview button.
3. If the raw file is binary and projection produces useful text, the default view can be processed with a banner that explains what is happening.
4. Projection metadata must include plugin id, plugin version, mode, render kind, and virtual path. Cache keys and review identity must distinguish raw and processed modes.
5. Projection failures should degrade to raw content or a binary placeholder with diagnostics. They should not make the whole file diff unusable.
6. Switching raw/processed for the same file should keep the current rendered diff visible until replacement content is ready. Switching to a different file resets rich-preview state so one file's processed view does not become another file's default.
7. Keep v1 static and narrow: no dynamic plugin ABI, no projected diff editing, and no virtual file tree formats such as XLSX/DOCX/ZIP until the product behavior is designed.
8. Per-file diff stats count the effective display mode: processed for formats that open processed by default (binary plists), raw otherwise.
