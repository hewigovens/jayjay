mod harness;

use gpui::{TestAppContext, VisualTestContext};
use harness::{install_test_globals, settle_visual};
use jayjay_gpui::app::{actions::OpenKeyboardShortcuts, menus};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::keyboard_shortcuts::KeyboardShortcutsView;
use jj_test::LinearFixture;

#[gpui::test]
fn shortcut_action_opens_complete_reference_window(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| {
        menus::install(cx);
        cx.dispatch_action(&OpenKeyboardShortcuts);
    });

    let any_window = cx.windows().last().copied().expect("shortcuts window");
    assert!(any_window.downcast::<KeyboardShortcutsView>().is_some());
    let mut shortcuts_cx = VisualTestContext::from_window(any_window, cx);
    settle_visual(&mut shortcuts_cx);

    for selector in [
        "keyboard-shortcuts-window",
        "keyboard-shortcuts-scroll",
        "shortcut-section-General",
        "shortcut-entry-Keyboard Shortcuts",
        "shortcut-section-Diff & Review",
        "shortcut-entry-Move Up / Down",
    ] {
        assert!(
            shortcuts_cx.debug_bounds(selector).is_some(),
            "missing shortcut reference selector {selector}"
        );
    }
}

#[gpui::test]
fn mod_slash_opens_keyboard_shortcuts(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    cx.update(menus::install);
    let (_view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let before = cx.cx.windows().len();

    cx.simulate_keystrokes(if cfg!(target_os = "macos") {
        "cmd-/"
    } else {
        "ctrl-/"
    });
    settle_visual(cx);

    assert_eq!(cx.cx.windows().len(), before + 1);
    assert!(
        cx.cx
            .windows()
            .last()
            .and_then(|window| window.downcast::<KeyboardShortcutsView>())
            .is_some()
    );
}
