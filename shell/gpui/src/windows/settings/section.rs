use crate::app::theme::Theme;
use crate::ui::icons::glyph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    Diff,
    Workflow,
    Integrations,
    Jujutsu,
    DataPrivacy,
    About,
}

impl SettingsSection {
    pub const ALL: [Self; 7] = [
        Self::Appearance,
        Self::Diff,
        Self::Workflow,
        Self::Integrations,
        Self::Jujutsu,
        Self::DataPrivacy,
        Self::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Diff => "Diff & Files",
            Self::Workflow => "Workflow",
            Self::Integrations => "Integrations",
            Self::Jujutsu => "Jujutsu",
            Self::DataPrivacy => "Data & Privacy",
            Self::About => "About",
        }
    }

    pub(super) fn glyph(self) -> &'static str {
        match self {
            Self::Appearance => glyph::EYE,
            Self::Diff => glyph::COLUMNS,
            Self::Workflow => glyph::LIST_CHECKS,
            Self::Integrations => glyph::GEAR,
            Self::Jujutsu => glyph::GIT_BRANCH,
            Self::DataPrivacy => glyph::HARD_DRIVE,
            Self::About => glyph::INFO,
        }
    }

    pub(super) fn color(self, t: &Theme) -> u32 {
        match self {
            Self::Appearance => t.tok_keyword,
            Self::Diff | Self::About => t.selected_accent,
            Self::Workflow => t.compare_accent,
            Self::Integrations => t.tok_type,
            Self::Jujutsu => t.success_fg,
            Self::DataPrivacy => t.change_id_prefix,
        }
    }
}
