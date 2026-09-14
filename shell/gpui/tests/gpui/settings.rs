use crate::harness::{install_test_globals, settle_visual};
use gpui::{
    Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, VisualTestContext, point,
    px,
};
use jayjay_gpui::app::actions::OpenAbout;
use jayjay_gpui::app::config::current as current_config;
use jayjay_gpui::windows::settings::cli_diagnostics::CliDiagnostics;
use jayjay_gpui::windows::settings::tools::AiToolStatuses;
use jayjay_gpui::windows::settings::{SettingsSection, SettingsView};

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
        .debug_bounds("settings-nav-Integrations")
        .expect("Integrations nav row");
    settings_cx.simulate_click(tools_nav.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    assert!(
        settings_cx
            .debug_bounds("settings-tool-row-Codex CLI")
            .is_some()
    );

    scroll_settings(&mut settings_cx, -500.);
    assert!(settings_cx.debug_bounds("settings-cli-section").is_some());
    assert!(settings_cx.debug_bounds("settings-tool-row-jj").is_some());
    assert!(settings_cx.debug_bounds("settings-tool-row-gh").is_some());
    assert!(
        settings_cx
            .debug_bounds("settings-tool-row-origin")
            .is_some()
    );
    let copy_config_icon = settings_cx
        .debug_bounds("settings-copy-jj-tool-config")
        .expect("tool config copy button");
    settings_cx.simulate_click(copy_config_icon.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    assert_eq!(
        settings_cx
            .cx
            .read_from_clipboard()
            .and_then(|item| item.text()),
        Some(jayjay_core::JJ_TOOL_CONFIG.to_owned())
    );
    assert!(
        settings_cx
            .debug_bounds("settings-copy-jj-tool-config-copied")
            .is_some(),
        "tool config copy should swap to a success checkmark"
    );

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
fn font_size_stepper_updates_config(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Appearance, cx));
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    let increase = settings_cx
        .debug_bounds("setting-font-size-increase")
        .expect("font size increase button");
    let before = increase.size.height;
    settings_cx.simulate_click(increase.center(), Modifiers::default());
    settle_visual(&mut settings_cx);

    settings_cx.cx.update(|cx| {
        assert_eq!(current_config(cx).font_size(), 13.);
    });
    let after = settings_cx
        .debug_bounds("setting-font-size-increase")
        .expect("resized font size increase button")
        .size
        .height;
    assert!(
        after > before,
        "font size control should resize with the global setting"
    );
}

#[gpui::test]
fn jujutsu_config_path_copy_writes_the_path_and_shows_feedback(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Jujutsu, cx));
    let any_window = cx.windows().last().copied().expect("settings window");
    let window = any_window
        .downcast::<SettingsView>()
        .expect("settings window view");
    let path = "/mock/config/jj/config.toml";
    window
        .update(cx, |view, _, cx| {
            view.set_jj_config_path(path.to_owned(), cx);
        })
        .expect("inject jj config path");
    let mut settings_cx = VisualTestContext::from_window(any_window, cx);
    settle_visual(&mut settings_cx);

    let copy_path = settings_cx
        .debug_bounds("jj-config-copy-path")
        .expect("config path copy button");
    settings_cx.simulate_click(copy_path.center(), Modifiers::default());
    settle_visual(&mut settings_cx);

    assert_eq!(
        settings_cx
            .cx
            .read_from_clipboard()
            .and_then(|item| item.text()),
        Some(path.to_owned())
    );
    assert!(
        settings_cx
            .debug_bounds("jj-config-copy-path-copied")
            .is_some(),
        "config path copy should swap to a success checkmark"
    );
}

#[gpui::test]
fn integrations_gate_cli_install_row_by_platform(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(SettingsView::open);
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    let cli_nav = settings_cx
        .debug_bounds("settings-nav-Integrations")
        .expect("Integrations nav row");
    settings_cx.simulate_click(cli_nav.center(), Modifiers::default());
    settle_visual(&mut settings_cx);

    scroll_settings(&mut settings_cx, -500.);
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
fn integrations_reuse_detection_snapshots_across_page_navigation(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Integrations, cx));
    let any_window = cx.windows().last().copied().expect("settings window");
    let window = any_window
        .downcast::<SettingsView>()
        .expect("settings window view");
    // Inject before settling: the async detection spawned by the direct Integrations open lands later and must not clobber this snapshot.
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
            let missing = jayjay_core::CliStatus {
                is_installed: false,
                version: String::new(),
                path: String::new(),
            };
            view.set_cli_diagnostics(
                CliDiagnostics {
                    jj: missing.clone(),
                    gh: missing.clone(),
                    glab: missing,
                    origin: jayjay_core::CliStatus {
                        is_installed: true,
                        version: "1.2.3".to_owned(),
                        path: "/mock/bin/origin".to_owned(),
                    },
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

    for _ in 0..2 {
        scroll_settings(&mut settings_cx, -500.);
        assert!(
            settings_cx
                .debug_bounds("settings-tool-state-jj-missing")
                .is_some()
        );
        assert!(
            settings_cx
                .debug_bounds("settings-tool-state-origin-found")
                .is_some()
        );
        click_setting(&mut settings_cx, "settings-nav-Appearance");
        click_setting(&mut settings_cx, "settings-nav-Integrations");
        assert!(settings_cx.debug_bounds("dd-btn-editor").is_some());
    }
}

#[gpui::test]
fn custom_tool_commands_are_editable_and_persisted(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Integrations, cx));
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    select_dropdown_option(&mut settings_cx, "dd-btn-editor", "dd-editor-custom");
    let editor = settings_cx
        .debug_bounds("setting-custom-editor-command")
        .expect("custom editor command input");
    settings_cx.simulate_click(editor.center(), Modifiers::default());
    settings_cx.simulate_input("code --reuse-window");
    settle_visual(&mut settings_cx);

    select_dropdown_option(&mut settings_cx, "dd-btn-terminal", "dd-terminal-custom");
    let terminal = settings_cx
        .debug_bounds("setting-custom-terminal-command")
        .expect("custom terminal command input");
    settings_cx.simulate_click(terminal.center(), Modifiers::default());
    settings_cx.simulate_input("foot --title JayJay");
    settle_visual(&mut settings_cx);

    click_setting(&mut settings_cx, "settings-nav-Diff & Files");
    click_setting(&mut settings_cx, "settings-nav-Integrations");
    let editor = settings_cx
        .debug_bounds("setting-custom-editor-command")
        .expect("custom editor command survives page navigation");
    settings_cx.simulate_click(editor.center(), Modifiers::default());
    settings_cx.simulate_keystrokes("end");
    settings_cx.simulate_input(" --wait");
    settle_visual(&mut settings_cx);

    settings_cx.cx.update(|cx| {
        let encoded = toml::to_string(&current_config(cx)).expect("serialize config");
        assert!(encoded.contains("custom_editor_command = \"code --reuse-window --wait\""));
        assert!(encoded.contains("custom_terminal_command = \"foot --title JayJay\""));
    });
}

#[gpui::test]
fn workflow_and_privacy_preferences_survive_page_navigation(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Workflow, cx));
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    click_setting(&mut settings_cx, "setting-workflow-confirm-abandon");
    click_setting(
        &mut settings_cx,
        "setting-workflow-confirm-workspace-delete",
    );
    settings_cx.cx.update(|cx| {
        let cfg = toml::Value::try_from(current_config(cx)).expect("serialized preferences");
        assert_eq!(
            cfg["features"]["skip_abandon_confirmation"].as_bool(),
            Some(true)
        );
        assert_eq!(
            cfg["features"]["skip_workspace_delete_confirmation"].as_bool(),
            Some(true)
        );
    });

    click_setting(&mut settings_cx, "settings-nav-Diff & Files");
    assert!(settings_cx.debug_bounds("setting-diff-sbs").is_some());
    assert!(
        settings_cx
            .debug_bounds("setting-workflow-confirm-abandon")
            .is_none()
    );
    click_setting(&mut settings_cx, "settings-nav-Workflow");
    click_setting(&mut settings_cx, "setting-workflow-confirm-abandon");
    click_setting(
        &mut settings_cx,
        "setting-workflow-confirm-workspace-delete",
    );
    settings_cx.cx.update(|cx| {
        let cfg = toml::Value::try_from(current_config(cx)).expect("serialized preferences");
        assert_eq!(
            cfg["features"]["skip_abandon_confirmation"].as_bool(),
            Some(false)
        );
        assert_eq!(
            cfg["features"]["skip_workspace_delete_confirmation"].as_bool(),
            Some(false)
        );
    });

    click_setting(&mut settings_cx, "settings-nav-Data & Privacy");
    click_setting(&mut settings_cx, "setting-privacy-telemetry");
    click_setting(&mut settings_cx, "settings-nav-About");
    assert!(
        settings_cx
            .debug_bounds("setting-privacy-telemetry")
            .is_none()
    );
    click_setting(&mut settings_cx, "settings-nav-Data & Privacy");
    assert!(
        settings_cx
            .debug_bounds("setting-privacy-telemetry")
            .is_some()
    );
    settings_cx
        .cx
        .update(|cx| assert!(!current_config(cx).telemetry.enabled));
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
    assert!(settings_cx.debug_bounds("settings-about-section").is_some());
    assert!(
        settings_cx
            .debug_bounds("setting-privacy-telemetry")
            .is_none()
    );
    assert!(settings_cx.debug_bounds("about-config-copy-path").is_none());
}

fn select_dropdown_option(
    cx: &mut VisualTestContext,
    button_selector: &'static str,
    option_selector: &'static str,
) {
    let button = cx
        .debug_bounds(button_selector)
        .unwrap_or_else(|| panic!("missing dropdown {button_selector}"));
    cx.simulate_click(button.center(), Modifiers::default());
    settle_visual(cx);
    let option = cx
        .debug_bounds(option_selector)
        .unwrap_or_else(|| panic!("missing dropdown option {option_selector}"));
    cx.simulate_click(option.center(), Modifiers::default());
    settle_visual(cx);
}

fn scroll_settings(cx: &mut VisualTestContext, delta: f32) {
    let scroll = cx
        .debug_bounds("settings-scroll")
        .expect("settings scroll area");
    cx.simulate_event(ScrollWheelEvent {
        position: scroll.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(delta))),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle_visual(cx);
}

fn click_setting(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing {selector}"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    settle_visual(cx);
}
