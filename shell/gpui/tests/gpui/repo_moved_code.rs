use std::fs;

use gpui::{Modifiers, TestAppContext, px, size};
use jj_test::{LinearFixture, run_jj_in};

use crate::harness::{open_fixture, select_file, settle_visual};

#[gpui::test]
fn moved_block_labels_jump_to_the_other_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let small = "fn small() {\n    x(1);\n}\n";
    let body: String = (0..80)
        .map(|i| format!("    let v{i} = compute({i});\n"))
        .collect();
    let big = format!("fn big() {{\n{body}}}\n");
    fs::write(fixture.path.join("moved.rs"), format!("{small}{big}")).unwrap();
    run_jj_in(&fixture.path, &["commit", "-m", "baseline"]);
    fs::write(fixture.path.join("moved.rs"), format!("{big}{small}")).unwrap();
    run_jj_in(&fixture.path, &["st"]);
    let (view, cx) = open_fixture(&fixture, cx);
    cx.simulate_resize(size(px(1600.), px(320.)));
    select_file(&view, "moved.rs", cx);
    settle_visual(cx);

    let label = cx
        .debug_bounds("moved-unified-0")
        .expect("jump label on the removed copy");
    let before = view.read_with(cx, |view, _| view.diff_scroll_offset_y());
    cx.simulate_click(label.center(), Modifiers::default());
    settle_visual(cx);
    let after = view.read_with(cx, |view, _| view.diff_scroll_offset_y());
    assert!(
        after < before,
        "diff did not scroll toward the added copy: {before:?} -> {after:?}"
    );
    assert!(cx.debug_bounds("moved-unified-0").is_none());
}
