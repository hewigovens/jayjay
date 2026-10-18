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
        Group {
            if let icon = setiIconForPath(path: path) {
                let size = 19.5 * typography.interfaceScale
                // Seti centers each glyph half an em above the baseline, well above the text it labels.
                Text(icon.glyph)
                    .font(.custom(Self.setiFontName, fixedSize: size))
                    .baselineOffset(-0.27 * size)
                    .fixedSize()
                    .padding(.leading, -CGFloat(icon.inkLeft) * size)
                    .frame(width: CGFloat(icon.inkWidth) * size, alignment: .leading)
            } else {
                // SF Symbols pad their ink by about an eighth of the point size on each side.
                Image(systemName: FileKind(path: path).symbolName)
                    .jayjayFont(12)
                    .padding(.horizontal, -0.12 * 12 * typography.interfaceScale)
            }
        }
        .foregroundStyle(.secondary)
        .accessibilityHidden(true)
    }
}
