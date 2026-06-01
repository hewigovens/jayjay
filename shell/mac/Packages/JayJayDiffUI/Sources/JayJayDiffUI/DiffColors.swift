import AppKit
import JayJayCore

/// Shared diff color theme used by unified and side-by-side renderers.
public struct DiffColors {
    private let palette: DiffThemeColors

    public init(isDark: Bool) {
        palette = diffThemeColors(isDark: isDark)
    }

    var gutterText: NSColor {
        NSColor(hex: palette.gutterFg)
    }

    public var contextText: NSColor {
        NSColor(hex: palette.textContext)
    }

    var addedText: NSColor {
        NSColor(hex: palette.textAdded)
    }

    var addedBg: NSColor {
        NSColor(hex: palette.addedBg)
    }

    var addedWordBg: NSColor {
        NSColor(hex: palette.addedWordBg)
    }

    var removedText: NSColor {
        NSColor(hex: palette.textRemoved)
    }

    var removedBg: NSColor {
        NSColor(hex: palette.removedBg)
    }

    var removedWordBg: NSColor {
        NSColor(hex: palette.removedWordBg)
    }

    var separatorBg: NSColor {
        NSColor(hex: palette.separatorBg)
    }

    var conflictHeaderBg: NSColor {
        NSColor(hex: palette.conflictHeaderBg)
    }

    var conflictSectionBg: NSColor {
        NSColor(hex: palette.conflictSectionBg)
    }

    var conflictContentBg: NSColor {
        NSColor(hex: palette.conflictContentBg)
    }

    var conflictHeaderText: NSColor {
        NSColor(hex: palette.conflictHeaderFg)
    }

    var conflictSectionText: NSColor {
        NSColor(hex: palette.conflictSectionFg)
    }

    var conflictStripe: NSColor {
        NSColor(hex: palette.conflictStripe, alpha: CGFloat(palette.conflictStripeAlpha))
    }

    var groupStripe: NSColor {
        NSColor(hex: palette.groupStripe, alpha: CGFloat(palette.groupStripeAlpha))
    }

    var findCurrentMatchBg: NSColor {
        NSColor(hex: palette.findMatchBg)
    }

    var findCurrentMatchText: NSColor {
        NSColor(hex: palette.findMatchFg)
    }

    /// Syntax tokens (GitHub-inspired)
    var keyword: NSColor {
        NSColor(hex: palette.tokKeyword)
    }

    var string: NSColor {
        NSColor(hex: palette.tokString)
    }

    var comment: NSColor {
        NSColor(hex: palette.tokComment)
    }

    var number: NSColor {
        NSColor(hex: palette.tokNumber)
    }

    var type: NSColor {
        NSColor(hex: palette.tokType)
    }

    public func tokenColor(_ token: SyntaxToken, fallback: NSColor) -> NSColor {
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

    func lineBg(_ line: DiffLine) -> NSColor {
        lineBg(line.style, conflictKind: line.conflictKind)
    }

    func lineBg(_ style: DiffSpanStyle, conflictKind: ConflictLineKind) -> NSColor {
        switch conflictKind {
            case .start:
                conflictHeaderBg
            case .end, .section:
                conflictSectionBg
            case .content:
                conflictContentBg
            case .added:
                addedBg
            case .removed:
                removedBg
            case .none:
                lineBg(style)
        }
    }

    func lineText(_ style: DiffSpanStyle) -> NSColor {
        switch style {
            case .added: addedText
            case .removed: removedText
            default: contextText
        }
    }

    func lineText(_ line: DiffLine) -> NSColor {
        lineText(line.style, conflictKind: line.conflictKind)
    }

    func lineText(_ style: DiffSpanStyle, conflictKind: ConflictLineKind) -> NSColor {
        switch conflictKind {
            case .start:
                conflictHeaderText
            case .end, .section:
                conflictSectionText
            case .added:
                addedText
            case .removed:
                removedText
            case .content, .none:
                lineText(style)
        }
    }
}
