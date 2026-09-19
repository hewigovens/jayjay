import SwiftUI

enum JayJayTextStyle {
    case title, body, secondary

    var size: CGFloat {
        switch self {
            case .title: 20
            case .body: 13
            case .secondary: 11
        }
    }

    var weight: Font.Weight {
        self == .title ? .semibold : .regular
    }
}

private struct JayJayTextStyleModifier: ViewModifier {
    @Environment(\.jayjayFontSize) private var fontSize
    let style: JayJayTextStyle

    func body(content: Content) -> some View {
        let size = style.size * fontSize / 13
        content.font(.system(size: size, weight: style.weight))
    }
}

extension View {
    func jayjayFont(_ style: JayJayTextStyle) -> some View {
        modifier(JayJayTextStyleModifier(style: style))
    }
}
