import JayJayCore
import SwiftUI

/// Rendering for the core `ShortId` model (an id plus its shortest-unique-prefix
/// length): the prefix is bold and highlighted, the remainder regular and dimmed.
extension ShortId {
    var compact: String {
        String(id.prefix(max(8, Int(shortLen))))
    }

    /// `font` is baked into both runs because a per-run weight needs an explicit font.
    func highlighted(scheme: ColorScheme, font: Font, prefixColor: Color? = nil) -> AttributedString {
        let shown = compact
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
