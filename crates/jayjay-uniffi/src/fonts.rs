use jayjay_core::{CodeFontChoices, FontSpecimen};

#[uniffi::export]
fn mono_font_family(preference: String) -> String {
    jayjay_core::mono_font_family(&preference)
}

#[uniffi::export]
fn is_user_font_path(path: String) -> bool {
    jayjay_core::is_user_font_path(std::path::Path::new(&path))
}

#[uniffi::export]
fn font_family_choices(families: Vec<String>) -> Vec<String> {
    jayjay_core::font_family_choices(families)
}

#[uniffi::remote(Record)]
pub struct CodeFontChoices {
    pub coding: Vec<String>,
    pub other: Vec<String>,
}

#[uniffi::export]
fn code_font_choices(monospaced_families: Vec<String>) -> CodeFontChoices {
    CodeFontChoices::new(monospaced_families)
}

#[uniffi::remote(Record)]
pub struct FontSpecimen {
    pub summary: String,
    pub change_id: String,
    pub path: String,
    pub diff: jayjay_core::diff::FileDiff,
}

#[uniffi::export]
fn font_specimen() -> FontSpecimen {
    FontSpecimen::default()
}
