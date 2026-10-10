use jj_lib::dsl_util::format_string;
use jj_lib::revset::{format_symbol, parse_symbol};

use crate::BookmarkInfo;

const BOOKMARK_STACK_JOIN: &str = " | trunk()..";
const ANCESTORS_PREFIX: &str = "::";
const LOCAL_BOOKMARK_PREFIX: &str = "bookmarks(exact:";
const REMOTE_BOOKMARK_PREFIX: &str = "remote_bookmarks(exact:";
const REMOTE_BOOKMARK_SEPARATOR: &str = ", exact:";

pub fn ancestors_revset(change_id: &str) -> String {
    format!("{ANCESTORS_PREFIX}change_id({change_id})")
}

pub fn bookmark_filter_revset(name: &str, remote: Option<&str>) -> String {
    let head = bookmark_head_revset(name, remote);
    format!("{head}{BOOKMARK_STACK_JOIN}{head}")
}

/// The workspace's working-copy change and all of its history, so the lane keeps trunk below it.
pub fn workspace_filter_revset(name: &str) -> String {
    format!("{ANCESTORS_PREFIX}{}", workspace_head_revset(name))
}

pub(super) fn workspace_head_revset(name: &str) -> String {
    format!("{}@", format_symbol(name))
}

/// A bare name fails to resolve while the bookmark is conflicted.
fn bookmark_head_revset(name: &str, remote: Option<&str>) -> String {
    match remote {
        Some(remote) => format!(
            "{REMOTE_BOOKMARK_PREFIX}{}{REMOTE_BOOKMARK_SEPARATOR}{})",
            format_string(name),
            format_string(remote)
        ),
        None => format!("{LOCAL_BOOKMARK_PREFIX}{})", format_string(name)),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BookmarkFilterTarget {
    pub name: String,
    pub head: String,
    pub revset: String,
}

impl BookmarkFilterTarget {
    fn new(title: String, name: &str, remote: Option<&str>) -> Self {
        Self {
            name: title,
            head: bookmark_head_revset(name, remote),
            revset: bookmark_filter_revset(name, remote),
        }
    }

    pub fn for_bookmark(bookmark: &BookmarkInfo) -> Vec<Self> {
        let local = bookmark
            .has_local_target
            .then(|| Self::new(bookmark.name.clone(), &bookmark.name, None));
        let remotes = bookmark
            .available_remotes
            .iter()
            .filter(|remote| !bookmark.tracked_remotes.contains(remote))
            .map(|remote| {
                Self::new(
                    format!("{}@{remote}", bookmark.name),
                    &bookmark.name,
                    Some(remote),
                )
            });
        local.into_iter().chain(remotes).collect()
    }
}

pub(super) fn workspace_filter_name(revset: &str) -> Option<String> {
    let symbol = revset.strip_prefix(ANCESTORS_PREFIX)?.strip_suffix('@')?;
    parse_symbol(symbol)
        .ok()
        .filter(|name| format_symbol(name) == symbol)
}

pub(super) fn bookmark_filter_name(revset: &str) -> Option<String> {
    let (symbol, repeated) = revset.split_once(BOOKMARK_STACK_JOIN)?;
    if symbol != repeated {
        return None;
    }
    if let Some(name) = symbol.strip_prefix(LOCAL_BOOKMARK_PREFIX) {
        return unquote(name.strip_suffix(')')?);
    }
    let args = symbol
        .strip_prefix(REMOTE_BOOKMARK_PREFIX)?
        .strip_suffix(')')?;
    let (name, remote) = args.split_once(REMOTE_BOOKMARK_SEPARATOR)?;
    Some(format!("{}@{}", unquote(name)?, unquote(remote)?))
}

fn unquote(symbol: &str) -> Option<String> {
    parse_symbol(symbol)
        .ok()
        .filter(|name| format_string(name) == symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmark_filters_quote_names_and_parse_back_to_them() {
        assert_eq!(
            bookmark_filter_revset("fix-a|b", None),
            "bookmarks(exact:\"fix-a|b\") | trunk()..bookmarks(exact:\"fix-a|b\")"
        );
        for (name, remote, parsed) in [
            ("feature\"x\\y", None, "feature\"x\\y"),
            ("work", Some("upstream"), "work@upstream"),
        ] {
            assert_eq!(
                bookmark_filter_name(&bookmark_filter_revset(name, remote)).as_deref(),
                Some(parsed)
            );
        }
        for other in ["\"a\" | trunk()..\"a\"", "a | trunk()..a", "::main"] {
            assert_eq!(bookmark_filter_name(other), None, "{other}");
        }
    }

    #[test]
    fn workspace_filters_quote_names_and_parse_back_to_them() {
        assert_eq!(workspace_filter_revset("my-font"), "::my-font@");
        assert_eq!(workspace_filter_revset("my font"), "::\"my font\"@");
        for name in ["my-font", "my font"] {
            assert_eq!(
                workspace_filter_name(&workspace_filter_revset(name)).as_deref(),
                Some(name)
            );
        }
        for other in ["::a@origin@", "::change_id(a)", "a@ | trunk()..a@"] {
            assert_eq!(workspace_filter_name(other), None, "{other}");
        }
    }
}
