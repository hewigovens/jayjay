import AppKit
import JayJayCore

extension AppSettings.FontFamily {
    static func installedCodeFontGroups() -> [FontFamilyGroup] {
        let choices = codeFontChoices(monospacedFamilies: installedFamilyNames {
            CTFontCreateWithFontDescriptor($0, 12, nil).hasFixedAdvances
        })
        return [
            FontFamilyGroup(title: "Coding Fonts", families: choices.coding.compactMap(Self.init(rawValue:))),
            FontFamilyGroup(title: "Other Monospaced", families: choices.other.compactMap(Self.init(rawValue:)))
        ]
    }

    static func installedUserFamilies() -> [AppSettings.FontFamily] {
        let families = installedFamilyNames { descriptor in
            (CTFontDescriptorCopyAttribute(descriptor, kCTFontURLAttribute) as? URL)
                .map { isUserFontPath(path: $0.path) } ?? false
        }
        return fontFamilyChoices(families: families).compactMap(Self.init(rawValue:))
    }

    private static func installedFamilyNames(where include: (CTFontDescriptor) -> Bool) -> [String] {
        let collection = CTFontCollectionCreateFromAvailableFonts(nil)
        let descriptors = CTFontCollectionCreateMatchingFontDescriptors(collection) as? [CTFontDescriptor] ?? []
        var checked = Set<String>()
        var families: [String] = []
        for descriptor in descriptors {
            guard let family = CTFontDescriptorCopyAttribute(descriptor, kCTFontFamilyNameAttribute) as? String,
                  checked.insert(family).inserted,
                  include(descriptor)
            else { continue }
            families.append(family)
        }
        return families
    }
}

private extension CTFont {
    /// The monospace trait misses some mono fonts (Nerd Font patches), so compare advances.
    var hasFixedAdvances: Bool {
        let probe = Array("iMW.0".utf16)
        var glyphs = [CGGlyph](repeating: 0, count: probe.count)
        guard CTFontGetGlyphsForCharacters(self, probe, &glyphs, probe.count) else { return false }
        var advances = [CGSize](repeating: .zero, count: glyphs.count)
        CTFontGetAdvancesForGlyphs(self, .horizontal, glyphs, &advances, glyphs.count)
        return advances.allSatisfy { abs($0.width - advances[0].width) < 0.01 }
    }
}
