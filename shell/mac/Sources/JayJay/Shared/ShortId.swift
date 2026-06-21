import JayJayCore
import SwiftUI

/// Rendering for the core `ShortId` model (an id plus its shortest-unique-prefix
/// length): the prefix is highlighted and the remainder dimmed.
extension ShortId {
    /// `id` truncated to `maxChars`, with the first `shortLen` characters colored.
    /// Returns an `AttributedString` so callers compose styled runs without the
    /// `Text(+)` operator (deprecated in macOS 26).
    func highlighted(scheme: ColorScheme, maxChars: Int = 12) -> AttributedString {
        let shown = String(id.prefix(maxChars))
        let n = max(0, min(Int(shortLen), shown.count))
        let split = shown.index(shown.startIndex, offsetBy: n)
        var attr = AttributedString(shown[..<split])
        attr.foregroundColor = AppColors.changeIdPrefix(scheme)
        var rest = AttributedString(shown[split...])
        rest.foregroundColor = .secondary
        attr.append(rest)
        return attr
    }
}
