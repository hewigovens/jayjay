use jayjay_core::BookmarkInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevsetEndpoint {
    pub(crate) rev: String,
    pub(crate) label: String,
}

pub fn bookmark_endpoint(name: &str) -> RevsetEndpoint {
    RevsetEndpoint {
        rev: quoted_symbol(name),
        label: name.to_string(),
    }
}

pub fn bookmark_endpoint_for_info(bookmark: &BookmarkInfo) -> RevsetEndpoint {
    if !bookmark.has_local_target
        && let Some(remote) = bookmark.available_remotes.first()
    {
        return bookmark_endpoint(&format!("{}@{remote}", bookmark.name));
    }
    bookmark_endpoint(&bookmark.name)
}

pub fn trunk_endpoint() -> RevsetEndpoint {
    RevsetEndpoint {
        rev: "trunk()".to_string(),
        label: "trunk".to_string(),
    }
}

pub fn quoted_symbol(symbol: &str) -> String {
    let escaped = symbol.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_bookmark_symbols() {
        assert_eq!(quoted_symbol("feature-x"), "\"feature-x\"");
        assert_eq!(quoted_symbol("feature\"x"), "\"feature\\\"x\"");
    }
}
