use tree_sitter_highlight::HighlightConfiguration;

use super::{HIGHLIGHT_NAMES, HighlightSpan, SyntaxToken, highlight_with_config};

pub(super) fn merge_block_and_inline(
    source: &str,
    block_spans: Vec<HighlightSpan>,
) -> Vec<HighlightSpan> {
    // Markdown has separate block and inline grammars; block structure wins where their spans overlap.
    let Some(config) = inline_config() else {
        return block_spans;
    };
    let inline_spans = inline_highlights(source, &config);
    merge_highlights(block_spans, inline_spans, source.len())
}

fn inline_config() -> Option<HighlightConfiguration> {
    let mut config = HighlightConfiguration::new(
        tree_sitter_md::INLINE_LANGUAGE.into(),
        "markdown_inline",
        tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
        "",
        "",
    )
    .ok()?;
    config.configure(&HIGHLIGHT_NAMES);
    Some(config)
}

fn inline_highlights(source: &str, config: &HighlightConfiguration) -> Vec<HighlightSpan> {
    let mut parser = tree_sitter::Parser::new();
    if parser
        .set_language(&tree_sitter_md::LANGUAGE.into())
        .is_err()
    {
        return vec![];
    }
    let Some(tree) = parser.parse(source, None) else {
        return vec![];
    };

    let mut inline_ranges = Vec::new();
    let mut nodes = vec![tree.root_node()];
    while let Some(node) = nodes.pop() {
        if matches!(node.kind(), "inline" | "pipe_table_cell") {
            inline_ranges.push(node.byte_range());
            continue;
        }
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        nodes.extend(children.into_iter().rev());
    }

    let mut spans = Vec::new();
    for range in inline_ranges {
        let Some(inline_source) = source.get(range.clone()) else {
            continue;
        };
        spans.extend(
            highlight_with_config(inline_source, config)
                .into_iter()
                .map(|span| HighlightSpan {
                    start: span.start + range.start,
                    end: span.end + range.start,
                    token: span.token,
                }),
        );
    }
    spans
}

fn merge_highlights(
    block: Vec<HighlightSpan>,
    inline: Vec<HighlightSpan>,
    source_len: usize,
) -> Vec<HighlightSpan> {
    let mut boundaries = Vec::with_capacity((block.len() + inline.len()) * 2 + 2);
    boundaries.extend([0, source_len]);
    for span in block.iter().chain(&inline) {
        boundaries.extend([span.start, span.end]);
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut block_index = 0;
    let mut inline_index = 0;
    let mut merged: Vec<HighlightSpan> = Vec::new();
    for range in boundaries.windows(2) {
        let start = range[0];
        let end = range[1];
        if start == end {
            continue;
        }

        while block.get(block_index).is_some_and(|span| span.end <= start) {
            block_index += 1;
        }
        while inline
            .get(inline_index)
            .is_some_and(|span| span.end <= start)
        {
            inline_index += 1;
        }

        let block_token = token_at(&block, block_index, start, end);
        let inline_token = token_at(&inline, inline_index, start, end);
        let token = if block_token == SyntaxToken::Plain {
            inline_token
        } else {
            block_token
        };

        if let Some(previous) = merged.last_mut()
            && previous.end == start
            && previous.token == token
        {
            previous.end = end;
        } else {
            merged.push(HighlightSpan { start, end, token });
        }
    }
    merged
}

fn token_at(spans: &[HighlightSpan], index: usize, start: usize, end: usize) -> SyntaxToken {
    spans
        .get(index)
        .filter(|span| span.start <= start && end <= span.end)
        .map_or(SyntaxToken::Plain, |span| span.token)
}
