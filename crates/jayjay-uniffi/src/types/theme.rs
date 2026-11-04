use jayjay_core::{ColorTheme, DiffThemeColors, RepoColors, ThemeSeed};

#[uniffi::remote(Record)]
pub struct ColorTheme {
    pub id: String,
    pub name: String,
    pub is_dark: bool,
    pub seed: ThemeSeed,
    pub colors: DiffThemeColors,
    pub repo: RepoColors,
}

#[uniffi::remote(Record)]
pub struct ThemeSeed {
    pub background: u32,
    pub foreground: u32,
    pub surface: u32,
    pub muted: u32,
    pub border: u32,
    pub accent: u32,
    pub selection: u32,
    pub red: u32,
    pub green: u32,
    pub yellow: u32,
    pub blue: u32,
    pub magenta: u32,
    pub cyan: u32,
    pub orange: u32,
}

#[uniffi::remote(Record)]
pub struct DiffThemeColors {
    pub added_bg: u32,
    pub removed_bg: u32,
    pub context_bg: u32,
    pub separator_bg: u32,
    pub conflict_header_bg: u32,
    pub conflict_section_bg: u32,
    pub conflict_content_bg: u32,
    pub conflict_header_fg: u32,
    pub conflict_section_fg: u32,
    pub conflict_stripe: u32,
    pub conflict_stripe_alpha: f64,
    pub added_word_bg: u32,
    pub removed_word_bg: u32,
    pub gutter_bg: u32,
    pub gutter_fg: u32,
    pub gutter_added_fg: u32,
    pub gutter_removed_fg: u32,
    pub text_context: u32,
    pub text_added: u32,
    pub text_removed: u32,
    pub text_dim: u32,
    pub tok_keyword: u32,
    pub tok_string: u32,
    pub tok_comment: u32,
    pub tok_number: u32,
    pub tok_type: u32,
    pub group_stripe: u32,
    pub group_stripe_alpha: f64,
    pub find_match_bg: u32,
    pub find_match_fg: u32,
}

#[uniffi::remote(Record)]
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
