mod description;
mod description_state;
mod header;
mod pane;

pub(crate) use description_state::DescriptionState;
pub(in crate::repo::window) use pane::detail_pane;
