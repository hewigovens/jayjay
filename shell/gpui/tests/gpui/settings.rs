use crate::harness::{install_test_globals, open_fixture, select_file, settle_visual};
use gpui::{
    Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, VisualTestContext, point,
    px, size,
};
use jayjay_core::{HostLogin, RepoHost, TokenSource};
use jayjay_gpui::app::actions::OpenAbout;
use jayjay_gpui::app::config::current as current_config;
use jayjay_gpui::windows::settings::cli_diagnostics::CliDiagnostics;
use jayjay_gpui::windows::settings::tools::AiToolStatuses;
use jayjay_gpui::windows::settings::{SettingsSection, SettingsView};
use jj_test::LinearFixture;

#[gpui::test]
fn settings_content_scrolls_and_jujutsu_config_loads_from_state(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(SettingsView::open);
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    assert!(settings_cx.debug_bounds("settings-scroll").is_some());
    let fonts_nav = settings_cx
        .debug_bounds("settings-nav-Fonts")
        .expect("Fonts nav row");
    settings_cx.simulate_click(fonts_nav.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    let font_button = settings_cx
        .debug_bounds("dd-btn-interface-font-family")
        .expect("interface font dropdown");
    settings_cx.simulate_click(font_button.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    let system_font = settings_cx
        .debug_bounds("dd-interface-font-family-system")
        .expect("system font option");
    settings_cx.simulate_click(system_font.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    settings_cx.cx.update(|cx| {
        assert_eq!(current_config(cx).interface_font_family, "");
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
    assert!(
        settings_cx
            .debug_bounds("settings-tool-row-GitHub")
            .is_some()
    );
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
fn page_heading_stays_visible_while_settings_scroll(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Fonts, cx));
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settings_cx.simulate_resize(size(px(760.), px(500.)));
    settle_visual(&mut settings_cx);

    let heading = settings_cx
        .debug_bounds("settings-page-title")
        .expect("page heading");
    let first_control = settings_cx
        .debug_bounds("dd-btn-interface-font-family")
        .expect("interface font");
    scroll_settings(&mut settings_cx, -2000.);
    let reset = settings_cx
        .debug_bounds("setting-reset-fonts")
        .expect("reset fonts");
    let scroll = settings_cx
        .debug_bounds("settings-scroll")
        .expect("settings viewport");
    assert!(reset.top() >= scroll.top() && reset.bottom() <= scroll.bottom());
    assert!(
        settings_cx
            .debug_bounds("dd-btn-interface-font-family")
            .is_none_or(|bounds| bounds.top() < first_control.top())
    );
    assert_eq!(
        settings_cx.debug_bounds("settings-page-title"),
        Some(heading)
    );
}

#[gpui::test]
fn interface_font_size_stepper_updates_config(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Fonts, cx));
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    let increase = settings_cx
        .debug_bounds("setting-interface-font-size-increase")
        .expect("font size increase button");
    let before = increase.size.height;
    settings_cx.simulate_click(increase.center(), Modifiers::default());
    settle_visual(&mut settings_cx);

    settings_cx.cx.update(|cx| {
        assert_eq!(current_config(cx).interface_font_size(), 13.);
        assert_eq!(current_config(cx).code_font_size(), 12.);
    });
    let after = settings_cx
        .debug_bounds("setting-interface-font-size-increase")
        .expect("resized font size increase button")
        .size
        .height;
    assert!(
        after > before,
        "font size control should resize with the global setting"
    );
}

#[gpui::test]
fn code_font_controls_update_the_code_font_and_reset(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| SettingsView::open_section(SettingsSection::Fonts, cx));
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);
    assert!(settings_cx.debug_bounds("settings-font-preview").is_some());

    for id in [
        "setting-code-font-size-increase",
        "setting-code-line-spacing-increase",
    ] {
        let button = settings_cx.debug_bounds(id).expect(id);
        settings_cx.simulate_click(button.center(), Modifiers::default());
        settle_visual(&mut settings_cx);
    }
    settings_cx.cx.update(|cx| {
        let config = current_config(cx);
        assert_eq!(
            (config.interface_font_size(), config.code_font_size()),
            (12., 13.)
        );
        assert_eq!(config.code_line_spacing, 1.1);
    });

    scroll_settings(&mut settings_cx, -500.);
    let reset = settings_cx
        .debug_bounds("setting-reset-fonts")
        .expect("reset fonts button");
    settings_cx.simulate_click(reset.center(), Modifiers::default());
    settle_visual(&mut settings_cx);
    settings_cx.cx.update(|cx| {
        let config = current_config(cx);
        assert_eq!(config.code_font_size(), 12.);
        assert_eq!(config.code_line_spacing, 1.);
    });
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
                    jj: missing,
                    origin: jayjay_core::CliStatus {
                        is_installed: true,
                        version: "1.2.3".to_owned(),
                        path: "/mock/bin/origin".to_owned(),
                    },
                    forge_logins: vec![
                        HostLogin {
                            host: RepoHost::GitHub,
                            source: Some(TokenSource::Stored),
                            signed_in: true,
                            can_sign_in: true,
                        },
                        HostLogin {
                            host: RepoHost::GitLab,
                            source: None,
                            signed_in: false,
                            can_sign_in: true,
                        },
                        HostLogin {
                            host: RepoHost::Codeberg,
                            source: None,
                            signed_in: false,
                            can_sign_in: false,
                        },
                    ],
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
        assert!(
            settings_cx
                .debug_bounds("settings-tool-state-GitHub-found")
                .is_some()
        );
        assert!(
            settings_cx
                .debug_bounds("settings-tool-state-GitLab-missing")
                .is_some()
        );
        for (selector, expected) in [
            ("settings-forge-GitHub-action", true),
            ("settings-forge-GitLab-action", true),
            ("settings-forge-Codeberg-action", false),
        ] {
            assert_eq!(
                settings_cx.debug_bounds(selector).is_some(),
                expected,
                "{selector}"
            );
        }
        click_setting(&mut settings_cx, "settings-nav-Appearance");
        click_setting(&mut settings_cx, "settings-nav-Integrations");
        assert!(settings_cx.debug_bounds("dd-btn-editor").is_some());
    }
}

#[gpui::test]
fn long_dropdowns_stay_inside_the_window_and_scroll_to_every_option(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| {
        jayjay_gpui::app::config::update(cx, |cfg| {
            cfg.tools.external_editor = "An editor with a very long saved name".to_owned();
        });
    });
    cx.update(|cx| SettingsView::open_section(SettingsSection::Integrations, cx));
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    let window_height = px(260.);
    settings_cx.simulate_resize(size(px(760.), window_height));
    crate::harness::zoom_to_max(&mut settings_cx);
    let editor = settings_cx
        .debug_bounds("dd-btn-editor")
        .expect("editor picker");
    assert!(
        editor.right() <= px(760.),
        "{editor:?} overflows the window"
    );

    click_setting(&mut settings_cx, "dd-btn-editor");
    let panel = settings_cx
        .debug_bounds("dropdown-panel")
        .expect("dropdown panel");
    assert!(
        panel.bottom() <= window_height,
        "{panel:?} overflows the window"
    );

    settings_cx.simulate_event(ScrollWheelEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-2000.))),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle_visual(&mut settings_cx);
    let last = settings_cx
        .debug_bounds("dd-editor-custom")
        .expect("last editor option");
    assert!(
        last.bottom() <= panel.bottom(),
        "{last:?} is still below {panel:?}"
    );
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
    let picker = settings_cx
        .debug_bounds("dd-btn-editor")
        .expect("editor picker");
    assert!(
        editor.size.width >= picker.size.width,
        "command input should remain wide enough to edit: {editor:?}"
    );
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

#[gpui::test]
fn changing_dark_diff_palette_recolors_open_diff_without_losing_selection_or_scroll_or_recoloring_chrome(
    cx: &mut TestAppContext,
) {
    use jayjay_gpui::app::config::{AppearanceMode, update as update_config};
    use jayjay_gpui::app::theme::Theme;
    use jj_test::{LinearFixture, run_jj_in};

    let fixture = LinearFixture::build();
    let content: String = (1..=120).map(|n| format!("line {n}\n")).collect();
    std::fs::write(fixture.path.join("wip1.txt"), content).expect("write wip1");
    run_jj_in(&fixture.path, &["st"]);

    let (view, repo_cx) = crate::harness::open_repo(fixture.path.clone(), cx);
    repo_cx.simulate_resize(size(px(1000.), px(500.)));
    settle_visual(repo_cx);
    crate::harness::select_file(&view, "wip1.txt", repo_cx);
    repo_cx.cx.update(|cx| {
        update_config(cx, |c| c.appearance = AppearanceMode::Dark);
    });
    settle_visual(repo_cx);
    view.update_in(repo_cx, |view, _, _| {
        view.set_diff_scroll_offset_y(px(-72.));
    });

    let (before_context_bg, before_success_fg) = repo_cx.cx.update(|cx| {
        let t = cx.global::<Theme>();
        (t.diff_context_bg, t.success_fg)
    });
    let before_selected_file = view.read_with(repo_cx, |view, cx| {
        view.view_model().read(cx).selected_file_ix
    });

    repo_cx.cx.update(|cx| {
        update_config(cx, |c| c.diff.palette.dark = "nord".to_owned());
    });
    settle_visual(repo_cx);

    let (after_context_bg, after_success_fg) = repo_cx.cx.update(|cx| {
        let t = cx.global::<Theme>();
        (t.diff_context_bg, t.success_fg)
    });
    assert_ne!(after_context_bg, before_context_bg);
    assert_eq!(
        after_context_bg,
        jayjay_core::DiffPalette::resolve("nord", true)
            .colors
            .context_bg
    );
    assert_eq!(after_success_fg, before_success_fg);
    view.read_with(repo_cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).selected_file_ix,
            before_selected_file
        );
        assert_eq!(view.diff_scroll_offset_y(), px(-72.));
    });
}

#[gpui::test]
fn unknown_stored_diff_palette_loads_as_default_and_survives_unrelated_config_updates(
    cx: &mut TestAppContext,
) {
    use jayjay_gpui::app::config::{AppConfig, AppConfigStore, update as update_config};
    use jayjay_gpui::app::theme::Theme;

    install_test_globals(cx);
    cx.update(|cx| {
        let mut cfg: AppConfig = toml::from_str(
            "appearance = \"dark\"\n[diff.palette]\ndark = \"future-dark-palette\"\n",
        )
        .expect("parse config");
        cfg.onboarding.completed = true;
        cx.set_global(AppConfigStore::new_ephemeral(cfg));
        update_config(cx, |c| c.diff.tree_file_list = true);

        let saved = current_config(cx);
        assert!(saved.diff.tree_file_list);
        assert_eq!(saved.diff.palette.dark, "future-dark-palette");
        assert_eq!(
            cx.global::<Theme>().diff_context_bg,
            jayjay_core::DiffThemeColors::dark().context_bg
        );
    });
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

#[gpui::test]
fn clearing_review_data_empties_the_store_and_repaints_open_repo_windows(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    let (repo, repo_cx) = open_fixture(&fixture, cx);

    let ix = select_file(&repo, "wip2.txt", repo_cx);
    let (change_id, identity) = repo.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let hunk = &vm.files.as_ref().expect("files loaded")[ix];
        let change = vm.selected_change().expect("selected change");
        (change.change_id.id.clone(), hunk.review_identity.clone())
    });
    repo.update_in(repo_cx, |view, _, cx| {
        view.toggle_reviewed(change_id, "wip2.txt".to_owned(), identity, cx);
    });
    repo_cx.cx.update(|cx| {
        jayjay_gpui::app::config::update(cx, |config| config.diff.hide_reviewed_files = true);
    });
    settle_visual(repo_cx);
    let visible = |repo_cx: &mut VisualTestContext| {
        repo.read_with(repo_cx, |view, cx| view.visible_file_paths(cx))
    };
    assert_eq!(visible(repo_cx), ["README.md", "feature.txt", "wip1.txt"]);
    assert!(repo_cx.debug_bounds("file-row-3").is_none());

    repo_cx
        .cx
        .update(|cx| SettingsView::open_section(SettingsSection::DataPrivacy, cx));
    let window = repo_cx
        .cx
        .windows()
        .last()
        .copied()
        .expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, &repo_cx.cx);
    settle_visual(&mut settings_cx);

    click_setting(&mut settings_cx, "settings-clear-review-data");
    assert!(settings_cx.debug_bounds("confirmation").is_some());
    click_setting(&mut settings_cx, "settings-clear-review-cancel");
    assert_eq!(visible(repo_cx), ["README.md", "feature.txt", "wip1.txt"]);

    click_setting(&mut settings_cx, "settings-clear-review-data");
    click_setting(&mut settings_cx, "settings-clear-review-submit");
    assert_eq!(
        visible(repo_cx),
        ["README.md", "feature.txt", "wip1.txt", "wip2.txt"]
    );
    assert!(repo_cx.debug_bounds("file-row-3").is_some());
}

#[gpui::test]
fn settings_sidebar_resizes_from_just_beside_its_divider(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(SettingsView::open);
    let window = cx.windows().last().copied().expect("settings window");
    let mut settings_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut settings_cx);

    let initial = settings_cx
        .debug_bounds("settings-nav-Fonts")
        .expect("Fonts nav row")
        .size
        .width;
    let handle = settings_cx
        .debug_bounds("settings-sidebar-resize-handle")
        .expect("sidebar resize handle");
    let start = point(handle.right() - px(8.), handle.center().y);
    let end = point(start.x + px(80.), start.y);
    settings_cx.simulate_mouse_down(start, gpui::MouseButton::Left, Modifiers::default());
    settings_cx.simulate_mouse_move(end, gpui::MouseButton::Left, Modifiers::default());
    settings_cx.simulate_mouse_up(end, gpui::MouseButton::Left, Modifiers::default());
    settle_visual(&mut settings_cx);

    let resized = settings_cx
        .debug_bounds("settings-nav-Fonts")
        .expect("Fonts nav row")
        .size
        .width;
    assert!(resized > initial + px(70.), "{initial:?} -> {resized:?}");
}
