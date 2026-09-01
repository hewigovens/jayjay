use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct ToolsConfig {
    pub(crate) external_editor: String,
    pub(crate) custom_editor_command: String,
    pub(crate) terminal: String,
    pub(crate) custom_terminal_command: String,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            external_editor: if cfg!(target_os = "linux") {
                "system".to_owned()
            } else {
                "vscode".to_owned()
            },
            custom_editor_command: String::new(),
            terminal: "terminal".to_owned(),
            custom_terminal_command: String::new(),
        }
    }
}
