use std::fs;

use crate::harness::{open_fixture, settle_visual};
use gpui::TestAppContext;
use jayjay_gpui::app::config;
use jj_test::{LinearFixture, run_jj_in};

#[gpui::test]
fn file_rows_show_the_folder_or_the_whole_move_in_both_layouts(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::create_dir_all(fixture.path.join("src/nested")).expect("create folders");
    fs::write(fixture.path.join("src/nested/lib.rs"), "fn lib() {}\n").expect("write lib.rs");
    fs::create_dir_all(fixture.path.join("docs")).expect("create docs");
    fs::rename(
        fixture.path.join("feature.txt"),
        fixture.path.join("docs/feature.txt"),
    )
    .expect("move feature.txt");
    run_jj_in(&fixture.path, &["st"]);
    let (_view, cx) = open_fixture(&fixture, cx);

    for tree in [false, true] {
        cx.cx
            .update(|cx| config::update(cx, |config| config.diff.tree_file_list = tree));
        settle_visual(cx);
        assert!(
            cx.debug_bounds("file-detail-src/nested/lib.rs=src/nested")
                .is_some(),
            "tree: {tree}"
        );
        assert!(
            cx.debug_bounds("file-detail-docs/feature.txt=feature.txt → docs/feature.txt")
                .is_some(),
            "tree: {tree}"
        );
        assert!(
            cx.debug_bounds("file-detail-wip1.txt=wip1.txt").is_none(),
            "tree: {tree}"
        );
    }
}
