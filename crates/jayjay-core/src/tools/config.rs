/// User-configured tool choices. Field names match the SwiftUI `AppSettings` keys so the same config flows through both shells.
#[derive(Debug, Clone, Default)]
pub struct ToolsConfig {
    pub external_editor: String,
    pub custom_editor_command: String,
    pub terminal: String,
    pub custom_terminal_command: String,
}
