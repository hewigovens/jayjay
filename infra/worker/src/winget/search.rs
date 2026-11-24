use serde::Deserialize;

use super::Package;

const QUERY_FIELDS: &[&str] = &[
    "PackageIdentifier",
    "PackageName",
    "Moniker",
    "Command",
    "Tag",
];

/// A WinGet REST `manifestSearch` body: `Query` and `Inclusions` widen the match, and every `Filters` entry must hold.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct SearchRequest {
    query: Option<RequestMatch>,
    inclusions: Option<Vec<PackageMatch>>,
    filters: Option<Vec<PackageMatch>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PackageMatch {
    package_match_field: String,
    request_match: RequestMatch,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RequestMatch {
    key_word: String,
    match_type: MatchType,
}

#[derive(Deserialize)]
enum MatchType {
    Exact,
    CaseInsensitive,
    StartsWith,
    Substring,
    Wildcard,
    Fuzzy,
    FuzzySubstring,
}

impl SearchRequest {
    pub(crate) fn matches(&self) -> bool {
        let inclusions = self.inclusions.as_deref().unwrap_or_default();
        let widened = self.query.is_none() && inclusions.is_empty()
            || self.query.as_ref().is_some_and(|query| {
                QUERY_FIELDS
                    .iter()
                    .any(|field| query.matches_any(Package::field_values(field)))
            })
            || inclusions.iter().any(PackageMatch::matches);
        widened
            && self
                .filters
                .as_deref()
                .unwrap_or_default()
                .iter()
                .all(PackageMatch::matches)
    }
}

impl PackageMatch {
    fn matches(&self) -> bool {
        self.request_match
            .matches_any(Package::field_values(&self.package_match_field))
    }
}

impl RequestMatch {
    fn matches_any(&self, values: &[&str]) -> bool {
        let keyword = self.key_word.to_lowercase();
        values.iter().any(|value| match self.match_type {
            MatchType::Exact => *value == self.key_word,
            MatchType::CaseInsensitive => value.eq_ignore_ascii_case(&self.key_word),
            MatchType::StartsWith => value.to_lowercase().starts_with(&keyword),
            MatchType::Substring | MatchType::Fuzzy | MatchType::FuzzySubstring => {
                value.to_lowercase().contains(&keyword)
            }
            MatchType::Wildcard => {
                wildcard_matches(keyword.as_bytes(), value.to_lowercase().as_bytes())
            }
        })
    }
}

fn wildcard_matches(pattern: &[u8], value: &[u8]) -> bool {
    let (mut p, mut v) = (0, 0);
    let mut star = None;
    while v < value.len() {
        if p < pattern.len() && pattern[p] == b'*' {
            star = Some((p, v));
            p += 1;
        } else if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == value[v]) {
            (p, v) = (p + 1, v + 1);
        } else if let Some((star_p, star_v)) = star {
            star = Some((star_p, star_v + 1));
            (p, v) = (star_p + 1, star_v + 1);
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|&byte| byte == b'*')
}
