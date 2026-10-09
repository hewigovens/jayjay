use std::path::PathBuf;

pub fn user_font_dirs() -> Vec<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .filter(|value| !value.is_empty())
        .map(|local| PathBuf::from(local).join(r"Microsoft\Windows\Fonts"))
        .into_iter()
        .collect()
}
