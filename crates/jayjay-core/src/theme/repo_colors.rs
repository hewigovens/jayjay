use super::ThemeSeed;
use super::color::{mix, readable_on};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepoColors {
    pub navigation_bg: Option<u32>,
    pub media_frame_bg: u32,
    pub graph_line: u32,
    pub workspace: u32,
    pub bookmark: u32,
    pub change_id_prefix: u32,
    pub commit_id_prefix: u32,
    pub file_added: u32,
    pub file_removed: u32,
    pub file_modified: u32,
    pub file_renamed: u32,
    pub file_lfs: u32,
    pub note_accent: u32,
}

impl RepoColors {
    pub fn dark() -> Self {
        let seed = ThemeSeed::dark();
        Self {
            navigation_bg: Some(0x1b2330),
            media_frame_bg: 0x14171c,
            graph_line: 0x3478f6,
            workspace: 0x42d96b,
            bookmark: 0xde81f4,
            change_id_prefix: 0xde81f4,
            commit_id_prefix: 0x78b7ff,
            file_added: seed.green,
            file_removed: seed.red,
            file_modified: seed.blue,
            file_renamed: seed.blue,
            file_lfs: seed.magenta,
            note_accent: seed.orange,
        }
    }

    pub fn light() -> Self {
        let seed = ThemeSeed::light();
        Self {
            navigation_bg: None,
            media_frame_bg: 0xeef0f3,
            graph_line: 0x5982b8,
            workspace: 0x128a3e,
            bookmark: 0x9635c9,
            change_id_prefix: 0x9635c9,
            commit_id_prefix: 0x175cd3,
            file_added: seed.green,
            file_removed: seed.red,
            file_modified: seed.blue,
            file_renamed: seed.blue,
            file_lfs: seed.magenta,
            note_accent: seed.orange,
        }
    }

    pub fn from_seed(seed: &ThemeSeed) -> Self {
        let bg = seed.background;
        let fg = seed.foreground;
        // Text tokens are drawn on the background, on the surface in the graph pane, and on selection fills.
        let text_floors = [(bg, 4.5), (seed.surface, 4.5), (seed.selection, 3.0)];
        let text = |color: u32| readable_on(color, fg, &text_floors);
        let id_floors = [(bg, 4.5), (seed.surface, 4.5), (seed.selection, 4.5)];
        let id = |color: u32| seed.vivid(color, &id_floors);
        Self {
            navigation_bg: Some(seed.surface),
            media_frame_bg: mix(bg, fg, if seed.is_dark() { 0.03 } else { 0.06 }),
            graph_line: readable_on(seed.blue, fg, &[(bg, 3.0), (seed.surface, 3.0)]),
            workspace: text(seed.green),
            bookmark: id(seed.magenta),
            change_id_prefix: id(seed.magenta),
            commit_id_prefix: id(seed.blue),
            file_added: text(seed.green),
            file_removed: text(seed.red),
            file_modified: text(seed.blue),
            file_renamed: text(seed.blue),
            file_lfs: text(seed.magenta),
            note_accent: text(seed.orange),
        }
    }
}
