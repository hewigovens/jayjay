import CoreText
import JayJayCore
import SwiftUI

struct FileIcon: View {
    let path: String

    @Environment(\.jayjayTypography) private var typography

    private static let setiFontName: String = {
        if let url = Bundle.main.url(forResource: "Seti", withExtension: "ttf") {
            CTFontManagerRegisterFontsForURL(url as CFURL, .process, nil)
        }
        return "seti"
    }()

    private static let baseSize: CGFloat = 19.5

    static func inkWidth(path: String, interfaceScale: CGFloat) -> CGFloat {
        let icon = setiIconForPath(path: path)
        return CGFloat(icon.inkWidth) * baseSize * interfaceScale * CGFloat(icon.scale)
    }

    var body: some View {
        let icon = setiIconForPath(path: path)
        let base = Self.baseSize * typography.interfaceScale
        let size = base * CGFloat(icon.scale)
        // Seti draws glyphs about half an em above the baseline; this puts every ink center at one height beside the text.
        Text(icon.glyph)
            .font(.custom(Self.setiFontName, fixedSize: size))
            .baselineOffset(0.23 * base - CGFloat(icon.inkCenter) * size)
            .fixedSize()
            .padding(.leading, -CGFloat(icon.inkLeft) * size)
            .frame(width: Self.inkWidth(path: path, interfaceScale: typography.interfaceScale), alignment: .leading)
            .foregroundStyle(.secondary)
            .accessibilityHidden(true)
    }
}
