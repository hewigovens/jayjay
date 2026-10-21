use gpui::{Pixels, Point, SharedString};

use crate::app::fonts::FontCatalog;

#[derive(Default)]
pub(super) struct FontCatalogState {
    pub catalog: Option<FontCatalog>,
    pub loading: bool,
    pub pending_dropdown: Option<(SharedString, Point<Pixels>)>,
}
