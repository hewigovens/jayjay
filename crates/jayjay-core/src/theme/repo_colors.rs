#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepoColors {
    pub graph_line: u32,
    pub workspace: u32,
    pub bookmark: u32,
    pub change_id_prefix: u32,
    pub commit_id_prefix: u32,
}

impl RepoColors {
    pub fn dark() -> Self {
        Self {
            graph_line: 0x3478f6,
            workspace: 0x42d96b,
            bookmark: 0xd86bf2,
            change_id_prefix: 0xc099f5,
            commit_id_prefix: 0x78b7ff,
        }
    }

    pub fn light() -> Self {
        Self {
            graph_line: 0x5982b8,
            workspace: 0x128a3e,
            bookmark: 0x9635c9,
            change_id_prefix: 0x7c4fc2,
            commit_id_prefix: 0x175cd3,
        }
    }
}

pub fn repo_colors(is_dark: bool) -> RepoColors {
    if is_dark {
        RepoColors::dark()
    } else {
        RepoColors::light()
    }
}
