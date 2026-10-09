use jayjay_core::FontSpecimen;

#[derive(uniffi::Record)]
pub struct MonoFontOption {
    id: String,
    title: String,
    font_names: Vec<String>,
}

#[uniffi::export]
fn mono_font_options() -> Vec<MonoFontOption> {
    jayjay_core::MONO_FONT_OPTIONS
        .iter()
        .map(|option| MonoFontOption {
            id: option.id.to_owned(),
            title: option.title.to_owned(),
            font_names: option
                .font_names
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
        })
        .collect()
}

#[uniffi::export]
fn is_user_font_path(path: String) -> bool {
    jayjay_core::is_user_font_path(std::path::Path::new(&path))
}

#[uniffi::export]
fn my_mono_font_families(families: Vec<String>) -> Vec<String> {
    jayjay_core::my_mono_font_families(families)
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
