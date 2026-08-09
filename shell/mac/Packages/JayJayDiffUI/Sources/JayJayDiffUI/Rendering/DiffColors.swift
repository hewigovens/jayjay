import AppKit
import JayJayCore

/// Shared diff color theme used by unified and side-by-side renderers.
public struct DiffColors {
    private let palette: DiffThemeColors
    private let isDark: Bool

    public init(isDark: Bool) {
        palette = diffThemeColors(isDark: isDark)
        self.isDark = isDark
    }

    /// Background band for embedded review-note rows, spanning gutter and content.
    var noteRowBg: NSColor {
        NSColor.systemOrange.withAlphaComponent(isDark ? 0.13 : 0.09)
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

    func spanText(_ span: DiffSpan, lineStyle: DiffSpanStyle, conflictKind: ConflictLineKind) -> NSColor {
        switch span.style {
            case .added:
                return addedText
            case .removed:
                return removedText
            default:
                let fallback = conflictKind == .none
                    ? contextText
                    : lineText(lineStyle, conflictKind: conflictKind)
                return tokenColor(span.token, fallback: fallback)
        }
    }

    func spanBackground(_ span: DiffSpan) -> NSColor {
        switch span.style {
            case .added: addedWordBg
            case .removed: removedWordBg
            default: .clear
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

    public func inlineBackground(lineStyle: DiffSpanStyle, spanStyle: DiffSpanStyle) -> NSColor {
        switch spanStyle {
            case .added: addedWordBg
            case .removed: removedWordBg
            default: lineBg(lineStyle)
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
