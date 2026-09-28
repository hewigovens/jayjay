use super::expressions::BookmarkFilterTarget;
use super::presets::{default_revset_preset, revset_presets};
use crate::BookmarkInfo;

pub fn typed_revset(text: &str, bookmarks: &[BookmarkInfo]) -> String {
    let text = text.trim();
    let preset = std::iter::once(default_revset_preset())
        .chain(revset_presets().iter().cloned())
        .find(|preset| preset.label.eq_ignore_ascii_case(text));
    if let Some(preset) = preset {
        return preset.revset;
    }
    bookmarks
        .iter()
        .flat_map(BookmarkFilterTarget::for_bookmark)
        .find(|target| target.name == text)
        .map_or_else(|| text.to_owned(), |target| target.revset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{bookmark_info, strings};
    use crate::revset::{DEFAULT_REVSET, bookmark_filter_revset};

    #[test]
    fn typed_names_pick_presets_and_bookmarks_before_parsing_as_revsets() {
        let bookmarks = [
            BookmarkInfo {
                available_remotes: strings(&["origin"]),
                tracked_remotes: strings(&["origin"]),
                ..bookmark_info("feature")
            },
            BookmarkInfo {
                available_remotes: strings(&["origin"]),
                has_local_target: false,
                ..bookmark_info("work")
            },
        ];
        assert_eq!(typed_revset(" mine ", &bookmarks), "mine()");
        assert_eq!(typed_revset("Default", &bookmarks), DEFAULT_REVSET);
        assert_eq!(
            typed_revset("feature", &bookmarks),
            bookmark_filter_revset("feature", None)
        );
        assert_eq!(
            typed_revset("work@origin", &bookmarks),
            bookmark_filter_revset("work", Some("origin"))
        );
        assert_eq!(typed_revset("work", &bookmarks), "work");
        assert_eq!(typed_revset("feature@origin", &bookmarks), "feature@origin");
        assert_eq!(typed_revset("::feature", &bookmarks), "::feature");
        assert_eq!(
            BookmarkFilterTarget::for_bookmark(&bookmarks[1])[0].head,
            "remote_bookmarks(exact:\"work\", exact:\"origin\")",
            "a remote-only bookmark is selected by its remote ref, not the local name it lacks"
        );
    }
}
