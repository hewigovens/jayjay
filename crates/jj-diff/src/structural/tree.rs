use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{ControlFlow, Range};

use tree_sitter::{Language, Node, ParseOptions, Parser};

const MAX_ATOMS: usize = 16_000;
const ROOT: u16 = u16::MAX;

pub(super) struct SyntaxTree<'a> {
    pub(super) atoms: Vec<Atom<'a>>,
    pub(super) subtrees: Vec<Subtree>,
}

pub(super) struct Atom<'a> {
    pub(super) key: (u16, &'a str),
    pub(super) range: Range<usize>,
}

pub(super) struct Subtree {
    pub(super) hash: u64,
    pub(super) atoms: Range<usize>,
}

struct Frame {
    kind: u16,
    first_atom: usize,
    hasher: DefaultHasher,
}

impl<'a> SyntaxTree<'a> {
    pub(super) fn parse(
        text: &'a str,
        language: &Language,
        check: &dyn Fn() -> Option<()>,
    ) -> Option<Self> {
        let mut parser = Parser::new();
        parser.set_language(language).ok()?;
        let mut progress = |_: &tree_sitter::ParseState| {
            check().map_or(ControlFlow::Break(()), ControlFlow::Continue)
        };
        let tree = parser.parse_with_options(
            &mut |offset, _| &text.as_bytes()[offset..],
            None,
            Some(ParseOptions::new().progress_callback(&mut progress)),
        );
        check()?;
        let tree = tree.filter(|tree| !tree.root_node().has_error())?;
        let mut syntax = Self {
            atoms: Vec::new(),
            subtrees: Vec::new(),
        };
        let mut frames: Vec<Frame> = Vec::new();
        let mut cursor = tree.walk();
        loop {
            check()?;
            let node = cursor.node();
            if !atomic(node) && cursor.goto_first_child() {
                frames.push(Frame {
                    kind: node.kind_id(),
                    first_atom: syntax.atoms.len(),
                    hasher: kind_hasher(node.kind_id()),
                });
                continue;
            }
            let (parent_kind, mut parent_hasher) = match frames.last_mut() {
                Some(frame) => (frame.kind, Some(&mut frame.hasher)),
                None => (ROOT, None),
            };
            syntax.push_gap(
                text,
                node.start_byte(),
                parent_kind,
                parent_hasher.as_deref_mut(),
            )?;
            syntax.push(text, node.byte_range(), node.kind_id(), parent_hasher)?;
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    syntax.push_gap(text, text.len(), ROOT, None)?;
                    return Some(syntax);
                }
                let frame = frames.pop().expect("a frame per entered node");
                let atoms = frame.first_atom..syntax.atoms.len();
                if atoms.is_empty() {
                    continue;
                }
                let hash = frame.hasher.finish();
                if let Some(parent) = frames.last_mut() {
                    hash.hash(&mut parent.hasher);
                }
                if atoms.len() >= 2 {
                    syntax.subtrees.push(Subtree { hash, atoms });
                }
            }
        }
    }

    pub(super) fn keys(&self, atoms: Range<usize>) -> impl Iterator<Item = (u16, &'a str)> {
        self.atoms[atoms].iter().map(|atom| atom.key)
    }

    pub(super) fn span(&self, atoms: &Range<usize>) -> Range<usize> {
        self.atoms[atoms.start].range.start..self.atoms[atoms.end - 1].range.end
    }

    /// Text a grammar leaves outside every leaf, such as the `1` in a CSS `1px`, becomes an atom of the enclosing kind.
    fn push_gap(
        &mut self,
        text: &'a str,
        end: usize,
        parent_kind: u16,
        parent: Option<&mut DefaultHasher>,
    ) -> Option<()> {
        let start = self.atoms.last().map_or(0, |atom| atom.range.end);
        if start < end {
            self.push(text, start..end, parent_kind, parent)?;
        }
        Some(())
    }

    fn push(
        &mut self,
        text: &'a str,
        range: Range<usize>,
        kind: u16,
        parent: Option<&mut DefaultHasher>,
    ) -> Option<()> {
        let source = &text[range.clone()];
        let trimmed = source.trim_start();
        let start = range.start + source.len() - trimmed.len();
        let source = trimmed.trim_end();
        if source.is_empty() {
            return Some(());
        }
        if self.atoms.len() == MAX_ATOMS {
            return None;
        }
        let mut hasher = kind_hasher(kind);
        source.hash(&mut hasher);
        if let Some(parent) = parent {
            hasher.finish().hash(parent);
        }
        self.atoms.push(Atom {
            key: (kind, source),
            range: start..start + source.len(),
        });
        Some(())
    }
}

fn kind_hasher(kind: u16) -> DefaultHasher {
    let mut hasher = DefaultHasher::new();
    kind.hash(&mut hasher);
    hasher
}

fn atomic(node: Node<'_>) -> bool {
    let kind = node.kind();
    node.child_count() == 0
        || kind.contains("string")
        || kind.contains("comment")
        || kind == "char_literal"
        || kind == "regex"
        || kind == "template_string"
}
