use gpui::{AppContext, Context};

use super::{SettingsView, cli_diagnostics, config, tools};

impl SettingsView {
    pub(super) fn ensure_jj_config_loaded(&mut self, cx: &mut Context<Self>) {
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

    /// Loads detection snapshots for Integrations: AI binaries, CLI versions, and the CLI install state (a no-op `None` on non-Linux platforms).
    pub(super) fn ensure_tools_loaded(&mut self, cx: &mut Context<Self>) {
        if self.tools_loading
            || (self.ai_tools.is_some()
                && self.cli_diagnostics.is_some()
                && self.cli_install.is_some())
        {
            return;
        }
        self.tools_loading = true;
        cx.spawn(async move |this, cx| {
            let (statuses, cli_install, diagnostics) = cx
                .background_spawn(async {
                    (
                        tools::load_ai_tool_statuses(),
                        crate::app::cli_install::load_state(),
                        cli_diagnostics::CliDiagnostics::load(),
                    )
                })
                .await;
            let _ = this.update(cx, move |view, cx| {
                view.tools_loading = false;
                // Don't clobber snapshots that arrived while this load ran (injected statuses, install/remove clicks).
                if view.ai_tools.is_none() {
                    view.ai_tools = Some(statuses);
                }
                if view.cli_diagnostics.is_none() {
                    view.cli_diagnostics = Some(diagnostics);
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
