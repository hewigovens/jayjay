import AppKit
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

    struct MonoFont: RawRepresentable, CaseIterable, Hashable, Identifiable {
        let rawValue: String

        init?(rawValue: String) {
            let id = Self.canonicalId(rawValue)
            guard Self.option(for: id) != nil else { return nil }
            self.rawValue = id
        }

        private init(validatedRawValue: String) {
            rawValue = validatedRawValue
        }

        var id: String {
            rawValue
        }

        static let system = MonoFont(validatedRawValue: "system")

        static var allCases: [MonoFont] {
            options.map { MonoFont(validatedRawValue: $0.id) }
        }

        var title: String {
            option?.title ?? rawValue
        }

        func nsFont(size: CGFloat) -> NSFont {
            if self == .system {
                return NSFont.monospacedSystemFont(ofSize: size, weight: .regular)
            }
            return resolvedFont(size: size)
                ?? NSFont.monospacedSystemFont(ofSize: size, weight: .regular)
        }

        var isInstalled: Bool {
            if self == .system {
                return true
            }
            return resolvedFont(size: 12) != nil
        }

        var nsFontName: String {
            if self == .system {
                return ""
            }
            return resolvedFont(size: 12)?.fontName ?? fontNames.first ?? ""
        }

        private var option: MonoFontOption? {
            Self.option(for: rawValue)
        }

        private var fontNames: [String] {
            option?.fontNames ?? []
        }

        private func resolvedFont(size: CGFloat) -> NSFont? {
            for name in fontNames {
                if let font = NSFont(name: name, size: size) {
                    return font
                }
            }
            guard let familyName = fontNames.first else { return nil }
            return Self.regularFont(inFamily: familyName, size: size)
        }

        private static func regularFont(inFamily familyName: String, size: CGFloat) -> NSFont? {
            guard let members = NSFontManager.shared.availableMembers(ofFontFamily: familyName) else {
                return nil
            }
            let regularMember = members.first { member in
                guard member.count > 1, let face = member[1] as? String else { return false }
                return face == "Regular" || face == "Book" || face == "Roman"
            }
            guard let fontName = (regularMember ?? members.first)?.first as? String else {
                return nil
            }
            return NSFont(name: fontName, size: size)
        }

        private static let options: [MonoFontOption] = monoFontOptions()

        private static let optionsById: [String: MonoFontOption] = Dictionary(
            uniqueKeysWithValues: options.map { ($0.id, $0) }
        )

        private static func option(for id: String) -> MonoFontOption? {
            optionsById[id]
        }

        private static func canonicalId(_ rawValue: String) -> String {
            switch rawValue {
                case "ioskeleymono-nl-nerd-font": "ioskeley-mono-nl-nerd-font"
                default: rawValue
            }
        }
    }
}
