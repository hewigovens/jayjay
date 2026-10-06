#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub tracked_remotes: Vec<String>,
}

impl TagInfo {
    pub fn remotes_of<'a>(tags: &'a [Self], name: &str) -> &'a [String] {
        tags.iter()
            .find(|tag| tag.name == name)
            .map_or(&[], |tag| tag.tracked_remotes.as_slice())
    }

    pub fn is_on_remote(tags: &[Self], name: &str) -> bool {
        !Self::remotes_of(tags, name).is_empty()
    }
}
