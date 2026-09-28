use super::expressions::BookmarkFilterTarget;
use super::state::RevsetFilterState;
use crate::BookmarkInfo;
use crate::fuzzy::rank;

const BOOKMARK_MATCH_LIMIT: usize = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevsetSuggestionKind {
    Current,
    Bookmark,
    Recent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevsetSuggestion {
    pub kind: RevsetSuggestionKind,
    pub title: String,
    pub revset: String,
}

impl RevsetSuggestion {
    fn new(kind: RevsetSuggestionKind, title: &str, revset: &str) -> Self {
        Self {
            kind,
            title: title.to_owned(),
            revset: revset.to_owned(),
        }
    }
}

impl RevsetFilterState {
    pub fn suggestions(&self, query: &str, bookmarks: &[BookmarkInfo]) -> Vec<RevsetSuggestion> {
        let query = query.trim();
        let typing = !query.is_empty() && query != self.revset;
        let mut suggestions = Vec::new();
        if typing {
            let targets: Vec<_> = bookmarks
                .iter()
                .flat_map(BookmarkFilterTarget::for_bookmark)
                .collect();
            let names: Vec<String> = targets.iter().map(|target| target.name.clone()).collect();
            suggestions.extend(
                rank(query, &names)
                    .into_iter()
                    .take(BOOKMARK_MATCH_LIMIT)
                    .map(|index| &targets[index as usize])
                    .map(|target| {
                        RevsetSuggestion::new(
                            RevsetSuggestionKind::Bookmark,
                            &target.name,
                            &target.revset,
                        )
                    }),
            );
        } else {
            suggestions.push(RevsetSuggestion::new(
                RevsetSuggestionKind::Current,
                &self.revset,
                &self.revset,
            ));
        }
        let lowered = query.to_lowercase();
        suggestions.extend(
            self.recent
                .iter()
                .filter(|revset| **revset != self.revset)
                .filter(|revset| !typing || revset.to_lowercase().contains(&lowered))
                .map(|revset| RevsetSuggestion::new(RevsetSuggestionKind::Recent, revset, revset)),
        );
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::bookmark_info;

    #[test]
    fn suggestions_offer_the_current_revset_until_the_user_types_something_else() {
        let bookmark = bookmark_info("feature");
        let mut state = RevsetFilterState::new("a()");
        state.apply("::feat");
        state.apply("b()");
        let kinds = |query: &str| {
            state
                .suggestions(query, std::slice::from_ref(&bookmark))
                .into_iter()
                .map(|suggestion| (suggestion.kind, suggestion.title))
                .collect::<Vec<_>>()
        };

        assert_eq!(
            kinds(""),
            [
                (RevsetSuggestionKind::Current, "b()".to_owned()),
                (RevsetSuggestionKind::Recent, "::feat".to_owned()),
            ]
        );
        assert_eq!(kinds("b()"), kinds(""));
        assert_eq!(
            kinds("feat"),
            [
                (RevsetSuggestionKind::Bookmark, "feature".to_owned()),
                (RevsetSuggestionKind::Recent, "::feat".to_owned()),
            ]
        );
    }
}
