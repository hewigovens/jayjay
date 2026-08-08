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
