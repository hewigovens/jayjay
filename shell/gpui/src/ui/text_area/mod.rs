mod action;
mod editing;
mod element;
mod input;
mod render;
mod state;

pub use action::key_bindings;
pub use state::TextArea;

pub(crate) use action::Newline;
pub(crate) use state::TextAreaUpdated;
pub(in crate::ui::text_area) use state::{LineLayout, TextLayout, TextLayoutKey};
