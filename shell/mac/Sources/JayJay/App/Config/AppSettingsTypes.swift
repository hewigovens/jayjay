import AppKit
import SwiftUI

extension AppSettings {
    enum AppearanceMode: String, CaseIterable, Identifiable {
        case system, light, dark

        var id: String {
            rawValue
        }

        var title: String {
            switch self {
                case .system: "System"
                case .light: "Light"
                case .dark: "Dark"
            }
        }

        var colorScheme: ColorScheme? {
            switch self {
                case .system: nil
                case .light: .light
                case .dark: .dark
            }
        }
    }

    enum MonoFont: String, CaseIterable, Identifiable {
        case system
        case menlo
        case sfMono = "sf-mono"
        case jetBrainsMono = "jetbrains-mono"
        case firaCode = "fira-code"
        case cascadiaCode = "cascadia-code"

        var id: String {
            rawValue
        }

        var title: String {
            switch self {
                case .system: "System Mono"
                case .menlo: "Menlo"
                case .sfMono: "SF Mono"
                case .jetBrainsMono: "JetBrains Mono"
                case .firaCode: "Fira Code"
                case .cascadiaCode: "Cascadia Code"
            }
        }

        func nsFont(size: CGFloat) -> NSFont {
            switch self {
                case .system:
                    NSFont.monospacedSystemFont(ofSize: size, weight: .regular)
                default:
                    NSFont(name: fontName, size: size)
                        ?? NSFont.monospacedSystemFont(ofSize: size, weight: .regular)
            }
        }

        var isInstalled: Bool {
            if self == .system { return true }
            return NSFont(name: fontName, size: 12) != nil
        }

        private var fontName: String {
            switch self {
                case .system: "Menlo"
                case .menlo: "Menlo"
                case .sfMono: "SF Mono"
                case .jetBrainsMono: "JetBrains Mono"
                case .firaCode: "Fira Code"
                case .cascadiaCode: "Cascadia Code"
            }
        }
    }

    enum ExternalEditor: String, CaseIterable, Identifiable {
        case vscode, zed, vim, custom

        var id: String {
            rawValue
        }

        var title: String {
            switch self {
                case .vscode: "Visual Studio Code"
                case .zed: "Zed"
                case .vim: "Vim"
                case .custom: "Custom"
            }
        }

        var command: String {
            switch self {
                case .vscode: "code"
                case .zed: "zed"
                case .vim: "vim"
                case .custom: ""
            }
        }

        var bundleId: String? {
            switch self {
                case .vscode: "com.microsoft.VSCode"
                case .zed: "dev.zed.Zed"
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

    enum Terminal: String, CaseIterable, Identifiable {
        case terminal, iterm, ghostty, custom

        var id: String {
            rawValue
        }

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
