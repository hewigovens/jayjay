use jayjay_core::{check_gh_environment, check_glab_environment, check_jj_environment};

use gpui::{AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Styled, div, px};

use super::SettingsView;
use super::cli_row;
use super::shared::{section_title, subsection_title};
use super::tools::{AiToolStatuses, binary_row, detected_cli_row};
use crate::app::cli_install;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;

pub(super) fn cli_section(
    ai_tools: Option<&AiToolStatuses>,
    cli_install: Option<Option<&crate::app::cli_install::CliInstallState>>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .debug_selector(|| "settings-cli-section".to_owned())
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(section_title("CLI", t))
        .child(version_control_rows(ai_tools, cli_install, t, cx))
        .child(forge_rows(t))
        .into_any_element()
}

fn version_control_rows(
    ai_tools: Option<&AiToolStatuses>,
    cli_install: Option<Option<&crate::app::cli_install::CliInstallState>>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let mut col = div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .child(subsection_title("Version control", t));
    if cli_install::supported() {
        if let Some(rows) = cli_row::command_line_rows(cli_install, t, cx) {
            col = col.child(rows);
        }
    } else {
        col = col.child(binary_row(
            "jayjay",
            glyph::INFO,
            ai_tools.map(|s| s.jayjay.as_deref()),
            "Not installed",
            t,
        ));
    }
    col.child(detected_cli_row(
        "jj",
        glyph::GIT_BRANCH,
        check_jj_environment(),
        t,
    ))
    .into_any_element()
}

fn forge_rows(t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .child(subsection_title("Forges", t))
        .child(detected_cli_row(
            "gh",
            glyph::GIT_MERGE,
            check_gh_environment(),
            t,
        ))
        .child(detected_cli_row(
            "glab",
            glyph::GIT_MERGE,
            check_glab_environment(),
            t,
        ))
}
