import AppKit
import JayJayDiffUI
import SwiftUI

/// Interface text uses the interface font; monospaced text (code, identifiers, revsets) uses the code font.
struct Typography: Equatable {
    /// Keep the scaling baseline stable so saved font sizes retain their meaning.
    static let referenceSize: Double = 12

    var interfaceFamily: AppSettings.FontFamily = .system
    var interfaceSize: Double = AppSettings.defaultFontSize
    var codeFamily: AppSettings.FontFamily = .system
    var codeSize: Double = AppSettings.defaultFontSize
    var codeLineSpacing: Double = 1
    var codeLigatures = true

    func font(_ size: CGFloat, weight: Font.Weight = .regular, design: Font.Design = .default) -> Font {
        let monospaced = design == .monospaced
        if (monospaced ? codeFamily : interfaceFamily) == .system {
            return .system(size: size * scale(monospaced: monospaced), weight: weight, design: design)
        }
        return Font(nsFont(size, monospaced: monospaced) as CTFont).weight(weight)
    }

    func nsFont(_ size: CGFloat, monospaced: Bool = false) -> NSFont {
        let scaled = size * scale(monospaced: monospaced)
        return monospaced
            ? codeFamily.nsFont(size: scaled, ligatures: codeLigatures)
            : interfaceFamily.nsFont(size: scaled, monospaced: false)
    }

    /// Short ids share one nominal size wherever they appear.
    var identifierFont: Font {
        font(11, design: .monospaced)
    }

    var editorFont: NSFont {
        nsFont(Self.referenceSize, monospaced: true)
    }

    var diffTypography: DiffTypography {
        DiffTypography(font: editorFont, lineSpacing: codeLineSpacing)
    }

    var interfaceScale: CGFloat {
        scale(monospaced: false)
    }

    private func scale(monospaced: Bool) -> CGFloat {
        (monospaced ? codeSize : interfaceSize) / Self.referenceSize
    }
}
