mod action;
mod editing;
mod element;
mod input;
mod render;
mod scroll;
mod state;

pub use action::key_bindings;
pub use state::TextArea;
pub(crate) use state::Tint;

pub(crate) use action::Newline;
pub(crate) use state::{LineLayout, TextLayout, TextLayoutKey};
pub(crate) use state::{TextAreaScrolled, TextAreaUpdated};
