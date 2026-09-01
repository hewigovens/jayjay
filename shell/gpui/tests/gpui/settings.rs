use crate::harness::{install_test_globals, settle_visual};
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::actions::OpenAbout;
use jayjay_gpui::app::config::current as current_config;
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
        .debug_bounds("settings-nav-Tools")
        .expect("Tools nav row");
    settings_cx.simulate_click(tools_nav.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    assert!(
        settings_cx
            .debug_bounds("settings-tool-row-Codex CLI")
            .is_some()
    );

    let cli_nav = settings_cx
        .debug_bounds("settings-nav-CLI")
        .expect("CLI nav row");
    settings_cx.simulate_click(cli_nav.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
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
fn cli_section_gates_cli_install_row_by_platform(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(SettingsView::open);
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    let cli_nav = settings_cx
        .debug_bounds("settings-nav-CLI")
        .expect("CLI nav row");
    settings_cx.simulate_click(cli_nav.center(), Modifiers::default());
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
}

#[gpui::test]
fn custom_tool_commands_are_editable_and_persisted(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Tools, cx));
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

    settings_cx.cx.update(|cx| {
        let encoded = toml::to_string(&current_config(cx)).expect("serialize config");
        assert!(encoded.contains("custom_editor_command = \"code --reuse-window\""));
        assert!(encoded.contains("custom_terminal_command = \"foot --title JayJay\""));
    });
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
