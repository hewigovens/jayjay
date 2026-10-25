use super::expressions::BookmarkFilterTarget;
use super::presets::{RevsetPreset, revset_presets};
use crate::BookmarkInfo;

pub fn typed_revset(
    text: &str,
    bookmarks: &[BookmarkInfo],
    default_revset: &RevsetPreset,
) -> String {
    let text = text.trim();
    let preset = std::iter::once(default_revset.clone())
        .chain(revset_presets())
        .find(|preset| {
            preset.id.eq_ignore_ascii_case(text) || preset.label.eq_ignore_ascii_case(text)
        });
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
    use crate::revset::{bookmark_filter_revset, default_revset_preset};

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
        let configured = RevsetPreset {
            revset: "::@".to_owned(),
            ..default_revset_preset()
        };
        let typed_revset =
            |text: &str, bookmarks: &[BookmarkInfo]| typed_revset(text, bookmarks, &configured);
        assert_eq!(typed_revset(" mine ", &bookmarks), "mine()");
        assert_eq!(typed_revset("Default", &bookmarks), "::@");
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
