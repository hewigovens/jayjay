import AppKit
import JayJayCore

extension AppSettings {
    struct FontFamily: RawRepresentable, Hashable, Identifiable {
        let rawValue: String

        init?(rawValue: String) {
            guard !rawValue.isEmpty else { return nil }
            let family = monoFontFamily(preference: rawValue)
            self.rawValue = family.isEmpty ? Self.system.rawValue : family
        }

        private init(validatedRawValue: String) {
            rawValue = validatedRawValue
        }

        var id: String {
            rawValue
        }

        static let system = FontFamily(validatedRawValue: "system")

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

        private func resolvedFont(size: CGFloat) -> NSFont? {
            NSFont(name: rawValue, size: size) ?? Self.regularFont(inFamily: rawValue, size: size)
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
