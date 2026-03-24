import AppKit

extension AppSettings {
    /// Supported external editors. Add new cases to extend — the `custom` case is always last.
    enum ExternalEditor: String, CaseIterable, Identifiable {
        case vscode, zed, xcode, androidStudio, vim, custom

        var id: String { rawValue }

        var title: String {
            switch self {
                case .vscode: "Visual Studio Code"
                case .zed: "Zed"
                case .xcode: "Xcode"
                case .androidStudio: "Android Studio"
                case .vim: "Vim"
                case .custom: "Custom"
            }
        }

        var command: String {
            switch self {
                case .vscode: "code"
                case .zed: "zed"
                case .xcode: "xed"
                case .androidStudio: "studio"
                case .vim: "vim"
                case .custom: ""
            }
        }

        var bundleId: String? {
            switch self {
                case .vscode: "com.microsoft.VSCode"
                case .zed: "dev.zed.Zed"
                case .xcode: "com.apple.dt.Xcode"
                case .androidStudio: "com.google.android.studio"
                default: nil
            }
        }

        var isTerminalEditor: Bool {
            switch self {
                case .vim: true
                default: false
            }
        }

        /// The tool name to pass to `jj resolve --tool`. Uses jj's built-in merge tool config
        /// which materializes proper git-style conflict markers for the editor.
        var jjMergeTool: String? {
            switch self {
                case .vscode: "vscode"
                case .zed: "zed"
                default: nil
            }
        }

        var isInstalled: Bool {
            if let bid = bundleId {
                return NSWorkspace.shared.urlForApplication(withBundleIdentifier: bid) != nil
            }
            return Self.findBinary(command) != nil
        }

        static func findBinary(_ name: String) -> String? {
            guard !name.isEmpty else { return nil }
            let paths = [
                "/opt/homebrew/bin/\(name)",
                "/usr/local/bin/\(name)",
                "\(NSHomeDirectory())/.local/bin/\(name)"
            ]
            return paths.first(where: { FileManager.default.isExecutableFile(atPath: $0) })
        }
    }
}
