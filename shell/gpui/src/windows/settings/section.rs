use crate::app::theme::Theme;
use crate::ui::icons::glyph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    Fonts,
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
        Self::Fonts,
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
            Self::Fonts => "Fonts",
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
            Self::Fonts => glyph::BRACES,
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
            Self::Appearance => t.syntax.keyword,
            Self::Fonts => t.syntax.string,
            Self::Diff | Self::About => t.selected_accent,
            Self::Workflow => t.compare_accent,
            Self::Integrations => t.syntax.ty,
            Self::Jujutsu => t.success_fg,
            Self::DataPrivacy => t.change_id_prefix,
        }
    }
}
