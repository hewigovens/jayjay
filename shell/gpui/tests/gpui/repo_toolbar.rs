use crate::harness::{open_fixture, settle_slide, settle_visual};
use gpui::{
    Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, VisualTestContext,
};
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

#[gpui::test]
fn double_clicking_toolbar_buttons_toggles_them_without_zooming(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    for selector in [
        "toolbar-sidebar-toggle",
        "toolbar-operation-log",
        "toolbar-refresh",
    ] {
        let at = cx.debug_bounds(selector).expect(selector).center();
        cx.simulate_click(at, Modifiers::default());
        cx.simulate_event(MouseDownEvent {
            button: MouseButton::Left,
            position: at,
            modifiers: Modifiers::default(),
            click_count: 2,
            first_mouse: false,
        });
        cx.simulate_event(MouseUpEvent {
            button: MouseButton::Left,
            position: at,
            modifiers: Modifiers::default(),
            click_count: 2,
        });
        settle_slide(cx);
    }
    view.read_with(cx, |view, _| assert!(!view.sidebar_hidden()));
    assert!(cx.debug_bounds("operation-log-pane").is_none());
}
