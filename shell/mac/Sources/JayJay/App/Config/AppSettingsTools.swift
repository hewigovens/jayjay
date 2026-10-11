import AppKit
import JayJayCore

/// Thin shim around `jayjay_core::tools` (exposed via uniffi). All real
/// editor / terminal launch logic lives in the Rust crate so both shells
/// share one implementation — see `crates/jayjay-core/src/tools/`.
extension AppSettings {
    func openInEditor(filePath: String, repoPath: String, line: UInt32? = nil) {
        _ = JayJayCore.openInEditor(
            repoPath: repoPath,
            filePath: filePath,
            line: line,
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
            line: nil,
            externalEditor: externalEditor.rawValue,
            customEditorCommand: customEditorCommand,
            terminal: terminal.rawValue,
            customTerminalCommand: customTerminalCommand
        )
    }

    var externalEditorOpensAtLine: Bool {
        editorOpensAtLine(externalEditor: externalEditor.rawValue)
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
