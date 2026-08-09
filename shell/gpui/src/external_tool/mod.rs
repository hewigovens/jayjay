mod actions;
mod diff;
mod load;
mod merge;
mod open;
mod render_diff;
mod render_merge;
mod view;

pub use jayjay_core::external_tools::{ExternalToolInvocation, parse_external_tool_invocation};
pub use open::open_external_tool;
pub use view::ExternalToolWindow;
