mod about;
mod appearance;
mod cli_row;
pub mod config;
mod diff;
mod dropdown;
pub(crate) mod shared;
pub mod tools;

use gpui::{
    AnyElement, App, AppContext, Bounds, ClickEvent, Context, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Point, Render, SharedString,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    px, rgb, size,
};

use dropdown::{OpenDropdown, dropdown_overlay};

use crate::app::config::{AppConfigStore, current as current_cfg};
use crate::app::theme::{Theme, observe_window_appearance, theme};
use crate::ui::icons::{self, glyph};
use crate::ui::logo::Logo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Appearance,
    Diff,
    Tools,
    Jujutsu,
    About,
}

const SECTIONS: &[(SettingsSection, &str, &str)] = &[
    (SettingsSection::Appearance, "Appearance", glyph::WHITESPACE),
    (SettingsSection::Diff, "Diff", glyph::COLUMNS),
    (SettingsSection::Tools, "Tools", glyph::GEAR),
    (SettingsSection::Jujutsu, "Jujutsu", glyph::GIT_BRANCH),
    (SettingsSection::About, "About", glyph::INFO),
];

pub struct SettingsView {
    section: SettingsSection,
    focus_handle: FocusHandle,
    open_dropdown: Option<OpenDropdown>,
    jj_config: Option<config::JjConfigSnapshot>,
    jj_config_loading: bool,
    ai_tools: Option<tools::AiToolStatuses>,
    tools_loading: bool,
    /// `None` until the Tools load lands; `Some(None)` when the CLI install surface is unavailable (no home directory).
    cli_install: Option<Option<crate::app::cli_install::CliInstallState>>,
    recently_copied: Option<SharedString>,
    logo: Logo,
}

impl SettingsView {
    pub fn open(cx: &mut App) {
        Self::open_section(SettingsSection::Appearance, cx);
    }

    pub fn open_section(section: SettingsSection, cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(680.), px(520.)), cx);
        let window_handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("JayJay Settings".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        let mut view = Self {
                            section,
                            focus_handle: cx.focus_handle(),
                            open_dropdown: None,
                            jj_config: None,
                            jj_config_loading: false,
                            ai_tools: None,
                            tools_loading: false,
                            cli_install: None,
                            recently_copied: None,
                            logo: Logo::load(cx),
                        };
                        // Direct opens must kick off the same lazy loads a sidebar click would.
                        match section {
                            SettingsSection::Tools => view.ensure_tools_loaded(cx),
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

    fn open_dropdown(
        &mut self,
        field_id: SharedString,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.open_dropdown = Some(OpenDropdown { field_id, anchor });
        cx.notify();
    }

    fn close_dropdown(&mut self, cx: &mut Context<Self>) {
        if self.open_dropdown.take().is_some() {
            cx.notify();
        }
    }

    fn mark_copied(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.recently_copied = Some(id.clone());
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1500))
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.recently_copied.as_ref() == Some(&id) {
                    view.recently_copied = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn ensure_jj_config_loaded(&mut self, cx: &mut Context<Self>) {
        if self.jj_config.is_some() || self.jj_config_loading {
            return;
        }
        self.jj_config_loading = true;
        cx.spawn(async move |this, cx| {
            let snapshot = cx
                .background_spawn(async { config::load_jj_config_snapshot() })
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.jj_config.is_none() {
                    view.jj_config = Some(snapshot);
                }
                view.jj_config_loading = false;
                cx.notify();
            });
        })
        .detach();
    }

    /// Loads every Tools-section snapshot: AI tool detection plus the CLI install state (a no-op `None` on non-Linux platforms).
    fn ensure_tools_loaded(&mut self, cx: &mut Context<Self>) {
        if self.ai_tools.is_some() || self.tools_loading {
            return;
        }
        self.tools_loading = true;
        cx.spawn(async move |this, cx| {
            let (statuses, cli_install) = cx
                .background_spawn(async {
                    (
                        tools::load_ai_tool_statuses(),
                        crate::app::cli_install::load_state(),
                    )
                })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.tools_loading = false;
                // Don't clobber snapshots that arrived while this load ran (injected statuses, install/remove clicks).
                if view.ai_tools.is_none() {
                    view.ai_tools = Some(statuses);
                }
                if view.cli_install.is_none() {
                    view.cli_install = Some(cli_install);
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Focusable for SettingsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cfg = current_cfg(cx);
        let t = theme(cx).clone();
        let active = self.section;
        let dropdown = self.open_dropdown.clone();
        let jj_config_loading = self.jj_config_loading;

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
                    .scrollbar_width(px(0.))
                    .px(px(28.))
                    .py(px(20.))
                    .child(section_body(
                        active,
                        &cfg,
                        LoadedSnapshots {
                            jj_config: self.jj_config.as_ref(),
                            jj_config_loading,
                            ai_tools: self.ai_tools.as_ref(),
                            cli_install: self.cli_install.as_ref().map(Option::as_ref),
                            recently_copied: self.recently_copied.as_ref(),
                        },
                        &self.logo,
                        &t,
                        cx,
                    )),
            );
        if let Some(d) = dropdown {
            root = root.child(dropdown_overlay(d, &t, cx));
        }
        root
    }
}

fn sidebar(active: SettingsSection, t: &Theme, cx: &mut Context<SettingsView>) -> AnyElement {
    let mut col = div()
        .flex()
        .flex_col()
        .w(px(180.))
        .h_full()
        .bg(rgb(t.sidebar_bg))
        .py(px(12.));

    for (sect, label, glyph_str) in SECTIONS {
        col = col.child(nav_button(*sect, label, glyph_str, *sect == active, t, cx));
    }
    col.into_any_element()
}

fn nav_button(
    sect: SettingsSection,
    label: &'static str,
    glyph_str: &'static str,
    active: bool,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let (bg, fg) = if active {
        (t.selected_bg, t.fg)
    } else {
        (t.sidebar_bg, t.fg_dim)
    };
    div()
        .id(SharedString::from(format!("settings-nav-{label}")))
        .debug_selector(move || format!("settings-nav-{label}"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(14.))
        .py(px(8.))
        .bg(rgb(bg))
        .text_size(px(12.))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _ev: &ClickEvent, _w, cx| {
            this.section = sect;
            if sect == SettingsSection::Jujutsu {
                this.ensure_jj_config_loaded(cx);
            }
            if sect == SettingsSection::Tools {
                this.ensure_tools_loaded(cx);
            }
            cx.notify();
        }))
        .child(icons::icon(glyph_str, 14., fg))
        .child(label)
        .into_any_element()
}

/// Section inputs loaded asynchronously after the window opens.
struct LoadedSnapshots<'a> {
    jj_config: Option<&'a config::JjConfigSnapshot>,
    jj_config_loading: bool,
    ai_tools: Option<&'a tools::AiToolStatuses>,
    /// Outer `None` while the Tools load is in flight; inner `None` when CLI install is unavailable.
    cli_install: Option<Option<&'a crate::app::cli_install::CliInstallState>>,
    recently_copied: Option<&'a SharedString>,
}

fn section_body(
    sect: SettingsSection,
    cfg: &crate::app::config::AppConfig,
    loaded: LoadedSnapshots<'_>,
    logo: &Logo,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    match sect {
        SettingsSection::Appearance => appearance::appearance_section(cfg, t, cx),
        SettingsSection::Diff => diff::diff_section(cfg, t),
        SettingsSection::Tools => tools::tools_section(
            cfg,
            loaded.ai_tools,
            loaded.cli_install,
            loaded.recently_copied,
            t,
            cx,
        ),
        SettingsSection::Jujutsu => config::jujutsu_section(
            loaded.jj_config,
            loaded.jj_config_loading,
            loaded.recently_copied,
            t,
            cx,
        ),
        SettingsSection::About => about::about_section(cfg, logo, t).into_any_element(),
    }
}
