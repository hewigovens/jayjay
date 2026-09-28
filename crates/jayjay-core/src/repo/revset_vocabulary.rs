use super::Repo;
use crate::revset::{RevsetName, RevsetVocabulary};
use crate::types::BookmarkInfo;

impl Repo {
    /// Names the revset bar can complete to; the caller passes the bookmarks it already loaded.
    pub fn revset_vocabulary(&self, bookmarks: &[BookmarkInfo]) -> RevsetVocabulary {
        RevsetVocabulary {
            aliases: self.revset_aliases(),
            bookmarks: bookmark_names(bookmarks),
            tags: self.tag_names(),
        }
    }

    fn tag_names(&self) -> Vec<RevsetName> {
        self.get_repo()
            .view()
            .local_tags()
            .map(|(name, _)| RevsetName::bare(name.as_str()))
            .collect()
    }
}

/// A bookmark is offered under the local name it may have and under each remote it exists on.
fn bookmark_names(bookmarks: &[BookmarkInfo]) -> Vec<RevsetName> {
    bookmarks
        .iter()
        .flat_map(|bookmark| {
            let local = bookmark
                .has_local_target
                .then(|| RevsetName::bookmark(&bookmark.name, None));
            let remotes = bookmark
                .available_remotes
                .iter()
                .map(|remote| RevsetName::bookmark(&bookmark.name, Some(remote)));
            local.into_iter().chain(remotes)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::mock::{bookmark_info, strings};
    use crate::types::BookmarkInfo;

    use super::bookmark_names;

    #[test]
    fn a_bookmark_is_offered_locally_and_on_each_remote_it_exists_on() {
        let tracked = BookmarkInfo {
            available_remotes: strings(&["origin", "upstream"]),
            ..bookmark_info("feature")
        };
        let remote_only = BookmarkInfo {
            has_local_target: false,
            available_remotes: strings(&["origin"]),
            ..bookmark_info("gone")
        };

        assert_eq!(
            bookmark_names(&[tracked, remote_only])
                .iter()
                .map(|name| name.symbol.as_str())
                .collect::<Vec<_>>(),
            [
                "feature",
                "feature@origin",
                "feature@upstream",
                "gone@origin"
            ]
        );
    }
}
