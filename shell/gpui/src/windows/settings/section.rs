use crate::app::theme::Theme;
use crate::ui::icons::glyph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    CodeFont,
    Diff,
    Workflow,
    Integrations,
    Jujutsu,
    DataPrivacy,
    About,
}

impl SettingsSection {
    pub const ALL: [Self; 8] = [
        Self::Appearance,
        Self::CodeFont,
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
            Self::CodeFont => "Code Font",
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
            Self::CodeFont => glyph::BRACES,
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
            Self::CodeFont => t.tok_string,
            Self::Diff | Self::About => t.selected_accent,
            Self::Workflow => t.compare_accent,
            Self::Integrations => t.tok_type,
            Self::Jujutsu => t.success_fg,
            Self::DataPrivacy => t.change_id_prefix,
        }
    }
}
