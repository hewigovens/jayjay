use crate::BookmarkInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevsetEndpoint {
    pub rev: String,
    pub label: String,
}

impl RevsetEndpoint {
    pub fn bookmark(name: &str) -> Self {
        Self {
            rev: quoted_symbol(name),
            label: name.to_owned(),
        }
    }

    /// A bookmark with no local target is only reachable under its remote-qualified name.
    pub fn for_bookmark(bookmark: &BookmarkInfo) -> Self {
        if !bookmark.has_local_target
            && let Some(remote) = bookmark.available_remotes.first()
        {
            return Self::bookmark(&format!("{}@{remote}", bookmark.name));
        }
        Self::bookmark(&bookmark.name)
    }

    pub fn trunk() -> Self {
        Self {
            rev: "trunk()".to_owned(),
            label: "trunk".to_owned(),
        }
    }
}

/// Bookmark names reach jj as revset symbols, where an unquoted name can parse as an operator.
pub fn quoted_symbol(symbol: &str) -> String {
    let escaped = symbol.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{bookmark_info, strings};

    #[test]
    fn quotes_and_escapes_bookmark_symbols() {
        assert_eq!(RevsetEndpoint::bookmark("feature-x").rev, "\"feature-x\"");
        assert_eq!(
            RevsetEndpoint::bookmark("feature\"x").rev,
            "\"feature\\\"x\""
        );
        assert_eq!(RevsetEndpoint::bookmark("feature-x").label, "feature-x");
    }

    #[test]
    fn a_remote_only_bookmark_is_selected_by_its_remote_qualified_name() {
        let mut bookmark = BookmarkInfo {
            is_tracking_remote: true,
            available_remotes: strings(&["origin"]),
            has_local_target: false,
            ..bookmark_info("feature")
        };

        assert_eq!(
            RevsetEndpoint::for_bookmark(&bookmark).rev,
            "\"feature@origin\""
        );

        bookmark.has_local_target = true;
        assert_eq!(RevsetEndpoint::for_bookmark(&bookmark).rev, "\"feature\"");
    }
}
