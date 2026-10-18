use gpui::Context;
use jayjay_core::{CliStatus, HostLogin, RepoHost};

use super::SettingsView;

#[derive(Clone, Debug)]
pub struct CliDiagnostics {
    pub jj: CliStatus,
    pub origin: CliStatus,
    pub forge_logins: Vec<HostLogin>,
}

impl CliDiagnostics {
    pub(super) fn load() -> Self {
        Self {
            jj: jayjay_core::check_jj_environment(),
            origin: jayjay_core::check_origin_environment(),
            forge_logins: RepoHost::TOKEN_HOSTS.map(RepoHost::login).to_vec(),
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
