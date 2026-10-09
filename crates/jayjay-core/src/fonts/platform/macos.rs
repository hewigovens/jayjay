use std::path::PathBuf;

pub fn user_font_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("/Library/Fonts")];
    dirs.extend(std::env::home_dir().map(|home| home.join("Library/Fonts")));
    dirs
}
