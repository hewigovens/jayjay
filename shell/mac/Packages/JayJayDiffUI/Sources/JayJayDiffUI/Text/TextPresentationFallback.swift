import AppKit

/// TextKit 1 falls back to Apple Color Emoji for symbols such as ↩ that default to text presentation; pick a text font first, as TextKit 2 does.
public final class TextPresentationFallback: NSObject, NSTextStorageDelegate {
    public static let shared = TextPresentationFallback()

    private static let textDefaultEmoji: CharacterSet = {
        var set = CharacterSet()
        for value in 0x80 ... 0x1FFFF {
            if let scalar = Unicode.Scalar(value), scalar.properties.isEmoji, !scalar.properties.isEmojiPresentation {
                set.insert(scalar)
            }
        }
        return set
    }()

    private var fallbacks: [String: NSFont?] = [:]

    public func textStorage(
        _ textStorage: NSTextStorage,
        willProcessEditing editedMask: NSTextStorageEditActions,
        range editedRange: NSRange,
        changeInLength delta: Int
    ) {
        let text = textStorage.string as NSString
        var searchRange = editedRange
        while searchRange.length > 0 {
            let found = text.rangeOfCharacter(from: Self.textDefaultEmoji, range: searchRange)
            guard found.location != NSNotFound else { return }
            searchRange = NSRange(location: NSMaxRange(found), length: NSMaxRange(searchRange) - NSMaxRange(found))
            guard Self.isStandalone(found, in: text),
                  let font = textStorage.attribute(.font, at: found.location, effectiveRange: nil) as? NSFont,
                  let fallback = fallback(for: text.substring(with: found), base: font)
            else { continue }
            textStorage.addAttribute(.font, value: fallback, range: found)
        }
    }

    private func fallback(for character: String, base: NSFont) -> NSFont? {
        guard !Self.covers(base as CTFont, character) else { return nil }
        let key = "\(base.fontName) \(base.pointSize) \(character)"
        if let cached = fallbacks[key] {
            return cached
        }
        let cascade = CTFontCopyDefaultCascadeListForLanguages(base as CTFont, nil) as? [CTFontDescriptor] ?? []
        let font = cascade.lazy
            .map { CTFontCreateWithFontDescriptor($0, base.pointSize, nil) }
            .first { !(CTFontCopyPostScriptName($0) as String).contains("Emoji") && Self.covers($0, character) }
            .map { $0 as NSFont }
        fallbacks[key] = .some(font)
        return font
    }

    private static func isStandalone(_ found: NSRange, in text: NSString) -> Bool {
        let cluster = text.rangeOfComposedCharacterSequence(at: found.location)
        return cluster == found || (cluster.length == found.length + 1 && text.character(at: NSMaxRange(found)) == 0xFE0E)
    }

    private static func covers(_ font: CTFont, _ character: String) -> Bool {
        let units = Array(character.utf16)
        var glyphs = [CGGlyph](repeating: 0, count: units.count)
        return CTFontGetGlyphsForCharacters(font, units, &glyphs, units.count)
    }
}
