use jayjay_core::JayError;
use jayjay_core::agent_skills::{AgentSkillInstallation, AgentSkillInstaller};

#[uniffi::remote(Record)]
pub struct AgentSkillInstallation {
    pub installed: bool,
    pub complete: bool,
}

#[uniffi::export]
fn agent_skill_installation() -> Result<AgentSkillInstallation, JayError> {
    AgentSkillInstaller::for_user()?.status()
}

#[uniffi::export]
fn set_agent_skill_installed(installed: bool) -> Result<(), JayError> {
    AgentSkillInstaller::for_user()?.set_installed(installed)
}
