import AppKit
import JayJayBindings

/// Shared diff color theme used by unified and side-by-side renderers.
struct DiffColors {
    let isDark: Bool

    var gutterText: NSColor { isDark ? NSColor(white: 0.45, alpha: 1) : NSColor(white: 0.65, alpha: 1) }
    var contextText: NSColor { isDark ? NSColor(white: 0.85, alpha: 1) : NSColor(white: 0.15, alpha: 1) }

    var addedText: NSColor { isDark ? NSColor(red: 0.47, green: 0.91, blue: 0.53, alpha: 1) : NSColor(red: 0.08, green: 0.47, blue: 0.17, alpha: 1) }
    var addedBg: NSColor { isDark ? NSColor(red: 0.07, green: 0.15, blue: 0.12, alpha: 1) : NSColor(red: 0.85, green: 0.98, blue: 0.88, alpha: 1) }
    var addedWordBg: NSColor { isDark ? NSColor(red: 0.15, green: 0.42, blue: 0.22, alpha: 0.5) : NSColor(red: 0.67, green: 0.93, blue: 0.73, alpha: 0.6) }

    var removedText: NSColor { isDark ? NSColor(red: 1, green: 0.48, blue: 0.45, alpha: 1) : NSColor(red: 0.82, green: 0.17, blue: 0.14, alpha: 1) }
    var removedBg: NSColor { isDark ? NSColor(red: 0.18, green: 0.08, blue: 0.08, alpha: 1) : NSColor(red: 1, green: 0.93, blue: 0.94, alpha: 1) }
    var removedWordBg: NSColor { isDark ? NSColor(red: 0.52, green: 0.15, blue: 0.15, alpha: 0.5) : NSColor(red: 1, green: 0.78, blue: 0.78, alpha: 0.6) }

    var separatorBg: NSColor { isDark ? NSColor(white: 0.16, alpha: 1) : NSColor(white: 0.94, alpha: 1) }

    // Syntax tokens (GitHub-inspired)
    var keyword: NSColor { isDark ? NSColor(red: 1, green: 0.48, blue: 0.45, alpha: 1) : NSColor(red: 0.84, green: 0.23, blue: 0.29, alpha: 1) }
    var string: NSColor { isDark ? NSColor(red: 0.65, green: 0.84, blue: 1, alpha: 1) : NSColor(red: 0.01, green: 0.18, blue: 0.38, alpha: 1) }
    var comment: NSColor { isDark ? NSColor(red: 0.55, green: 0.58, blue: 0.63, alpha: 1) : NSColor(red: 0.42, green: 0.45, blue: 0.49, alpha: 1) }
    var number: NSColor { isDark ? NSColor(red: 0.47, green: 0.75, blue: 1, alpha: 1) : NSColor(red: 0, green: 0.36, blue: 0.77, alpha: 1) }
    var type: NSColor { isDark ? NSColor(red: 0.82, green: 0.66, blue: 1, alpha: 1) : NSColor(red: 0.44, green: 0.26, blue: 0.76, alpha: 1) }

    func tokenColor(_ token: SyntaxToken, fallback: NSColor) -> NSColor {
        switch token {
        case .comment: comment
        case .keyword, .operator: keyword
        case .stringLit: string
        case .number: number
        case .type, .function, .attribute: type
        default: fallback
        }
    }

    func lineBg(_ style: DiffSpanStyle) -> NSColor {
        switch style {
        case .added: addedBg
        case .removed: removedBg
        case .separator: separatorBg
        default: .clear
        }
    }

    func lineText(_ style: DiffSpanStyle) -> NSColor {
        switch style {
        case .added: addedText
        case .removed: removedText
        default: contextText
        }
    }

    func markerColor(_ marker: String) -> NSColor {
        switch marker {
        case "+": addedText
        case "-": removedText
        default: gutterText
        }
    }
}
