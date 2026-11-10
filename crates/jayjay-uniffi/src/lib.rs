uniffi::setup_scaffolding!();

#[cfg(feature = "desktop")]
mod agent_skills;
#[cfg(feature = "desktop")]
mod ai;
mod avatar;
#[cfg(feature = "desktop")]
mod bookmark_sync;
#[cfg(feature = "desktop")]
mod cli;
mod commit_message;
mod compare;
#[cfg(feature = "desktop")]
mod config;
#[cfg(feature = "desktop")]
mod converge;
mod dag;
mod diff;
mod diff_edit;
mod error;
mod file_tree;
#[cfg(feature = "desktop")]
mod flock;
mod fonts;
#[cfg(feature = "desktop")]
mod forge;
mod fuzzy;
mod l10n;
#[cfg(feature = "desktop")]
mod log_file;
mod markdown;
#[cfg(feature = "desktop")]
mod merge_scroll;
mod network;
mod overview;
mod palette;
mod placeholder;
mod projection;
#[cfg(feature = "desktop")]
mod repo;
#[cfg(feature = "desktop")]
mod repositories;
mod review;
#[cfg(feature = "desktop")]
mod shared_review_store;
mod shortcuts;
#[cfg(feature = "desktop")]
mod ssh;
mod theme;
mod tool_config;
mod trunk;
mod types;

#[cfg(feature = "desktop")]
pub use cli::*;
pub use compare::*;
pub use dag::*;
pub use diff::*;
pub use error::*;
pub use file_tree::*;
pub use fonts::*;
pub use fuzzy::*;
pub use l10n::*;
pub use markdown::*;
pub use network::*;
pub use palette::*;
pub use placeholder::*;
#[cfg(feature = "desktop")]
pub use repo::*;
#[cfg(feature = "desktop")]
pub use repositories::*;
pub use review::*;
pub use theme::*;
pub use tool_config::*;
pub use types::*;
