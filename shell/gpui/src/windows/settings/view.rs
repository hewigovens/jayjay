use super::sidebar::sidebar;
use super::{
    SettingsSection, about, appearance, cli_diagnostics, config, data_privacy, diff, integrations,
    tools, workflow,
};

use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    px, rgb, size,
};

use super::dropdown::{OpenDropdown, dropdown_overlay};

use crate::app::config::{AppConfigStore, current as current_cfg};
use crate::app::theme::{Theme, observe_window_appearance};
use crate::ui::logo::Logo;
use crate::ui::text_area::TextArea;

#[cfg(target_os = "macos")]
const CUSTOM_TERMINAL_PLACEHOLDER: &str = "e.g. Terminal";
#[cfg(not(target_os = "macos"))]
const CUSTOM_TERMINAL_PLACEHOLDER: &str = "e.g. foot --title JayJay";

pub struct SettingsView {
    pub(super) section: SettingsSection,
    focus_handle: FocusHandle,
    pub(super) scroll: ScrollHandle,
    pub(super) open_dropdown: Option<OpenDropdown>,
    pub(super) jj_config: Option<config::JjConfigSnapshot>,
    pub(super) jj_config_loading: bool,
    pub(super) ai_tools: Option<tools::AiToolStatuses>,
    pub(super) cli_diagnostics: Option<cli_diagnostics::CliDiagnostics>,
    pub(super) tools_loading: bool,
    /// `None` until the Integrations load lands; `Some(None)` when the CLI install surface is unavailable (no home directory).
    pub(super) cli_install: Option<Option<crate::app::cli_install::CliInstallState>>,
    pub(super) recently_copied: Option<SharedString>,
    pub(super) custom_editor_command: Entity<TextArea>,
    pub(super) custom_terminal_command: Entity<TextArea>,
    logo: Logo,
}

impl SettingsView {
    pub fn open(cx: &mut App) {
        Self::open_section(SettingsSection::Appearance, cx);
    }

    pub fn open_section(section: SettingsSection, cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(760.), px(560.)), cx);
        let window_handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(720.), px(500.))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("JayJay Settings".into()),
                        ..Default::default()
                    }),
                    ..crate::app::window_options()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        let cfg = current_cfg(cx);
                        let custom_editor_command = tools::persisted_command(
                            cfg.tools.custom_editor_command.clone(),
                            "e.g. code --reuse-window",
                            |cfg, value| cfg.tools.custom_editor_command = value,
                            cx,
                        );
                        let custom_terminal_command = tools::persisted_command(
                            cfg.tools.custom_terminal_command.clone(),
                            CUSTOM_TERMINAL_PLACEHOLDER,
                            |cfg, value| cfg.tools.custom_terminal_command = value,
                            cx,
                        );
                        let mut view = Self {
                            section,
                            focus_handle: cx.focus_handle(),
                            scroll: ScrollHandle::new(),
                            open_dropdown: None,
                            jj_config: None,
                            jj_config_loading: false,
                            ai_tools: None,
                            cli_diagnostics: None,
                            tools_loading: false,
                            cli_install: None,
                            recently_copied: None,
                            custom_editor_command,
                            custom_terminal_command,
                            logo: Logo::load(cx),
                        };
                        // Direct opens must kick off the same lazy loads a sidebar click would.
                        match section {
                            SettingsSection::Integrations => view.ensure_tools_loaded(cx),
                            SettingsSection::Jujutsu => view.ensure_jj_config_loaded(cx),
                            _ => {}
                        }
                        view
                    })
                },
            )
            .ok();
        if let Some(handle) = window_handle {
            let _ = handle.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                let h = view.focus_handle(cx);
                window.focus(&h, cx);
            });
        }
    }
}

impl Focusable for SettingsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cfg = current_cfg(cx);
        let t = crate::app::theme::theme_for_window(window, cx).clone();
        let active = self.section;
        let dropdown = self.open_dropdown.clone();

        let mut root = div()
            .track_focus(&self.focus_handle)
            .key_context("SettingsView")
            .on_action(
                cx.listener(|_, _: &crate::app::actions::CloseWindow, window, _cx| {
                    window.remove_window();
                }),
            )
            .on_action(
                cx.listener(|_, _: &crate::app::actions::Dismiss, window, _cx| {
                    window.remove_window();
                }),
            )
            .relative()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg))
            .child(sidebar(active, &t, cx))
            .child(crate::ui::primitives::divider_v(&t))
            .child(
                div()
                    .id("settings-scroll")
                    .debug_selector(|| "settings-scroll".to_owned())
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .scrollbar_width(px(0.))
                    .px(px(28.))
                    .py(px(20.))
                    .child(self.section_body(&cfg, &t, cx)),
            );
        if let Some(d) = dropdown {
            root = root.child(dropdown_overlay(d, &t, cx));
        }
        root
    }
}

impl SettingsView {
    fn section_body(
        &self,
        cfg: &crate::app::config::AppConfig,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.section {
            SettingsSection::Appearance => appearance::appearance_section(cfg, t, cx),
            SettingsSection::Diff => diff::diff_section(cfg, t),
            SettingsSection::Workflow => workflow::workflow_section(cfg, t),
            SettingsSection::Integrations => integrations::integrations_section(self, cfg, t, cx),
            SettingsSection::Jujutsu => config::jujutsu_section(
                self.jj_config.as_ref(),
                self.jj_config_loading,
                self.recently_copied.as_ref(),
                t,
                cx,
            ),
            SettingsSection::DataPrivacy => data_privacy::data_privacy_section(cfg, t),
            SettingsSection::About => about::about_section(&self.logo, t).into_any_element(),
        }
    }
}
