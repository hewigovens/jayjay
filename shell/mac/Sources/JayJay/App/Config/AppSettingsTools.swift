import AppKit
import JayJayCore

/// Thin shim around `jayjay_core::tools` (exposed via uniffi). All real
/// editor / terminal launch logic lives in the Rust crate so both shells
/// share one implementation — see `crates/jayjay-core/src/tools/`.
extension AppSettings {
    func openInEditor(filePath: String, repoPath: String) {
        _ = JayJayCore.openInEditor(
            repoPath: repoPath,
            filePath: filePath,
            externalEditor: externalEditor.rawValue,
            customEditorCommand: customEditorCommand,
            terminal: terminal.rawValue,
            customTerminalCommand: customTerminalCommand
        )
    }

    @discardableResult
    func openInEditor(absolutePath: String, cwd: String? = nil) -> Bool {
        JayJayCore.openInEditor(
            repoPath: cwd ?? NSHomeDirectory(),
            filePath: absolutePath,
            externalEditor: externalEditor.rawValue,
            customEditorCommand: customEditorCommand,
            terminal: terminal.rawValue,
            customTerminalCommand: customTerminalCommand
        )
    }

    func openInTerminal(at path: String, command: String? = nil) {
        _ = JayJayCore.openInTerminal(
            repoPath: path,
            command: command,
            terminal: terminal.rawValue,
            customTerminalCommand: customTerminalCommand
        )
    }
}
