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
            Self::Appearance => glyph::PALETTE,
            Self::Fonts => glyph::BRACES,
            Self::Diff => glyph::DIFF,
            Self::Workflow => glyph::LIST_CHECKS,
            Self::Integrations => glyph::WRENCH,
            Self::Jujutsu => glyph::GIT_BRANCH,
            Self::DataPrivacy => glyph::SHIELD,
            Self::About => glyph::INFO,
        }
    }

    pub(super) fn color(self, t: &Theme) -> u32 {
        match self {
            Self::Appearance => t.change_id_prefix,
            Self::Fonts => t.selected_accent,
            Self::Diff | Self::About => t.selected_accent,
            Self::Workflow => t.compare_accent,
            Self::Integrations => t.syntax.ty,
            Self::Jujutsu => t.success_fg,
            Self::DataPrivacy => t.change_id_prefix,
        }
    }
}
