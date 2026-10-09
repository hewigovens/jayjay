import AppKit
import JayJayCore

extension AppSettings {
    struct FontFamily: RawRepresentable, CaseIterable, Hashable, Identifiable {
        let rawValue: String

        init?(rawValue: String) {
            guard !rawValue.isEmpty else { return nil }
            self.rawValue = Self.canonicalId(rawValue)
        }

        private init(validatedRawValue: String) {
            rawValue = validatedRawValue
        }

        var id: String {
            rawValue
        }

        static let system = FontFamily(validatedRawValue: "system")

        static var allCases: [FontFamily] {
            options.map { FontFamily(validatedRawValue: $0.id) }
        }

        var title: String {
            option?.title ?? rawValue
        }

        func nsFont(size: CGFloat, monospaced: Bool = true, ligatures: Bool = true) -> NSFont {
            let systemFont = monospaced
                ? NSFont.monospacedSystemFont(ofSize: size, weight: .regular)
                : NSFont.systemFont(ofSize: size)
            let font = self == .system ? systemFont : resolvedFont(size: size) ?? systemFont
            return ligatures ? font : font.withoutLigatures
        }

        var hasLigatures: Bool {
            let probe = "-> => != == === <= >= :: www fi fl"
            let font = nsFont(size: 12)
            return font.glyphs(for: probe) != font.withoutLigatures.glyphs(for: probe)
        }

        var isInstalled: Bool {
            self == .system || resolvedFont(size: 12) != nil
        }

        private var option: MonoFontOption? {
            Self.option(for: rawValue)
        }

        private var fontNames: [String] {
            option?.fontNames ?? [rawValue]
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

private extension NSFont {
    func glyphs(for text: String) -> [CGGlyph] {
        let line = CTLineCreateWithAttributedString(NSAttributedString(string: text, attributes: [.font: self]))
        let runs = CTLineGetGlyphRuns(line) as? [CTRun] ?? []
        return runs.flatMap { run in
            var glyphs = [CGGlyph](repeating: 0, count: CTRunGetGlyphCount(run))
            CTRunGetGlyphs(run, CFRange(), &glyphs)
            return glyphs
        }
    }

    var withoutLigatures: NSFont {
        let features: [[NSFontDescriptor.FeatureKey: Int]] = [
            [.typeIdentifier: kLigaturesType, .selectorIdentifier: kCommonLigaturesOffSelector],
            [.typeIdentifier: kContextualAlternatesType, .selectorIdentifier: kContextualAlternatesOffSelector]
        ]
        return NSFont(descriptor: fontDescriptor.addingAttributes([.featureSettings: features]), size: pointSize) ?? self
    }
}
