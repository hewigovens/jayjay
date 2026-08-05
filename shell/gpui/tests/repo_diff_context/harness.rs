use gpui::VisualTestContext;
use jayjay_core::diff::ContextRegion;
use jayjay_gpui::repo::RepoWindow;

#[path = "../harness.rs"]
mod shared;

pub(super) use shared::{
    install_test_globals, load_selected_change_files, select_file, selector, settle_visual,
};

pub(super) fn largest_region(
    view: &gpui::Entity<RepoWindow>,
    cx: &mut VisualTestContext,
) -> ContextRegion {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .current_diff
            .as_ref()
            .expect("diff loaded")
            .lines
            .iter()
            .filter_map(|line| line.context_region)
            .max_by_key(|region| region.line_count)
            .expect("collapsed context region")
    })
}
