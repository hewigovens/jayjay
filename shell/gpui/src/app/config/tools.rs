use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct ToolsConfig {
    pub external_editor: String,
    pub custom_editor_command: String,
    pub terminal: String,
    pub custom_terminal_command: String,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            external_editor: "vscode".to_owned(),
            custom_editor_command: String::new(),
            terminal: "terminal".to_owned(),
            custom_terminal_command: String::new(),
        }
    }
}
