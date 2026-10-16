use std::time::{Duration, Instant};

use tree_sitter::Language;

use super::anchors::moved_subtrees;
use super::tree::SyntaxTree;
use crate::types::{DiffLine, DiffSpanStyle, FileDiff, LineIndex, LineSpan, MovedRow};

const MAX_BYTES: usize = 256 * 1024;
const MAX_LINES: usize = 8_000;
const BUDGET: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MovedLines {
    pub(super) old_lines: LineSpan,
    pub(super) new_lines: LineSpan,
}

impl MovedLines {
    pub(super) fn find(diff: &FileDiff, old: &str, new: &str) -> Option<Vec<Self>> {
        if old.len().saturating_add(new.len()) > MAX_BYTES
            || old.lines().count().saturating_add(new.lines().count()) > MAX_LINES
        {
            return None;
        }
        let grammar = crate::syntax::grammar(&diff.language)?;
        let deadline = Instant::now() + BUDGET;
        let check = || (Instant::now() < deadline).then_some(());
        let old = Side::parse(old, &grammar, &check)?;
        let new = Side::parse(new, &grammar, &check)?;
        let rows = diff.kept_rows();
        let aligned =
            |old_atom, new_atom| rows.get(&old.line_of(old_atom)) == Some(&new.line_of(new_atom));
        let moved = moved_subtrees(&old.tree, &new.tree, &aligned);
        check()?;
        Some(
            moved
                .iter()
                .filter_map(|(old_atoms, new_atoms)| {
                    Some(Self {
                        old_lines: old.lines.lines_within(old.text, old.tree.span(old_atoms))?,
                        new_lines: new.lines.lines_within(new.text, new.tree.span(new_atoms))?,
                    })
                })
                .collect(),
        )
    }

    pub(super) fn row(&self, line: &DiffLine) -> Option<MovedRow> {
        let (number, own, other) = match line.style {
            DiffSpanStyle::Removed => (line.old_line_no?, self.old_lines, self.new_lines),
            DiffSpanStyle::Added => (line.new_line_no?, self.new_lines, self.old_lines),
            _ => return None,
        };
        (own.start..own.start + own.count)
            .contains(&number)
            .then_some(MovedRow {
                counterpart_line: other.start,
                starts_block: number == own.start,
            })
    }
}

struct Side<'a> {
    text: &'a str,
    tree: SyntaxTree<'a>,
    lines: LineIndex,
}

impl<'a> Side<'a> {
    fn parse(text: &'a str, grammar: &Language, check: &dyn Fn() -> Option<()>) -> Option<Self> {
        Some(Self {
            text,
            tree: SyntaxTree::parse(text, grammar, check)?,
            lines: LineIndex::from_text(text),
        })
    }

    fn line_of(&self, atom: usize) -> usize {
        self.lines.line_at(self.tree.atoms[atom].range.start)
    }
}
