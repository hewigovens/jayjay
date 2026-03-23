import AppKit

extension AppSettings {
    enum Terminal: String, CaseIterable, Identifiable {
        case terminal, iterm, ghostty, custom

        var id: String { rawValue }

        var title: String {
            switch self {
                case .terminal: "Terminal"
                case .iterm: "iTerm2"
                case .ghostty: "Ghostty"
                case .custom: "Custom"
            }
        }

        var bundleId: String {
            switch self {
                case .terminal: "com.apple.Terminal"
                case .iterm: "com.googlecode.iterm2"
                case .ghostty: "com.mitchellh.ghostty"
                case .custom: ""
            }
        }

        var isInstalled: Bool {
            if self == .terminal || self == .custom { return true }
            return NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId) != nil
        }
    }
}
