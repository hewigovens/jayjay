mod colors;
mod content_row;
mod gutter;
mod hunk_tag;
mod note_row;

pub use colors::{line_bg_color, line_text_color};
pub use content_row::{conflict_stripe_overlay, content_row, selection_overlay};
pub use gutter::{
    GUTTER_NUMBER_WIDTH, NOTE_DOT_WIDTH, REVIEW_STRIPE_WIDTH, content_row_tint, gutter_cell,
    gutter_column, interactive_gutter_column, interactive_gutter_row, review_stripe,
    review_stripe_spacer,
};
pub use hunk_tag::tag_for_hunk;
pub use note_row::{note_content_row, note_dot_cell, note_gutter_row};

#[cfg(test)]
mod tests;
