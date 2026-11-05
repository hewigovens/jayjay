use std::sync::LazyLock;

use super::{ColorTheme, catppuccin, default, dracula, gruvbox, nord, one, solarized, tokyo_night};

pub(super) static CATALOG: LazyLock<Vec<ColorTheme>> = LazyLock::new(|| {
    vec![
        default::light(),
        solarized::light(),
        gruvbox::light(),
        catppuccin::light(),
        one::light(),
        default::dark(),
        solarized::dark(),
        gruvbox::dark(),
        catppuccin::dark(),
        one::dark(),
        nord::dark(),
        dracula::dark(),
        tokyo_night::dark(),
    ]
});

pub fn color_themes() -> Vec<ColorTheme> {
    CATALOG.iter().map(ColorTheme::localized).collect()
}
