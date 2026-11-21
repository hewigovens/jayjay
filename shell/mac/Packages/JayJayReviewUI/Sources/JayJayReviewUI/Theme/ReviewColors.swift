import SwiftUI

/// From review-web's `base.css` (`--jj-kind-*`, `--jj-reviewed-ink`, `--jj-note-ink`).
enum ReviewColors {
    static func kind(_ kind: FileKind) -> Color {
        switch kind {
            case .added, .copied: dynamic(light: 0x1C7C3A, dark: 0x5FD17F)
            case .removed: dynamic(light: 0xB42318, dark: 0xFF8A80)
            case .modified: dynamic(light: 0x175CD3, dark: 0x7CB4FF)
            case .renamed: dynamic(light: 0xA21CAF, dark: 0xE28CF0)
        }
    }

    static let reviewed = dynamic(light: 0x1C7C3A, dark: 0x5FD17F)
    static let note = dynamic(light: 0xB45309, dark: 0xEDC18A)

    private static func dynamic(light: UInt32, dark: UInt32) -> Color {
        #if canImport(UIKit)
            Color(UIColor { $0.userInterfaceStyle == .dark ? UIColor(Color(rgb: dark)) : UIColor(Color(rgb: light)) })
        #else
            Color(NSColor(name: nil) { appearance in
                appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua ? NSColor(Color(rgb: dark)) : NSColor(Color(rgb: light))
            })
        #endif
    }
}
