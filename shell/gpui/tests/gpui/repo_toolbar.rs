use crate::harness::{open_fixture, settle_visual};
use gpui::{TestAppContext, VisualTestContext};
use jj_test::LinearFixture;

fn set_editor(editor: &str, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        jayjay_gpui::app::config::update(cx, |c| c.tools.external_editor = editor.to_owned());
        window.refresh();
    });
    settle_visual(cx);
}

#[gpui::test]
fn tool_buttons_show_the_configured_apps_mark_or_the_generic_symbol(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (_view, cx) = open_fixture(&fixture, cx);

    set_editor("zed", cx);
    assert!(cx.debug_bounds("toolbar-mark-editor.zed").is_some());
    assert!(
        cx.debug_bounds("toolbar-mark-generic").is_some(),
        "the default system terminal has no mark"
    );

    set_editor("custom", cx);
    assert!(cx.debug_bounds("toolbar-mark-editor.zed").is_none());
}
