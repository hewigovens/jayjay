import AppKit
import SwiftUI

public struct DiffTypography: Equatable {
    public var font: NSFont
    public var interfaceFont: NSFont
    /// Line height multiple; 1 keeps the font's natural line height.
    public var lineSpacing: Double

    public init(
        font: NSFont = .monospacedSystemFont(ofSize: 12, weight: .regular),
        lineSpacing: Double = 1,
        interfaceFont: NSFont = .systemFont(ofSize: 11)
    ) {
        self.font = font
        self.interfaceFont = interfaceFont
        self.lineSpacing = lineSpacing
    }
}

private struct DiffTypographyKey: EnvironmentKey {
    static let defaultValue = DiffTypography()
}

public extension EnvironmentValues {
    var diffTypography: DiffTypography {
        get { self[DiffTypographyKey.self] }
        set { self[DiffTypographyKey.self] = newValue }
    }
}
