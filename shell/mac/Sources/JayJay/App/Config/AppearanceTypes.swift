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
                case .light: "☀ Light"
                case .dark: "☾ Dark"
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
}
