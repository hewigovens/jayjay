use jayjay_core::{CodeFontChoices, font_family_choices};

use super::platform;

pub(crate) struct FontCatalog {
    pub(crate) interface: Vec<String>,
    pub(crate) code: CodeFontChoices,
}

impl FontCatalog {
    pub(crate) fn load() -> Self {
        Self {
            interface: font_family_choices(platform::user_families()),
            code: CodeFontChoices::new(platform::monospaced_families()),
        }
    }
}
