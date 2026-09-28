use std::collections::HashSet;

use super::expressions::symbol_text;
use super::functions::FUNCTIONS;
use crate::utf16::{byte_offset, utf16_offset};

const COMPLETION_LIMIT: usize = 20;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RevsetCompletionKind {
    Function,
    Alias,
    Bookmark,
    Tag,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevsetCompletion {
    pub kind: RevsetCompletionKind,
    /// Text to insert in place of `start .. start + len`.
    pub text: String,
    /// UTF-16 offset of the replaced text, counted from the start of the input.
    pub start: u32,
    /// UTF-16 length of the replaced text.
    pub len: u32,
}

/// A name a completion can offer and the revset text that stands for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevsetName {
    pub name: String,
    /// The symbol a shell inserts, quoted when jj would not read the name as one.
    pub symbol: String,
}

impl RevsetName {
    pub(crate) fn bare(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            symbol: symbol_text(name),
        }
    }

    /// A function name the user calls with `(`.
    pub(crate) fn called(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            symbol: format!("{}(", symbol_text(name)),
        }
    }

    /// A bookmark, on `remote` when it is a remote bookmark.
    pub(crate) fn bookmark(name: &str, remote: Option<&str>) -> Self {
        match remote {
            Some(remote) => Self {
                name: format!("{name}@{remote}"),
                symbol: format!("{}@{}", symbol_text(name), symbol_text(remote)),
            },
            None => Self::bare(name),
        }
    }
}

/// What a position can offer besides jj's built-in functions; both shells load it with the repository.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RevsetVocabulary {
    pub aliases: Vec<RevsetName>,
    pub bookmarks: Vec<RevsetName>,
    pub tags: Vec<RevsetName>,
}

/// Candidates that fit the symbol the cursor sits in, matched on the part before the cursor, each replacing the whole symbol.
pub fn revset_completions(
    text: &str,
    cursor: u32,
    vocabulary: &RevsetVocabulary,
) -> Vec<RevsetCompletion> {
    let Some(token) = Token::at(text, cursor) else {
        return Vec::new();
    };
    let functions = FUNCTIONS.iter().map(|(name, arguments)| {
        let call = format!("{name}{}", arguments.opening());
        (RevsetCompletionKind::Function, *name, call)
    });
    let names = [
        (RevsetCompletionKind::Alias, &vocabulary.aliases),
        (RevsetCompletionKind::Bookmark, &vocabulary.bookmarks),
        (RevsetCompletionKind::Tag, &vocabulary.tags),
    ]
    .into_iter()
    .flat_map(|(kind, names)| {
        names
            .iter()
            .map(move |name| (kind, name.name.as_str(), name.symbol.clone()))
    });
    let mut seen = HashSet::new();
    let mut matched: Vec<_> = functions
        .chain(names)
        .filter(|(_, name, _)| token.matches(name))
        // A bookmark named like a function is a different revset, so only a call shadows a call and a symbol a symbol.
        .filter(|(_, name, insertion)| seen.insert((*name, insertion.contains('('))))
        .collect();
    matched.sort_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));
    matched.truncate(COMPLETION_LIMIT);
    matched
        .into_iter()
        .map(|(kind, name, insertion)| RevsetCompletion {
            kind,
            // The call the user already opened stays as it is.
            text: if token.is_called && insertion.contains('(') {
                name.to_owned()
            } else {
                insertion
            },
            start: token.start as u32,
            len: token.len as u32,
        })
        .collect()
}

/// The symbol the cursor sits in.
struct Token<'a> {
    typed: &'a str,
    is_called: bool,
    /// UTF-16 offsets, as AppKit reports a caret.
    start: usize,
    len: usize,
}

impl<'a> Token<'a> {
    fn at(text: &'a str, cursor: u32) -> Option<Self> {
        let caret = byte_offset(text, cursor as usize);
        let mut start = None;
        for (index, ch) in text[..caret].char_indices().rev() {
            // A run of dots is one of jj's range operators, not part of the symbol.
            if !is_symbol_char(ch) || (ch == '.' && text[..index].ends_with('.')) {
                break;
            }
            start = Some(index);
        }
        let start = start?;
        // Inside a quoted string pattern a revset name is not what the argument takes.
        if is_quoted(&text[..start]) {
            return None;
        }
        let rest = &text[caret..];
        let end = caret
            + rest
                .char_indices()
                .find(|(index, ch)| {
                    !is_symbol_char(*ch) || (*ch == '.' && rest[index + 1..].starts_with('.'))
                })
                .map_or(rest.len(), |(index, _)| index);
        let start_utf16 = utf16_offset(text, start);
        Some(Self {
            typed: &text[start..caret],
            is_called: text[end..].starts_with('('),
            start: start_utf16,
            len: utf16_offset(text, end) - start_utf16,
        })
    }

    fn matches(&self, name: &str) -> bool {
        name.get(..self.typed.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(self.typed))
    }
}

/// The characters jj reads as part of an unquoted symbol.
fn is_symbol_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '.' | '-' | '+' | '*' | '/' | '@')
}

/// Whether `text` ends inside a string literal: `"…"` escapes with a backslash, `'…'` is raw.
fn is_quoted(text: &str) -> bool {
    let mut open = None;
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        match (open, ch) {
            (Some('"'), '\\') => {
                chars.next();
            }
            (Some(quote), ch) if ch == quote => open = None,
            (None, '"' | '\'') => open = Some(ch),
            _ => {}
        }
    }
    open.is_some()
}

#[cfg(test)]
mod tests {
    use jj_lib::revset::parse_symbol;

    use super::*;

    fn vocabulary() -> RevsetVocabulary {
        RevsetVocabulary {
            aliases: vec![RevsetName::called("reviewed"), RevsetName::bare("wip")],
            bookmarks: vec![
                RevsetName::bookmark("fix-a|b", None),
                RevsetName::bookmark("fix", Some("origin")),
                RevsetName::bookmark("main", None),
            ],
            tags: vec![RevsetName::bare("v1.0.0")],
        }
    }

    fn completions(text: &str, cursor: u32) -> Vec<(RevsetCompletionKind, String)> {
        let completions = revset_completions(text, cursor, &vocabulary());
        assert!(
            completions
                .windows(2)
                .all(|pair| (pair[0].kind, &pair[0].text) < (pair[1].kind, &pair[1].text)),
            "candidates should be ordered: {completions:?}"
        );
        completions
            .into_iter()
            .map(|completion| (completion.kind, completion.text))
            .collect()
    }

    #[test]
    fn candidates_follow_the_functions_aliases_and_names_that_fit_the_token() {
        assert_eq!(
            completions("mine() | fix", 12),
            [
                (RevsetCompletionKind::Bookmark, "\"fix-a|b\"".to_owned()),
                (RevsetCompletionKind::Bookmark, "fix@origin".to_owned()),
            ]
        );
        assert_eq!(
            completions("ancestors(mai", 13),
            [(RevsetCompletionKind::Bookmark, "main".to_owned())]
        );
        assert_eq!(
            completions("mine() | fix@or", 15),
            [(RevsetCompletionKind::Bookmark, "fix@origin".to_owned())],
            "a remote bookmark is one token"
        );
    }

    #[test]
    fn a_range_operator_ends_the_symbol_before_the_cursor() {
        for text in ["main..fix", "main...fix"] {
            assert_eq!(
                completions(text, text.len() as u32),
                [
                    (RevsetCompletionKind::Bookmark, "\"fix-a|b\"".to_owned()),
                    (RevsetCompletionKind::Bookmark, "fix@origin".to_owned()),
                ],
                ".. is an operator, not part of the name being typed: {text}"
            );
        }
        assert_eq!(
            completions("main..", 6),
            [],
            "nothing to replace after an operator"
        );
        assert_eq!(
            completions("v1.0.0..v1", 10),
            [(RevsetCompletionKind::Tag, "v1.0.0".to_owned())],
            "a single dot stays inside a name"
        );
    }

    #[test]
    fn a_quoted_argument_offers_nothing() {
        for text in [
            "description(\"fix",
            "description(\"fix bug",
            "description(\"fix \\\"bug",
            "author('fix bug",
        ] {
            assert_eq!(
                completions(text, text.len() as u32),
                [],
                "{text} is a string pattern, not a symbol"
            );
        }
        let closed = "description(\"fix\") | fix";
        assert_eq!(
            completions(closed, closed.len() as u32).len(),
            2,
            "a closed pattern does not hide the symbol after the operator"
        );
    }

    #[test]
    fn a_token_completes_to_a_function_whose_call_is_closed_or_left_open() {
        assert_eq!(
            completions("mine() | min", 12),
            [(RevsetCompletionKind::Function, "mine()".to_owned())]
        );
        assert_eq!(
            completions("anc", 3),
            [(RevsetCompletionKind::Function, "ancestors(".to_owned())]
        );
        assert_eq!(
            completions("mine() | au", 12),
            [
                (RevsetCompletionKind::Function, "author(".to_owned()),
                (RevsetCompletionKind::Function, "author_date(".to_owned()),
                (RevsetCompletionKind::Function, "author_email(".to_owned()),
                (RevsetCompletionKind::Function, "author_name(".to_owned()),
            ]
        );
    }

    #[test]
    fn aliases_offer_their_name_and_tags_their_symbol() {
        assert_eq!(
            completions("rev", 3),
            [(RevsetCompletionKind::Alias, "reviewed(".to_owned())]
        );
        assert_eq!(
            completions("wip", 3),
            [(RevsetCompletionKind::Alias, "wip".to_owned())]
        );
        assert_eq!(
            completions("v1", 2),
            [(RevsetCompletionKind::Tag, "v1.0.0".to_owned())]
        );
    }

    #[test]
    fn the_whole_symbol_is_replaced_when_the_cursor_sits_inside_it() {
        let completions = revset_completions("@ | main..", 6, &vocabulary());
        assert_eq!(
            completions,
            [RevsetCompletion {
                kind: RevsetCompletionKind::Bookmark,
                text: "main".to_owned(),
                start: 4,
                len: 4,
            }],
            "`ma|in` matches on `ma` and replaces `main`, stopping at the range operator"
        );

        let called = revset_completions("anc(@)", 2, &vocabulary());
        assert_eq!(
            (called[0].text.as_str(), called[0].start, called[0].len),
            ("ancestors", 0, 3),
            "an already opened call keeps its parenthesis"
        );
    }

    #[test]
    fn a_bookmark_named_like_a_function_stays_a_separate_choice() {
        let vocabulary = RevsetVocabulary {
            bookmarks: vec![RevsetName::bookmark("mine", None)],
            ..vocabulary()
        };
        assert_eq!(
            revset_completions("min", 3, &vocabulary)
                .into_iter()
                .map(|completion| (completion.kind, completion.text))
                .collect::<Vec<_>>(),
            [
                (RevsetCompletionKind::Function, "mine()".to_owned()),
                (RevsetCompletionKind::Bookmark, "mine".to_owned()),
            ]
        );
    }

    #[test]
    fn a_built_in_function_wins_over_an_alias_of_the_same_name() {
        let vocabulary = RevsetVocabulary {
            aliases: vec![RevsetName::called("trunk")],
            ..vocabulary()
        };
        assert_eq!(
            revset_completions("tru", 3, &vocabulary),
            [RevsetCompletion {
                kind: RevsetCompletionKind::Function,
                text: "trunk()".to_owned(),
                start: 0,
                len: 3,
            }]
        );
    }

    #[test]
    fn the_replaced_range_covers_the_symbol_before_the_cursor_in_utf16_units() {
        for text in ["😀 ancestors(", "mine() | "] {
            let cursor = text.encode_utf16().count() as u32;
            assert!(
                revset_completions(text, cursor, &vocabulary()).is_empty(),
                "the cursor is not in a symbol: {text}"
            );
        }

        let text = "😀 mai";
        let completions =
            revset_completions(text, text.encode_utf16().count() as u32, &vocabulary());
        assert_eq!((completions[0].start, completions[0].len), (3, 3));
    }

    #[test]
    fn a_name_is_quoted_only_when_jj_would_not_read_it_as_a_symbol() {
        assert_eq!(RevsetName::bare("main").symbol, "main");
        assert_eq!(RevsetName::bare("v1.0.0").symbol, "v1.0.0");
        assert_eq!(RevsetName::bare("fix-a|b").symbol, "\"fix-a|b\"");
        assert_eq!(RevsetName::called("wip").symbol, "wip(");
        assert_eq!(
            RevsetName::bookmark("fix-a|b", Some("origin")).symbol,
            "\"fix-a|b\"@origin"
        );
        for name in ["main", "v1.0.0", "fix-a|b"] {
            assert_eq!(
                parse_symbol(&RevsetName::bare(name).symbol).ok().as_deref(),
                Some(name),
                "{name}"
            );
        }
    }
}
