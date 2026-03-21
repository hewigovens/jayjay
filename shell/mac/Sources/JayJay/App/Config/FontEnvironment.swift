import AppKit
import SwiftUI

/// Default font size used as the reference point for all scaled fonts.
private let defaultFontSize: Double = 12.0

private struct JayJayFontSizeKey: EnvironmentKey {
    static let defaultValue: Double = defaultFontSize
}

private struct JayJayFontFamilyKey: EnvironmentKey {
    static let defaultValue: AppSettings.MonoFont = .system
}

extension EnvironmentValues {
    var jayjayFontSize: Double {
        get { self[JayJayFontSizeKey.self] }
        set { self[JayJayFontSizeKey.self] = newValue }
    }

    var jayjayFontFamily: AppSettings.MonoFont {
        get { self[JayJayFontFamilyKey.self] }
        set { self[JayJayFontFamilyKey.self] = newValue }
    }
}

private struct JayJayFontModifier: ViewModifier {
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    let size: CGFloat
    let weight: Font.Weight
    let design: Font.Design

    func body(content: Content) -> some View {
        let scaled = size * (baseFontSize / defaultFontSize)
        if design == .monospaced || design == .default, fontFamily != .system {
            content.font(Font(fontFamily.nsFont(size: scaled) as CTFont))
        } else {
            content.font(.system(size: scaled, weight: weight, design: design))
        }
    }
}

extension View {
    func jayjayFont(
        _ size: CGFloat,
        weight: Font.Weight = .regular,
        design: Font.Design = .default
    ) -> some View {
        modifier(JayJayFontModifier(size: size, weight: weight, design: design))
    }
}
