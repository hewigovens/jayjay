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

    var body: some View {
        let icon = setiIconForPath(path: path)
        let base = 19.5 * typography.interfaceScale
        let size = base * CGFloat(icon.scale)
        // Seti draws glyphs about half an em above the baseline; this puts every ink center at one height beside the text.
        Text(icon.glyph)
            .font(.custom(Self.setiFontName, fixedSize: size))
            .baselineOffset(0.23 * base - CGFloat(icon.inkCenter) * size)
            .fixedSize()
            .padding(.leading, -CGFloat(icon.inkLeft) * size)
            .frame(width: CGFloat(icon.inkWidth) * size, alignment: .leading)
            .foregroundStyle(.secondary)
            .accessibilityHidden(true)
    }
}
