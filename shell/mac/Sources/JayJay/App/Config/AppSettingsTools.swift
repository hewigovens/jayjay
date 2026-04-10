import AppKit

extension AppSettings {
    func openInEditor(filePath: String, repoPath: String) {
        let fullPath = URL(fileURLWithPath: repoPath).appendingPathComponent(filePath).path
        _ = openInEditor(absolutePath: fullPath, cwd: repoPath)
    }

    /// Opens an absolute path in the user-configured editor; returns false if the editor's CLI is unavailable.
    @discardableResult
    func openInEditor(absolutePath: String, cwd: String? = nil) -> Bool {
        let cmd = externalEditor == .custom ? customEditorCommand : externalEditor.command
        guard !cmd.isEmpty else { return false }

        if externalEditor.isTerminalEditor {
            let escapedPath = absolutePath.replacingOccurrences(of: "'", with: "'\\''")
            openInTerminal(at: cwd ?? NSHomeDirectory(), command: "\(cmd) '\(escapedPath)'")
            return true
        }

        guard let binary = ExternalEditor.findBinary(cmd) else { return false }
        let process = Process()
        process.executableURL = URL(fileURLWithPath: binary)
        process.arguments = [absolutePath]
        do {
            try process.run()
            return true
        } catch {
            return false
        }
    }

    func openInTerminal(at path: String, command: String? = nil) {
        switch terminal {
            case .terminal, .custom:
                openViaAppleScript(
                    appName: terminal == .custom ? customTerminalCommand : "Terminal",
                    command: command ?? "cd '\(path.replacingOccurrences(of: "'", with: "'\\''"))'"
                )
            case .iterm:
                let cdCmd = command ?? "cd '\(path.replacingOccurrences(of: "'", with: "'\\''"))'"
                let script = """
                tell application "iTerm2"
                    activate
                    try
                        tell current window
                            create tab with default profile command "/bin/zsh"
                            tell current session
                                write text "\(cdCmd)"
                            end tell
                        end tell
                    on error
                        create window with default profile command "/bin/zsh"
                        tell current window
                            tell current session
                                write text "\(cdCmd)"
                            end tell
                        end tell
                    end try
                end tell
                """
                if let appleScript = NSAppleScript(source: script) {
                    appleScript.executeAndReturnError(nil)
                }
            case .ghostty:
                guard let appURL = NSWorkspace.shared.urlForApplication(withBundleIdentifier: terminal.bundleId)
                else { return }
                let process = Process()
                process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
                process.arguments = ["-na", appURL.path, "--args", "--working-directory=\(path)"]
                if let command {
                    process.arguments?.append(contentsOf: ["-e", command])
                }
                try? process.run()
        }
    }

    private func openViaAppleScript(appName: String, command: String) {
        let escapedCmd = command
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        let escapedApp = appName
            .replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        let script = "tell application \"\(escapedApp)\" to do script \"\(escapedCmd)\""
        if let appleScript = NSAppleScript(source: script) {
            appleScript.executeAndReturnError(nil)
        }
    }
}
