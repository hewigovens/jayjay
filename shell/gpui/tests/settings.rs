mod support;

use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::actions::OpenAbout;
use jayjay_gpui::app::config::current as current_config;
use jayjay_gpui::windows::settings::tools::AiToolStatuses;
use jayjay_gpui::windows::settings::{SettingsSection, SettingsView};
use support::{install_test_globals, settle_visual};

#[gpui::test]
fn settings_content_scrolls_and_jujutsu_config_loads_from_state(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(SettingsView::open);
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    assert!(settings_cx.debug_bounds("settings-scroll").is_some());
    let font_button = settings_cx
        .debug_bounds("dd-btn-font-family")
        .expect("font family dropdown");
    settings_cx.simulate_click(font_button.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    let system_font = settings_cx
        .debug_bounds("dd-font-family-system")
        .expect("system font option");
    settings_cx.simulate_click(system_font.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    settings_cx.cx.update(|cx| {
        assert_eq!(current_config(cx).font_family, "system");
    });

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
fn tools_section_gates_cli_install_row_by_platform(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(SettingsView::open);
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    let tools_nav = settings_cx
        .debug_bounds("settings-nav-Tools")
        .expect("Tools nav row");
    settings_cx.simulate_click(tools_nav.center(), Modifiers::default());
    settle_visual(&mut settings_cx);

    let row = settings_cx.debug_bounds("settings-cli-install-row");
    if cfg!(target_os = "linux") {
        assert!(
            row.is_some(),
            "Linux settings should offer the jayjay CLI install row"
        );
    } else {
        assert!(
            row.is_none(),
            "CLI install belongs to the macOS app bundle / is unsupported on Windows; the row must stay hidden"
        );
    }
}

#[gpui::test]
fn tools_ai_provider_rows_reflect_mocked_detection_states(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Tools, cx));
    let any_window = cx.windows().last().copied().expect("settings window");
    let window = any_window
        .downcast::<SettingsView>()
        .expect("settings window view");
    // Inject before settling: the async detection spawned by the direct Tools open lands later and must not clobber this snapshot.
    window
        .update(cx, |view, _, cx| {
            view.set_ai_tool_statuses(
                AiToolStatuses {
                    codex: Some("/mock/bin/codex".to_owned()),
                    claude: None,
                    jayjay: Some("/mock/bin/jayjay".to_owned()),
                },
                cx,
            );
        })
        .expect("inject ai tool statuses");
    let mut settings_cx = VisualTestContext::from_window(any_window, cx);
    settle_visual(&mut settings_cx);

    assert!(
        settings_cx
            .debug_bounds("settings-tool-state-Codex CLI-found")
            .is_some(),
        "detected codex should render as found with its resolved path"
    );
    assert!(
        settings_cx
            .debug_bounds("settings-tool-state-Claude CLI-missing")
            .is_some(),
        "undetected claude should render as missing"
    );
    assert!(
        settings_cx
            .debug_bounds("settings-tool-state-Claude CLI-found")
            .is_none(),
        "a missing provider must not also render a found marker"
    );
    assert!(
        settings_cx
            .debug_bounds("settings-tool-state-jayjay-found")
            .is_some(),
        "detected jayjay CLI should render as found"
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
