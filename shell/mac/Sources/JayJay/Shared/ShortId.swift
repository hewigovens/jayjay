import JayJayCore
import SwiftUI

/// Rendering for the core `ShortId` model (an id plus its shortest-unique-prefix
/// length): the prefix is highlighted and the remainder dimmed.
extension ShortId {
    /// Muted violet, light/dark adaptive, matching the DAG change-id highlight.
    static func prefixColor(_ scheme: ColorScheme) -> Color {
        scheme == .dark
            ? Color(red: 0x9B / 255.0, green: 0x7F / 255.0, blue: 0xCF / 255.0)
            : Color(red: 0x7C / 255.0, green: 0x4F / 255.0, blue: 0xC2 / 255.0)
    }

    /// `id` truncated to `maxChars`, with the first `shortLen` characters colored.
    func highlightedText(scheme: ColorScheme, maxChars: Int = 12) -> Text {
        let shown = String(id.prefix(maxChars))
        let n = max(0, min(Int(shortLen), shown.count))
        let split = shown.index(shown.startIndex, offsetBy: n)
        return Text(String(shown[..<split])).foregroundColor(Self.prefixColor(scheme))
            + Text(String(shown[split...])).foregroundColor(.secondary).fontWeight(.regular)
    }
}
