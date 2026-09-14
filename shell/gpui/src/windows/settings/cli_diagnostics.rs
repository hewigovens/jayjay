use gpui::Context;
use jayjay_core::CliStatus;

use super::SettingsView;

#[derive(Clone, Debug)]
pub struct CliDiagnostics {
    pub jj: CliStatus,
    pub gh: CliStatus,
    pub glab: CliStatus,
    pub origin: CliStatus,
}

impl CliDiagnostics {
    pub(super) fn load() -> Self {
        Self {
            jj: jayjay_core::check_jj_environment(),
            gh: jayjay_core::check_gh_environment(),
            glab: jayjay_core::check_glab_environment(),
            origin: jayjay_core::check_origin_environment(),
        }
    }
}

impl SettingsView {
    /// Replaces the detection snapshot without an in-flight probe overwriting it.
    pub fn set_cli_diagnostics(&mut self, diagnostics: CliDiagnostics, cx: &mut Context<Self>) {
        self.cli_diagnostics = Some(diagnostics);
        cx.notify();
    }
}
