mod find_bar;
mod header;
mod mouse;
mod placeholders;
mod render;
mod sbs_body;
mod state;
mod unified_body;
mod wrap_cache;

pub use render::diff_view;
pub use state::{
    DetailMode, DiffViewMode, DiffViewState, FindState, MarkdownPreviewContent, SvgPreviewContent,
};
pub(crate) use wrap_cache::DiffWrapCache;
