use std::path::Path;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::SettingsView;
use super::shared::detail_row;
use crate::app::cli_install::{self, CliInstallState, EntryStatus};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::button;

/// `install` is `None` while the Tools load is in flight and `Some(None)` when no install state could be resolved.
pub(super) fn command_line_rows(
    install: Option<Option<&CliInstallState>>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> Option<AnyElement> {
    if !cli_install::supported() {
        return None;
    }
    let mut col = div().flex().flex_col().w_full().gap(px(2.));
    match install {
        Some(Some(state)) => {
            col = col.child(install_row(state, t, cx));
            for (text, color) in hint_lines(state, t) {
                col = col.child(hint_line(text, color));
            }
        }
        loading_or_unavailable => {
            let detail = if loading_or_unavailable.is_none() {
                "Checking…"
            } else {
                "Unavailable: no home directory"
            };
            col = col.child(
                detail_row(glyph::TERMINAL, "jayjay", detail, 11., t.fg_faint, t)
                    .debug_selector(|| "settings-cli-install-row".to_owned()),
            );
        }
    }
    Some(col.into_any_element())
}

fn install_row(state: &CliInstallState, t: &Theme, cx: &mut Context<SettingsView>) -> AnyElement {
    let install_path = state.install_path().display().to_string();
    let mut row = detail_row(glyph::TERMINAL, "jayjay", install_path, 11., t.fg_dim, t)
        .debug_selector(|| "settings-cli-install-row".to_owned())
        .py(px(4.));
    match &state.status {
        EntryStatus::NotInstalled => {
            row = row.child(action_button(
                "settings-cli-install-btn",
                "Install",
                t,
                cx,
                SettingsView::cli_install_clicked,
            ));
        }
        EntryStatus::Installed { .. } => {
            row = row
                .child(action_button(
                    "settings-cli-remove-btn",
                    "Remove",
                    t,
                    cx,
                    SettingsView::cli_remove_clicked,
                ))
                .child(icons::icon(glyph::CHECK, 13., t.tag_added_fg));
        }
        EntryStatus::Broken { .. } => {
            row = row
                .child(action_button(
                    "settings-cli-reinstall-btn",
                    "Reinstall",
                    t,
                    cx,
                    SettingsView::cli_install_clicked,
                ))
                .child(action_button(
                    "settings-cli-remove-btn",
                    "Remove",
                    t,
                    cx,
                    SettingsView::cli_remove_clicked,
                ))
                .child(icons::icon(glyph::WARNING, 13., t.error_fg));
        }
        // Not ours to touch: no Install/Remove controls, only the manual-removal hint below.
        EntryStatus::Unmanaged { .. } => {
            row = row.child(icons::icon(glyph::WARNING, 13., t.fg_faint));
        }
    }
    row.into_any_element()
}

fn hint_lines(state: &CliInstallState, t: &Theme) -> Vec<(String, u32)> {
    let mut lines = Vec::new();
    if let Some(error) = &state.error {
        lines.push((error.clone(), t.error_fg));
    }
    if let EntryStatus::Broken { target } = &state.status {
        lines.push((
            format!("Link target is missing: {}", target.display()),
            t.error_fg,
        ));
    }
    if let EntryStatus::Unmanaged { target } = &state.status {
        let what = match target {
            Some(target) => format!("This entry links to {}", target.display()),
            None => "A file JayJay didn't install is already here".to_owned(),
        };
        lines.push((
            format!("{what}; remove it manually if JayJay should manage this command."),
            t.fg_faint,
        ));
    }
    match (&state.status, &state.path_hint) {
        (EntryStatus::NotInstalled, _) => lines.push((
            "Adds a 'jayjay' command that opens repositories in JayJay.".to_owned(),
            t.fg_faint,
        )),
        (_, Some(hint)) => lines.push((hint.clone(), t.fg_faint)),
        _ => {}
    }
    lines
}

fn hint_line(text: String, color: u32) -> AnyElement {
    div()
        .w_full()
        .px(px(8.))
        .text_size(ui_font_size(11.))
        .text_color(rgb(color))
        .child(SharedString::from(text))
        .into_any_element()
}

fn action_button(
    id: &'static str,
    label: &'static str,
    t: &Theme,
    cx: &mut Context<SettingsView>,
    on_click: fn(&mut SettingsView, &mut Context<SettingsView>),
) -> AnyElement {
    button(SharedString::from(id), label, t, false)
        .debug_selector(move || id.to_owned())
        .on_click(cx.listener(move |view, _ev, _window, cx| on_click(view, cx)))
        .into_any_element()
}

impl SettingsView {
    fn cli_install_clicked(&mut self, cx: &mut Context<Self>) {
        self.apply_cli_action(cli_install::perform_install, cx);
    }

    fn cli_remove_clicked(&mut self, cx: &mut Context<Self>) {
        self.apply_cli_action(cli_install::perform_uninstall, cx);
    }

    /// Runs the fs action, then rebuilds the snapshot so status, PATH hint, and error reflect the new on-disk state.
    fn apply_cli_action(
        &mut self,
        action: fn(&Path) -> Result<(), String>,
        cx: &mut Context<Self>,
    ) {
        let Some(bin_dir) = self
            .cli_install
            .as_ref()
            .and_then(|loaded| loaded.as_ref())
            .map(|s| s.bin_dir.clone())
        else {
            return;
        };
        let error = action(&bin_dir).err();
        let mut state = cli_install::state_for(&bin_dir);
        state.error = error;
        self.cli_install = Some(Some(state));
        cx.notify();
    }
}
