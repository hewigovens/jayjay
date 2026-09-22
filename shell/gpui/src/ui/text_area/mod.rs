mod action;
mod editing;
mod element;
mod input;
mod render;
mod scroll;
mod state;

pub use action::key_bindings;
pub use state::TextArea;

pub(crate) use action::Newline;
pub(in crate::ui::text_area) use state::{LineLayout, TextLayout, TextLayoutKey};
pub(crate) use state::{TextAreaScrolled, TextAreaUpdated};
