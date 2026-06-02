import AppKit
import JayJayCore

extension AppSettings {
    /// Supported external editors. Add new cases to extend — the `custom` case is always last.
    enum ExternalEditor: String, CaseIterable, Identifiable {
        case vscode, vscodium, zed, xcode, vim, custom

        var id: String {
            rawValue
        }

        var title: String {
            switch self {
                case .vscode: "Visual Studio Code"
                case .vscodium: "VSCodium"
                case .zed: "Zed"
                case .xcode: "Xcode"
                case .vim: "Vim"
                case .custom: "Custom"
            }
        }

        var command: String {
            switch self {
                case .vscode: "code"
                case .vscodium: "codium"
                case .zed: "zed"
                case .xcode: "xed"
                case .vim: "vim"
                case .custom: ""
            }
        }

        var bundleId: String? {
            switch self {
                case .vscode: "com.microsoft.VSCode"
                case .vscodium: "com.vscodium.codium"
                case .zed: "dev.zed.Zed"
                case .xcode: "com.apple.dt.Xcode"
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
                case .vscodium: "vscodium"
                case .zed: "zed"
                default: nil
            }
        }

        var isInstalled: Bool {
            if let bid = bundleId {
                return NSWorkspace.shared.urlForApplication(withBundleIdentifier: bid) != nil
            }
            return findBinary(name: command) != nil
        }
    }
}
