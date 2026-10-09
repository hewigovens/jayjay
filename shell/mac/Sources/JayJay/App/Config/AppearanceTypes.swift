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
}
