use std::path::Path;

const RUST_KEYWORDS: &str = "abstract as async await become box break const continue crate do dyn else enum extern false final fn for gen if impl in let loop macro match mod move mut override priv pub ref return self static struct super trait true try type typeof unsafe unsized use virtual where while yield";
const SWIFT_KEYWORDS: &str = "Any as associatedtype break case catch class continue default defer deinit do else enum extension fallthrough false fileprivate for func guard if import in init inout internal is let nil operator private protocol public repeat rethrows return self static struct subscript super switch throw throws true try typealias var where while";

#[derive(Clone, Copy)]
pub(crate) enum Identifier {
    Message,
    Variable,
}

impl Identifier {
    pub(crate) fn check(self, path: &Path, name: &str) {
        let (kind, separator) = match self {
            Self::Message => ("message", '-'),
            Self::Variable => ("name", '_'),
        };
        let valid = name.split(separator).all(|word| {
            word.starts_with(|c: char| c.is_ascii_lowercase())
                && word
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        });
        let keyword = RUST_KEYWORDS.split(' ').any(|k| k == snake(name))
            || SWIFT_KEYWORDS.split(' ').any(|k| k == camel(name));
        assert!(
            valid && !keyword,
            "{}: {kind} `{name}` must be lowercase words joined by `{separator}` and not a Rust or Swift keyword",
            path.display()
        );
    }
}

pub(crate) fn snake(name: &str) -> String {
    name.replace('-', "_")
}

pub(crate) fn camel(name: &str) -> String {
    let pascal = pascal(name);
    let mut chars = pascal.chars();
    chars
        .next()
        .map(|first| first.to_ascii_lowercase().to_string() + chars.as_str())
        .unwrap_or_default()
}

pub(crate) fn pascal(name: &str) -> String {
    name.split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect()
}
