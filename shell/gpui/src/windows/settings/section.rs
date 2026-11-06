use jayjay_i18n::settings;

use crate::app::theme::Theme;
use crate::ui::icons::glyph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    Fonts,
    Diff,
    KeyboardShortcuts,
    Workflow,
    Integrations,
    Jujutsu,
    DataPrivacy,
    Flock,
    About,
}

impl SettingsSection {
    pub const ALL: [Self; 10] = [
        Self::Appearance,
        Self::Fonts,
        Self::Diff,
        Self::KeyboardShortcuts,
        Self::Workflow,
        Self::Integrations,
        Self::Jujutsu,
        Self::DataPrivacy,
        Self::Flock,
        Self::About,
    ];

    pub fn label(self) -> String {
        match self {
            Self::Appearance => settings::page_appearance(),
            Self::Fonts => settings::page_fonts(),
            Self::KeyboardShortcuts => jayjay_i18n::menu::keyboard_shortcuts(),
            Self::Diff => settings::page_diff(),
            Self::Workflow => settings::page_workflow(),
            Self::Integrations => settings::page_integrations(),
            Self::Jujutsu => settings::page_jujutsu(),
            Self::DataPrivacy => settings::page_data_privacy(),
            Self::Flock => settings::page_flock(),
            Self::About => settings::page_about(),
        }
    }

    pub(super) fn glyph(self) -> &'static str {
        match self {
            Self::Appearance => glyph::PALETTE,
            Self::Fonts => glyph::CASE_SENSITIVE,
            Self::KeyboardShortcuts => glyph::KEYBOARD,
            Self::Diff => glyph::DIFF,
            Self::Workflow => glyph::LIST_CHECKS,
            Self::Integrations => glyph::WRENCH,
            Self::Jujutsu => glyph::GIT_BRANCH,
            Self::DataPrivacy => glyph::SHIELD,
            Self::Flock => glyph::BIRD,
            Self::About => glyph::INFO,
        }
    }

    pub(super) fn color(self, t: &Theme) -> u32 {
        match self {
            Self::Appearance => t.change_id_prefix,
            Self::Fonts | Self::KeyboardShortcuts => t.selected_accent,
            Self::Diff | Self::Flock | Self::About => t.selected_accent,
            Self::Workflow => t.compare_accent,
            Self::Integrations => t.syntax.ty,
            Self::Jujutsu => t.success_fg,
            Self::DataPrivacy => t.change_id_prefix,
        }
    }
}
