import AppKit
import JayJayCore

extension AppSettings.FontFamily {
    static func installedMyFonts() -> [AppSettings.FontFamily] {
        let families = userFamilyNames { CTFontCreateWithFontDescriptor($0, 12, nil).hasFixedAdvances }
        return myMonoFontFamilies(families: families).compactMap(Self.init(rawValue:))
    }

    static func installedUserFamilies() -> [AppSettings.FontFamily] {
        userFamilyNames { _ in true }
            .sorted { $0.localizedCaseInsensitiveCompare($1) == .orderedAscending }
            .compactMap(Self.init(rawValue:))
    }

    private static func userFamilyNames(where include: (CTFontDescriptor) -> Bool) -> [String] {
        let collection = CTFontCollectionCreateFromAvailableFonts(nil)
        let descriptors = CTFontCollectionCreateMatchingFontDescriptors(collection) as? [CTFontDescriptor] ?? []
        var checked = Set<String>()
        var families: [String] = []
        for descriptor in descriptors {
            guard let url = CTFontDescriptorCopyAttribute(descriptor, kCTFontURLAttribute) as? URL,
                  isUserFontPath(path: url.path),
                  let family = CTFontDescriptorCopyAttribute(descriptor, kCTFontFamilyNameAttribute) as? String,
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
