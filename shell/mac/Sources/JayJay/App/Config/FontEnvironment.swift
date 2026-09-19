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

extension AppSettings.MonoFont {
    func scaledFont(
        _ size: CGFloat,
        baseSize: Double,
        weight: Font.Weight = .regular,
        design: Font.Design = .default
    ) -> Font {
        let scaled = size * (baseSize / defaultFontSize)
        if design == .monospaced || design == .default, self != .system {
            return Font(nsFont(size: scaled) as CTFont).weight(weight)
        }
        return .system(size: scaled, weight: weight, design: design)
    }

    /// Short ids share one nominal size wherever they appear.
    func identifierFont(baseSize: Double) -> Font {
        scaledFont(11, baseSize: baseSize, design: .monospaced)
    }
}

private struct JayJayFontModifier: ViewModifier {
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    let size: CGFloat
    let weight: Font.Weight
    let design: Font.Design

    func body(content: Content) -> some View {
        content.font(fontFamily.scaledFont(size, baseSize: baseFontSize, weight: weight, design: design))
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
