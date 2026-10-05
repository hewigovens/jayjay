import AppKit
import SwiftUI

public struct DiffTypography: Equatable {
    public var fontFamily: String
    public var fontSize: Double

    public init(fontFamily: String = "", fontSize: Double = 12) {
        self.fontFamily = fontFamily
        self.fontSize = fontSize
    }

    var font: NSFont {
        NSFont(name: fontFamily, size: fontSize) ?? .monospacedSystemFont(ofSize: fontSize, weight: .regular)
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
