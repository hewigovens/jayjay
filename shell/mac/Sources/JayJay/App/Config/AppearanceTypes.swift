import JayJayCore
import SwiftUI

extension AppSettings {
    enum AppearanceMode: String, CaseIterable, Identifiable {
        case system, light, dark

        var id: String {
            rawValue
        }

        var title: String {
            switch self {
                case .system: Localized.Settings.appearanceSystem
                case .light: "☀ \(Localized.Settings.appearanceLight)"
                case .dark: "☾ \(Localized.Settings.appearanceDark)"
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
}
