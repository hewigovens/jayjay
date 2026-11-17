import SwiftUI

/// Every editor and terminal shares one monochrome tile, so the toolbar never carries a third-party icon's colours.
struct ToolbarAppMark: View {
    enum Glyph {
        case asset(String, size: CGFloat = 15)
        case symbol(String, size: CGFloat = 12)
    }

    let glyph: Glyph

    var body: some View {
        RoundedRectangle(cornerRadius: 6, style: .continuous)
            .fill(.primary.opacity(0.1))
            .frame(width: 22, height: 22)
            .overlay {
                switch glyph {
                    case let .asset(name, size):
                        Image(name)
                            .resizable()
                            .scaledToFit()
                            .frame(width: size, height: size)
                    case let .symbol(name, size):
                        Image(systemName: name)
                            .font(.system(size: size, weight: .semibold))
                }
            }
    }
}
