mod support;

use gpui::{Focusable, Modifiers, TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::config::{self, AppearanceMode};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::command_palette::CommandPalette;
use jj_test::LinearFixture;
use support::*;

#[gpui::test]
fn command_palette_ctrl_n_enter_dispatches_selected_action(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);

    palette_cx.simulate_input("theme");
    palette_cx.simulate_keystrokes("ctrl-n ctrl-n enter");

    assert_eq!(
        palette_cx.cx.read(|cx| config::current(cx).appearance),
        AppearanceMode::Dark
    );
}

#[gpui::test]
fn command_palette_supports_line_editing_keys(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let (palette, cx) = cx.add_window_view(|_, cx| CommandPalette::new("".into(), None, cx));
    let cx: &mut VisualTestContext = cx;

    cx.focus(&palette);
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-left");
    cx.simulate_input("|");

    palette.read_with(cx, |palette, _| {
        assert_eq!(palette.query_text(), "alpha beta |gamma");
    });

    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-backspace");

    palette.read_with(cx, |palette, _| {
        assert_eq!(palette.query_text(), "alpha beta ");
    });

    cx.simulate_input("gamma");
    cx.simulate_keystrokes("ctrl-u");

    palette.read_with(cx, |palette, _| {
        assert_eq!(palette.query_text(), "");
    });

    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("ctrl-a alt-right ctrl-k");

    palette.read_with(cx, |palette, _| {
        assert_eq!(palette.query_text(), "alpha");
    });
}

#[gpui::test]
fn command_palette_renders_input_caret(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);

    assert!(palette_cx.debug_bounds("command-palette-caret").is_some());
    palette_cx.simulate_input("jj status");
    assert!(palette_cx.debug_bounds("command-palette-caret").is_some());
}

#[gpui::test]
fn command_palette_mouse_click_dispatches_action(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);
    palette_cx.simulate_input("dark");

    let row = palette_cx
        .debug_bounds("command-palette-action-theme-dark")
        .expect("theme dark row bounds");
    palette_cx.simulate_click(row.center(), Modifiers::default());

    assert_eq!(
        palette_cx.cx.read(|cx| config::current(cx).appearance),
        AppearanceMode::Dark
    );
}

#[gpui::test]
fn command_palette_toggles_hide_git_lfs_diffs(cx: &mut TestAppContext) {
    install_test_globals(cx);
    assert!(cx.update(|cx| config::current(cx).diff.hide_git_lfs));
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);
    palette_cx.simulate_input("lfs");

    let row = palette_cx
        .debug_bounds("command-palette-action-toggle-hide-git-lfs-backed-files")
        .expect("hide Git LFS row bounds");
    palette_cx.simulate_click(row.center(), Modifiers::default());

    assert!(
        !palette_cx
            .cx
            .read(|cx| config::current(cx).diff.hide_git_lfs)
    );
}

#[gpui::test]
fn command_palette_renders_operation_log_action(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);
    palette_cx.simulate_input("op log");

    assert!(
        palette_cx
            .debug_bounds("command-palette-action-operation-log")
            .is_some()
    );
}

#[gpui::test]
fn find_bar_supports_line_editing_keys(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    view.update_in(cx, |view, window, cx| {
        view.open_find(cx);
        view.focus_handle(cx).focus(window, cx);
    });
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-left");
    cx.simulate_input("|");

    view.read_with(cx, |view, _| {
        assert_eq!(view.find_query_text(), Some("alpha beta |gamma"));
    });

    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-backspace");

    view.read_with(cx, |view, _| {
        assert_eq!(view.find_query_text(), Some("alpha beta "));
    });

    cx.simulate_input("gamma");
    cx.simulate_keystrokes("ctrl-u");

    view.read_with(cx, |view, _| {
        assert_eq!(view.find_query_text(), Some(""));
    });
}
