pub mod revset;
mod stacked_pr;
pub mod toggles;
pub(crate) mod toolbar;
pub mod view_model;
pub mod window;

pub(crate) use stacked_pr::CoreStackedPrProvider;
pub use stacked_pr::StackedPrProvider;
pub use window::{ActivePane, RepoWindow, open_repo_window};
