mod support;

use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::actions::OpenAbout;
use jayjay_gpui::windows::settings::SettingsView;
use support::{install_test_globals, settle_visual};

#[gpui::test]
fn settings_content_scrolls_and_jujutsu_config_loads_from_state(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(SettingsView::open);
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    assert!(settings_cx.debug_bounds("settings-scroll").is_some());
    let tools_nav = settings_cx
        .debug_bounds("settings-nav-Tools")
        .expect("Tools nav row");
    settings_cx.simulate_click(tools_nav.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    assert!(
        settings_cx
            .debug_bounds("settings-tool-row-Codex CLI")
            .is_some()
    );
    assert!(settings_cx.debug_bounds("settings-tool-row-jj").is_some());

    let nav = settings_cx
        .debug_bounds("settings-nav-Jujutsu")
        .expect("Jujutsu nav row");
    settings_cx.simulate_click(nav.center(), Modifiers::default());
    settle_visual(&mut settings_cx);

    assert!(
        settings_cx
            .debug_bounds("settings-jujutsu-section")
            .is_some(),
        "Jujutsu settings pane should render"
    );
    assert!(
        settings_cx.debug_bounds("jj-config-status").is_some()
            || settings_cx.debug_bounds("jj-config-path-row").is_some()
            || settings_cx.debug_bounds("jj-config-row").is_some(),
        "Jujutsu pane should render either loading/error status or config rows"
    );
}

#[gpui::test]
fn open_about_action_opens_about_settings_section(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| {
        jayjay_gpui::app::menus::install(cx);
        cx.dispatch_action(&OpenAbout);
    });

    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    assert!(settings_cx.debug_bounds("settings-nav-About").is_some());
    assert!(
        settings_cx
            .debug_bounds("setting-about-telemetry")
            .is_some()
    );
    assert!(settings_cx.debug_bounds("about-config-copy-path").is_none());
}
