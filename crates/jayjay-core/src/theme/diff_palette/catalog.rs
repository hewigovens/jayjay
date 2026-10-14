use super::{
    DiffPalette, catppuccin, default, dracula, gruvbox, nord, one, solarized, tokyo_night,
};

pub fn diff_palettes() -> Vec<DiffPalette> {
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
}
