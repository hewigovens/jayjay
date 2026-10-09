use std::path::PathBuf;

pub fn user_font_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("/usr/local/share/fonts")];
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::home_dir().map(|home| home.join(".local/share")));
    dirs.extend(data_home.map(|data| data.join("fonts")));
    dirs.extend(std::env::home_dir().map(|home| home.join(".fonts")));
    dirs
}
