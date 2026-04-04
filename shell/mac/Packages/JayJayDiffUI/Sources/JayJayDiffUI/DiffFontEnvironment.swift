import AppKit
import SwiftUI

/// Default font size used as the reference point for all scaled fonts.
private let defaultFontSize: Double = 12.0

private struct DiffFontSizeKey: EnvironmentKey {
    static let defaultValue: Double = defaultFontSize
}

private struct DiffFontFamilyKey: EnvironmentKey {
    static let defaultValue: String = ""
}

public extension EnvironmentValues {
    var diffFontSize: Double {
        get { self[DiffFontSizeKey.self] }
        set { self[DiffFontSizeKey.self] = newValue }
    }

    var diffFontFamily: String {
        get { self[DiffFontFamilyKey.self] }
        set { self[DiffFontFamilyKey.self] = newValue }
    }
}

struct DiffFontModifier: ViewModifier {
    @Environment(\.diffFontSize) private var baseFontSize
    @Environment(\.diffFontFamily) private var fontFamily

    let size: CGFloat
    let weight: Font.Weight
    let design: Font.Design

    func body(content: Content) -> some View {
        let scaled = size * (baseFontSize / defaultFontSize)
        if (design == .monospaced || design == .default),
           !fontFamily.isEmpty,
           let font = NSFont(name: fontFamily, size: scaled)
        {
            content.font(Font(font as CTFont))
        } else {
            content.font(.system(size: scaled, weight: weight, design: design))
        }
    }
}

public extension View {
    func diffFont(
        _ size: CGFloat,
        weight: Font.Weight = .regular,
        design: Font.Design = .default
    ) -> some View {
        modifier(DiffFontModifier(size: size, weight: weight, design: design))
    }
}
