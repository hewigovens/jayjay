import JayJayCore
import SwiftUI

/// Rendering for the core `ShortId` model (an id plus its shortest-unique-prefix
/// length): the prefix is bold and highlighted, the remainder regular and dimmed.
extension ShortId {
    /// `font` is baked into both runs because a per-run weight needs an explicit font.
    func highlighted(scheme: ColorScheme, font: Font, maxChars: Int = 8, prefixColor: Color? = nil) -> AttributedString {
        let shown = String(id.prefix(max(maxChars, Int(shortLen))))
        let n = max(0, min(Int(shortLen), shown.count))
        let split = shown.index(shown.startIndex, offsetBy: n)
        var attr = AttributedString(shown[..<split])
        attr.foregroundColor = prefixColor ?? AppColors.changeIdPrefix(scheme)
        attr.font = font.weight(.bold)
        var rest = AttributedString(shown[split...])
        rest.foregroundColor = .secondary
        rest.font = font
        attr.append(rest)
        return attr
    }
}
