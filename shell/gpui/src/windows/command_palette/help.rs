//! Searchable palette help topics that open the public user guide at a topic anchor.

use std::sync::OnceLock;

use serde::Deserialize;

use super::actions::ACTIONS;
use crate::app::links::GUIDE_URL;

// Same source file the SwiftUI shell bundles for its palette help topics, so both shells stay aligned (shell-parity: searchable help topics).
const HELP_FEATURES_JSON: &str = include_str!("../../../../mac/Resources/HelpFeatures.json");

/// One help topic: palette title, fuzzy-search haystack, and the public-guide anchor it opens.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HelpTopic {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub guide_anchor: String,
}

impl HelpTopic {
    pub(super) fn palette_title(&self) -> String {
        format!("Help: {}", self.title)
    }

    /// The public guide opened at this topic's anchor; GPUI has no bundled Help Book, so this mirrors SwiftUI's online-guide fallback.
    pub(super) fn guide_url(&self) -> String {
        format!("{GUIDE_URL}#{}", self.guide_anchor)
    }

    pub(super) fn search_text(&self) -> String {
        format!(
            "{} {} {} {}",
            self.palette_title(),
            self.keywords.join(" "),
            self.summary,
            self.category
        )
    }
}

pub(super) fn topics() -> &'static [HelpTopic] {
    static TOPICS: OnceLock<Vec<HelpTopic>> = OnceLock::new();
    // Fall back to no topics on a malformed edit, matching SwiftUI's decode behavior; the unit test below gates drift.
    TOPICS.get_or_init(|| serde_json::from_str(HELP_FEATURES_JSON).unwrap_or_default())
}

/// Palette rows list `ACTIONS` first, then help topics; map a row index past the actions to its topic.
pub(super) fn topic_for_row(row_ix: usize) -> Option<&'static HelpTopic> {
    row_ix
        .checked_sub(ACTIONS.len())
        .and_then(|ix| topics().get(ix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topics_load_from_the_shared_swiftui_source() {
        let topics = topics();
        assert!(!topics.is_empty(), "bundled HelpFeatures.json must parse");
        for topic in topics {
            assert!(!topic.title.is_empty(), "topic {} needs a title", topic.id);
            assert!(
                !topic.guide_anchor.is_empty(),
                "topic {} needs a guide anchor",
                topic.id
            );
        }
    }

    #[test]
    fn guide_url_appends_topic_anchor_to_canonical_guide() {
        let topic = topics()
            .iter()
            .find(|topic| topic.id == "review-notes")
            .expect("review-notes topic");
        assert_eq!(
            topic.guide_url(),
            "https://jayjay.hewig.dev/guide.html#review-notes"
        );
    }

    #[test]
    fn topic_for_row_maps_indices_past_actions() {
        assert!(topic_for_row(0).is_none());
        assert!(topic_for_row(ACTIONS.len() - 1).is_none());
        let first = topic_for_row(ACTIONS.len()).expect("first help topic row");
        assert_eq!(first.id, topics()[0].id);
        assert!(topic_for_row(ACTIONS.len() + topics().len()).is_none());
    }
}
