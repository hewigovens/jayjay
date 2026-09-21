import SwiftUI

/// Fonts are baked per run because `jayjayFont` applies one weight to the whole `Text`.
struct IdentifierText: View {
    enum PrefixStyle {
        case plain
        case changeId
        case commitId
    }

    let value: String
    var prefixLength: Int = 0
    var prefixStyle: PrefixStyle = .plain
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        text
            .foregroundStyle(.secondary)
            .textSelection(.enabled)
    }

    private var text: Text {
        let regular = fontFamily.identifierFont(baseSize: baseFontSize)
        let emphasized = min(prefixLength, value.count)
        guard emphasized > 0 else { return Text(value).font(regular) }
        let split = value.index(value.startIndex, offsetBy: emphasized)
        var attr = AttributedString(value[..<split])
        attr.font = regular.weight(.bold)
        attr.foregroundColor = prefixColor
        var rest = AttributedString(value[split...])
        rest.font = regular
        attr.append(rest)
        return Text(attr)
    }

    private var prefixColor: Color {
        switch prefixStyle {
            case .plain: .primary
            case .changeId: AppColors.changeIdPrefix(colorScheme)
            case .commitId: AppColors.commitIdPrefix(colorScheme)
        }
    }
}
